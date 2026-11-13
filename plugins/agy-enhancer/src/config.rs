use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// Default context window limit: 200k tokens.
pub const DEFAULT_CONTEXT_LIMIT_TOKENS: u64 = 200_000;
pub const DEFAULT_TIGHT_THRESHOLD: f64 = 0.60;
pub const DEFAULT_CRITICAL_THRESHOLD: f64 = 0.85;

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

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct PartialThresholdConfig {
    #[serde(default)]
    pub tight: Option<f64>,
    #[serde(default)]
    pub critical: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct PartialMessagesConfig {
    #[serde(default)]
    pub nominal: Option<String>,
    #[serde(default)]
    pub tight: Option<String>,
    #[serde(default)]
    pub critical: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModelProfileConfig {
    pub match_pattern: String,
    #[serde(default)]
    pub context_limit: Option<u64>,
    #[serde(default)]
    pub thresholds: Option<PartialThresholdConfig>,
    #[serde(default)]
    pub messages: Option<PartialMessagesConfig>,
    #[serde(default)]
    pub slice_invariants_enabled: Option<bool>,
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
    #[serde(default)]
    pub model_profiles: Vec<ModelProfileConfig>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            context_limit: default_context_limit(),
            thresholds: ThresholdConfig::default(),
            messages: MessagesConfig::default(),
            memory_radar: MemoryRadarConfig::default(),
            slice_invariants: SliceInvariantsConfig::default(),
            model_profiles: Vec::new(),
        }
    }
}

impl MonitorConfig {
    /// Returns a config copy with the first matching `model_profiles` entry applied.
    pub fn resolve_for_model(&self, model_name: Option<&str>) -> Self {
        let Some(raw_name) = model_name else {
            return self.clone();
        };
        let needle_target = raw_name.trim().to_lowercase();
        if needle_target.is_empty() {
            return self.clone();
        }

        let Some(profile) = self.model_profiles.iter().find(|p| {
            let pat = p.match_pattern.trim().to_lowercase();
            !pat.is_empty() && needle_target.contains(&pat)
        }) else {
            return self.clone();
        };

        let mut resolved = self.clone();
        if let Some(limit) = profile.context_limit {
            resolved.context_limit = limit;
        }
        if let Some(ref th) = profile.thresholds {
            if let Some(tight) = th.tight {
                resolved.thresholds.tight = tight;
            }
            if let Some(critical) = th.critical {
                resolved.thresholds.critical = critical;
            }
        }
        if let Some(ref msgs) = profile.messages {
            if let Some(ref nominal) = msgs.nominal {
                resolved.messages.nominal = nominal.clone();
            }
            if let Some(ref tight) = msgs.tight {
                resolved.messages.tight = tight.clone();
            }
            if let Some(ref critical) = msgs.critical {
                resolved.messages.critical = critical.clone();
            }
        }
        if let Some(enabled) = profile.slice_invariants_enabled {
            resolved.slice_invariants.enabled = enabled;
        }
        resolved
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
    String::new()
}

fn default_tight_message() -> String {
    "[Context Posture: TIGHT] Avoid blind full-file dumps (use outline/rg; offload broad cross-file audits to research subagent). If a fix failed or spans modules, do NOT repeat local tweaks—trace paired channels & align Where/Who/When first.".to_string()
}

fn default_critical_message() -> String {
    "[Context Posture: CRITICAL] Working memory near ceiling. Offload remaining discovery, log forensics, or isolated subtasks to research/self subagents; keep primary reads strictly rg-anchored, suppress verbose CLI output, and converge directly on verified completion.".to_string()
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
