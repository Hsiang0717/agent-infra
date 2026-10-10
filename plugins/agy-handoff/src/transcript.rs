use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TranscriptStep {
    step_index: Option<u64>,
    #[serde(rename = "type")]
    step_type: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    status: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    truncated_fields: Option<Vec<String>>,
}

pub struct ExtractedResponse {
    pub step_index: u64,
    pub content: String,
}

pub fn get_last_planner_response(
    app_data_dir: &Path,
    session_id: &str,
) -> Result<ExtractedResponse, String> {
    let logs_dir = app_data_dir
        .join("brain")
        .join(session_id)
        .join(".system_generated")
        .join("logs");

    let transcript_path = logs_dir.join("transcript.jsonl");
    if !transcript_path.exists() {
        return Err(format!(
            "transcript.jsonl not found at: {}",
            transcript_path.display()
        ));
    }

    let file = File::open(&transcript_path)
        .map_err(|e| format!("Failed to open {}: {}", transcript_path.display(), e))?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        if let Ok(l) = line {
            if !l.trim().is_empty() {
                lines.push(l);
            }
        }
    }

    // Iterate backwards to find the last PLANNER_RESPONSE with content
    for line in lines.iter().rev() {
        if let Ok(step) = serde_json::from_str::<TranscriptStep>(line) {
            let is_planner_response = step
                .step_type
                .as_deref()
                .map(|t| t == "PLANNER_RESPONSE")
                .unwrap_or(false);

            if !is_planner_response {
                continue;
            }

            let idx = step.step_index.unwrap_or(0);
            let is_content_truncated = step
                .truncated_fields
                .as_ref()
                .map(|fields| fields.iter().any(|f| f == "content"))
                .unwrap_or(false);

            if is_content_truncated {
                // Fetch full content from transcript_full.jsonl
                let full_path = logs_dir.join("transcript_full.jsonl");
                if full_path.exists() {
                    if let Ok(full_content) = fetch_from_full_transcript(&full_path, idx) {
                        if !full_content.trim().is_empty() {
                            return Ok(ExtractedResponse {
                                step_index: idx,
                                content: full_content,
                            });
                        }
                    }
                }
            }

            if let Some(content) = step.content {
                if !content.trim().is_empty() {
                    return Ok(ExtractedResponse {
                        step_index: idx,
                        content,
                    });
                }
            }
        }
    }

    Err(format!(
        "No completed PLANNER_RESPONSE with content found in session {}",
        session_id
    ))
}

fn fetch_from_full_transcript(full_path: &PathBuf, target_step_index: u64) -> Result<String, String> {
    let file = File::open(full_path)
        .map_err(|e| format!("Failed to open {}: {}", full_path.display(), e))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.trim().is_empty() {
                continue;
            }
            if let Ok(step) = serde_json::from_str::<TranscriptStep>(&l) {
                if step.step_index == Some(target_step_index) {
                    if let Some(c) = step.content {
                        return Ok(c);
                    }
                }
            }
        }
    }

    Err(format!(
        "Step index {} not found in {}",
        target_step_index,
        full_path.display()
    ))
}
