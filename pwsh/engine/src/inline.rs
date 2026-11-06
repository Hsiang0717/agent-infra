use std::fs;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use crossterm::{
    cursor::{position, Hide, MoveTo, Show},
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
    dismissed_commands: std::collections::HashSet<String>,
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
            dismissed_commands: std::collections::HashSet::new(),
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

        // 1. Smart Items: Virtual aliases + Project scripts (per-group deduplication)
        let mut smart = Vec::new();
        let aliases = self.db.get_all_aliases().unwrap_or_default();
        let mut seen_smart = std::collections::HashSet::new();

        for a in &aliases {
            if self.dismissed_commands.contains(&a.expansion) {
                continue;
            }
            if seen_smart.insert(a.expansion.clone()) {
                smart.push(InlineItem {
                    command: a.expansion.clone(),
                    label: format!("[{}] ➔ {}", a.trigger, a.expansion),
                    source: "Virtual Alias".to_string(),
                    cwd: self.cwd.to_string_lossy().to_string(),
                    alias: Some(a.trigger.clone()),
                    id: Some(a.id),
                    is_alias: true,
                });
            }
        }

        let ctx = Prober::probe(&self.cwd);
        for action in ctx.actions {
            if self.dismissed_commands.contains(&action.command) {
                continue;
            }
            if seen_smart.insert(action.command.clone()) {
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

        // 2. Local Items (independent per-group deduplication so Local always retains its history & alias badges)
        let git_root_str = ctx.git_root.as_ref().map(|p| p.to_string_lossy().to_string());
        let cwd_str = self.cwd.to_string_lossy().to_string();
        let local_records = self.db.get_local_history(&cwd_str, git_root_str.as_deref(), 200).unwrap_or_default();
        let mut local = Vec::new();
        let mut seen_local = std::collections::HashSet::new();
        for r in local_records {
            if self.dismissed_commands.contains(&r.command) {
                continue;
            }
            if seen_local.insert(r.command.clone()) {
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

        // 3. Global Items (independent per-group deduplication)
        let global_records = self.db.get_global_history(&cwd_str, 500).unwrap_or_default();
        let mut global = Vec::new();
        let mut seen_global = std::collections::HashSet::new();
        for r in global_records {
            if self.dismissed_commands.contains(&r.command) {
                continue;
            }
            if seen_global.insert(r.command.clone()) {
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

        // Filter by query if non-empty, giving priority boost to exact/prefix alias trigger matches
        let filter_items = |items: Vec<InlineItem>, m: &mut FuzzyEngine| -> Vec<InlineItem> {
            if query.is_empty() {
                return items;
            }
            let q_lower = query.to_lowercase();
            let mut scored = Vec::new();
            for item in items {
                let text = format!("{} {}", item.label, item.command);
                if let Some(mut s) = m.score(&text, query) {
                    if let Some(ref al) = item.alias {
                        let al_lower = al.to_lowercase();
                        if al_lower == q_lower {
                            s += 100_000;
                        } else if al_lower.starts_with(&q_lower) {
                            s += 25_000;
                        }
                    }
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
        let (orig_x, orig_y) = position().unwrap_or((0, 0));
        // 直接開啟 Windows CONOUT$ 主控台螢幕緩衝區，徹底繞過 PowerShell ScriptBlock 對 stdout 的管線攔截
        let mut conout = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONOUT$")?;
        // 先換行至當前提示列正下方，避免覆蓋 PowerShell 提示字元與已輸入文字
        let _ = conout.write_all(b"\r\n");
        let _ = conout.flush();
        execute!(conout, Hide)?;

        let backend = CrosstermBackend::new(conout);
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(INLINE_HEIGHT),
            },
        )?;

        let mut exit_action: Option<(String, String)> = None; // ("TAB" / "EXEC", cmd)
        let mut viewport_top_y: u16 = orig_y.saturating_add(1);

        loop {
            terminal.draw(|f| {
                viewport_top_y = f.area().y;
                self.render(f);
            })?;

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
                                    let target_info = self
                                        .current_items()
                                        .get(self.selected_index)
                                        .map(|it| (it.command.clone(), it.alias.clone()));
                                    if let Some((cmd, old_alias)) = target_info {
                                        if let Some(ref old_tr) = old_alias {
                                            let _ = self.db.remove_alias(old_tr);
                                        }
                                        let _ = self.db.remove_aliases_for_expansion(&cmd);
                                        let _ = self.db.add_alias(&trigger, &cmd, None, None);
                                        let _ = self.reload_data();
                                        if let Some(pos) = self.current_items().iter().position(|it| it.command == cmd) {
                                            self.selected_index = pos;
                                        }
                                    }
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

        // 清空內聯選單區域（僅清除 viewport_top_y 以下），並將游標精準歸位回原提示列
        let new_prompt_y = viewport_top_y.saturating_sub(1);
        let scrolled = orig_y.saturating_sub(new_prompt_y);
        disable_raw_mode()?;
        if let Ok(mut out) = fs::OpenOptions::new().read(true).write(true).open("CONOUT$") {
            let _ = execute!(
                out,
                MoveTo(0, viewport_top_y),
                CtClear(ClearType::FromCursorDown),
                MoveTo(orig_x, new_prompt_y),
                Show
            );
            let _ = out.flush();
        }

        // 輸出回傳動作與捲動行數給 PowerShell（透過暫存檔傳遞，避免污染終端機 stdout）
        if let Some(ref path) = self.output_file {
            let content = match exit_action {
                Some((ref action, ref cmd)) => format!("{}\t{}\t{}", action, scrolled, cmd),
                None => format!("ESC\t{}\t", scrolled),
            };
            let _ = fs::write(path, content);
        } else if let Some((action, cmd)) = exit_action {
            println!("{}\t{}", action, cmd);
        }

        Ok(())
    }

    pub fn render_passive_preview(&self) {
        let Ok((orig_x, orig_y)) = position() else { return };
        let Ok((term_w, term_h)) = crossterm::terminal::size() else { return };
        if term_w < 30 || orig_y + 2 >= term_h {
            return;
        }
        let avail_h = (term_h - orig_y - 1).min(8);
        if avail_h < 4 {
            return;
        }
        let Ok(mut conout) = fs::OpenOptions::new().read(true).write(true).open("CONOUT$") else {
            return;
        };
        let area = ratatui::layout::Rect::new(0, orig_y + 1, term_w, avail_h);
        let backend = CrosstermBackend::new(conout);
        if let Ok(mut term) = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(area),
            },
        ) {
            let _ = term.draw(|f| self.render(f));
        }
        if let Ok(mut out) = fs::OpenOptions::new().read(true).write(true).open("CONOUT$") {
            let _ = execute!(out, MoveTo(orig_x, orig_y), Show);
            let _ = out.flush();
        }
    }

    pub fn clear_passive_preview() {
        let Ok((orig_x, orig_y)) = position() else { return };
        let Ok((_, term_h)) = crossterm::terminal::size() else { return };
        if orig_y + 1 >= term_h {
            return;
        }
        if let Ok(mut out) = fs::OpenOptions::new().read(true).write(true).open("CONOUT$") {
            let _ = execute!(
                out,
                MoveTo(0, orig_y + 1),
                CtClear(ClearType::FromCursorDown),
                MoveTo(orig_x, orig_y),
                Show
            );
            let _ = out.flush();
        }
    }

    fn delete_current_item(&mut self) {
        let (is_alias, alias_trigger, cmd, id) = {
            if let Some(item) = self.current_items().get(self.selected_index) {
                (item.is_alias, item.alias.clone(), item.command.clone(), item.id)
            } else {
                return;
            }
        };

        if let Some(ref trigger) = alias_trigger {
            let _ = self.db.remove_alias(trigger);
        }
        if !is_alias {
            if let Some(history_id) = id {
                let _ = self.db.delete_history(history_id);
            }
            let _ = self.db.delete_command(&cmd);
            self.dismissed_commands.insert(cmd);
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

        // 左欄：候選指令清單（扣除上下邊框 2 行高度，避免第 5/6 項被裁切不可見）
        let list_height = main_chunks[0].height.saturating_sub(2).max(1) as usize;
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

                let clean_cmd = item.command.replace("\r\n", " ↵ ").replace('\n', " ↵ ");
                let clean_label = item.label.replace("\r\n", " ↵ ").replace('\n', " ↵ ");

                // 若有別名，高亮顯示別名
                let display_text = if let Some(ref al) = item.alias {
                    format!("{}. [{}] {}", num, al, clean_cmd)
                } else {
                    format!("{}. {}", num, clean_label)
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
            let clean_cmd = item.command.replace("\r\n", " ↵ ").replace('\n', " ↵ ");
            detail_lines.push(Line::from(vec![
                Span::styled("指令: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(clean_cmd, Style::default().fg(Color::White)),
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
