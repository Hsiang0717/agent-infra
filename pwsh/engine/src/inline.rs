use std::fs;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use crossterm::{
    cursor::{Hide, MoveUp, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear as CtClear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal, TerminalOptions, Viewport,
};

use crate::db::Database;
use crate::matcher::FuzzyEngine;
use crate::prober::Prober;

#[derive(Debug, Clone)]
pub struct InlineItem {
    pub command: String,
    pub label: String,
    pub source: String,
    pub cwd: String,
    pub alias: Option<String>,
    pub id: Option<i64>,
    pub is_alias: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Smart = 0,
    Local = 1,
    Global = 2,
}

impl Group {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            Group::Smart => "★ Smart / Alias",
            Group::Local => "● Local",
            Group::Global => "○ Global",
        }
    }

    pub fn prev(&self) -> Group {
        match self {
            Group::Smart => Group::Global,
            Group::Local => Group::Smart,
            Group::Global => Group::Local,
        }
    }

    pub fn next(&self) -> Group {
        match self {
            Group::Smart => Group::Local,
            Group::Local => Group::Global,
            Group::Global => Group::Smart,
        }
    }
}

enum InlineState {
    Normal,
    AliasInput { buffer: String },
}

pub struct InlineRunner {
    cwd: PathBuf,
    query: String,
    output_file: Option<PathBuf>,
    db: Database,
    active_group: Group,
    selected_index: usize,
    state: InlineState,
    smart_items: Vec<InlineItem>,
    local_items: Vec<InlineItem>,
    global_items: Vec<InlineItem>,
}

impl InlineRunner {
    pub fn new(cwd: &Path, query: &str, output_file: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::open_default()?;
        let mut runner = Self {
            cwd: cwd.to_path_buf(),
            query: query.trim().to_string(),
            output_file,
            db,
            active_group: Group::Smart,
            selected_index: 0,
            state: InlineState::Normal,
            smart_items: Vec::new(),
            local_items: Vec::new(),
            global_items: Vec::new(),
        };
        runner.reload_data()?;
        Ok(runner)
    }

    pub fn reload_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut matcher = FuzzyEngine::new();
        let query = &self.query;

        // 1. Smart Items: Project scripts + Virtual aliases
        let mut smart = Vec::new();
        let aliases = self.db.get_all_aliases().unwrap_or_default();
        let mut seen_commands = std::collections::HashSet::new();

        for a in &aliases {
            smart.push(InlineItem {
                command: a.expansion.clone(),
                label: format!("[{}] ➔ {}", a.trigger, a.expansion),
                source: "Virtual Alias".to_string(),
                cwd: self.cwd.to_string_lossy().to_string(),
                alias: Some(a.trigger.clone()),
                id: Some(a.id),
                is_alias: true,
            });
            seen_commands.insert(a.expansion.clone());
        }

        let ctx = Prober::probe(&self.cwd);
        for action in ctx.actions {
            if seen_commands.insert(action.command.clone()) {
                smart.push(InlineItem {
                    command: action.command.clone(),
                    label: action.label,
                    source: action.source,
                    cwd: self.cwd.to_string_lossy().to_string(),
                    alias: None,
                    id: None,
                    is_alias: false,
                });
            }
        }

        // 2. Local Items
        let git_root_str = ctx.git_root.as_ref().map(|p| p.to_string_lossy().to_string());
        let cwd_str = self.cwd.to_string_lossy().to_string();
        let local_records = self.db.get_local_history(&cwd_str, git_root_str.as_deref(), 200).unwrap_or_default();
        let mut local = Vec::new();
        for r in local_records {
            if seen_commands.insert(r.command.clone()) {
                let alias = aliases.iter().find(|a| a.expansion == r.command).map(|a| a.trigger.clone());
                local.push(InlineItem {
                    command: r.command.clone(),
                    label: r.command.clone(),
                    source: if r.git_root.is_some() { "Git Repository History".to_string() } else { "Local Directory History".to_string() },
                    cwd: r.cwd,
                    alias,
                    id: Some(r.id),
                    is_alias: false,
                });
            }
        }

