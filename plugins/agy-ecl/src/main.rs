use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Chunk size for backward scanning in transcript (64 KB).
const CHUNK_SIZE: usize = 64 * 1024;

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
    tool_calls: Option<serde_json::Value>,
}

// ─── External Cognitive Layer (ECL) Auto-Shield ──────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct EclNode {
    pub raw_coord: String,
    pub rel_path: String,
    pub symbol: Option<String>,
    pub effect: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct EclCluster {
    pub id: String,
    pub summary: Option<String>,
    pub nodes: Vec<EclNode>,
    pub rules: Vec<String>,
}

fn unquote_trimmed(s: &str) -> String {
    let mut t = s.trim();
    while (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        if t.len() >= 2 {
            t = &t[1..t.len() - 1];
        } else {
            break;
        }
    }
    t.replace("\\\"", "\"").trim().to_string()
}

fn normalize_path_for_match(p: &str) -> String {
    let mut s = unquote_trimmed(p).replace('\\', "/");
    while s.starts_with("./") {
        s = s[2..].to_string();
    }
    s.to_lowercase()
}

/// Parses `.agent/ecl.yaml` using a schema-tolerant, pattern-driven state machine.
/// Recognizes flexible key aliases:
/// - Cluster ID: `cluster_id`, `id`, `cluster`, `name`
/// - Coordinates: any `<path>#L<start>-<end>` or keys `id`, `node`, `path`, `file`, `target`, `location`
/// - Symbols: `symbol`, `fn`, `func`, `struct`, `class`
/// - Rules: `rules`, `invariants`, `guards`, `constraints`
pub fn parse_ecl_yaml(content: &str) -> Vec<EclCluster> {
    let mut clusters: Vec<EclCluster> = Vec::new();
    let mut current: Option<EclCluster> = None;
    let mut in_rules = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = raw_line.len() - raw_line.trim_start().len();

        // Detect new cluster header (e.g., `  - cluster_id: foo` or `  - id: foo` at top cluster level)
        if indent <= 4 && trimmed.starts_with("- ") {
            let after_dash = trimmed[2..].trim();
            for key in &["cluster_id:", "id:", "cluster:", "name:"] {
                if let Some(rest) = after_dash.strip_prefix(key) {
                    if let Some(prev) = current.take() {
                        if !prev.nodes.is_empty() || !prev.rules.is_empty() {
                            clusters.push(prev);
                        }
                    }
                    current = Some(EclCluster {
                        id: unquote_trimmed(rest),
                        summary: None,
                        nodes: Vec::new(),
                        rules: Vec::new(),
                    });
                    in_rules = false;
                    break;
                }
            }
        }

        let Some(ref mut cluster) = current else {
            continue;
        };

        // Check if entering or leaving a rules/invariants block
        let key_part = trimmed.split(':').next().unwrap_or("").trim_start_matches("- ").trim();
        if matches!(key_part, "rules" | "invariants" | "guards" | "constraints") && trimmed.ends_with(':') {
            in_rules = true;
            continue;
        } else if trimmed.ends_with(':') && !trimmed.starts_with("- ") {
            in_rules = false;
        }

        if in_rules {
            if let Some(rule_item) = trimmed.strip_prefix("- ") {
                let rule_text = unquote_trimmed(rule_item);
                if !rule_text.is_empty() && !rule_text.starts_with("[TODO") {
                    cluster.rules.push(rule_text);
                }
                continue;
            } else if !trimmed.starts_with('-') && trimmed.contains(':') {
                in_rules = false;
            }
        }

        // Detect business intent / summary for the cluster
        let clean_line = trimmed.trim_start_matches("- ").trim();
        for sum_key in &["summary:", "intent:", "description:"] {
            if let Some(rest) = clean_line.strip_prefix(sum_key) {
                let s = unquote_trimmed(rest);
                if !s.is_empty() && !s.starts_with("[TODO") && cluster.summary.is_none() {
                    cluster.summary = Some(s);
                }
            }
        }

        // Detect symbol alias for the most recent node
        for sym_key in &["symbol:", "fn:", "func:", "struct:", "class:"] {
            if let Some(rest) = clean_line.strip_prefix(sym_key) {
                let sym = unquote_trimmed(rest);
                if !sym.is_empty() {
                    if let Some(last_node) = cluster.nodes.last_mut() {
                        if last_node.symbol.is_none() {
                            last_node.symbol = Some(sym);
                        }
                    }
                }
            }
        }

        // Detect causal side-effect / temporal behavior for the most recent node
        for eff_key in &["effect:", "impact:", "action:", "when:"] {
            if let Some(rest) = clean_line.strip_prefix(eff_key) {
                let eff = unquote_trimmed(rest);
                if !eff.is_empty() && !eff.starts_with("[TODO") {
                    if let Some(last_node) = cluster.nodes.last_mut() {
                        if last_node.effect.is_none() {
                            last_node.effect = Some(eff);
                        }
                    }
                }
            }
        }

        // Pattern-first coordinate detection: any token containing `#L<digits>` or explicit path keys
        for coord_key in &["id:", "node:", "path:", "file:", "target:", "location:", "entrypoint:"] {
            if let Some(rest) = clean_line.strip_prefix(coord_key) {
                let val = unquote_trimmed(rest);
                if !val.is_empty() && (val.contains('/') || val.contains('\\') || val.contains("#L") || val.ends_with(".rs") || val.ends_with(".py") || val.ends_with(".ts") || val.ends_with(".tsx") || val.ends_with(".js") || val.ends_with(".go")) {
                    let rel_path = val.split('#').next().unwrap_or(&val).trim().to_string();
                    if !rel_path.is_empty() {
                        cluster.nodes.push(EclNode {
                            raw_coord: val,
                            rel_path,
                            symbol: None,
                            effect: None,
                        });
                    }
                }
            }
        }
    }

    if let Some(prev) = current.take() {
        if !prev.nodes.is_empty() || !prev.rules.is_empty() {
            clusters.push(prev);
        }
    }

    clusters
}

