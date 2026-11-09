use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// Default context window limit: 256k tokens.
pub const DEFAULT_CONTEXT_LIMIT_TOKENS: u64 = 256_000;
pub const DEFAULT_TIGHT_THRESHOLD: f64 = 0.60;
pub const DEFAULT_CRITICAL_THRESHOLD: f64 = 0.90;

// ─── Configuration Structures ───────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MemoryRadarConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_memory_active_template")]
    pub active_template: String,
    #[serde(default = "default_memory_files_template")]
    pub files_template: String,
    #[serde(default = "default_memory_ready_template")]
    pub ready_template: String,
}

impl Default for MemoryRadarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            active_template: default_memory_active_template(),
            files_template: default_memory_files_template(),
            ready_template: default_memory_ready_template(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_memory_active_template() -> String {
    "[Project Memory: active (.agents/memory/pitfalls.md - {count} entries; query via rg)]".to_string()
}

fn default_memory_files_template() -> String {
    "[Project Memory: active (.agents/memory/ - {count} files; query via rg)]".to_string()
}

fn default_memory_ready_template() -> String {
    "[Project Memory: ready (.agents/memory/pitfalls.md; distill non-obvious traps)]".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SliceInvariantsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_slice_invariants_template")]
    pub template: String,
}

impl Default for SliceInvariantsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            template: default_slice_invariants_template(),
        }
    }
}

pub fn default_slice_invariants_template() -> String {
    "[Target Invariant: {details} — match verbatim in edit_file]".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MonitorConfig {
    #[serde(default = "default_context_limit")]
    pub context_limit: u64,
    #[serde(default)]
    pub thresholds: ThresholdConfig,
    #[serde(default)]
    pub messages: MessagesConfig,
    #[serde(default)]
    pub memory_radar: MemoryRadarConfig,
    #[serde(default)]
    pub slice_invariants: SliceInvariantsConfig,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            context_limit: default_context_limit(),
            thresholds: ThresholdConfig::default(),
            messages: MessagesConfig::default(),
            memory_radar: MemoryRadarConfig::default(),
            slice_invariants: SliceInvariantsConfig::default(),
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
    "[Context Posture: NOMINAL] Working memory is abundant. Proceed with topological clarity, surgical precision, and end-to-end autonomy.".to_string()
}

fn default_tight_message() -> String {
    "[Context Posture: TIGHT] Working memory is tightening. Avoid blind full-file dumps (use outline/rg; offload broad cross-file audits to research subagent). If a fix failed or spans modules, do NOT repeat local tweaks—trace paired channels & align Where/Who/When first.".to_string()
}

fn default_critical_message() -> String {
    "[Context Posture: CRITICAL] Working memory near ceiling. Cease broad exploration, converge on active atomic changes, and prepare /handoff.".to_string()
}

// ─── Config Discovery & Loading ─────────────────────────────────────────────

pub fn load_config(workspace_root: Option<&Path>) -> (MonitorConfig, Option<PathBuf>) {
    let mut candidates = Vec::new();

    // 1. Workspace override (.agents/enhancer.json, .agents/context-monitor.json, etc.)
    if let Some(ws) = workspace_root {
        candidates.push(ws.join(".agents").join("enhancer.json"));
        candidates.push(ws.join(".agents").join("enhancer").join("config.json"));
        candidates.push(ws.join(".agents").join("context-monitor.json"));
        candidates.push(ws.join(".agents").join("context-monitor").join("config.json"));
        candidates.push(ws.join("plugins").join("agy-enhancer").join("config.json"));
        candidates.push(ws.join("plugins").join("agy-context-monitor").join("config.json"));
        candidates.push(ws.join("enhancer.json"));
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

    // 3. Global user configuration (~/.gemini/config/plugins/agy-enhancer/config.json, fallback to agy-context-monitor)
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        let global_base = PathBuf::from(home).join(".gemini").join("config").join("plugins");
        candidates.push(global_base.join("agy-enhancer").join("config.json"));
        candidates.push(global_base.join("agy-context-monitor").join("config.json"));
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
