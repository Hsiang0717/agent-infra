use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HistoryEntry {
    #[serde(rename = "conversationId")]
    conversation_id: Option<String>,
    workspace: Option<String>,
    #[allow(dead_code)]
    timestamp: Option<u64>,
}

pub struct SessionResolution {
    pub session_id: String,
    pub source: &'static str,
    pub app_data_dir: PathBuf,
}

pub fn get_app_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ANTIGRAVITY_APP_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir.trim());
        }
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(user_profile).join(".gemini").join("antigravity-cli");
        if p.exists() {
            return p;
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".gemini").join("antigravity-cli");
        if p.exists() {
            return p;
        }
    }

    PathBuf::from(".gemini/antigravity-cli")
}

pub fn resolve_session_id(explicit: Option<String>) -> Result<SessionResolution, String> {
    let app_data_dir = get_app_data_dir();

    // 1. Explicit argument
    if let Some(id) = explicit {
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(SessionResolution {
                session_id: trimmed,
                source: "CLI argument (--session)",
                app_data_dir,
            });
        }
    }

    // 2. ANTIGRAVITY_CONVERSATION_ID environment variable
    if let Ok(env_id) = std::env::var("ANTIGRAVITY_CONVERSATION_ID") {
        let trimmed = env_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(SessionResolution {
                session_id: trimmed,
                source: "Environment (ANTIGRAVITY_CONVERSATION_ID)",
                app_data_dir,
            });
        }
    }

    // 3. history.jsonl lookup
    let history_path = app_data_dir.join("history.jsonl");
    if history_path.exists() {
        if let Ok(entries) = read_history_entries(&history_path) {
            // First pass: match current workspace
            if let Ok(cwd) = std::env::current_dir() {
                let cwd_str = normalize_path(&cwd);
                for entry in entries.iter().rev() {
                    if let (Some(ws), Some(cid)) = (&entry.workspace, &entry.conversation_id) {
                        if !cid.trim().is_empty() && normalize_path(Path::new(ws)) == cwd_str {
                            return Ok(SessionResolution {
                                session_id: cid.trim().to_string(),
                                source: "history.jsonl (workspace match)",
                                app_data_dir,
                            });
                        }
                    }
                }
            }

            // Second pass: latest entry regardless of workspace
            for entry in entries.iter().rev() {
                if let Some(cid) = &entry.conversation_id {
                    let trimmed = cid.trim().to_string();
                    if !trimmed.is_empty() {
                        return Ok(SessionResolution {
                            session_id: trimmed,
                            source: "history.jsonl (most recent)",
                            app_data_dir,
                        });
                    }
                }
            }
        }
    }

    // 4. Fallback: scan brain/ directory for newest transcript.jsonl
    let brain_dir = app_data_dir.join("brain");
    if brain_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&brain_dir) {
            let mut candidates: Vec<(String, std::time::SystemTime)> = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        let transcript = path.join(".system_generated").join("logs").join("transcript.jsonl");
                        let mtime = if transcript.exists() {
                            transcript.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH)
                        } else {
                            entry.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH)
                        };
                        candidates.push((file_name.to_string(), mtime));
                    }
                }
            }

            candidates.sort_by(|a, b| b.1.cmp(&a.1));
            if let Some((newest_id, _)) = candidates.first() {
                return Ok(SessionResolution {
                    session_id: newest_id.clone(),
                    source: "brain/ filesystem (latest mtime)",
                    app_data_dir,
                });
            }
        }
    }

    Err("Could not locate any active session ID across CLI args, env, history.jsonl, or brain/ directory.".to_string())
}

fn read_history_entries(path: &Path) -> std::io::Result<Vec<HistoryEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&l) {
                entries.push(entry);
            }
        }
    }

    Ok(entries)
}

fn normalize_path(p: &Path) -> String {
    let s = p.to_string_lossy().to_string();
    s.replace('/', "\\").trim_end_matches('\\').to_lowercase()
}
