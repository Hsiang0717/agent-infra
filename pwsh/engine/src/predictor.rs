use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::db::{AliasRecord, Database, HistoryRecord};
use crate::matcher::FuzzyEngine;
use crate::prober::{Prober, ProjectContext};

#[derive(Debug, Clone)]
pub struct PredictionItem {
    pub tier: char, // 'S' = ★ Smart, 'L' = ● Local, 'G' = ○ Global
    pub command: String,
    pub tooltip: String,
    pub score: u32,
}

struct CachedContext {
    context: ProjectContext,
    timestamp: Instant,
}

struct CachedDbSnapshot {
    data_version: i64,
    cwd: String,
    aliases: Vec<AliasRecord>,
    recent_dirs: Vec<(String, i64)>,
    local_history: Vec<HistoryRecord>,
    global_history: Vec<HistoryRecord>,
}

pub struct PredictorEngine {
    fuzzy: FuzzyEngine,
    context_cache: HashMap<String, CachedContext>,
    db_snapshot: Option<CachedDbSnapshot>,
}

impl PredictorEngine {
    pub fn new() -> Self {
        Self {
            fuzzy: FuzzyEngine::new(),
            context_cache: HashMap::new(),
            db_snapshot: None,
        }
    }

    pub fn invalidate_snapshot(&mut self) {
        self.db_snapshot = None;
    }

    pub fn get_project_context(&mut self, cwd: &str) -> ProjectContext {
        let now = Instant::now();
        if let Some(cached) = self.context_cache.get(cwd) {
            if now.duration_since(cached.timestamp) < Duration::from_secs(10) {
                return cached.context.clone();
            }
        }
        let ctx = Prober::probe(Path::new(cwd));
        self.context_cache.insert(
            cwd.to_string(),
            CachedContext {
                context: ctx.clone(),
                timestamp: now,
            },
        );
        ctx
    }

    fn ensure_snapshot(&mut self, db: &Database, cwd: &str, git_root: Option<&str>) {
        let ver = db.get_data_version();
        let needs_reload = match &self.db_snapshot {
            Some(snap) => snap.data_version != ver || !snap.cwd.eq_ignore_ascii_case(cwd),
            None => true,
        };

        if needs_reload {
            let aliases = db.get_all_aliases().unwrap_or_default();
            let recent_dirs = db.get_recent_directories(25).unwrap_or_default();
            let local_history = db.get_local_history(cwd, git_root, 500).unwrap_or_default();
            let global_history = db.get_global_history(cwd, 2500).unwrap_or_default();
            self.db_snapshot = Some(CachedDbSnapshot {
                data_version: ver,
                cwd: cwd.to_string(),
                aliases,
                recent_dirs,
                local_history,
                global_history,
            });
        }
    }

    fn match_score(
        fuzzy: &mut FuzzyEngine,
        candidate: &str,
        query: &str,
        query_lower: &str,
        allow_fuzzy: bool,
    ) -> Option<u32> {
        if query.is_empty() {
            return Some(100);
        }
        let cand_lower = candidate.to_lowercase();
        if cand_lower == *query_lower {
            return None;
        }
        if cand_lower.starts_with(query_lower) {
            return Some(2500);
        }
        if cand_lower.contains(query_lower) {
            if cand_lower.contains(&format!(" {}", query_lower))
                || cand_lower.contains(&format!("-{}", query_lower))
                || cand_lower.contains(&format!("\\{}", query_lower))
                || cand_lower.contains(&format!("/{}", query_lower))
            {
                return Some(1400);
            }
            return Some(1000);
        }
        if allow_fuzzy && query.chars().count() >= 2 {
            if let Some(fscore) = fuzzy.score(candidate, query) {
                let min_threshold = (query.chars().count() as u32) * 22;
                if fscore >= min_threshold {
                    return Some(300 + fscore.min(600));
                }
            }
        }
        None
    }

