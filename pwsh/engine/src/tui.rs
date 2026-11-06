use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::matcher::FuzzyEngine;
use crate::prober::{Prober, ProjectContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryTab {
    All = 0,
    SmartAlias = 1,
    Local = 2,
    Global = 3,
}

impl CategoryTab {
    pub fn next(&self) -> Self {
        match self {
            Self::All => Self::SmartAlias,
            Self::SmartAlias => Self::Local,
            Self::Local => Self::Global,
            Self::Global => Self::All,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::All => Self::Global,
            Self::SmartAlias => Self::All,
            Self::Local => Self::SmartAlias,
            Self::Global => Self::Local,
        }
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ItemKind {
    DevAction { label: String, source: String },
    Alias { id: i64, trigger: String, description: Option<String> },
    LocalHistory { id: i64, freq: i64, last_time: String },
    GlobalHistory { id: i64, freq: i64, last_time: String, cwd: String },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CandidateItem {
    pub id: Option<i64>,
    pub command: String,
    pub kind: ItemKind,
    pub cwd: Option<String>,
    pub git_root: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub alias_trigger: Option<String>,
    pub score: u32,
}

#[derive(Debug, Clone)]
pub struct AliasModalState {
    pub target_command: String,
    pub input_trigger: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub action: String, // "execute" | "insert"
    pub command: String,
}

pub struct AppState {
    pub active_tab: CategoryTab,
    pub query: String,
    pub selected_index: usize,
    pub candidates: Vec<CandidateItem>,
    pub all_items: Vec<CandidateItem>,
    pub alias_modal: Option<AliasModalState>,
    pub status_message: Option<(String, Instant)>,
    pub cwd: PathBuf,
    pub project: ProjectContext,
}

impl AppState {
    pub fn new(cwd: PathBuf, initial_query: String, db: &Database) -> Self {
        let project = Prober::probe(&cwd);
        let cwd_str = cwd.to_string_lossy().to_string();

        let mut all_items = Vec::new();

        // 1. Load Virtual Aliases
        if let Ok(aliases) = db.get_all_aliases() {
            for a in aliases {
                all_items.push(CandidateItem {
                    id: Some(a.id),
                    command: a.expansion.clone(),
                    kind: ItemKind::Alias {
                        id: a.id,
                        trigger: a.trigger.clone(),
                        description: a.description.clone(),
                    },
                    cwd: None,
                    git_root: None,
                    exit_code: Some(0),
                    duration_ms: None,
                    alias_trigger: Some(a.trigger),
                    score: 1000,
                });
            }
        }

        // 2. Load Probed Dev Actions
        for action in &project.actions {
            all_items.push(CandidateItem {
                id: None,
                command: action.command.clone(),
                kind: ItemKind::DevAction {
                    label: action.label.clone(),
                    source: action.source.clone(),
                },
                cwd: project.root.as_ref().map(|p| p.to_string_lossy().to_string()),
                git_root: project.git_root.as_ref().map(|p| p.to_string_lossy().to_string()),
                exit_code: Some(0),
                duration_ms: None,
                alias_trigger: None,
                score: 900,
            });
        }

        // 3. Load Local History
        let git_root_str = project.git_root.as_ref().map(|p| p.to_string_lossy().to_string());
        if let Ok(locals) = db.get_local_history(&cwd_str, git_root_str.as_deref(), 200) {
            for h in locals {
                all_items.push(CandidateItem {
                    id: Some(h.id),
                    command: h.command.clone(),
                    kind: ItemKind::LocalHistory {
                        id: h.id,
                        freq: h.freq,
                        last_time: h.created_at.clone(),
                    },
                    cwd: Some(h.cwd.clone()),
                    git_root: h.git_root.clone(),
                    exit_code: h.exit_code,
                    duration_ms: h.duration_ms,
                    alias_trigger: None,
                    score: 500 + (h.freq.min(50) as u32) * 5,
                });
            }
        }

        // 4. Load Global History
        if let Ok(globals) = db.get_global_history(&cwd_str, 500) {
            for h in globals {
                all_items.push(CandidateItem {
                    id: Some(h.id),
                    command: h.command.clone(),
                    kind: ItemKind::GlobalHistory {
                        id: h.id,
                        freq: h.freq,
                        last_time: h.created_at.clone(),
                        cwd: h.cwd.clone(),
                    },
                    cwd: Some(h.cwd.clone()),
                    git_root: h.git_root.clone(),
                    exit_code: h.exit_code,
                    duration_ms: h.duration_ms,
                    alias_trigger: None,
                    score: 100 + (h.freq.min(50) as u32) * 2,
                });
            }
        }

        // Mark existing aliases on commands
        let alias_map: std::collections::HashMap<String, String> = all_items
            .iter()
            .filter_map(|item| {
                if let ItemKind::Alias { ref trigger, .. } = item.kind {
                    Some((item.command.clone(), trigger.clone()))
                } else {
                    None
                }
            })
            .collect();

        for item in &mut all_items {
            if item.alias_trigger.is_none() {
                if let Some(t) = alias_map.get(&item.command) {
                    item.alias_trigger = Some(t.clone());
                }
            }
        }

        let mut app = Self {
            active_tab: CategoryTab::All,
            query: initial_query,
            selected_index: 0,
            candidates: Vec::new(),
            all_items,
            alias_modal: None,
            status_message: None,
            cwd,
            project,
        };

        app.filter_and_rank();
        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn filter_and_rank(&mut self) {
        let mut matcher = FuzzyEngine::new();
        let query_trimmed = self.query.trim();

        let mut filtered: Vec<CandidateItem> = self
            .all_items
            .iter()
            .filter(|item| match self.active_tab {
                CategoryTab::All => true,
                CategoryTab::SmartAlias => matches!(item.kind, ItemKind::DevAction { .. } | ItemKind::Alias { .. }),
                CategoryTab::Local => matches!(item.kind, ItemKind::LocalHistory { .. }),
                CategoryTab::Global => matches!(item.kind, ItemKind::GlobalHistory { .. }),
            })
            .filter_map(|item| {
                if query_trimmed.is_empty() {
                    let mut cloned = item.clone();
                    cloned.score = match item.kind {
                        ItemKind::Alias { .. } => 1000,
                        ItemKind::DevAction { .. } => 800,
                        ItemKind::LocalHistory { freq, .. } => 500 + (freq.min(50) as u32) * 5,
                        ItemKind::GlobalHistory { freq, .. } => 100 + (freq.min(50) as u32) * 2,
                    };
                    Some(cloned)
                } else {
                    // Match against command or alias trigger
                    let match_target = if let Some(ref t) = item.alias_trigger {
                        format!("{} {}", t, item.command)
                    } else {
                        item.command.clone()
                    };

                    matcher.score(&match_target, query_trimmed).map(|score| {
                        let mut cloned = item.clone();
                        let boost = match item.kind {
                            ItemKind::Alias { .. } => 500,
                            ItemKind::DevAction { .. } => 300,
                            ItemKind::LocalHistory { .. } => 150,
                            ItemKind::GlobalHistory { .. } => 0,
                        };
                        cloned.score = score + boost;
                        cloned
                    })
                }
            })
            .collect();

        // Sort descending by score
        filtered.sort_by(|a, b| b.score.cmp(&a.score));

        // Deduplicate commands within the filtered list (keeping highest score)
        let mut seen = std::collections::HashSet::new();
        let mut deduped = Vec::new();
        for item in filtered {
            let key = format!("{:?}:{}", std::mem::discriminant(&item.kind), item.command);
            if seen.insert(key) {
                deduped.push(item);
            }
        }

        self.candidates = deduped;
        if self.selected_index >= self.candidates.len() && !self.candidates.is_empty() {
            self.selected_index = self.candidates.len() - 1;
        }
    }

    pub fn reload_data(&mut self, db: &Database) {
        *self = Self::new(self.cwd.clone(), self.query.clone(), db);
    }
}

pub fn run(
    cwd: &Path,
    initial_query: &str,
    output_file: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open_default()?;
    let _ = db.seed_psreadline_history_if_needed();

    // Setup terminal using stderr to prevent interfering with stdout pipes
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(cwd.to_path_buf(), initial_query.to_string(), &db);

    let res = run_loop(&mut terminal, &mut state, &db, output_file);

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = execute!(io::stderr(), LeaveAlternateScreen, cursor::Show);

    res
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
    db: &Database,
    output_file: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Clear status message after 3 seconds
        if let Some((_, time)) = state.status_message {
            if time.elapsed().as_secs() >= 3 {
                state.status_message = None;
            }
        }

        terminal.draw(|f| render_ui(f, state))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // 1. When Alias Modal is open
                if let Some(ref mut modal) = state.alias_modal {
                    match key.code {
                        KeyCode::Enter => {
                            let trigger = modal.input_trigger.trim().to_string();
                            let cmd = modal.target_command.clone();
                            if !trigger.is_empty() {
                                if let Err(e) = db.add_alias(&trigger, &cmd, None, None) {
                                    state.set_status(format!("❌ 別名儲存失敗: {e}"));
                                } else {
                                    state.set_status(format!("✔ 已設定虛擬別名 [{trigger}] ➔ {cmd}"));
                                    state.alias_modal = None;
                                    state.reload_data(db);
                                }
                            } else {
                                state.alias_modal = None;
                            }
                        }
                        KeyCode::Esc => {
                            state.alias_modal = None;
                            state.set_status("取消別名設定");
                        }
                        KeyCode::Backspace => {
                            modal.input_trigger.pop();
                        }
                        KeyCode::Char(c) => {
                            if !c.is_control() {
                                modal.input_trigger.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // 2. Normal Mode Keybindings
                match key.code {
                    // Esc or Ctrl+C: Cancel
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }

                    // Left / Right: Switch Category Tabs
                    KeyCode::Left => {
                        state.active_tab = state.active_tab.prev();
                        state.selected_index = 0;
                        state.filter_and_rank();
                    }
                    KeyCode::Right => {
                        state.active_tab = state.active_tab.next();
                        state.selected_index = 0;
                        state.filter_and_rank();
                    }

                    // Up / Down: Select Candidate
                    KeyCode::Up => {
                        if !state.candidates.is_empty() {
                            if state.selected_index > 0 {
                                state.selected_index -= 1;
                            } else {
                                state.selected_index = state.candidates.len() - 1;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if !state.candidates.is_empty() {
                            if state.selected_index + 1 < state.candidates.len() {
                                state.selected_index += 1;
                            } else {
                                state.selected_index = 0;
                            }
                        }
                    }

                    // Enter: Execute immediately
                    KeyCode::Enter => {
                        if let Some(candidate) = state.candidates.get(state.selected_index) {
                            write_result(output_file, "execute", &candidate.command)?;
                            return Ok(());
                        }
                    }

                    // Tab: Insert into prompt without executing
                    KeyCode::Tab => {
                        if let Some(candidate) = state.candidates.get(state.selected_index) {
                            write_result(output_file, "insert", &candidate.command)?;
                            return Ok(());
                        }
                    }

                    // 'a' or 'A': Open Alias Modal to name / overwrite virtual alias
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        if let Some(candidate) = state.candidates.get(state.selected_index) {
                            state.alias_modal = Some(AliasModalState {
                                target_command: candidate.command.clone(),
                                input_trigger: candidate.alias_trigger.clone().unwrap_or_default(),
                            });
                        }
                    }

                    // Delete: Delete selected item from SQLite (History or Alias)
                    KeyCode::Delete => {
                        if let Some(candidate) = state.candidates.get(state.selected_index).cloned() {
                            match &candidate.kind {
                                ItemKind::Alias { trigger, .. } => {
                                    if let Ok(true) = db.remove_alias(trigger) {
                                        state.set_status(format!("✔ 已刪除虛擬別名 [{trigger}]"));
                                        state.reload_data(db);
                                    }
                                }
                                ItemKind::LocalHistory { id, .. } | ItemKind::GlobalHistory { id, .. } => {
                                    if let Ok(true) = db.delete_history(*id) {
                                        state.set_status(format!("✔ 已從歷史刪除: {}", candidate.command));
                                        state.all_items.retain(|i| i.id != Some(*id));
                                        state.filter_and_rank();
                                    }
                                }
                                ItemKind::DevAction { label, .. } => {
                                    state.set_status(format!("⚠️ 專案指令 [{label}] 來自設定檔，無法刪除"));
                                }
                            }
                        }
                    }

                    // Typing into Search Query
                    KeyCode::Backspace => {
                        state.query.pop();
                        state.selected_index = 0;
                        state.filter_and_rank();
                    }
                    KeyCode::Char(c) => {
                        if !c.is_control() {
                            state.query.push(c);
                            state.selected_index = 0;
                            state.filter_and_rank();
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}

fn write_result(output_file: Option<&Path>, action: &str, command: &str) -> io::Result<()> {
    if let Some(path) = output_file {
        let result = SearchResult {
            action: action.to_string(),
            command: command.to_string(),
        };
        let json = serde_json::to_string(&result)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
    }
    Ok(())
}

fn render_ui(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // Outer container block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" ⚡ Smart-Hub 智慧指令中心 ")
        .title_alignment(Alignment::Left);

    let inner_area = outer_block.inner(size);
    f.render_widget(outer_block, size);

    // Vertical sections: Search (3), Tabs (3), Main (Fill), Status/Help (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner_area);

    // 1. Search Box
    let search_title = Span::styled(
        " 🔍 即時搜尋 / 別名檢索 ",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    );
    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(search_title);

    let query_line = Line::from(vec![
        Span::styled(" ❯ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(&state.query),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]);
    let search_widget = Paragraph::new(query_line).block(search_block);
    f.render_widget(search_widget, chunks[0]);

    // 2. Category Tabs (Left / Right to switch)
    let tab_titles = vec![
        Line::from(if state.active_tab == CategoryTab::All {
            " ◄ 1. 全部 (All) ► "
        } else {
            " 1. 全部 (All) "
        }),
        Line::from(if state.active_tab == CategoryTab::SmartAlias {
            " ◄ 2. ★ 智能/Alias ► "
        } else {
            " 2. ★ 智能/Alias "
        }),
        Line::from(if state.active_tab == CategoryTab::Local {
            " ◄ 3. ● 本地 (Local) ► "
        } else {
            " 3. ● 本地 (Local) "
        }),
        Line::from(if state.active_tab == CategoryTab::Global {
            " ◄ 4. ○ 全域 (Global) ► "
        } else {
            " 4. ○ 全域 (Global) "
        }),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(" 🏷️ 分類過濾 (← / → 切換) ", Style::default().fg(Color::Blue))),
        )
        .select(state.active_tab.to_index())
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
    f.render_widget(tabs, chunks[1]);

    // 3. Main Area: Master-Detail Layout (Horizontal 52% : 48%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(chunks[2]);

    // Left: Candidates List
    let list_title = format!(" 候選清單 (共 {} 筆) ", state.candidates.len());
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(list_title, Style::default().fg(Color::White)));

    let list_items: Vec<ListItem> = state
        .candidates
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.selected_index;
            let (prefix_badge, badge_color) = match &item.kind {
                ItemKind::DevAction { label, .. } => (format!("★ [{label}]"), Color::Yellow),
                ItemKind::Alias { trigger, .. } => (format!("★ [{trigger}]"), Color::Magenta),
                ItemKind::LocalHistory { .. } => ("● [local]".to_string(), Color::Green),
                ItemKind::GlobalHistory { .. } => ("○ [global]".to_string(), Color::Blue),
            };

            let pointer = if is_selected { "▶ " } else { "  " };

            let mut spans = vec![
                Span::styled(
                    pointer,
                    if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(
                    format!("{:<14} ", prefix_badge),
                    Style::default().fg(badge_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &item.command,
                    if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
            ];

            if let Some(ref t) = item.alias_trigger {
                if !matches!(item.kind, ItemKind::Alias { .. }) {
                    spans.push(Span::styled(
                        format!(" (alias: {t})"),
                        Style::default().fg(Color::LightMagenta),
                    ));
                }
            }

            let style = if is_selected {
                Style::default().bg(Color::Rgb(30, 40, 60))
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let list_widget = List::new(list_items).block(list_block);
    f.render_widget(list_widget, main_chunks[0]);

    // Right: Detailed Preview Pane
    let selected_candidate = state.candidates.get(state.selected_index);
    render_preview_pane(f, main_chunks[1], selected_candidate, &state.project);

    // 4. Status and Shortcuts Bar
    render_status_bar(f, chunks[3], state);

    // 5. Alias Modal (if active)
    if let Some(ref modal) = state.alias_modal {
        render_alias_modal(f, size, modal);
    }
}

fn render_preview_pane(
    f: &mut Frame,
    area: Rect,
    candidate: Option<&CandidateItem>,
    project: &ProjectContext,
) {
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" 📋 詳細資訊預覽 ", Style::default().fg(Color::Cyan)));

    let mut lines = Vec::new();

    if let Some(item) = candidate {
        // Command block
        lines.push(Line::from(vec![Span::styled(
            "指令內容:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  {}", item.command),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::raw(""));

        // Source and Alias
        let source_desc = match &item.kind {
            ItemKind::DevAction { label, source } => format!("專案開發腳本 ({source} - {label})"),
            ItemKind::Alias { trigger, description, .. } => {
                let desc = description.as_deref().unwrap_or("自定義別名");
                format!("虛擬別名 [{trigger}] - {desc}")
            }
            ItemKind::LocalHistory { freq, .. } => format!("當前目錄歷史紀錄 (累計執行 {freq} 次)"),
            ItemKind::GlobalHistory { freq, cwd, .. } => {
                format!("全域歷史紀錄 (累計執行 {freq} 次, 於 {cwd})")
            }
        };

        lines.push(Line::from(vec![
            Span::styled("來源類型: ", Style::default().fg(Color::Gray)),
            Span::styled(source_desc, Style::default().fg(Color::Cyan)),
        ]));

        // Virtual Alias Status
        if let Some(ref t) = item.alias_trigger {
            lines.push(Line::from(vec![
                Span::styled("虛擬別名: ", Style::default().fg(Color::LightMagenta)),
                Span::styled(
                    format!("{t} (按 A 鍵可修改 / 覆蓋)",),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("虛擬別名: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "無 (按 A 鍵即可為此指令命名虛擬別名)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        // Execution Directory
        if let Some(ref dir) = item.cwd {
            lines.push(Line::from(vec![
                Span::styled("執行路徑: ", Style::default().fg(Color::Gray)),
                Span::styled(dir, Style::default().fg(Color::LightBlue)),
            ]));
        }

        // Stats (Frequency, Last executed time, Exit code)
        match &item.kind {
            ItemKind::LocalHistory { last_time, freq, .. } | ItemKind::GlobalHistory { last_time, freq, .. } => {
                lines.push(Line::from(vec![
                    Span::styled("累計次數: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{freq} 次"), Style::default().fg(Color::Green)),
                    Span::styled(" │ 最後執行: ", Style::default().fg(Color::Gray)),
                    Span::styled(last_time, Style::default().fg(Color::LightCyan)),
                ]));

                let status_span = match item.exit_code {
                    Some(0) => Span::styled("成功 (Exit 0)", Style::default().fg(Color::Green)),
                    Some(code) => Span::styled(format!("失敗 (Exit {code})"), Style::default().fg(Color::Red)),
                    None => Span::styled("未知", Style::default().fg(Color::DarkGray)),
                };

                let duration_span = if let Some(ms) = item.duration_ms {
                    format!("{ms} ms")
                } else {
                    "-".to_string()
                };

                lines.push(Line::from(vec![
                    Span::styled("執行狀態: ", Style::default().fg(Color::Gray)),
                    status_span,
                    Span::styled(" │ 耗時: ", Style::default().fg(Color::Gray)),
                    Span::styled(duration_span, Style::default().fg(Color::White)),
                ]));
            }
            _ => {}
        }

        // Project Environment
        if let Some(ref p_type) = project.project_type {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled("專案環境: ", Style::default().fg(Color::Yellow)),
                Span::styled(p_type, Style::default().fg(Color::White)),
            ]));
        }
        if let Some(ref git) = project.git_root {
            lines.push(Line::from(vec![
                Span::styled("Git 根目錄: ", Style::default().fg(Color::DarkGray)),
                Span::styled(git.to_string_lossy(), Style::default().fg(Color::DarkGray)),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "（無符合條件的候選指令）",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let preview_widget = Paragraph::new(lines)
        .block(preview_block)
        .wrap(Wrap { trim: false });
    f.render_widget(preview_widget, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let content = if let Some((ref msg, _)) = state.status_message {
        Line::from(vec![
            Span::styled(" 📢 ", Style::default().fg(Color::Yellow)),
            Span::styled(msg, Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [Enter]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" 立即執行 │"),
            Span::styled(" [Tab]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" 回填命令列 │"),
            Span::styled(" [←/→]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" 分類 │"),
            Span::styled(" [↑/↓]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" 選擇 │"),
            Span::styled(" [A]", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
            Span::raw(" 設為別名 │"),
            Span::styled(" [Del]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" 刪除 │"),
            Span::styled(" [Esc]", Style::default().fg(Color::Gray)),
            Span::raw(" 離開"),
        ])
    };

    let p = Paragraph::new(content).block(block);
    f.render_widget(p, area);
}

fn render_alias_modal(f: &mut Frame, area: Rect, modal: &AliasModalState) {
    let popup_area = centered_rect(65, 35, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::LightMagenta))
        .title(" 🏷️ 設定虛擬別名 (Virtual Alias) ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .split(inner);

    // Target command
    let cmd_line = Line::from(vec![
        Span::styled("指令內容: ", Style::default().fg(Color::Gray)),
        Span::styled(&modal.target_command, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(cmd_line).wrap(Wrap { trim: true }), chunks[0]);

    // Input Trigger
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" 請輸入別名 Trigger (若重複將直接覆蓋) ");

    let input_text = Line::from(vec![
        Span::styled(" ❯ ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
        Span::styled(&modal.input_trigger, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("█", Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(input_text).block(input_block), chunks[1]);

    // Help hints
    let help_text = Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" 確認儲存  │  "),
        Span::styled("[Esc]", Style::default().fg(Color::Gray)),
        Span::raw(" 取消返回"),
    ]);
    f.render_widget(Paragraph::new(help_text).alignment(Alignment::Center), chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
