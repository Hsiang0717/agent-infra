use crate::config::{MonitorConfig, ThresholdConfig, DEFAULT_CONTEXT_LIMIT_TOKENS};
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Chunk size for backward scanning in transcript (64 KB).
pub(crate) const CHUNK_SIZE: usize = 64 * 1024;

#[derive(Deserialize, Debug)]
pub(crate) struct TranscriptStep {
    #[serde(rename = "type")]
    pub(crate) step_type: Option<String>,
    pub(crate) input_tokens: Option<u64>,
    pub(crate) output_tokens: Option<u64>,
    pub(crate) cache_read_tokens: Option<u64>,
    pub(crate) content: Option<serde_json::Value>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContextTier {
    Nominal,
    Tight,
    Critical,
}

impl ContextTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContextTier::Nominal => "NOMINAL",
            ContextTier::Tight => "TIGHT",
            ContextTier::Critical => "CRITICAL",
        }
    }
}

// ─── Tier Classification & Message Formatting ───────────────────────────────

pub fn normalize_threshold(val: f64) -> f64 {
    if val >= 1.0 {
        val / 100.0
    } else {
        val
    }
}

pub fn classify_tier(used_percentage: f64, thresholds: &ThresholdConfig) -> ContextTier {
    let tight_pct = normalize_threshold(thresholds.tight) * 100.0;
    let crit_pct = normalize_threshold(thresholds.critical) * 100.0;

    if used_percentage >= crit_pct {
        ContextTier::Critical
    } else if used_percentage >= tight_pct {
        ContextTier::Tight
    } else {
        ContextTier::Nominal
    }
}

pub fn human_format(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

pub fn interpolate_template(
    template: &str,
    tier: ContextTier,
    used_pct: f64,
    used_tokens: u64,
    limit_tokens: u64,
) -> String {
    let remain_tokens = limit_tokens.saturating_sub(used_tokens);
    let mut result = template.to_string();

    if result.contains("{tier}") {
        result = result.replace("{tier}", tier.as_str());
    }
    if result.contains("{pct}") {
        result = result.replace("{pct}", &format!("{:.1}%", used_pct));
    }
    if result.contains("{used}") {
        result = result.replace("{used}", &human_format(used_tokens));
    }
    if result.contains("{limit}") {
        result = result.replace("{limit}", &human_format(limit_tokens));
    }
    if result.contains("{remain}") {
        result = result.replace("{remain}", &human_format(remain_tokens));
    }
    if result.contains("{used_raw}") {
        result = result.replace("{used_raw}", &used_tokens.to_string());
    }
    if result.contains("{limit_raw}") {
        result = result.replace("{limit_raw}", &limit_tokens.to_string());
    }
    if result.contains("{remain_raw}") {
        result = result.replace("{remain_raw}", &remain_tokens.to_string());
    }

    result
}

pub fn build_semantic_message(used_tokens: u64, config: &MonitorConfig) -> String {
    let limit = if config.context_limit == 0 {
        DEFAULT_CONTEXT_LIMIT_TOKENS
    } else {
        config.context_limit
    };

    let used_pct = ((used_tokens as f64 / limit as f64) * 100.0).clamp(0.0, 100.0);
    let tier = classify_tier(used_pct, &config.thresholds);

    let template = match tier {
        ContextTier::Nominal => &config.messages.nominal,
        ContextTier::Tight => &config.messages.tight,
        ContextTier::Critical => &config.messages.critical,
    };

    interpolate_template(template, tier, used_pct, used_tokens, limit)
}

// ─── Backward Transcript Probe ──────────────────────────────────────────────

/// Scans the transcript file backwards in chunks to find the latest PLANNER_RESPONSE
/// containing input_tokens and optional cache_read_tokens.
/// Returns (total_context_tokens, output_tokens).
pub fn probe_transcript_tokens(path: &Path) -> (u64, u64) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return (0, 0),
    };

    let file_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return (0, 0),
    };

    if file_len == 0 {
        return (0, 0);
    }

    let mut pos = file_len;
    let mut leftover = Vec::new();

    while pos > 0 {
        let chunk_start = pos.saturating_sub(CHUNK_SIZE as u64);
        let read_len = (pos - chunk_start) as usize;

        if file.seek(SeekFrom::Start(chunk_start)).is_err() {
            break;
        }

        let mut buf = vec![0u8; read_len];
        if file.read_exact(&mut buf).is_err() {
            break;
        }

        buf.extend_from_slice(&leftover);
        leftover.clear();

        let mut lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();

        if chunk_start > 0 && !lines.is_empty() {
            leftover = lines.remove(0).to_vec();
        }

        for line in lines.into_iter().rev() {
            if line.is_empty() || line.iter().all(|b| b.is_ascii_whitespace()) {
                continue;
            }

            if let Ok(line_str) = std::str::from_utf8(line) {
                if let Ok(step) = serde_json::from_str::<TranscriptStep>(line_str.trim()) {
                    if step.step_type.as_deref() == Some("PLANNER_RESPONSE") {
                        if let Some(input) = step.input_tokens {
                            let cached = step.cache_read_tokens.unwrap_or(0);
                            let output = step.output_tokens.unwrap_or(0);
                            return (input + cached, output);
                        }
                    }
                }
            }
        }

        pos = chunk_start;
    }

    if !leftover.is_empty() {
        if let Ok(line_str) = std::str::from_utf8(&leftover) {
            if let Ok(step) = serde_json::from_str::<TranscriptStep>(line_str.trim()) {
                if step.step_type.as_deref() == Some("PLANNER_RESPONSE") {
                    if let Some(input) = step.input_tokens {
                        let cached = step.cache_read_tokens.unwrap_or(0);
                        let output = step.output_tokens.unwrap_or(0);
                        return (input + cached, output);
                    }
                }
            }
        }
    }

    (0, 0)
}
