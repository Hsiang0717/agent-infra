use std::io::{self, Read};
use std::path::Path;

// ─── PreToolUse CRLF & BOM Auto-Normalizer ───────────────────────────────────

pub fn normalize_file_crlf_if_needed(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("bat") || ext.eq_ignore_ascii_case("cmd") {
            return false;
        }
    }

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return false,
    };

    // Skip binary files (containing null bytes)
    if bytes.contains(&0) {
        return false;
    }

    let has_crlf = bytes.windows(2).any(|w| w == b"\r\n");
    let has_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);

    if !has_crlf && !has_bom {
        return false;
    }

    let text = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let cleaned = text.trim_start_matches('\u{feff}');
    let normalized = cleaned.replace("\r\n", "\n");
    std::fs::write(path, normalized.as_bytes()).is_ok()
}

pub(crate) fn strip_line_prefix_from_line(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let after_l = trimmed.strip_prefix('L').unwrap_or(trimmed);
    let digit_count = after_l.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_count == 0 {
        return None;
    }
    let rest = &after_l[digit_count..];
    if rest.starts_with(": ") {
        Some(&rest[2..])
    } else if rest.starts_with(':') {
        Some(&rest[1..])
    } else if rest.starts_with(" | ") {
        Some(&rest[3..])
    } else if rest.starts_with("| ") {
        Some(&rest[2..])
    } else if rest.starts_with("│ ") {
        Some(&rest["│ ".len()..])
    } else if rest.starts_with(" │ ") {
        Some(&rest[" │ ".len()..])
    } else {
        None
    }
}

pub(crate) fn strip_line_number_prefixes(text: &str) -> (String, bool) {
    let mut modified = false;
    let mut cleaned_lines = Vec::new();
    let has_trailing_newline = text.ends_with('\n');
    let is_crlf = text.contains("\r\n");
    let norm = text.replace("\r\n", "\n");

    for line in norm.lines() {
        if let Some(stripped) = strip_line_prefix_from_line(line) {
            modified = true;
            cleaned_lines.push(stripped);
        } else if line.trim().is_empty() {
            cleaned_lines.push("");
        } else {
            cleaned_lines.push(line);
        }
    }

    let mut result = cleaned_lines.join(if is_crlf { "\r\n" } else { "\n" });
    if has_trailing_newline {
        result.push_str(if is_crlf { "\r\n" } else { "\n" });
    }
    (result, modified)
}

