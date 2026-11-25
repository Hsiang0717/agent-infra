use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Tail window size (128 KB ≈ recent 15-25 turns) for scanning transcript tool calls
/// and session-window deduplication without starving long multi-phase conversations.
const DEDUP_WINDOW_BYTES: usize = 128 * 1024;

/// Grace period for mtime comparisons to avoid false Advisory triggers immediately
/// after `git clone` or `git checkout`.
const MTIME_GRACE_SECS: u64 = 5;

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
    content: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeHealth {
    Fresh,
    Modified,
    MissingFile,
    MissingSymbol(String),
}

impl NodeHealth {
    fn is_broken(&self) -> bool {
        matches!(self, NodeHealth::MissingFile | NodeHealth::MissingSymbol(_))
    }
}

#[derive(Debug, Default, Clone)]
pub struct DedupState {
    pub seen_read_clusters: HashSet<String>,
    pub seen_cleanup_clusters: HashSet<String>,
    pub seen_post_edit_clusters: HashSet<String>,
}

fn unquote_trimmed(s: &str) -> String {
    let mut t = s.trim();
    // Handle JSON-stringified tool arguments from agy CLI (e.g. `"\"C:\\\\Users\\\\...\""`).
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        if let Ok(parsed) = serde_json::from_str::<String>(t) {
            let inner = parsed.trim();
            if (inner.starts_with('"') && inner.ends_with('"'))
                || (inner.starts_with('\'') && inner.ends_with('\''))
            {
                return unquote_trimmed(inner);
            }
            return inner.to_string();
        }
    }
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
    while s.contains("//") {
        s = s.replace("//", "/");
    }
    while s.starts_with("./") {
        s = s[2..].to_string();
    }
    s.to_lowercase()
}

