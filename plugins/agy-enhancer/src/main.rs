mod config;
mod invariants;
mod posture;
mod sanitizer;

pub use config::*;
pub use invariants::*;
pub use posture::*;
pub use sanitizer::*;

use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

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
                println!("agy-enhancer v0.2.0 (Agent-Native Runtime Enhancer Core)");
                println!("Usage:");
                println!("  agy-enhancer                   Run as PreInvocation hook (reads stdin)");
                println!("  agy-enhancer --pre-edit        Run as PreToolUse edit_file auto-sanitizer/normalizer");
                println!("  agy-enhancer --config          Print currently resolved configuration");
                println!("  agy-enhancer --probe <file>    Probe token counts and invariants from transcript file");
                println!("  agy-enhancer --simulate <num>  Simulate semantic message for token count");
                println!("  agy-enhancer --memory [dir]    Inspect project epistemic memory radar");
                println!("  agy-enhancer --version         Show version");
                return;
            }
            "--version" | "-v" => {
                println!("agy-enhancer 0.2.0");
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
                    if cfg.slice_invariants.enabled {
                        if let Some(hint) = probe_recent_read_invariants_with_template(p, &cfg.slice_invariants.template) {
                            println!("Target Invariant: {}", hint);
                        }
                    }
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
            "--memory" => {
                let target_dir = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
                    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                let (cfg, _) = load_config(Some(&target_dir));
                let hint = probe_project_memory_with_config(Some(&target_dir), &cfg.memory_radar);
                println!("Inspecting Project Memory at: {}", target_dir.display());
                if let Some(h) = hint {
                    println!("Memory Radar Output: {}", h);
                } else {
                    println!("Memory Radar Output: (None - .agents/memory/ directory not present)");
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
    let mut message = build_semantic_message(used_tokens, &cfg);

    if cfg.slice_invariants.enabled {
        if let Some(hint) = probe_recent_read_invariants_with_template(transcript_path, &cfg.slice_invariants.template) {
            message.push(' ');
            message.push_str(&hint);
        }
    }

    if cfg.memory_radar.enabled {
        if let Some(mem_hint) = probe_project_memory_with_config(ws_root, &cfg.memory_radar) {
            message.push(' ');
            message.push_str(&mem_hint);
        }
    }

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
    use std::fs::File;

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

    #[test]
    fn test_sanitize_edit_target_blank_line_gap_in_file() {
        let file_content = "import json\n\nfrom fastapi import APIRouter\nfrom fastapi.responses import StreamingResponse\n";
        let missing_blank_target = "import json\nfrom fastapi import APIRouter\nfrom fastapi.responses import StreamingResponse";
        let aligned = sanitize_edit_target(missing_blank_target, file_content);
        assert_eq!(
            aligned,
            Some("import json\n\nfrom fastapi import APIRouter\nfrom fastapi.responses import StreamingResponse".to_string())
        );
    }

    #[test]
    fn test_sanitize_edit_target_blank_line_gap_in_target() {
        let file_content = "import json\nfrom fastapi import APIRouter\n";
        let extra_blank_target = "import json\n\nfrom fastapi import APIRouter";
        let aligned = sanitize_edit_target(extra_blank_target, file_content);
        assert_eq!(
            aligned,
            Some("import json\nfrom fastapi import APIRouter".to_string())
        );
    }

    #[test]
    fn test_sanitize_edit_target_multiple_blank_gaps_ambiguous() {
        let file_content = "def foo():\n    let a = 1;\n\n    let b = 2;\n\ndef bar():\n    let a = 1;\n\n    let b = 2;\n";
        let target = "    let a = 1;\n    let b = 2;";
        assert_eq!(sanitize_edit_target(target, file_content), None);
    }

    #[test]
    fn test_sanitize_edit_target_crlf_blank_line_tolerance() {
        let file_content = "import json\r\n\r\nfrom fastapi import APIRouter\r\n";
        let target = "import json\nfrom fastapi import APIRouter";
        let aligned = sanitize_edit_target(target, file_content);
        assert_eq!(
            aligned,
            Some("import json\r\n\r\nfrom fastapi import APIRouter".to_string())
        );
    }

    #[test]
    fn test_slice_invariants_real_world_case() {
        let content = "Showing lines 56-62 of 322\n56:    - Use spatial directional anchors\n57:    - [geo label] is transparent\n58:    - ONLY when a label sits directly on top of a solid line\n59:    - CRITICAL SYNTAX GUARD: NEVER put midway or sloped inside a standalone \\\\node at (...)!\n60:    - Angle Arcs: ALWAYS specify explicit physical units\n61: 3. Language & Character Set Discipline:\n62:    - ALL node labels, annotations ($A$, $B$, \\\\theta)";
        let inv = SliceInvariants::detect_from_read_output(content).expect("Should detect invariants");
        assert_eq!(inv.odd_indents, vec![3]);
        assert!(inv.has_double_backslashes);
        assert!(!inv.has_tabs);
        let hint = inv.format_hint();
        assert!(hint.contains("3-space indent"));
        assert!(hint.contains(r"\\ escape"));
    }

    #[test]
    fn test_slice_invariants_standard_clean_code() {
        let py_content = "Showing lines 10-15 of 100\n10: def compute_total(items):\n11:     total = 0\n12:     for item in items:\n13:         total += item.price\n14:     return total\n";
        assert_eq!(SliceInvariants::detect_from_read_output(py_content), None);

        let ts_content = "Showing lines 1-4 of 50\n1: export function add(a: number, b: number): number {\n2:   return a + b;\n3: }\n";
        assert_eq!(SliceInvariants::detect_from_read_output(ts_content), None);
    }

    #[test]
    fn test_slice_invariants_tabs_and_trailing() {
        let tab_content = "Showing lines 1-2 of 10\n1:\tfunc main() {\n2:\t\tprintln(\"hello\")\n";
        let inv = SliceInvariants::detect_from_read_output(tab_content).expect("Should detect tabs");
        assert!(inv.has_tabs);
        assert!(inv.format_hint().contains(r"tab (\t) indent"));

        let trailing_content = "Showing lines 1-2 of 10\n1:    line with trailing space   \n2:    normal line\n";
        let inv2 = SliceInvariants::detect_from_read_output(trailing_content).expect("Should detect trailing ws");
        assert!(inv2.has_trailing_ws);
        assert!(inv2.format_hint().contains("trailing whitespace"));
    }

    #[test]
    fn test_probe_project_memory_cases() {
        let temp_dir = env::temp_dir().join(format!("test_mem_radar_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Case 1: No .agents/memory dir -> None
        assert_eq!(probe_project_memory(Some(&temp_dir)), None);

        // Case 2: .agents/memory exists but empty
        let mem_dir = temp_dir.join(".agents").join("memory");
        std::fs::create_dir_all(&mem_dir).unwrap();
        let hint_ready = probe_project_memory(Some(&temp_dir)).expect("Should return ready hint");
        assert!(hint_ready.contains("Project Memory: ready"));

        // Case 3: pitfalls.md exists with 2 entries
        let pitfalls = mem_dir.join("pitfalls.md");
        std::fs::write(&pitfalls, "# Pitfalls\n\n- [Windows]: CRLF issue ➔ Normalize file\n- [Git]: Avoid rebase -i ➔ Soft reset\n").unwrap();
        let hint_active = probe_project_memory(Some(&temp_dir)).expect("Should return active hint");
        assert!(hint_active.contains("Project Memory: active"));
        assert!(hint_active.contains("2 entries"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
