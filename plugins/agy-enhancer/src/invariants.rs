use crate::config::{default_slice_invariants_template, MemoryRadarConfig};
use crate::posture::{TranscriptStep, CHUNK_SIZE};
use crate::sanitizer::strip_line_prefix_from_line;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

// ─── Target Invariant Detector (Slice Quirks) ────────────────────────────────

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SliceInvariants {
    pub odd_indents: Vec<usize>,
    pub has_tabs: bool,
    pub has_double_backslashes: bool,
}

impl SliceInvariants {
    pub fn is_empty(&self) -> bool {
        self.odd_indents.is_empty() && !self.has_tabs && !self.has_double_backslashes
    }

    pub fn merge(&mut self, other: SliceInvariants) {
        let mut set: std::collections::BTreeSet<usize> = self.odd_indents.iter().copied().collect();
        set.extend(other.odd_indents);
        self.odd_indents = set.into_iter().collect();
        self.has_tabs |= other.has_tabs;
        self.has_double_backslashes |= other.has_double_backslashes;
    }

    pub fn detect_from_read_output(content: &str) -> Option<Self> {
        let mut odd_indents_set = std::collections::BTreeSet::new();
        let mut has_tabs = false;
        let mut has_double_backslashes = false;
        let mut found_file_lines = 0;

        for line in content.lines() {
            if let Some(file_line) = strip_line_prefix_from_line(line) {
                found_file_lines += 1;

                let trimmed_start = file_line.trim_start();
                if trimmed_start.is_empty() {
                    continue;
                }

                // Count leading whitespace
                let ws_len = file_line.chars().take_while(|c| c.is_whitespace()).count();
                if ws_len > 0 {
                    let ws_str: String = file_line.chars().take(ws_len).collect();
                    if ws_str.contains('\t') {
                        has_tabs = true;
                    } else if ws_len % 2 != 0 {
                        // Ignore standard C/Rust/JS block doc-comment continuation lines (` * `)
                        let is_block_comment_star = ws_len == 1 && trimmed_start.starts_with('*');
                        if !is_block_comment_star {
                            odd_indents_set.insert(ws_len);
                        }
                    }
                }

                // Check for double backslashes in code line (e.g. \\node, \\\\, \\d+)
                if file_line.contains(r"\\") {
                    has_double_backslashes = true;
                }
            }
        }

        if found_file_lines == 0 {
            return None;
        }

        let inv = SliceInvariants {
            odd_indents: odd_indents_set.into_iter().collect(),
            has_tabs,
            has_double_backslashes,
        };

        if inv.is_empty() {
            None
        } else {
            Some(inv)
        }
    }

    pub fn format_hint_with_template(&self, template: &str) -> String {
        let mut parts = Vec::new();
        if !self.odd_indents.is_empty() {
            let indents: Vec<String> = self.odd_indents.iter().map(|n| format!("{}-space", n)).collect();
            parts.push(format!("{} indent", indents.join("/")));
        }
        if self.has_tabs {
            parts.push(r"tab (\t) indent".to_string());
        }
        if self.has_double_backslashes {
            parts.push(r"\\ escape (JSON requires \\\\)".to_string());
        }
        let details = parts.join("; ");
        template.replace("{details}", &details)
    }

    pub fn format_hint(&self) -> String {
        self.format_hint_with_template(&default_slice_invariants_template())
    }
}

pub fn probe_recent_read_invariants_with_template(path: &Path, template: &str) -> Option<String> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let file_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return None,
    };

    if file_len == 0 {
        return None;
    }

    let chunk_start = file_len.saturating_sub(CHUNK_SIZE as u64);
    let read_len = (file_len - chunk_start) as usize;

    if file.seek(SeekFrom::Start(chunk_start)).is_err() {
        return None;
    }

    let mut buf = vec![0u8; read_len];
    if file.read_exact(&mut buf).is_err() {
        return None;
    }

    let lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();
    let mut merged = SliceInvariants::default();

    for line in lines.into_iter().rev() {
        if line.is_empty() || line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }

        if let Ok(line_str) = std::str::from_utf8(line) {
            if let Ok(step) = serde_json::from_str::<TranscriptStep>(line_str.trim()) {
                let st = step.step_type.as_deref().unwrap_or("");
                if st == "EPHEMERAL_MESSAGE" || st == "SYSTEM_SDK" || st == "TOOL_CALL" {
                    continue;
                }

                if st == "GENERIC" {
                    let content_str = match step.content {
                        Some(serde_json::Value::String(ref s)) => s.as_str(),
                        _ => "",
                    };

                    if content_str.contains("Showing lines ") {
                        if let Some(inv) = SliceInvariants::detect_from_read_output(content_str) {
                            merged.merge(inv);
                        }
                    }
                    continue;
                }

                // Reached PLANNER_RESPONSE or USER_INPUT boundary of the latest turn
                break;
            }
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged.format_hint_with_template(template))
    }
}

