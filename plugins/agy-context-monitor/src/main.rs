use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Default context window limit: 256k tokens.
pub const DEFAULT_CONTEXT_LIMIT_TOKENS: u64 = 256_000;
pub const DEFAULT_TIGHT_THRESHOLD: f64 = 0.60;
pub const DEFAULT_CRITICAL_THRESHOLD: f64 = 0.90;

/// Chunk size for backward scanning in transcript (64 KB).
const CHUNK_SIZE: usize = 64 * 1024;

// ─── Configuration Structures ───────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MonitorConfig {
    #[serde(default = "default_context_limit")]
    pub context_limit: u64,
    #[serde(default)]
    pub thresholds: ThresholdConfig,
    #[serde(default)]
    pub messages: MessagesConfig,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            context_limit: default_context_limit(),
            thresholds: ThresholdConfig::default(),
            messages: MessagesConfig::default(),
        }
    }
}

fn default_context_limit() -> u64 {
    DEFAULT_CONTEXT_LIMIT_TOKENS
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThresholdConfig {
    #[serde(default = "default_tight_threshold")]
    pub tight: f64,
    #[serde(default = "default_critical_threshold")]
    pub critical: f64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            tight: DEFAULT_TIGHT_THRESHOLD,
            critical: DEFAULT_CRITICAL_THRESHOLD,
        }
    }
}

fn default_tight_threshold() -> f64 {
    DEFAULT_TIGHT_THRESHOLD
}

fn default_critical_threshold() -> f64 {
    DEFAULT_CRITICAL_THRESHOLD
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MessagesConfig {
    #[serde(default = "default_nominal_message")]
    pub nominal: String,
    #[serde(default = "default_tight_message")]
    pub tight: String,
    #[serde(default = "default_critical_message")]
    pub critical: String,
}

impl Default for MessagesConfig {
    fn default() -> Self {
        Self {
            nominal: default_nominal_message(),
            tight: default_tight_message(),
            critical: default_critical_message(),
        }
    }
}

fn default_nominal_message() -> String {
    "[Context Posture: NOMINAL] Working memory is abundant. Proceed with full surgical precision and end-to-end autonomy.".to_string()
}

fn default_tight_message() -> String {
    "[Context Posture: TIGHT] Working memory is tightening. Constrain read slices and offload expansive exploration to the research subagent.".to_string()
}

fn default_critical_message() -> String {
    "[Context Posture: CRITICAL] Working memory near ceiling. Cease broad exploration, converge on active atomic changes, and prepare /handoff.".to_string()
}

// ─── Config Discovery & Loading ─────────────────────────────────────────────

pub fn load_config(workspace_root: Option<&Path>) -> (MonitorConfig, Option<PathBuf>) {
    let mut candidates = Vec::new();

    // 1. Workspace override (.agents/context-monitor.json or plugins/agy-context-monitor/config.json)
    if let Some(ws) = workspace_root {
        candidates.push(ws.join(".agents").join("context-monitor.json"));
        candidates.push(ws.join(".agents").join("context-monitor").join("config.json"));
        candidates.push(ws.join("plugins").join("agy-context-monitor").join("config.json"));
        candidates.push(ws.join("context-monitor.json"));
    }

    // 2. Local plugin directory (adjacent to binary or in plugin root)
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("config.json"));
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join("config.json"));
            }
        }
    }

    // 3. Global user configuration (~/.gemini/config/plugins/agy-context-monitor/config.json)
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        candidates.push(
            PathBuf::from(home)
                .join(".gemini")
                .join("config")
                .join("plugins")
                .join("agy-context-monitor")
                .join("config.json"),
        );
    }

    for path in candidates {
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let cleaned = content.trim().trim_start_matches('\u{feff}');
                if let Ok(cfg) = serde_json::from_str::<MonitorConfig>(cleaned) {
                    return (cfg, Some(path));
                }
            }
        }
    }

    (MonitorConfig::default(), None)
}

// ─── Data Structures ─────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Deserialize, Debug, Default)]
struct HookInput {
    #[serde(rename = "conversationId")]
    conversation_id: Option<String>,
    #[serde(rename = "transcriptPath")]
    transcript_path: Option<String>,
    #[serde(rename = "modelName")]
    model_name: Option<String>,
    #[serde(rename = "workspacePaths")]
    workspace_paths: Option<Vec<String>>,
    #[serde(rename = "invocationNum")]
    invocation_num: Option<u64>,
}

#[derive(Serialize, Debug)]
struct HookOutput {
    #[serde(rename = "injectSteps")]
    inject_steps: Vec<InjectStep>,
}