pub(crate) fn find_unique_blank_line_tolerant_match(candidate_target: &str, file_content: &str) -> Option<String> {
    let norm_target = candidate_target.replace("\r\n", "\n");
    let norm_content = file_content.replace("\r\n", "\n");

    let target_lines: Vec<&str> = norm_target.split('\n').collect();
    let target_meaningful: Vec<&str> = target_lines
        .iter()
        .map(|l| l.trim_end())
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Guard: require at least 2 non-blank lines (single-line targets already substring-match before trailing ws)
    if target_meaningful.len() < 2 {
        return None;
    }

    let file_lines: Vec<&str> = norm_content.split('\n').collect();
    let mut matches = Vec::new();

    let n_meaningful = target_meaningful.len();
    for start_idx in 0..file_lines.len() {
        if file_lines[start_idx].trim().is_empty() {
            continue;
        }
        if file_lines[start_idx].trim_end() != target_meaningful[0] {
            continue;
        }

        let mut meaningful_idx = 1;
        let mut end_idx = start_idx;
        if n_meaningful > 1 {
            for curr_idx in (start_idx + 1)..file_lines.len() {
                let line = file_lines[curr_idx];
                if line.trim().is_empty() {
                    continue;
                }
                if line.trim_end() == target_meaningful[meaningful_idx] {
                    meaningful_idx += 1;
                    end_idx = curr_idx;
                    if meaningful_idx == n_meaningful {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        if meaningful_idx == n_meaningful {
            matches.push((start_idx, end_idx));
            if matches.len() > 1 {
                return None;
            }
        }
    }

    if matches.len() == 1 {
        let (start, end) = matches[0];
        let matched_slice = file_lines[start..=end].join("\n");
        if file_content.contains("\r\n") {
            Some(matched_slice.replace('\n', "\r\n"))
        } else {
            Some(matched_slice)
        }
    } else {
        None
    }
}

pub(crate) fn sanitize_edit_target(raw_target: &str, file_content: &str) -> Option<String> {
    if raw_target.is_empty() {
        return None;
    }
    // 1. Verbatim check: if already matches, never touch it
    if file_content.contains(raw_target) {
        return None;
    }
    let norm_target = raw_target.replace("\r\n", "\n");
    let norm_content = file_content.replace("\r\n", "\n");
    if norm_content.contains(&norm_target) {
        return None;
    }

    // 2. Strip line prefixes if present
    let (cleaned, modified) = strip_line_number_prefixes(raw_target);
    let candidate = if modified {
        let norm_cleaned = cleaned.replace("\r\n", "\n");
        if norm_content.contains(&norm_cleaned) {
            if file_content.contains("\r\n") && !cleaned.contains("\r\n") {
                return Some(cleaned.replace('\n', "\r\n"));
            } else {
                return Some(cleaned);
            }
        }
        cleaned
    } else {
        raw_target.to_string()
    };

    // 3. Blank-line & whitespace-tolerant alignment
    if let Some(aligned) = find_unique_blank_line_tolerant_match(&candidate, file_content) {
        return Some(aligned);
    }

    None
}

pub(crate) fn sanitize_replacement_if_dirty(raw_replacement: &str) -> Option<String> {
    if raw_replacement.is_empty() {
        return None;
    }
    let (cleaned, modified) = strip_line_number_prefixes(raw_replacement);
    if modified {
        Some(cleaned)
    } else {
        None
    }
}

pub(crate) fn handle_pre_edit_hook(explicit_file: Option<&str>) {
    if let Some(f) = explicit_file {
        normalize_file_crlf_if_needed(Path::new(f));
        println!("{{\"decision\":\"allow\"}}");
        return;
    }

    let mut raw_stdin = String::new();
    if io::stdin().read_to_string(&mut raw_stdin).is_ok() {
        let cleaned = raw_stdin.trim().trim_start_matches('\u{feff}');
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(cleaned) {
            let workspace_root = val
                .get("workspacePaths")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str());

            if let Some(args) = val.get("toolCall").and_then(|tc| tc.get("args")) {
                let path_opt = args
                    .get("Path")
                    .or_else(|| args.get("TargetFile"))
                    .or_else(|| args.get("path"))
                    .and_then(|v| v.as_str());
                if let Some(p) = path_opt {
                    let target_path = Path::new(p);
                    let full_path = if target_path.is_absolute() {
                        target_path.to_path_buf()
                    } else if let Some(ws) = workspace_root {
                        Path::new(ws).join(target_path)
                    } else {
                        target_path.to_path_buf()
                    };
                    normalize_file_crlf_if_needed(&full_path);

                    if let Ok(content) = std::fs::read_to_string(&full_path) {
                        let target_opt = args.get("Target").and_then(|v| v.as_str());
                        let replacement_opt = args.get("Replacement").and_then(|v| v.as_str());

                        if let Some(target) = target_opt {
                            if let Some(cleaned_target) = sanitize_edit_target(target, &content) {
                                let mut overwrite = serde_json::Map::new();
                                overwrite.insert("Target".to_string(), serde_json::Value::String(cleaned_target));

                                if let Some(replacement) = replacement_opt {
                                    if let Some(cleaned_rep) = sanitize_replacement_if_dirty(replacement) {
                                        overwrite.insert("Replacement".to_string(), serde_json::Value::String(cleaned_rep));
                                    }
                                }

                                let resp = serde_json::json!({
                                    "decision": "allow",
                                    "overwrite": overwrite
                                });
                                println!("{}", resp);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{{\"decision\":\"allow\"}}");
}
