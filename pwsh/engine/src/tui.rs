use std::io::{self, stderr, Stderr};
use std::path::Path;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

use crate::db::Database;
use crate::matcher::FuzzyEngine;
use crate::prober::{Prober, ProjectContext};

#[derive(Debug, Clone)]
pub enum CandidateKind {
    SmartAction {
        label: String,
        command: String,
        source: String,
    },
    Alias {
        trigger: String,
        expansion: String,
        desc: Option<String>,
    },
    LocalHistory {
        id: i64,
        command: String,
        time: String,
    },
    GlobalHistory {
        id: i64,
        command: String,
        #[allow(dead_code)]
        time: String,
        cwd: String,
    },
    DirectoryHop {
        path: String,
        label: String,
    },
}

impl CandidateKind {
    pub fn get_command(&self) -> String {
        match self {
            Self::SmartAction { command, .. } => command.clone(),
            Self::Alias { expansion, .. } => expansion.clone(),
            Self::LocalHistory { command, .. } => command.clone(),
            Self::GlobalHistory { command, .. } => command.clone(),
            Self::DirectoryHop { path, .. } => format!("cd '{}'", path),
        }
    }

    pub fn get_search_text(&self) -> String {
        match self {
            Self::SmartAction { label, command, .. } => format!("{} {}", label, command),
            Self::Alias { trigger, expansion, desc, .. } => {
                format!("{} {} {}", trigger, expansion, desc.as_deref().unwrap_or(""))
            }
            Self::LocalHistory { command, .. } => command.clone(),
            Self::GlobalHistory { command, .. } => command.clone(),
            Self::DirectoryHop { path, label } => format!("cd {} {}", label, path),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierFilter {
    All,
    Smart,
    Local,
    Global,
}

impl TierFilter {
    pub fn next(&self) -> Self {
        match self {
            Self::All => Self::Smart,
            Self::Smart => Self::Local,
            Self::Local => Self::Global,
            Self::Global => Self::All,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Tiers",
            Self::Smart => "★ Smart/Alias Only",
            Self::Local => "● Local Only",
            Self::Global => "○ Global Only",
        }
    }
}

#[allow(dead_code)]
pub struct App {
    pub query: String,
    pub cwd: String,
    pub project_context: ProjectContext,
    pub all_items: Vec<CandidateKind>,
    pub filtered_items: Vec<CandidateKind>,
    pub list_state: ListState,
    pub filter: TierFilter,
    pub status_message: Option<(String, Color)>,
}

impl App {
    pub fn new(db: &Database, cwd: &str, initial_query: Option<&str>) -> Self {
        let path = Path::new(cwd);
        let project_context = Prober::probe(path);

        let mut all_items = Vec::new();

        // 1. Smart Actions from Prober
        for act in &project_context.actions {
            all_items.push(CandidateKind::SmartAction {
                label: act.label.clone(),
                command: act.command.clone(),
                source: act.source.clone(),
            });
        }

        // 2. Aliases
        if let Ok(aliases) = db.get_all_aliases() {
            for a in aliases {
                all_items.push(CandidateKind::Alias {
                    trigger: a.trigger,
                    expansion: a.expansion,
                    desc: a.description,
                });
            }
        }

        // 3. Directory hops (frequent directories)
        if let Ok(dirs) = db.get_recent_directories(20) {
            for (dir_path, _) in dirs {
                if dir_path != cwd {
                    let folder_name = Path::new(&dir_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| dir_path.clone());
                    all_items.push(CandidateKind::DirectoryHop {
                        path: dir_path,
                        label: folder_name,
                    });
                }
            }
        }

        // 4. Local history
        if let Ok(locals) = db.get_local_history(cwd, 200) {
            let mut seen = std::collections::HashSet::new();
            for item in locals {
                if seen.insert(item.command.clone()) {
                    all_items.push(CandidateKind::LocalHistory {
                        id: item.id,
                        command: item.command,
                        time: item.created_at,
                    });
                }
            }
        }

        // 5. Global history
        if let Ok(globals) = db.get_global_history(300) {
            let mut seen = std::collections::HashSet::new();
            for item in globals {
                if item.cwd != cwd && seen.insert(item.command.clone()) {
                    all_items.push(CandidateKind::GlobalHistory {
                        id: item.id,
                        command: item.command,
                        time: item.created_at,
                        cwd: item.cwd,
                    });
                }
            }
        }

        let query = initial_query.unwrap_or("").to_string();
        let mut app = Self {
            query,
            cwd: cwd.to_string(),
            project_context,
            all_items,
            filtered_items: Vec::new(),
            list_state: ListState::default(),
            filter: TierFilter::All,
            status_message: None,
        };

        app.recompute_filtered();
        if !app.filtered_items.is_empty() {
            app.list_state.select(Some(0));
        }

        app
    }

    pub fn recompute_filtered(&mut self) {
        let mut fuzzy = FuzzyEngine::new();
        let query = self.query.trim();

        let eligible: Vec<CandidateKind> = self
            .all_items
            .iter()
            .filter(|item| match self.filter {
                TierFilter::All => true,
                TierFilter::Smart => matches!(
                    item,
                    CandidateKind::SmartAction { .. }
                        | CandidateKind::Alias { .. }
                        | CandidateKind::DirectoryHop { .. }
                ),
                TierFilter::Local => matches!(item, CandidateKind::LocalHistory { .. }),
                TierFilter::Global => matches!(item, CandidateKind::GlobalHistory { .. }),
            })
            .cloned()
            .collect();

        let ranked = fuzzy.filter_and_rank(
            eligible,
            |item| item.get_search_text(),
            query,
            |item| match item {
                CandidateKind::SmartAction { .. } => 300,
                CandidateKind::Alias { .. } => 250,
                CandidateKind::DirectoryHop { .. } => 200,
                CandidateKind::LocalHistory { .. } => 150,
                CandidateKind::GlobalHistory { .. } => 50,
            },
        );

        self.filtered_items = ranked.into_iter().map(|s| s.item).collect();

        // Adjust selection index
        if self.filtered_items.is_empty() {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(0);
            if current >= self.filtered_items.len() {
                self.list_state.select(Some(self.filtered_items.len() - 1));
            } else {
                self.list_state.select(Some(current));
            }
        }
    }

    pub fn delete_selected(&mut self, db: &Database) -> bool {
        if let Some(idx) = self.list_state.selected() {
            if idx < self.filtered_items.len() {
                let item = self.filtered_items[idx].clone();
                match item {
                    CandidateKind::LocalHistory { id, command, .. }
                    | CandidateKind::GlobalHistory { id, command, .. } => {
                        if db.delete_history(id).unwrap_or(false) {
                            self.status_message = Some((
                                format!("✓ 已從歷史記錄清除: {}", command),
                                Color::LightRed,
                            ));
                            // Remove from all_items
                            self.all_items.retain(|i| match i {
                                CandidateKind::LocalHistory { id: i_id, .. }
                                | CandidateKind::GlobalHistory { id: i_id, .. } => *i_id != id,
                                _ => true,
                            });
                            self.recompute_filtered();
                            return true;
                        }
                    }
                    CandidateKind::Alias { trigger, .. } => {
                        if db.remove_alias(&trigger).unwrap_or(false) {
                            self.status_message =
                                Some((format!("✓ 已刪除別名: {}", trigger), Color::Yellow));
                            self.all_items.retain(|i| match i {
                                CandidateKind::Alias { trigger: t, .. } => t != &trigger,
                                _ => true,
                            });
                            self.recompute_filtered();
                            return true;
                        }
                    }
                    _ => {
                        self.status_message =
                            Some(("無法刪除動態生成的環境操作項目".to_string(), Color::DarkGray));
                    }
                }
            }
        }
        false
    }
}

pub fn run_tui(db: &Database, cwd: &str, initial_query: Option<&str>) -> io::Result<Option<String>> {
    enable_raw_mode()?;
    let mut err_out = stderr();
    execute!(err_out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(err_out);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db, cwd, initial_query);
    let result = run_loop(&mut terminal, &mut app, db);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stderr>>,
    app: &mut App,
    db: &Database,
) -> io::Result<Option<String>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Ignore key release events
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = app.list_state.selected() {
                            if let Some(item) = app.filtered_items.get(idx) {
                                return Ok(Some(item.get_command()));
                            }
                        }
                        if !app.query.is_empty() {
                            return Ok(Some(app.query.clone()));
                        }
                        return Ok(None);
                    }
                    KeyCode::Up => {
                        if let Some(idx) = app.list_state.selected() {
                            if idx > 0 {
                                app.list_state.select(Some(idx - 1));
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(idx) = app.list_state.selected() {
                            if idx + 1 < app.filtered_items.len() {
                                app.list_state.select(Some(idx + 1));
                            }
                        }
                    }
                    KeyCode::Tab => {
                        app.filter = app.filter.next();
                        app.recompute_filtered();
                    }
                    KeyCode::Delete => {
                        app.delete_selected(db);
                    }
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
                        app.delete_selected(db);
                    }
                    KeyCode::Backspace => {
                        if !app.query.is_empty() {
                            app.query.pop();
                            app.recompute_filtered();
                        }
                    }
                    KeyCode::Char(c) => {
                        app.query.push(c);
                        app.recompute_filtered();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Query Input
            Constraint::Min(5),    // Hierarchical Candidate List
            Constraint::Length(1), // Keybinding Help & Toast
        ])
        .split(f.area());

    // 1. Input Box
    let proj_tag = app
        .project_context
        .project_type
        .as_deref()
        .unwrap_or("General");
    let input_title = format!(" 🚀 Smart Hub [{}] Filter: [{}] ", proj_tag, app.filter.label());

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(input_title);

    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(&app.query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]);
    let input_p = Paragraph::new(input_line).block(input_block);
    f.render_widget(input_p, chunks[0]);