#[derive(Serialize, Debug)]
struct InjectStep {
    #[serde(rename = "ephemeralMessage")]
    ephemeral_message: String,
}

#[derive(Deserialize, Debug)]
struct TranscriptStep {
    #[serde(rename = "type")]
    step_type: Option<String>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
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

fn strip_line_prefix_from_line(line: &str) -> Option<&str> {
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

fn strip_line_number_prefixes(text: &str) -> (String, bool) {
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

fn sanitize_edit_target(raw_target: &str, file_content: &str) -> Option<String> {
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

    // 2. Strip line prefixes
    let (cleaned, modified) = strip_line_number_prefixes(raw_target);
    if !modified {
        return None;
    }

    // 3. Double check: does cleaned target actually match the file?
    let norm_cleaned = cleaned.replace("\r\n", "\n");
    if norm_content.contains(&norm_cleaned) {
        if file_content.contains("\r\n") && !cleaned.contains("\r\n") {
            Some(cleaned.replace('\n', "\r\n"))
        } else {
            Some(cleaned)
        }
    } else {
        None
    }
}

fn sanitize_replacement_if_dirty(raw_replacement: &str) -> Option<String> {
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

fn handle_pre_edit_hook(explicit_file: Option<&str>) {
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

// ─── Main Entry Point ────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let cmd = &args[1];
        match cmd.as_str() {
            "--pre-edit" => {
                handle_pre_edit_hook(args.get(2).map(|s| s.as_str()));
                return;
            }
            "--help" | "-h" => {
                println!("agy-context-monitor v0.1.0 (Native Context & PreToolUse Hook)");
                println!("Usage:");
                println!("  agy-context-monitor                   Run as PreInvocation hook (reads stdin)");
                println!("  agy-context-monitor --pre-edit        Run as PreToolUse edit_file CRLF normalizer");
                println!("  agy-context-monitor --config          Print currently resolved configuration");
                println!("  agy-context-monitor --probe <file>    Probe token counts from transcript file");
                println!("  agy-context-monitor --simulate <num>  Simulate semantic message for token count");
                println!("  agy-context-monitor --version         Show version");
                return;
            }
            "--version" | "-v" => {
                println!("agy-context-monitor 0.1.0");
                return;
            }
            "--config" => {
                let (cfg, path) = load_config(None);
                println!("Resolved Config Source: {}", path.map(|p| p.display().to_string()).unwrap_or_else(|| "Built-in Defaults".to_string()));
                if let Ok(json) = serde_json::to_string_pretty(&cfg) {
                    println!("{}", json);
                }
                return;
            }
            "--probe" => {
                if let Some(path_str) = args.get(2) {
                    let p = Path::new(path_str);
                    let (cfg, cfg_path) = load_config(None);
                    let (used, output) = probe_transcript_tokens(p);
                    let msg = build_semantic_message(used, &cfg);
                    println!("Config Source   : {}", cfg_path.map(|p| p.display().to_string()).unwrap_or_else(|| "Defaults".to_string()));
                    println!("Transcript Path : {}", path_str);
                    println!("Total Used      : {} tokens ({})", used, human_format(used));
                    println!("Last Output     : {} tokens ({})", output, human_format(output));
                    println!("Limit           : {} tokens ({})", cfg.context_limit, human_format(cfg.context_limit));
                    println!("Message         : {}", msg);
                } else {
                    eprintln!("Error: missing transcript path for --probe");
                }
                return;
            }
            "--simulate" => {
                if let Some(tokens_str) = args.get(2) {
                    let tokens: u64 = tokens_str.parse().unwrap_or(0);
                    let (cfg, _) = load_config(None);
                    let msg = build_semantic_message(tokens, &cfg);
                    println!("Simulated Tokens: {}", tokens);
                    println!("Message: {}", msg);
                } else {
                    eprintln!("Error: missing token count for --simulate");
                }
                return;
            }
            _ => {}
        }
    }

    // Standard Hook Execution via Stdin
    let mut raw_stdin = String::new();
    if io::stdin().read_to_string(&mut raw_stdin).is_err() {
        emit_empty_response();
        return;
    }

    let cleaned = raw_stdin.trim().trim_start_matches('\u{feff}');
    if cleaned.is_empty() {
        emit_empty_response();
        return;
    }

    let input: HookInput = match serde_json::from_str(cleaned) {
        Ok(inp) => inp,
        Err(_) => {
            emit_empty_response();
            return;
        }
    };

    let transcript_path = match input.transcript_path.as_deref() {
        Some(p) if !p.is_empty() => Path::new(p),
        _ => {
            emit_empty_response();
            return;
        }
    };

    let ws_root = input
        .workspace_paths
        .as_ref()
        .and_then(|v| v.first())
        .map(Path::new);

    let (cfg, _) = load_config(ws_root);
    let (used_tokens, _) = probe_transcript_tokens(transcript_path);
    let message = build_semantic_message(used_tokens, &cfg);

    let output = HookOutput {
        inject_steps: vec![InjectStep {
            ephemeral_message: message,
        }],
    };

    if let Ok(json_str) = serde_json::to_string(&output) {
        println!("{}", json_str);
    } else {
        emit_empty_response();
    }
}

fn emit_empty_response() {
    println!("{{\"injectSteps\":[]}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_format() {
        assert_eq!(human_format(500), "500");
        assert_eq!(human_format(1_500), "1.5K");
        assert_eq!(human_format(50_629), "50.6K");
        assert_eq!(human_format(256_000), "256.0K");
        assert_eq!(human_format(1_048_576), "1.0M");
    }

    #[test]
    fn test_tier_classification_defaults() {
        let thresholds = ThresholdConfig::default();
        assert_eq!(classify_tier(0.0, &thresholds), ContextTier::Nominal);
        assert_eq!(classify_tier(59.9, &thresholds), ContextTier::Nominal);
        assert_eq!(classify_tier(60.0, &thresholds), ContextTier::Tight);
        assert_eq!(classify_tier(89.9, &thresholds), ContextTier::Tight);
        assert_eq!(classify_tier(90.0, &thresholds), ContextTier::Critical);
        assert_eq!(classify_tier(100.0, &thresholds), ContextTier::Critical);
    }

    #[test]
    fn test_tier_classification_custom_thresholds() {
        let thresholds = ThresholdConfig {
            tight: 0.50,
            critical: 0.80,
        };
        assert_eq!(classify_tier(49.0, &thresholds), ContextTier::Nominal);
        assert_eq!(classify_tier(50.0, &thresholds), ContextTier::Tight);
        assert_eq!(classify_tier(79.0, &thresholds), ContextTier::Tight);
        assert_eq!(classify_tier(80.0, &thresholds), ContextTier::Critical);
    }

    #[test]
    fn test_template_interpolation() {
        let template = "[Context: {tier}] {pct} ({used}/{limit}) - {remain} left";
        let res = interpolate_template(template, ContextTier::Nominal, 20.0, 51_200, 256_000);
        assert_eq!(res, "[Context: NOMINAL] 20.0% (51.2K/256.0K) - 204.8K left");
    }

    #[test]
    fn test_semantic_message_content() {
        let cfg = MonitorConfig::default();
        let nom = build_semantic_message(25_600, &cfg);
        assert!(nom.contains("NOMINAL"));
        assert!(!nom.contains("%"));

        let tight = build_semantic_message(179_200, &cfg);
        assert!(tight.contains("TIGHT"));
        assert!(!tight.contains("%"));

        let crit = build_semantic_message(240_000, &cfg);
        assert!(crit.contains("CRITICAL"));
        assert!(!crit.contains("%"));
    }

    #[test]
    fn test_probe_transcript_tokens() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("test_transcript_{}.jsonl", std::process::id()));

        let sample_lines = r#"{"type":"USER_INPUT","content":"hello"}
{"type":"PLANNER_RESPONSE","input_tokens":1200,"output_tokens":150,"cache_read_tokens":5000}
{"type":"TOOL_CALL","name":"run_command"}
{"type":"PLANNER_RESPONSE","input_tokens":3000,"output_tokens":200,"cache_read_tokens":20000}
{"type":"TOOL_RESULT","content":"success"}
"#;

        {
            let mut f = File::create(&test_file).unwrap();
            f.write_all(sample_lines.as_bytes()).unwrap();
        }

        let (used, output) = probe_transcript_tokens(&test_file);
        let _ = std::fs::remove_file(&test_file);

        assert_eq!(used, 23_000);
        assert_eq!(output, 200);
    }

    #[test]
    fn test_normalize_file_crlf() {
        let temp_dir = std::env::temp_dir();
        let toml_file = temp_dir.join(format!("test_crlf_{}.toml", std::process::id()));
        let bat_file = temp_dir.join(format!("test_crlf_{}.bat", std::process::id()));
        let bom_file = temp_dir.join(format!("test_bom_{}.txt", std::process::id()));

        std::fs::write(&toml_file, "a = 1\r\nb = 2\r\n").unwrap();
        std::fs::write(&bat_file, "@echo off\r\necho hi\r\n").unwrap();
        // UTF-8 BOM + CRLF
        let mut bom_bytes = vec![0xEF, 0xBB, 0xBF];
        bom_bytes.extend_from_slice("hello\r\nworld\r\n".as_bytes());
        std::fs::write(&bom_file, &bom_bytes).unwrap();

        assert!(normalize_file_crlf_if_needed(&toml_file));
        assert_eq!(std::fs::read_to_string(&toml_file).unwrap(), "a = 1\nb = 2\n");
        // Second call returns false when already LF
        assert!(!normalize_file_crlf_if_needed(&toml_file));

        // .bat files must retain CRLF
        assert!(!normalize_file_crlf_if_needed(&bat_file));
        assert_eq!(std::fs::read_to_string(&bat_file).unwrap(), "@echo off\r\necho hi\r\n");

        // BOM file must be stripped of BOM and normalized to LF
        assert!(normalize_file_crlf_if_needed(&bom_file));
        let bom_content = std::fs::read(&bom_file).unwrap();
        assert!(!bom_content.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert_eq!(std::str::from_utf8(&bom_content).unwrap(), "hello\nworld\n");

        let _ = std::fs::remove_file(&toml_file);
        let _ = std::fs::remove_file(&bat_file);
        let _ = std::fs::remove_file(&bom_file);
    }

    #[test]
    fn test_strip_line_prefix_patterns() {
        // read_file style
        assert_eq!(strip_line_prefix_from_line("30:     let cleaned = text;"), Some("    let cleaned = text;"));
        assert_eq!(strip_line_prefix_from_line("1: # Title"), Some("# Title"));
        assert_eq!(strip_line_prefix_from_line("415: fn handle_pre_edit_hook() {"), Some("fn handle_pre_edit_hook() {"));
        assert_eq!(strip_line_prefix_from_line("31:"), Some(""));
        assert_eq!(strip_line_prefix_from_line("31: "), Some(""));

        // rg -n style
        assert_eq!(strip_line_prefix_from_line("30:let cleaned = text;"), Some("let cleaned = text;"));

        // pipe / outline styles
        assert_eq!(strip_line_prefix_from_line("30 |     let cleaned = text;"), Some("    let cleaned = text;"));
        assert_eq!(strip_line_prefix_from_line("30 │     let cleaned = text;"), Some("    let cleaned = text;"));
        assert_eq!(strip_line_prefix_from_line("L30:     let cleaned = text;"), Some("    let cleaned = text;"));

        // Non-line-number text should return None
        assert_eq!(strip_line_prefix_from_line("let x = 1;"), None);
        assert_eq!(strip_line_prefix_from_line("404 Not Found"), None);
        assert_eq!(strip_line_prefix_from_line("HTTP/1.1 200 OK"), None);
    }

    #[test]
    fn test_strip_line_number_prefixes_multiline() {
        let dirty = "30:     let x = 1;\n31: \n32:     let y = 2;\n";
        let (cleaned, modified) = strip_line_number_prefixes(dirty);
        assert!(modified);
        assert_eq!(cleaned, "    let x = 1;\n\n    let y = 2;\n");
    }

    #[test]
    fn test_sanitize_edit_target_verbatim_untouched() {
        let file_content = "10: this is actual line number in file\n20: next line\n";
        let target = "10: this is actual line number in file";
        // Since verbatim matches, it MUST NOT alter target
        assert_eq!(sanitize_edit_target(target, file_content), None);
    }

    #[test]
    fn test_sanitize_edit_target_fixes_read_file_artifacts() {
        let file_content = "fn compute() {\n    let a = 10;\n    let b = 20;\n    a + b\n}\n";
        let dirty_target = "2:     let a = 10;\n3:     let b = 20;";
        let sanitized = sanitize_edit_target(dirty_target, file_content);
        assert_eq!(sanitized, Some("    let a = 10;\n    let b = 20;".to_string()));
    }

    #[test]
    fn test_sanitize_edit_target_unmatched_returns_none() {
        let file_content = "fn compute() {\n    let a = 10;\n}\n";
        let dirty_target = "2:     let totally_different = 999;";
        assert_eq!(sanitize_edit_target(dirty_target, file_content), None);
    }

    #[test]
    fn test_sanitize_replacement_handling() {
        let dirty_rep = "2:     let a = 100;\n3:     let b = 200;";
        assert_eq!(
            sanitize_replacement_if_dirty(dirty_rep),
            Some("    let a = 100;\n    let b = 200;".to_string())
        );

        let clean_rep = "    let a = 100;\n    let b = 200;";
        assert_eq!(sanitize_replacement_if_dirty(clean_rep), None);
    }
}