        // 3. Global Items
        let global_records = self.db.get_global_history(&cwd_str, 500).unwrap_or_default();
        let mut global = Vec::new();
        for r in global_records {
            if seen_commands.insert(r.command.clone()) {
                let alias = aliases.iter().find(|a| a.expansion == r.command).map(|a| a.trigger.clone());
                global.push(InlineItem {
                    command: r.command.clone(),
                    label: r.command.clone(),
                    source: "Global History".to_string(),
                    cwd: r.cwd,
                    alias,
                    id: Some(r.id),
                    is_alias: false,
                });
            }
        }

        // Filter by query if non-empty
        let filter_items = |items: Vec<InlineItem>, m: &mut FuzzyEngine| -> Vec<InlineItem> {
            if query.is_empty() {
                return items;
            }
            let mut scored = Vec::new();
            for item in items {
                let text = format!("{} {}", item.label, item.command);
                if let Some(s) = m.score(&text, query) {
                    scored.push((s, item));
                }
            }
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            scored.into_iter().map(|(_, it)| it).collect()
        };

        self.smart_items = filter_items(smart, &mut matcher);
        self.local_items = filter_items(local, &mut matcher);
        self.global_items = filter_items(global, &mut matcher);

        // Auto-switch to first non-empty group if current group is empty
        if self.current_items().is_empty() {
            if !self.smart_items.is_empty() {
                self.active_group = Group::Smart;
            } else if !self.local_items.is_empty() {
                self.active_group = Group::Local;
            } else if !self.global_items.is_empty() {
                self.active_group = Group::Global;
            }
        }