    // 2. Candidate List
    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = app.list_state.selected() == Some(i);
            let cursor = if is_selected { "▸ " } else { "  " };

            let line = match item {
                CandidateKind::SmartAction { label, source, .. } => Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow)),
                    Span::styled("[★ Smart] ", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
                    Span::styled(label, Style::default().fg(Color::White)),
                    Span::styled(format!(" ({})", source), Style::default().fg(Color::DarkGray)),
                ]),
                CandidateKind::Alias { trigger, expansion, desc } => Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow)),
                    Span::styled("[⚡ Alias] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ➔ ", trigger), Style::default().fg(Color::LightMagenta)),
                    Span::styled(expansion, Style::default().fg(Color::White)),
                    Span::styled(
                        desc.as_ref().map(|d| format!(" ({})", d)).unwrap_or_default(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                CandidateKind::DirectoryHop { label, path } => Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow)),
                    Span::styled("[📁 Nav] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("cd {} ", label), Style::default().fg(Color::LightBlue)),
                    Span::styled(format!("({})", path), Style::default().fg(Color::DarkGray)),
                ]),
                CandidateKind::LocalHistory { command, time, .. } => Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow)),
                    Span::styled("[● Local] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(command, Style::default().fg(Color::White)),
                    Span::styled(format!("  [{}]", time), Style::default().fg(Color::DarkGray)),
                ]),
                CandidateKind::GlobalHistory { command, cwd, .. } => Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow)),
                    Span::styled("[○ Global] ", Style::default().fg(Color::DarkGray)),
                    Span::styled(command, Style::default().fg(Color::Gray)),
                    Span::styled(format!("  (in {})", cwd), Style::default().fg(Color::DarkGray)),
                ]),
            };

            let item_style = if is_selected {
                Style::default().bg(Color::Rgb(35, 45, 60))
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list_title = format!(" 候選清單 ({} 項) ", app.filtered_items.len());
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(list_title);

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // 3. Footer / Status
    let footer_line = if let Some((msg, color)) = &app.status_message {
        Line::from(vec![Span::styled(msg, Style::default().fg(*color).add_modifier(Modifier::BOLD))])
    } else {
        Line::from(vec![
            Span::styled(" [Enter] 執行 ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(" [Tab] 切換層級 ", Style::default().fg(Color::Black).bg(Color::Yellow)),
            Span::raw(" "),
            Span::styled(" [Del / Alt+D] 刪除錯指令 ", Style::default().fg(Color::White).bg(Color::Red)),
            Span::raw(" "),
            Span::styled(" [Esc] 取消 ", Style::default().fg(Color::DarkGray)),
        ])
    };

    let footer = Paragraph::new(footer_line).alignment(Alignment::Left);
    f.render_widget(footer, chunks[2]);
}