fn path_matches_node(norm_accessed: &str, norm_node: &str) -> bool {
    !norm_node.is_empty()
        && (norm_accessed == norm_node || norm_accessed.ends_with(&format!("/{}", norm_node)))
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn symbol_exists_in_content(content: &str, symbol: &str) -> bool {
    if symbol.is_empty() {
        return true;
    }
    let bytes = content.as_bytes();
    let sym_len = symbol.len();
    let mut start = 0;
    while let Some(rel_idx) = content[start..].find(symbol) {
        let idx = start + rel_idx;
        let left_ok = idx == 0 || !is_ident_byte(bytes[idx - 1]);
        let right_idx = idx + sym_len;
        let right_ok = right_idx == bytes.len() || !is_ident_byte(bytes[right_idx]);
        if left_ok && right_ok {
            return true;
        }
        start = idx + 1;
        while start < bytes.len() && !content.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

fn check_node_health(
    project_root: Option<&Path>,
    node: &EclNode,
    ecl_mtime: Option<SystemTime>,
) -> NodeHealth {
    let Some(root) = project_root else {
        return NodeHealth::Fresh;
    };

    let full_path = root.join(&node.rel_path);
    if !full_path.is_file() {
        return NodeHealth::MissingFile;
    }

    if let Some(ref sym) = node.symbol {
        if let Ok(text) = std::fs::read_to_string(&full_path) {
            if !symbol_exists_in_content(&text, sym) {
                return NodeHealth::MissingSymbol(sym.clone());
            }
        }
    }

    if let (Some(ecl_ts), Ok(meta)) = (ecl_mtime, std::fs::metadata(&full_path)) {
        if let Ok(file_ts) = meta.modified() {
            if let Some(threshold) = ecl_ts.checked_add(Duration::from_secs(MTIME_GRACE_SECS)) {
                if file_ts > threshold {
                    return NodeHealth::Modified;
                }
            }
        }
    }

    NodeHealth::Fresh
}

fn extract_marker_cluster_ids(content: &str, prefix: &str, out: &mut HashSet<String>) {
    let mut search_from = 0;
    while let Some(rel_idx) = content[search_from..].find(prefix) {
        let id_start = search_from + rel_idx + prefix.len();
        let rest = &content[id_start..];
        if let Some(end_offset) = rest.find([':', ')']) {
            let cid = rest[..end_offset].trim();
            if !cid.is_empty() {
                out.insert(cid.to_string());
            }
        }
        search_from = id_start;
    }
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
        let key_part = trimmed
            .split(':')
            .next()
            .unwrap_or("")
            .trim_start_matches("- ")
            .trim();
        if matches!(key_part, "rules" | "invariants" | "guards" | "constraints")
            && trimmed.ends_with(':')
        {
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
        for coord_key in &[
            "id:",
            "node:",
            "path:",
            "file:",
            "target:",
            "location:",
            "entrypoint:",
        ] {
            if let Some(rest) = clean_line.strip_prefix(coord_key) {
                let val = unquote_trimmed(rest);
                if !val.is_empty()
                    && (val.contains('/')
                        || val.contains('\\')
                        || val.contains("#L")
                        || val.ends_with(".rs")
                        || val.ends_with(".py")
                        || val.ends_with(".ts")
                        || val.ends_with(".tsx")
                        || val.ends_with(".js")
                        || val.ends_with(".go"))
                {
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

/// Evaluates read and edit/write accesses against parsed ECL clusters using AAEP empirical guards:
/// - Session-window deduplication (injects each cluster at most once per recent transcript window)
/// - Live filesystem verification (path existence, symbol presence, mtime freshness + 5s grace)
/// - Silent Drop on broken accessed targets (prevents pink-elephant active poisoning)
/// - Empirical Overhaul Auto-Mute (auto-silences read injections when >=50% of >=2 clusters are broken)
/// - Separate post-edit / cleanup cues for `edit_file` and `write_file` without re-spamming old rules
pub fn evaluate_ecl_clusters(
    ecl_content: &str,
    read_files: &[String],
    edited_files: &[String],
    project_root: Option<&Path>,
    ecl_mtime: Option<SystemTime>,
    dedup: &DedupState,
) -> Option<String> {
    if read_files.is_empty() && edited_files.is_empty() {
        return None;
    }

    let clusters = parse_ecl_yaml(ecl_content);
    if clusters.is_empty() {
        return None;
    }

    let norm_reads: Vec<String> = read_files
        .iter()
        .map(|f| normalize_path_for_match(f))
        .filter(|f| !f.is_empty() && !f.ends_with("ecl.yaml"))
        .collect();

    let norm_edits: Vec<String> = edited_files
        .iter()
        .map(|f| normalize_path_for_match(f))
        .filter(|f| !f.is_empty() && !f.ends_with("ecl.yaml"))
        .collect();

    if norm_reads.is_empty() && norm_edits.is_empty() {
        return None;
    }

    // Precompute node health per cluster when project_root is available, and detect
    // global architectural overhaul (>=50% of >=2 active clusters have broken references).
    let cluster_healths: Vec<Vec<NodeHealth>> = clusters
        .iter()
        .map(|c| {
            c.nodes
                .iter()
                .map(|n| check_node_health(project_root, n, ecl_mtime))
                .collect()
        })
        .collect();

    let mut active_cluster_count = 0usize;
    let mut broken_cluster_count = 0usize;
    if project_root.is_some() {
        for (c, healths) in clusters.iter().zip(cluster_healths.iter()) {
            if !c.nodes.is_empty() && !c.rules.is_empty() {
                active_cluster_count += 1;
                if healths.iter().any(NodeHealth::is_broken) {
                    broken_cluster_count += 1;
                }
            }
        }
    }
    let is_global_overhaul =
        active_cluster_count >= 2 && broken_cluster_count * 2 >= active_cluster_count;

    let mut matched_hints: Vec<String> = Vec::new();
    let mut handled_by_edit: HashSet<String> = HashSet::new();

    // 1. Evaluate edit_file / write_file hits first (compact post-edit or cleanup cues; never re-spam rules).
    for (c, healths) in clusters.iter().zip(cluster_healths.iter()) {
        if c.nodes.is_empty() {
            continue;
        }

        let edited_indices: Vec<usize> = c
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, n)| {
                let norm_node = normalize_path_for_match(&n.rel_path);
                if norm_edits
                    .iter()
                    .any(|acc| path_matches_node(acc, &norm_node))
                {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        if edited_indices.is_empty() {
            continue;
        }
        handled_by_edit.insert(c.id.clone());

        // Check if this edit broke any tracked node in this cluster.
        let mut broken_reasons: Vec<String> = Vec::new();
        for &idx in &edited_indices {
            match &healths[idx] {
                NodeHealth::MissingFile => {
                    broken_reasons.push(format!("file `{}` removed", c.nodes[idx].rel_path));
                }
                NodeHealth::MissingSymbol(sym) => {
                    broken_reasons.push(format!(
                        "symbol `{}` no longer found in `{}`",
                        sym, c.nodes[idx].rel_path
                    ));
                }
                NodeHealth::Fresh | NodeHealth::Modified => {}
            }
        }

        if !broken_reasons.is_empty() {
            if !dedup.seen_cleanup_clusters.contains(&c.id) {
                let suffix = if is_global_overhaul {
                    "overhaul detected — prune/update .agent/ecl.yaml or re-run /ecl-bootstrap before closure"
                } else {
                    "update or prune .agent/ecl.yaml before closure"
                };
                matched_hints.push(format!(
                    "[ECL Cleanup ({}): {} — {}]",
                    c.id,
                    broken_reasons.join(", "),
                    suffix
                ));
            }
            continue;
        }

        // Healthy edit: emit a single compact post-edit verification cue (skip during global overhaul or if already emitted).
        if !is_global_overhaul
            && !c.rules.is_empty()
            && !dedup.seen_post_edit_clusters.contains(&c.id)
        {
            let edited_paths: Vec<&str> = edited_indices
                .iter()
                .map(|&i| c.nodes[i].rel_path.as_str())
                .collect();
            let other_nodes: Vec<String> = c
                .nodes
                .iter()
                .enumerate()
                .filter(|(idx, _)| !edited_indices.contains(idx))
                .map(|(idx, n)| {
                    if healths[idx].is_broken() {
                        format!("{} [missing]", n.rel_path)
                    } else {
                        n.rel_path.clone()
                    }
                })
                .collect();

            if other_nodes.is_empty() {
                matched_hints.push(format!(
                    "[ECL Post-Edit ({}): {} modified — if invariants changed, update .agent/ecl.yaml or run /ecl-sync]",
                    c.id,
                    edited_paths.join(", ")
                ));
            } else {
                matched_hints.push(format!(
                    "[ECL Post-Edit ({}): {} modified — verify downstream ({}) and update .agent/ecl.yaml or run /ecl-sync if invariants changed]",
                    c.id,
                    edited_paths.join(", "),
                    other_nodes.join(", ")
                ));
            }
        }
    }

    // 2. Evaluate read_file / outline hits (with global overhaul auto-mute, dedup, and stale silent-drop).
    if !is_global_overhaul {
        for (c, healths) in clusters.iter().zip(cluster_healths.iter()) {
            if c.rules.is_empty()
                || handled_by_edit.contains(&c.id)
                || dedup.seen_read_clusters.contains(&c.id)
            {
                continue;
            }

            let read_indices: Vec<usize> = c
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(idx, n)| {
                    let norm_node = normalize_path_for_match(&n.rel_path);
                    if norm_reads
                        .iter()
                        .any(|acc| path_matches_node(acc, &norm_node))
                    {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();

            if read_indices.is_empty() {
                continue;
            }

            // Silent Drop if every accessed node in this cluster is structurally broken
            // (prevents pink-elephant active poisoning from obsolete rules).
            if read_indices.iter().all(|&idx| healths[idx].is_broken()) {
                continue;
            }

            let has_modified = healths.iter().any(|h| matches!(h, NodeHealth::Modified));
            let has_partial_break = healths.iter().any(NodeHealth::is_broken);

            let chain_str: Vec<String> = c
                .nodes
                .iter()
                .zip(healths.iter())
                .map(|(n, h)| {
                    let base = match (&n.symbol, &n.effect) {
                        (Some(sym), Some(eff)) => format!("{} ({} -> {})", n.raw_coord, sym, eff),
                        (Some(sym), None) => format!("{} ({})", n.raw_coord, sym),
                        (None, Some(eff)) => format!("{} ({})", n.raw_coord, eff),
                        (None, None) => n.raw_coord.clone(),
                    };
                    if h.is_broken() {
                        format!("{} [missing]", base)
                    } else {
                        base
                    }
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

            if has_modified || has_partial_break {
                matched_hints.push(format!(
                    "[ECL Advisory ({}) | Code changed since last sync — verify against live code | Chain: {} | Rules: {}]",
                    header,
                    chain_str.join(" ==> "),
                    rules_str.join("; ")
                ));
            } else {
                matched_hints.push(format!(
                    "[ECL Shield ({}) | Chain: {} | Rules: {}]",
                    header,
                    chain_str.join(" ==> "),
                    rules_str.join("; ")
                ));
            }
        }
    }

    if matched_hints.is_empty() {
        None
    } else {
        Some(matched_hints.join(" "))
    }
}

/// Pure string-matching helper retained for direct unit testing without filesystem state.
pub fn match_ecl_clusters(ecl_content: &str, accessed_files: &[String]) -> Option<String> {
    evaluate_ecl_clusters(
        ecl_content,
        accessed_files,
        &[],
        None,
        None,
        &DedupState::default(),
    )
}

/// Extracts recently read/edited file paths and prior ECL injection markers from the tail window
/// of `transcript.jsonl`, then evaluates them against `.agent/ecl.yaml`.
pub fn probe_ecl_shield(transcript_path: &Path, workspace_root: Option<&Path>) -> Option<String> {
    let mut file = File::open(transcript_path).ok()?;
    let file_len = file.metadata().ok()?.len();
    if file_len == 0 {
        return None;
    }

    let chunk_start = file_len.saturating_sub(DEDUP_WINDOW_BYTES as u64);
    let read_len = (file_len - chunk_start) as usize;
    file.seek(SeekFrom::Start(chunk_start)).ok()?;

    let mut buf = vec![0u8; read_len];
    file.read_exact(&mut buf).ok()?;

    let lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();
    let mut read_files: Vec<String> = Vec::new();
    let mut edited_files: Vec<String> = Vec::new();
    let mut dedup = DedupState::default();
    let mut captured_latest_turn = false;

    for line in lines.into_iter().rev() {
        if line.is_empty() || line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }

        if let Ok(line_str) = std::str::from_utf8(line) {
            if let Ok(step) = serde_json::from_str::<TranscriptStep>(line_str.trim()) {
                let st = step.step_type.as_deref().unwrap_or("");

                // Collect prior ECL injections across the entire tail window for deduplication.
                if st == "SYSTEM_MESSAGE" || st == "EPHEMERAL_MESSAGE" {
                    if let Some(ref content) = step.content {
                        extract_marker_cluster_ids(
                            content,
                            "[ECL Shield (",
                            &mut dedup.seen_read_clusters,
                        );
                        extract_marker_cluster_ids(
                            content,
                            "[ECL Advisory (",
                            &mut dedup.seen_read_clusters,
                        );
                        extract_marker_cluster_ids(
                            content,
                            "[ECL Cleanup (",
                            &mut dedup.seen_cleanup_clusters,
                        );
                        extract_marker_cluster_ids(
                            content,
                            "[ECL Post-Edit (",
                            &mut dedup.seen_post_edit_clusters,
                        );
                    }
                    continue;
                }

                if captured_latest_turn {
                    continue;
                }

                if st == "SYSTEM_SDK" || st == "GENERIC" {
                    continue;
                }

                if st == "PLANNER_RESPONSE" {
                    if let Some(serde_json::Value::Array(calls)) = step.tool_calls {
                        for call in calls {
                            let tool_name =
                                call.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            if tool_name.ends_with("read_file") {
                                if let Some(p) = call
                                    .get("args")
                                    .and_then(|a| a.get("Path"))
                                    .and_then(|v| v.as_str())
                                {
                                    read_files.push(unquote_trimmed(p));
                                }
                            } else if tool_name.ends_with("edit_file")
                                || tool_name.ends_with("write_file")
                            {
                                if let Some(p) = call
                                    .get("args")
                                    .and_then(|a| a.get("Path"))
                                    .and_then(|v| v.as_str())
                                {
                                    edited_files.push(unquote_trimmed(p));
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
                                            read_files.push(unquote_trimmed(target));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    captured_latest_turn = true;
                    continue;
                }

                // Latest non-system step wasn't a PLANNER_RESPONSE (e.g. fresh USER_INPUT):
                // nothing to inject for this turn.
                return None;
            }
        }
    }

    if read_files.is_empty() && edited_files.is_empty() {
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
        for f_str in read_files.iter().chain(edited_files.iter()) {
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
    let project_root = ecl_path.parent().and_then(Path::parent);
    let ecl_mtime = std::fs::metadata(&ecl_path)
        .and_then(|m| m.modified())
        .ok();
    let ecl_content = std::fs::read_to_string(&ecl_path).ok()?;

    evaluate_ecl_clusters(
        &ecl_content,
        &read_files,
        &edited_files,
        project_root,
        ecl_mtime,
        &dedup,
    )
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
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn make_temp_dir(label: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "agy_ecl_test_{}_{}_{}",
            std::process::id(),
            label,
            id
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

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
        assert!(hit1.contains(
            "ECL Shield (seek_scrub_polling_exclusivity: Synchronizes user timeline slider dragging"
        ));
        assert!(hit1.contains("src/ui_callbacks.rs#L73-80 (PendingSeekState) ==> src/app_timer.rs#L28-558 (start_playback_polling_timer -> Skips slider overwrite while user_scrub is active)"));
        assert!(hit1.contains("用戶拖曳進度條期間"));

        // 2. Hit via downstream chain node + double-JSON-escaped agy CLI path format
        let hit2 = match_ecl_clusters(
            ecl_yaml,
            &[r#""C:\\Users\\Admin\\rs-yt-player\\src\\library.rs""#.to_string()],
        )
        .expect("Should match downstream chain node with escaped CLI path");
        assert!(hit2.contains("ECL Shield (flexible_alias_cluster)"));
        assert!(hit2.contains(
            "src/stream_reader/cache.rs#L3-10 (ByteCache) ==> src/library.rs#L402-511 (prune_cache_async)"
        ));
        assert!(hit2.contains("MAX_BYTE_CACHE_BYTES = 32MB"));

        // 3. Non-matching file -> None (0 token noise)
        assert_eq!(
            match_ecl_clusters(ecl_yaml, &["src/unrelated_module.rs".to_string()]),
            None
        );
    }

    #[test]
    fn test_silent_drop_on_broken_read_and_cleanup_on_edit() {
        let dir = make_temp_dir("broken_read_edit");
        fs::create_dir_all(dir.join("src")).unwrap();
        // Symbol `PendingSeekState` was renamed to `SeekDraft`, while `start_polling` still exists.
        fs::write(dir.join("src/ui.rs"), "pub struct SeekDraft {}\n").unwrap();
        fs::write(dir.join("src/timer.rs"), "pub fn start_polling() {}\n").unwrap();

        let ecl_yaml = r#"
clusters:
  - cluster_id: seek_flow
    summary: Seek sync
    entrypoint:
      id: src/ui.rs#L1-5
      symbol: PendingSeekState
    chain:
      - node: src/timer.rs#L1-5
        symbol: start_polling
    rules:
      - Block timer during scrub
"#;

        // Reading broken node -> Silent Drop (no stale rules injected)
        let read_broken = evaluate_ecl_clusters(
            ecl_yaml,
            &[dir.join("src/ui.rs").to_string_lossy().to_string()],
            &[],
            Some(&dir),
            None,
            &DedupState::default(),
        );
        assert_eq!(read_broken, None);

        // Reading healthy sibling node -> Advisory with partial `[missing]` annotation
        let read_sibling = evaluate_ecl_clusters(
            ecl_yaml,
            &[dir.join("src/timer.rs").to_string_lossy().to_string()],
            &[],
            Some(&dir),
            None,
            &DedupState::default(),
        )
        .expect("Healthy sibling should receive Advisory");
        assert!(read_sibling.starts_with("[ECL Advisory (seek_flow: Seek sync)"));
        assert!(read_sibling.contains("src/ui.rs#L1-5 (PendingSeekState) [missing]"));

        // Editing broken node -> 1-line Cleanup WITHOUT old rules
        let edit_broken = evaluate_ecl_clusters(
            ecl_yaml,
            &[],
            &[dir.join("src/ui.rs").to_string_lossy().to_string()],
            Some(&dir),
            None,
            &DedupState::default(),
        )
        .expect("Editing broken node should trigger Cleanup");
        assert!(edit_broken.contains("[ECL Cleanup (seek_flow):"));
        assert!(edit_broken.contains("PendingSeekState"));
        assert!(!edit_broken.contains("Block timer during scrub"));

        // Editing healthy node -> compact Post-Edit WITHOUT repeating rules
        let edit_healthy = evaluate_ecl_clusters(
            ecl_yaml,
            &[],
            &[dir.join("src/timer.rs").to_string_lossy().to_string()],
            Some(&dir),
            None,
            &DedupState::default(),
        )
        .expect("Editing healthy node should trigger Post-Edit");
        assert!(edit_healthy.contains("[ECL Post-Edit (seek_flow):"));
        assert!(!edit_healthy.contains("Block timer during scrub"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_mtime_advisory_and_grace_window() {
        let dir = make_temp_dir("mtime_grace");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/a.rs"), "pub fn alpha() {}\n").unwrap();

        let ecl_yaml = r#"
clusters:
  - cluster_id: alpha_flow
    entrypoint:
      id: src/a.rs#L1-5
      symbol: alpha
    rules:
      - Keep alpha idempotent
"#;

        let now = SystemTime::now();
        // ecl.yaml is 2s older than file (within 5s grace window) -> Fresh Shield
        let within_grace = now.checked_sub(Duration::from_secs(2));
        let fresh_hit = evaluate_ecl_clusters(
            ecl_yaml,
            &[dir.join("src/a.rs").to_string_lossy().to_string()],
            &[],
            Some(&dir),
            within_grace,
            &DedupState::default(),
        )
        .expect("Should emit Fresh Shield within grace window");
        assert!(fresh_hit.starts_with("[ECL Shield (alpha_flow)"));

        // ecl.yaml is 30s older than file (outside 5s grace window) -> Advisory
        let stale_ts = now.checked_sub(Duration::from_secs(30));
        let advisory_hit = evaluate_ecl_clusters(
            ecl_yaml,
            &[dir.join("src/a.rs").to_string_lossy().to_string()],
            &[],
            Some(&dir),
            stale_ts,
            &DedupState::default(),
        )
        .expect("Should emit Advisory when file is newer than ecl.yaml");
        assert!(advisory_hit.starts_with("[ECL Advisory (alpha_flow)"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_global_overhaul_auto_mute() {
        let dir = make_temp_dir("global_overhaul");
        fs::create_dir_all(dir.join("src")).unwrap();
        // Cluster 1 broken (old_a gone), Cluster 2 broken (file missing), Cluster 3 still intact.
        fs::write(dir.join("src/a.rs"), "pub fn new_a() {}\n").unwrap();
        fs::write(dir.join("src/c.rs"), "pub fn keep_c() {}\n").unwrap();

        let ecl_yaml = r#"
clusters:
  - cluster_id: c1
    entrypoint:
      id: src/a.rs#L1-3
      symbol: old_a
    rules:
      - Rule 1
  - cluster_id: c2
    entrypoint:
      id: src/deleted_b.rs#L1-3
      symbol: old_b
    rules:
      - Rule 2
  - cluster_id: c3
    entrypoint:
      id: src/c.rs#L1-3
      symbol: keep_c
    rules:
      - Rule 3
"#;

        // 2 out of 3 clusters broken (>=50%) -> even reading healthy c3 is auto-muted!
        let muted = evaluate_ecl_clusters(
            ecl_yaml,
            &[dir.join("src/c.rs").to_string_lossy().to_string()],
            &[],
            Some(&dir),
            None,
            &DedupState::default(),
        );
        assert_eq!(muted, None);

        // Editing broken a.rs still surfaces a 1-line overhaul Cleanup hint (without old rules)
        let cleanup = evaluate_ecl_clusters(
            ecl_yaml,
            &[],
            &[dir.join("src/a.rs").to_string_lossy().to_string()],
            Some(&dir),
            None,
            &DedupState::default(),
        )
        .expect("Overhaul edit should emit concise cleanup cue");
        assert!(cleanup.contains("[ECL Cleanup (c1):"));
        assert!(cleanup.contains("/ecl-bootstrap"));
        assert!(!cleanup.contains("Rule 1"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_transcript_dedup_and_write_file_tracking() {
        let dir = make_temp_dir("transcript_dedup");
        fs::create_dir_all(dir.join(".agent")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();

        fs::write(dir.join("src/core.rs"), "pub fn init_core() {}\n").unwrap();
        fs::write(
            dir.join(".agent/ecl.yaml"),
            r#"
clusters:
  - cluster_id: core_boot
    summary: Boot sequence
    entrypoint:
      id: src/core.rs#L1-5
      symbol: init_core
    rules:
      - Must initialize logger before core
"#,
        )
        .unwrap();

        let core_path_json =
            serde_json::to_string(&dir.join("src/core.rs").to_string_lossy().to_string()).unwrap();

        // Turn 1: first read_file -> injects Shield once
        let t1 = dir.join("t1.jsonl");
        let turn1_line = serde_json::json!({
            "type": "PLANNER_RESPONSE",
            "tool_calls": [{
                "name": "read_file",
                "args": { "Path": core_path_json }
            }]
        });
        fs::write(&t1, format!("{}\n", turn1_line)).unwrap();
        let res1 = probe_ecl_shield(&t1, Some(&dir)).expect("First read should inject Shield");
        assert!(res1.contains("[ECL Shield (core_boot: Boot sequence)"));

        // Turn 2: SYSTEM_MESSAGE recorded prior Shield + another read_file -> deduped (None!)
        let t2 = dir.join("t2.jsonl");
        let sys_msg = serde_json::json!({
            "source": "SYSTEM_SDK",
            "type": "SYSTEM_MESSAGE",
            "content": format!("Injected:\n{}", res1)
        });
        fs::write(&t2, format!("{}\n{}\n{}\n", turn1_line, sys_msg, turn1_line)).unwrap();
        assert_eq!(probe_ecl_shield(&t2, Some(&dir)), None);

        // Turn 3: write_file overwrote core.rs removing `init_core` -> triggers Cleanup
        fs::write(dir.join("src/core.rs"), "pub fn boot_v2() {}\n").unwrap();
        let t3 = dir.join("t3.jsonl");
        let write_line = serde_json::json!({
            "type": "PLANNER_RESPONSE",
            "tool_calls": [{
                "name": "write_file",
                "args": { "Path": core_path_json }
            }]
        });
        fs::write(&t3, format!("{}\n{}\n{}\n", turn1_line, sys_msg, write_line)).unwrap();
        let res3 = probe_ecl_shield(&t3, Some(&dir)).expect("write_file breaking symbol triggers Cleanup");
        assert!(res3.contains("[ECL Cleanup (core_boot):"));
        assert!(res3.contains("init_core"));
        assert!(!res3.contains("Must initialize logger before core"));

        let _ = fs::remove_dir_all(&dir);
    }
}