/// Matches a list of accessed file paths against parsed ECL clusters and formats a compact `[ECL Shield]` hint.
pub fn match_ecl_clusters(ecl_content: &str, accessed_files: &[String]) -> Option<String> {
    if accessed_files.is_empty() {
        return None;
    }

    let clusters = parse_ecl_yaml(ecl_content);
    if clusters.is_empty() {
        return None;
    }

    let norm_accessed: Vec<String> = accessed_files
        .iter()
        .map(|f| normalize_path_for_match(f))
        .filter(|f| !f.is_empty() && !f.ends_with("ecl.yaml"))
        .collect();

    if norm_accessed.is_empty() {
        return None;
    }

    let mut matched_hints: Vec<String> = Vec::new();

    for c in clusters {
        let is_hit = c.nodes.iter().any(|n| {
            let norm_node = normalize_path_for_match(&n.rel_path);
            !norm_node.is_empty()
                && norm_accessed.iter().any(|acc| {
                    acc == &norm_node || acc.ends_with(&format!("/{}", norm_node))
                })
        });

        if is_hit && !c.rules.is_empty() {
            let chain_str: Vec<String> = c
                .nodes
                .iter()
                .map(|n| match (&n.symbol, &n.effect) {
                    (Some(sym), Some(eff)) => format!("{} ({} -> {})", n.raw_coord, sym, eff),
                    (Some(sym), None) => format!("{} ({})", n.raw_coord, sym),
                    (None, Some(eff)) => format!("{} ({})", n.raw_coord, eff),
                    (None, None) => n.raw_coord.clone(),
                })
                .collect();

            let rules_str: Vec<String> = c
                .rules
                .iter()
                .enumerate()
                .map(|(idx, r)| format!("({}) {}", idx + 1, r))
                .collect();

            let header = match &c.summary {
                Some(sum) => format!("{}: {}", c.id, sum),
                None => c.id.clone(),
            };

            matched_hints.push(format!(
                "[ECL Shield ({}) | Chain: {} | Rules: {}]",
                header,
                chain_str.join(" ==> "),
                rules_str.join("; ")
            ));
        }
    }

    if matched_hints.is_empty() {
        None
    } else {
        Some(matched_hints.join(" "))
    }
}