        self.clamp_selection();
        Ok(())
    }

    fn current_items(&self) -> &Vec<InlineItem> {
        match self.active_group {
            Group::Smart => &self.smart_items,
            Group::Local => &self.local_items,
            Group::Global => &self.global_items,
        }
    }

    fn clamp_selection(&mut self) {
        let len = self.current_items().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        const INLINE_HEIGHT: u16 = 8;

        enable_raw_mode()?;
        let mut stdout_handle = stdout();
        execute!(stdout_handle, Hide)?;

        let backend = CrosstermBackend::new(stdout_handle);
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(INLINE_HEIGHT),
            },
        )?;

        let mut exit_action: Option<(String, String)> = None; // ("TAB" / "EXEC", cmd)

        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match &mut self.state {
                    InlineState::Normal => {
                        match key.code {
                            // ← / → 切換分組
                            KeyCode::Left => {
                                self.active_group = self.active_group.prev();
                                self.selected_index = 0;
                                self.clamp_selection();
                            }
                            KeyCode::Right => {
                                self.active_group = self.active_group.next();
                                self.selected_index = 0;
                                self.clamp_selection();
                            }
                            // ↑ / ↓ 上下選擇
                            KeyCode::Up => {
                                self.selected_index = self.selected_index.saturating_sub(1);
                                self.clamp_selection();
                            }
                            KeyCode::Down => {
                                let len = self.current_items().len();
                                if len > 0 && self.selected_index + 1 < len {
                                    self.selected_index += 1;
                                }
                            }
                            // Tab 回填至終端機命令列，並恢復原生操作
                            KeyCode::Tab => {
                                if let Some(item) = self.current_items().get(self.selected_index) {
                                    exit_action = Some(("TAB".to_string(), item.command.clone()));
                                }
                                break;
                            }
                            // Enter 直接執行
                            KeyCode::Enter => {
                                if let Some(item) = self.current_items().get(self.selected_index) {
                                    exit_action = Some(("EXEC".to_string(), item.command.clone()));
                                }
                                break;
                            }
                            // a 鍵：命名/覆蓋虛擬別名
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if let Some(item) = self.current_items().get(self.selected_index) {
                                    let default_val = item.alias.clone().unwrap_or_default();
                                    self.state = InlineState::AliasInput { buffer: default_val };
                                }
                            }
                            // Del 鍵：刪除該筆紀錄
                            KeyCode::Delete => {
                                self.delete_current_item();
                            }
                            // Esc 鍵：取消退出，恢復原生操作
                            KeyCode::Esc => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    InlineState::AliasInput { buffer } => {
                        match key.code {
                            KeyCode::Char(c) => {
                                buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                buffer.pop();
                            }
                            KeyCode::Enter => {
                                let trigger = buffer.trim().to_string();
                                if !trigger.is_empty() {
                                    if let Some(item) = self.current_items().get(self.selected_index) {
                                        let _ = self.db.add_alias(&trigger, &item.command, None, None);
                                    }
                                    let _ = self.reload_data();
                                }
                                self.state = InlineState::Normal;
                            }
                            KeyCode::Esc => {
                                self.state = InlineState::Normal;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // 清空內聯選單區域，還原終端機畫面
        disable_raw_mode()?;
        let mut out = stdout();
        let _ = execute!(out, MoveUp(INLINE_HEIGHT), CtClear(ClearType::FromCursorDown), Show);
        let _ = out.flush();

        // 輸出回傳動作給 PowerShell
        if let Some((action, cmd)) = exit_action {
            if let Some(ref path) = self.output_file {
                let content = format!("{}\t{}", action, cmd);
                let _ = fs::write(path, content);
            }
            println!("{}\t{}", action, cmd);
        }

        Ok(())
    }

    fn delete_current_item(&mut self) {
        let (is_alias, alias_trigger, cmd, id) = {
            if let Some(item) = self.current_items().get(self.selected_index) {
                (item.is_alias, item.alias.clone(), item.command.clone(), item.id)
            } else {
                return;
            }
        };

        if is_alias {
            if let Some(ref trigger) = alias_trigger {
                let _ = self.db.remove_alias(trigger);
            }
        } else {
            if let Some(history_id) = id {
                let _ = self.db.delete_history(history_id);
            }
            let _ = self.db.delete_command(&cmd);
        }

        let _ = self.reload_data();
    }

    fn render(&self, f: &mut ratatui::Frame) {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 分組標籤列
                Constraint::Min(4),    // 核心左右排版區 (List 50% | Detail 50%)
                Constraint::Length(1), // 狀態與快捷鍵指示列
            ])
            .split(size);

        // 1. 頂部：分組切換標籤 (左右切換)
        let tab_smart = format!("★ Smart ({})", self.smart_items.len());
        let tab_local = format!("● Local ({})", self.local_items.len());
        let tab_global = format!("○ Global ({})", self.global_items.len());

        let smart_span = if self.active_group == Group::Smart {
            Span::styled(format!(" [ ◀ {} ▶ ] ", tab_smart), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("   {}   ", tab_smart), Style::default().fg(Color::DarkGray))
        };

        let local_span = if self.active_group == Group::Local {
            Span::styled(format!(" [ ◀ {} ▶ ] ", tab_local), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("   {}   ", tab_local), Style::default().fg(Color::DarkGray))
        };

        let global_span = if self.active_group == Group::Global {
            Span::styled(format!(" [ ◀ {} ▶ ] ", tab_global), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("   {}   ", tab_global), Style::default().fg(Color::DarkGray))
        };

        let header_line = Line::from(vec![
            Span::raw(" "),
            smart_span,
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            local_span,
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            global_span,
        ]);
        f.render_widget(Paragraph::new(header_line), chunks[0]);

        // 2. 核心左右排版 (左欄 List 50% | 右欄 Detail 50%)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let items = self.current_items();
        let total_items = items.len();

        // 左欄：候選指令清單
        let list_height = main_chunks[0].height as usize;
        let start_idx = if self.selected_index >= list_height {
            self.selected_index - list_height + 1
        } else {
            0
        };
        let end_idx = (start_idx + list_height).min(total_items);

        let mut list_lines = Vec::new();
        if total_items == 0 {
            list_lines.push(Line::from(Span::styled("  (無相符指令)", Style::default().fg(Color::DarkGray))));
        } else {
            for i in start_idx..end_idx {
                let item = &items[i];
                let is_selected = i == self.selected_index;
                let num = i + 1;

                let (prefix, style) = if is_selected {
                    (" ▶ ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
                } else {
                    ("   ", Style::default().fg(Color::White))
                };

                // 若有別名，高亮顯示別名
                let display_text = if let Some(ref al) = item.alias {
                    format!("{}. [{}] {}", num, al, item.command)
                } else {
                    format!("{}. {}", num, item.label)
                };

                list_lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(display_text, style),
                ]));
            }
        }

        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(format!(" 候選清單 [{}/{}] ", if total_items == 0 { 0 } else { self.selected_index + 1 }, total_items), Style::default().fg(Color::Cyan)));
        f.render_widget(Paragraph::new(list_lines).block(list_block), main_chunks[0]);

        // 右欄：詳細資訊預覽卡片
        let mut detail_lines = Vec::new();
        if let Some(item) = items.get(self.selected_index) {
            detail_lines.push(Line::from(vec![
                Span::styled("指令: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(&item.command, Style::default().fg(Color::White)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("分組: ", Style::default().fg(Color::Cyan)),
                Span::styled(&item.source, Style::default().fg(Color::White)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("路徑: ", Style::default().fg(Color::Magenta)),
                Span::styled(&item.cwd, Style::default().fg(Color::DarkGray)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("別名: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                if let Some(ref al) = item.alias {
                    Span::styled(format!("[{}] (按 a 可重新覆蓋)", al), Style::default().fg(Color::LightGreen))
                } else {
                    Span::styled("(無，按 a 可直接命名)", Style::default().fg(Color::DarkGray))
                },
            ]));
        } else {
            detail_lines.push(Line::from(Span::styled("暫無選中項目", Style::default().fg(Color::DarkGray))));
        }

        let detail_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" 詳細預覽 ", Style::default().fg(Color::Yellow)));
        f.render_widget(Paragraph::new(detail_lines).block(detail_block), main_chunks[1]);

        // 3. 底部：快捷鍵指示列 / 別名輸入框
        let bottom_widget = match &self.state {
            InlineState::Normal => {
                let hint = Line::from(vec![
                    Span::styled(" [↑↓]", Style::default().fg(Color::Yellow)),
                    Span::raw("選擇 "),
                    Span::styled("[←→]", Style::default().fg(Color::Yellow)),
                    Span::raw("切換分組 "),
                    Span::styled("[Tab]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("回填 ", Style::default().fg(Color::Cyan)),
                    Span::styled("[Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled("執行 ", Style::default().fg(Color::Green)),
                    Span::styled("[a]", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled("命名別名 ", Style::default().fg(Color::Magenta)),
                    Span::styled("[Del]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled("刪除 ", Style::default().fg(Color::Red)),
                    Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
                    Span::styled("取消", Style::default().fg(Color::DarkGray)),
                ]);
                Paragraph::new(hint).alignment(Alignment::Left)
            }
            InlineState::AliasInput { buffer } => {
                let prompt = Line::from(vec![
                    Span::styled(" [命名別名] ", Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::raw(" 請輸入別名 Trigger: "),
                    Span::styled(format!("{}_", buffer), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled("  (按 Enter 儲存/覆蓋，按 Esc 放棄)", Style::default().fg(Color::DarkGray)),
                ]);
                Paragraph::new(prompt).alignment(Alignment::Left)
            }
        };

        f.render_widget(bottom_widget, chunks[2]);
    }
}