    pub fn predict(&mut self, db: &Database, cwd: &str, raw_query: &str) -> Vec<PredictionItem> {
        let query = raw_query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        let project_ctx = self.get_project_context(cwd);
        let git_root_str = project_ctx
            .git_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        self.ensure_snapshot(db, cwd, git_root_str.as_deref());
        let snap = match &self.db_snapshot {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut seen = HashSet::new();
        seen.insert(query_lower.clone());

        let mut results = Vec::new();

        // =====================================================================
        // Tier 1: [★ Smart] (Virtual Aliases + Project Prober + Directory Hops)
        // =====================================================================
        let mut smart_candidates: Vec<PredictionItem> = Vec::new();

        // 1a. Virtual Aliases
        for a in &snap.aliases {
            let trig_lower = a.trigger.to_lowercase();
            let desc_text = a
                .description
                .as_deref()
                .map(|d| format!(" ({})", d))
                .unwrap_or_default();

            let score_opt = if trig_lower == query_lower {
                Some(5000)
            } else if trig_lower.starts_with(&query_lower) {
                Some(3800)
            } else {
                Self::match_score(&mut self.fuzzy, &a.expansion, query, &query_lower, true)
            };

            if let Some(sc) = score_opt {
                smart_candidates.push(PredictionItem {
                    tier: 'S',
                    command: a.expansion.clone(),
                    tooltip: format!("★ 虛別名 [{}] ➔ {}{}", a.trigger, a.expansion, desc_text),
                    score: sc + 500,
                });
            }
        }

        // 1b. Project Context Actions
        for (idx, act) in project_ctx.actions.iter().enumerate() {
            let cmd_score = Self::match_score(&mut self.fuzzy, &act.command, query, &query_lower, true);
            let lbl_score = Self::match_score(&mut self.fuzzy, &act.label, query, &query_lower, false);
            if let Some(sc) = cmd_score.or(lbl_score) {
                let order_boost = (20usize.saturating_sub(idx) as u32) * 5;
                smart_candidates.push(PredictionItem {
                    tier: 'S',
                    command: act.command.clone(),
                    tooltip: format!("★ 專案環境感知 [{}] ➔ {}", act.source, act.label),
                    score: sc + 300 + order_boost,
                });
            }
        }

        // 1c. Directory Hops
        let is_cd_query = query_lower.starts_with("cd ")
            || query_lower == "cd"
            || query_lower.starts_with("pushd ")
            || query_lower.starts_with("sl ");
        for (rank, (dir_path, visits)) in snap.recent_dirs.iter().enumerate() {
            if dir_path.eq_ignore_ascii_case(cwd) {
                continue;
            }
            let folder_name = Path::new(dir_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir_path.clone());
            let hop_cmd = format!("cd '{}'", dir_path);

            let sc_opt = if is_cd_query {
                let sub_q = query_lower
                    .trim_start_matches("cd")
                    .trim_start_matches("pushd")
                    .trim_start_matches("sl")
                    .trim();
                if sub_q.is_empty() {
                    Some(1800)
                } else {
                    Self::match_score(&mut self.fuzzy, &folder_name, sub_q, sub_q, true)
                        .or_else(|| Self::match_score(&mut self.fuzzy, dir_path, sub_q, sub_q, false))
                }
            } else if folder_name.to_lowercase().starts_with(&query_lower) && query.len() >= 2 {
                Some(1200)
            } else {
                None
            };

            if let Some(sc) = sc_opt {
                let freq_boost =
                    ((*visits).min(30) as u32) * 10 + (25usize.saturating_sub(rank) as u32) * 5;
                smart_candidates.push(PredictionItem {
                    tier: 'S',
                    command: hop_cmd,
                    tooltip: format!("★ 常用目錄跳轉 ({} 次造訪) ➔ {}", visits, dir_path),
                    score: sc + freq_boost,
                });
            }
        }

        smart_candidates.sort_by(|a, b| b.score.cmp(&a.score));
        let mut smart_count = 0;
        for item in smart_candidates {
            if seen.insert(item.command.to_lowercase()) {
                results.push(item);
                smart_count += 1;
                if smart_count >= 4 {
                    break;
                }
            }
        }

        // =====================================================================
        // Tier 2: [● Local] (Current Directory & Current Git Repo History)
        // =====================================================================
        let mut local_candidates: Vec<PredictionItem> = Vec::new();
        for (rank, rec) in snap.local_history.iter().enumerate() {
            let cmd_lower = rec.command.to_lowercase();
            if seen.contains(&cmd_lower) {
                continue;
            }
            let allow_fuzzy = rank < 100;
            if let Some(sc) =
                Self::match_score(&mut self.fuzzy, &rec.command, query, &query_lower, allow_fuzzy)
            {
                let exact_cwd_boost = if rec.cwd.eq_ignore_ascii_case(cwd) { 250 } else { 0 };
                let recency_boost = (500usize.saturating_sub(rank) as u32) / 2;
                let freq_boost = (rec.freq.min(25) as u32) * 12;
                let tooltip = if rec.cwd.eq_ignore_ascii_case(cwd) {
                    format!("● 當前資料夾歷史 (執行 {} 次 · {})", rec.freq, rec.created_at)
                } else {
                    format!(
                        "● 同專案歷史 [{}] (執行 {} 次 · {})",
                        short_path(&rec.cwd),
                        rec.freq,
                        rec.created_at
                    )
                };
                local_candidates.push(PredictionItem {
                    tier: 'L',
                    command: rec.command.clone(),
                    tooltip,
                    score: sc + exact_cwd_boost + recency_boost + freq_boost,
                });
            }
        }

        local_candidates.sort_by(|a, b| b.score.cmp(&a.score));
        let mut local_count = 0;
        for item in local_candidates {
            if seen.insert(item.command.to_lowercase()) {
                results.push(item);
                local_count += 1;
                if local_count >= 5 {
                    break;
                }
            }
        }

        // =====================================================================
        // Tier 3: [○ Global] (Global SQLite History across all directories)
        // =====================================================================
        let mut global_candidates: Vec<PredictionItem> = Vec::new();
        for (rank, rec) in snap.global_history.iter().enumerate() {
            let cmd_lower = rec.command.to_lowercase();
            if seen.contains(&cmd_lower) {
                continue;
            }
            let allow_fuzzy = rank < 120;
            if let Some(sc) =
                Self::match_score(&mut self.fuzzy, &rec.command, query, &query_lower, allow_fuzzy)
            {
                let recency_boost = (2500usize.saturating_sub(rank) as u32) / 10;
                let freq_boost = (rec.freq.min(25) as u32) * 10;
                let tooltip = if rec.cwd.is_empty() {
                    format!("○ 全域歷史 (執行 {} 次)", rec.freq)
                } else {
                    format!(
                        "○ 全域歷史 [{}] (執行 {} 次 · {})",
                        short_path(&rec.cwd),
                        rec.freq,
                        rec.created_at
                    )
                };
                global_candidates.push(PredictionItem {
                    tier: 'G',
                    command: rec.command.clone(),
                    tooltip,
                    score: sc + recency_boost + freq_boost,
                });
            }
        }

        global_candidates.sort_by(|a, b| b.score.cmp(&a.score));
        let mut global_count = 0;
        for item in global_candidates {
            if seen.insert(item.command.to_lowercase()) {
                results.push(item);
                global_count += 1;
                if global_count >= 12 {
                    break;
                }
            }
        }

        results
    }
}

fn short_path(path: &str) -> String {
    let p = Path::new(path);
    let components: Vec<_> = p.components().collect();
    if components.len() <= 2 {
        return path.to_string();
    }
    let last_two = &components[components.len() - 2..];
    format!(
        "…\\{}",
        last_two
            .iter()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("\\")
    )
}

pub fn escape_tsv_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

pub fn unescape_tsv_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Persistent stdin/stdout worker loop for ultra-low-latency (<0.5ms) ICommandPredictor queries
pub fn run_serve(db: &Database) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = BufWriter::new(stdout.lock());
    let mut engine = PredictorEngine::new();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line
                    .trim_start_matches('\u{feff}')
                    .trim_end_matches(&['\r', '\n'][..]);
                if trimmed.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = trimmed.splitn(4, '\t').collect();
                match parts[0] {
                    "PREDICT" if parts.len() >= 3 => {
                        let cwd = unescape_tsv_field(parts[1]);
                        let query = unescape_tsv_field(parts[2]);
                        let items = engine.predict(db, &cwd, &query);
                        if let Ok(runner) = crate::inline::InlineRunner::new(Path::new(&cwd), &query, None) {
                            runner.render_passive_preview();
                        }
                        let _ = writeln!(writer, "OK\t{}", items.len());
                        for it in items {
                            let _ = writeln!(
                                writer,
                                "{}\t{}\t{}",
                                it.tier,
                                escape_tsv_field(&it.command),
                                escape_tsv_field(&it.tooltip)
                            );
                        }
                        let _ = writer.flush();
                    }
                    "RECORD" if parts.len() >= 4 => {
                        crate::inline::InlineRunner::clear_passive_preview();
                        let cwd = unescape_tsv_field(parts[1]);
                        let exit_code: Option<i32> = parts[2].parse().ok();
                        let cmd = unescape_tsv_field(parts[3]);
                        let git_root = engine
                            .get_project_context(&cwd)
                            .git_root
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = db.record(&cmd, &cwd, git_root.as_deref(), exit_code, None);
                        engine.invalidate_snapshot();
                        let _ = writeln!(writer, "OK");
                        let _ = writer.flush();
                    }
                    "PING" => {
                        let _ = writeln!(writer, "PONG");
                        let _ = writer.flush();
                    }
                    "QUIT" => break,
                    _ => {
                        let _ = writeln!(writer, "ERR\tUnknown command");
                        let _ = writer.flush();
                    }
                }
            }
            Err(_) => break,
        }
    }
}