/// Extracts recently read/inspected file paths from the latest tool invocation in `transcript.jsonl`
/// and matches them against `<workspace>/.agent/ecl.yaml` (or ancestor `.agent/ecl.yaml` of the accessed file).
pub fn probe_ecl_shield(transcript_path: &Path, workspace_root: Option<&Path>) -> Option<String> {
    let mut file = File::open(transcript_path).ok()?;
    let file_len = file.metadata().ok()?.len();
    if file_len == 0 {
        return None;
    }

    let chunk_start = file_len.saturating_sub(CHUNK_SIZE as u64);
    let read_len = (file_len - chunk_start) as usize;
    file.seek(SeekFrom::Start(chunk_start)).ok()?;

    let mut buf = vec![0u8; read_len];
    file.read_exact(&mut buf).ok()?;

    let lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();
    let mut accessed_files: Vec<String> = Vec::new();

    for line in lines.into_iter().rev() {
        if line.is_empty() || line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }

        if let Ok(line_str) = std::str::from_utf8(line) {
            if let Ok(step) = serde_json::from_str::<TranscriptStep>(line_str.trim()) {
                let st = step.step_type.as_deref().unwrap_or("");
                if st == "EPHEMERAL_MESSAGE" || st == "SYSTEM_SDK" || st == "GENERIC" {
                    continue;
                }

                if st == "PLANNER_RESPONSE" {
                    if let Some(serde_json::Value::Array(calls)) = step.tool_calls {
                        for call in calls {
                            let tool_name = call.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            if tool_name.ends_with("read_file") || tool_name.ends_with("edit_file") {
                                if let Some(p) = call
                                    .get("args")
                                    .and_then(|a| a.get("Path"))
                                    .and_then(|v| v.as_str())
                                {
                                    accessed_files.push(unquote_trimmed(p));
                                }
                            } else if tool_name.ends_with("run_command") {
                                if let Some(cmd) = call
                                    .get("args")
                                    .and_then(|a| a.get("CommandLine"))
                                    .and_then(|v| v.as_str())
                                {
                                    let clean_cmd = unquote_trimmed(cmd);
                                    if let Some(rest) = clean_cmd.strip_prefix("outline ") {
                                        let target = rest.split_whitespace().next().unwrap_or("");
                                        if !target.is_empty() {
                                            accessed_files.push(unquote_trimmed(target));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break;
                }

                break;
            }
        }
    }

    if accessed_files.is_empty() {
        return None;
    }

    let mut ecl_candidate: Option<PathBuf> = None;
    if let Some(ws) = workspace_root {
        let p = ws.join(".agent").join("ecl.yaml");
        if p.is_file() {
            ecl_candidate = Some(p);
        }
    }

    if ecl_candidate.is_none() {
        for f_str in &accessed_files {
            let mut cur = Path::new(f_str).parent();
            while let Some(dir) = cur {
                let candidate = dir.join(".agent").join("ecl.yaml");
                if candidate.is_file() {
                    ecl_candidate = Some(candidate);
                    break;
                }
                cur = dir.parent();
            }
            if ecl_candidate.is_some() {
                break;
            }
        }
    }

    let ecl_path = ecl_candidate?;
    let ecl_content = std::fs::read_to_string(&ecl_path).ok()?;
    match_ecl_clusters(&ecl_content, &accessed_files)
}

fn main() {
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

    if let Some(ecl_hint) = probe_ecl_shield(transcript_path, ws_root) {
        let output = HookOutput {
            inject_steps: vec![InjectStep {
                ephemeral_message: ecl_hint,
            }],
        };
        if let Ok(json_str) = serde_json::to_string(&output) {
            println!("{}", json_str);
            return;
        }
    }

    emit_empty_response();
}

fn emit_empty_response() {
    println!("{{}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecl_shield_matching_and_flexible_keys() {
        let ecl_yaml = r#"
clusters:
  - cluster_id: seek_scrub_polling_exclusivity
    summary: Synchronizes user timeline slider dragging with the 100ms background polling timer
    entrypoint:
      id: src/ui_callbacks.rs#L73-80
      symbol: PendingSeekState
    chain:
      - node: src/app_timer.rs#L28-558
        symbol: start_playback_polling_timer
        effect: Skips slider overwrite while user_scrub is active
    rules:
      - 用戶拖曳進度條期間，必須嚴格阻斷 100ms Polling Timer 覆寫 UI 滑桿位置

  - id: flexible_alias_cluster
    target: src/stream_reader/cache.rs#L3-10
    struct: ByteCache
    downstream:
      - path: src/library.rs#L402-511
        fn: prune_cache_async
    invariants:
      - MAX_BYTE_CACHE_BYTES = 32MB 上限不可移除
"#;

        // 1. Hit via entrypoint path (Windows absolute path with backslashes)
        let hit1 = match_ecl_clusters(
            ecl_yaml,
            &[r"C:\Users\Admin\rs-yt-player\src\ui_callbacks.rs".to_string()],
        )
        .expect("Should match entrypoint");
        assert!(hit1.contains("ECL Shield (seek_scrub_polling_exclusivity: Synchronizes user timeline slider dragging"));
        assert!(hit1.contains("src/ui_callbacks.rs#L73-80 (PendingSeekState) ==> src/app_timer.rs#L28-558 (start_playback_polling_timer -> Skips slider overwrite while user_scrub is active)"));
        assert!(hit1.contains("用戶拖曳進度條期間"));

        // 2. Hit via downstream chain node + flexible alias keys (`id`, `target`, `path`, `invariants`)
        let hit2 = match_ecl_clusters(
            ecl_yaml,
            &["C:/Users/Admin/rs-yt-player/src/library.rs".to_string()],
        )
        .expect("Should match downstream chain node with flexible keys");
        assert!(hit2.contains("ECL Shield (flexible_alias_cluster)"));
        assert!(hit2.contains("src/stream_reader/cache.rs#L3-10 (ByteCache) ==> src/library.rs#L402-511 (prune_cache_async)"));
        assert!(hit2.contains("MAX_BYTE_CACHE_BYTES = 32MB"));

        // 3. Non-matching file -> None (0 token noise)
        assert_eq!(
            match_ecl_clusters(ecl_yaml, &["src/unrelated_module.rs".to_string()]),
            None
        );
    }
}