/// Checks whether a recent `EPHEMERAL_MESSAGE` in the last 64KB chunk already contains `marker`.
/// If `max_planner_turns` is `Some(n)`, stops scanning once `n` `PLANNER_RESPONSE` steps have been passed.
pub fn has_recent_ephemeral_marker(path: &Path, marker: &str, max_planner_turns: Option<usize>) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let file_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return false,
    };

    if file_len == 0 {
        return false;
    }

    let chunk_start = file_len.saturating_sub(CHUNK_SIZE as u64);
    let read_len = (file_len - chunk_start) as usize;

    if file.seek(SeekFrom::Start(chunk_start)).is_err() {
        return false;
    }

    let mut buf = vec![0u8; read_len];
    if file.read_exact(&mut buf).is_err() {
        return false;
    }

    let lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();
    let mut planner_turns_seen = 0;

    for line in lines.into_iter().rev() {
        if line.is_empty() || line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }

        if let Ok(line_str) = std::str::from_utf8(line) {
            if let Ok(step) = serde_json::from_str::<TranscriptStep>(line_str.trim()) {
                let st = step.step_type.as_deref().unwrap_or("");
                if st == "PLANNER_RESPONSE" {
                    planner_turns_seen += 1;
                    if let Some(max_turns) = max_planner_turns {
                        if planner_turns_seen >= max_turns {
                            return false;
                        }
                    }
                } else if st == "EPHEMERAL_MESSAGE" {
                    if let Some(serde_json::Value::String(ref s)) = step.content {
                        if s.contains(marker) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn probe_recent_read_invariants(path: &Path) -> Option<String> {
    probe_recent_read_invariants_with_template(path, &default_slice_invariants_template())
}

// ─── Epistemic Memory Radar ──────────────────────────────────────────────────

/// Probes workspace `.agents/memory/` and `.agents/memory/pitfalls.md` to awaken the model's awareness.
/// Uses templates from `MemoryRadarConfig`.
pub fn probe_project_memory_with_config(workspace_root: Option<&Path>, cfg: &MemoryRadarConfig) -> Option<String> {
    let ws = workspace_root?;
    let memory_dir = ws.join(".agents").join("memory");
    if !memory_dir.exists() {
        return None;
    }

    let pitfalls_path = memory_dir.join("pitfalls.md");
    if pitfalls_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pitfalls_path) {
            let mut entries = 0;
            for line in content.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("- **`[")
                    || trimmed.starts_with("- **[")
                    || trimmed.starts_with("- `[")
                    || trimmed.starts_with("- [")
                    || trimmed.starts_with("* **`[")
                    || trimmed.starts_with("* **[")
                    || trimmed.starts_with("* `[")
                    || trimmed.starts_with("* [")
                    || (trimmed.starts_with('[') && trimmed.contains("]:"))
                {
                    entries += 1;
                }
            }
            if entries > 0 {
                return Some(cfg.active_template.replace("{count}", &entries.to_string()));
            } else {
                return Some(cfg.ready_template.clone());
            }
        }
    }

    if let Ok(read_dir) = std::fs::read_dir(&memory_dir) {
        let mut count = 0;
        for entry in read_dir.flatten() {
            if entry.path().is_file() {
                count += 1;
            }
        }
        if count > 0 {
            return Some(cfg.files_template.replace("{count}", &count.to_string()));
        }
    }

    Some(cfg.ready_template.clone())
}

pub fn probe_project_memory(workspace_root: Option<&Path>) -> Option<String> {
    probe_project_memory_with_config(workspace_root, &MemoryRadarConfig::default())
}
