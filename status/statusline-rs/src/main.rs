use serde::Deserialize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

// ─── ANSI Palette & Constants ────────────────────────────────────────────────
const R: &str = "\x1b[0m";
const B: &str = "\x1b[1m";

const FG_GRAY: &str = "\x1b[38;2;125;135;145m";
const FG_DIM_GRAY: &str = "\x1b[38;2;75;85;95m";
const FG_BRIGHT_WHITE: &str = "\x1b[38;2;240;240;240m";
const FG_SAGE: &str = "\x1b[38;2;145;175;155m";
const FG_OAT: &str = "\x1b[38;2;215;185;145m";
const FG_FOG_BLUE: &str = "\x1b[38;2;140;170;195m";
const FG_DUSTY_ROSE: &str = "\x1b[38;2;195;155;170m";
const FG_TERRACOTTA: &str = "\x1b[38;2;205;130;120m";
const FG_MUTED_CYAN: &str = "\x1b[38;2;135;175;180m";
const FG_LAVENDER: &str = "\x1b[38;2;175;160;190m";

// ─── Data Structures ─────────────────────────────────────────────────────────
#[allow(dead_code)]
#[derive(Deserialize, Default, Debug)]
struct Payload {
    agent_state: Option<String>,
    terminal_width: Option<u16>,
    context_window: Option<ContextWindow>,
    sandbox: Option<Sandbox>,
    artifact_count: Option<u32>,
    subagents: Option<serde_json::Value>,
    task_count: Option<u32>,
    model: Option<Model>,
    cwd: Option<String>,
    plan_tier: Option<String>,
    email: Option<String>,
    quota: Option<Quotas>,
    nerdfont: Option<bool>,
    classic: Option<bool>,
    theme: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
struct ContextWindow {
    used_percentage: Option<f64>,
    context_window_size: Option<u64>,
}

#[derive(Deserialize, Default, Debug)]
struct Sandbox {
    enabled: Option<bool>,
    allow_network: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
struct Model {
    id: Option<String>,
    display_name: Option<String>,
    effort: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Default, Debug)]
struct Quotas {
    #[serde(rename = "gemini-5h")]
    gemini_5h: Option<QuotaItem>,
    #[serde(rename = "gemini-weekly")]
    gemini_weekly: Option<QuotaItem>,
    #[serde(rename = "3p-5h")]
    tp_5h: Option<QuotaItem>,
    #[serde(rename = "3p-weekly")]
    tp_weekly: Option<QuotaItem>,
}

#[derive(Deserialize, Default, Debug)]
struct QuotaItem {
    remaining_fraction: Option<f64>,
    reset_in_seconds: Option<i64>,
}

// ─── Logger ──────────────────────────────────────────────────────────────────
fn log_diagnostic(msg: &str) {
    if let Ok(temp) = env::var("TEMP") {
        let log_path = PathBuf::from(temp).join("antigravity-statusline.log");
        let now = chrono_or_fallback();
        let pid = std::process::id();
        let line = format!("[{}] [PID {}] {}\r\n", now, pid, msg);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = f.write_all(line.as_bytes());
        }
    }
}

fn chrono_or_fallback() -> String {
    // Windows timestamp without chrono dependency
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond, st.wMilliseconds
        )
    }
}

// ─── Win32 Non-Blocking Stdin Reader ─────────────────────────────────────────
fn read_stdin_safely() -> String {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{GetStdHandle, STD_INPUT_HANDLE};
        use windows_sys::Win32::System::Pipes::PeekNamedPipe;

        unsafe {
            let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
            if h_stdin != std::ptr::null_mut() && h_stdin != -1isize as _ {
                let mut avail: u32 = 0;
                let mut got_bytes = false;
                for _ in 0..12 {
                    if PeekNamedPipe(h_stdin, std::ptr::null_mut(), 0, std::ptr::null_mut(), &mut avail, std::ptr::null_mut()) != 0 {
                        if avail > 0 {
                            got_bytes = true;
                            break;
                        }
                    } else {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                if got_bytes {
                    let mut buffer = String::new();
                    if io::stdin().read_to_string(&mut buffer).is_ok() {
                        return buffer;
                    }
                } else if avail == 0 {
                    // No data in pipe buffer
                    return String::new();
                }
            }
        }
    }

    let mut buf = String::new();
    let _ = io::stdin().read_to_string(&mut buf);
    buf
}

// ─── Probes: Git & Power ─────────────────────────────────────────────────────
fn find_git_dir(start: &Path) -> Option<PathBuf> {
    let mut curr = start;
    loop {
        let candidate = curr.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if candidate.is_file() {
            if let Ok(content) = fs::read_to_string(&candidate) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(target) = trimmed.strip_prefix("gitdir:") {
                        let p = target.trim();
                        let target_path = if Path::new(p).is_relative() {
                            curr.join(p)
                        } else {
                            PathBuf::from(p)
                        };
                        if target_path.is_dir() {
                            return Some(target_path);
                        }
                    }
                }
            }
        }
        match curr.parent() {
            Some(parent) if parent != curr => curr = parent,
            _ => break,
        }
    }
    None
}

fn get_git_branch(cwd: &str) -> String {
    let start_path = if cwd.is_empty() {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        PathBuf::from(cwd)
    };

    if let Some(git_dir) = find_git_dir(&start_path) {
        let head_path = git_dir.join("HEAD");
        if let Ok(head_content) = fs::read_to_string(head_path) {
            let trimmed = head_content.trim();
            if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                return branch.trim().to_string();
            }
            if trimmed.len() >= 7 {
                return trimmed[..7].to_string();
            }
        }
    }
    String::new()
}

struct PowerInfo {
    percent: u8,
    is_ac: bool,
}

fn get_power_status() -> Option<PowerInfo> {
    if env::var("ANTIGRAVITY_STATUS_NO_POWER").unwrap_or_default() == "1" {
        return None;
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        unsafe {
            let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
            if GetSystemPowerStatus(&mut status) != 0 {
                if status.BatteryLifePercent <= 100 {
                    return Some(PowerInfo {
                        percent: status.BatteryLifePercent,
                        is_ac: status.ACLineStatus == 1,
                    });
                }
            }
        }
    }
    None
}

// ─── Formatters & Layout ─────────────────────────────────────────────────────
fn human_format(num: u64) -> String {
    if num == 0 {
        return "0".to_string();
    }
    if num >= 1_000_000 {
        let val = (num as f64) / 1_000_000.0;
        return format!("{:.1}M", val);
    }
    if num >= 1_000 {
        let val = (num as f64) / 1_000.0;
        return format!("{:.1}K", val);
    }
    num.to_string()
}

fn format_reset_time(seconds: i64, compact: bool) -> String {
    if seconds <= 0 {
        return String::new();
    }
    let days = seconds / 86400;
    let rem = seconds % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let mins = rem / 60;

    if compact {
        if days > 0 {
            if hours > 0 {
                return format!("{}d{}h", days, hours);
            }
            return format!("{}d", days);
        }
        if hours > 0 {
            if mins > 0 {
                return format!("{}h{}m", hours, mins);
            }
            return format!("{}h", hours);
        }
        if mins > 0 {
            return format!("{}m", mins);
        }
        "<1m".to_string()
    } else {
        if days > 0 {
            if hours > 0 {
                return format!("{}d {}h", days, hours);
            }
            return format!("{}d", days);
        }
        if hours > 0 {
            if mins > 0 {
                return format!("{}h {}m", hours, mins);
            }
            return format!("{}h", hours);
        }
        if mins > 0 {
            return format!("{}m", mins);
        }
        "<1m".to_string()
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return chars.iter().take(max_len).collect();
    }
    let prefix: String = chars.iter().take(max_len - 3).collect();
    format!("{}...", prefix)
}

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }

        let u = c as u32;
        if (0xfe00..=0xfe0f).contains(&u) {
            continue;
        }
        if (0x4e00..=0x9fff).contains(&u)
            || (0x3000..=0x303f).contains(&u)
            || (0xff00..=0xffef).contains(&u)
            || u == 0x231b
            || (0x1f000..=0x1faff).contains(&u)
            || (0x1f300..=0x1f9ff).contains(&u)
        {
            len += 2;
        } else {
            len += 1;
        }
    }
    len
}

fn strip_separator(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut in_esc = false;

    // Phase 1: skip leading whitespace and ANSI sequences before separator
    while i < chars.len() {
        let c = chars[i];
        if c == '\x1b' {
            in_esc = true;
            i += 1;
            continue;
        }
        if in_esc {
            if c == 'm' {
                in_esc = false;
            }
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        break;
    }

    // Phase 2: check if current char is a separator char
    if i < chars.len() && (chars[i] == '|' || chars[i] == '/' || chars[i] == '·' || chars[i] == '╱') {
        i += 1;

        // Phase 3: skip trailing whitespace and ANSI sequences after separator
        in_esc = false;
        while i < chars.len() {
            let c = chars[i];
            if c == '\x1b' {
                in_esc = true;
                i += 1;
                continue;
            }
            if in_esc {
                if c == 'm' {
                    in_esc = false;
                }
                i += 1;
                continue;
            }
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            break;
        }
        return chars[i..].iter().collect();
    }

    s.to_string()
}

fn has_separator(s: &str) -> bool {
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
            continue;
        }
        if in_esc {
            if c == 'm' {
                in_esc = false;
            }
            continue;
        }
        if c.is_whitespace() {
            continue;
        }
        return c == '|' || c == '/' || c == '·' || c == '╱';
    }
    false
}

fn format_flex_wrap_line(
    left_items: &[String],
    right_items: &[String],
    total_width: usize,
    dot_sep: &str,
) -> Vec<String> {
    let max_content = total_width.saturating_sub(4).max(1);

    let mut left_str = String::new();
    let mut is_first_left = true;
    for item in left_items {
        if !item.is_empty() && visible_len(item) > 0 {
            if is_first_left {
                left_str.push_str(&strip_separator(item));
                is_first_left = false;
            } else {
                left_str.push_str(item);
            }
        }
    }

    let mut right_str = String::new();
    for item in right_items {
        if !item.is_empty() && visible_len(item) > 0 {
            right_str.push_str(item);
        }
    }

    let left_vis = visible_len(&left_str);
    let right_vis = visible_len(&right_str);
    let min_gap = if !left_str.is_empty() && !right_str.is_empty() { 1 } else { 0 };

    if left_vis + right_vis + min_gap <= max_content {
        let pad = max_content.saturating_sub(left_vis + right_vis);
        let spaces = " ".repeat(pad);
        return vec![format!(
            "{}│{} {}{}{}{} │{}",
            FG_GRAY, R, left_str, spaces, right_str, FG_GRAY, R
        )];
    }

    // Wrap mode: flatten items into a single sequential flow
    let mut all_items = Vec::new();
    for item in left_items {
        if !item.is_empty() && visible_len(item) > 0 {
            all_items.push(item.clone());
        }
    }
    let mut is_first_right = true;
    for item in right_items {
        if !item.is_empty() && visible_len(item) > 0 {
            let mut proc = item.clone();
            if is_first_right {
                if !has_separator(&proc) {
                    proc = format!("{}{}", dot_sep, proc);
                }
                is_first_right = false;
            }
            all_items.push(proc);
        }
    }

    let mut lines = Vec::new();
    let mut cur_items = Vec::new();
    let mut cur_vis = 0;

    for item in all_items {
        let is_first = cur_items.is_empty();
        let mut proc_item = if is_first { strip_separator(&item) } else { item };
        let mut item_vis = visible_len(&proc_item);

        if item_vis > max_content {
            proc_item = truncate_string(&proc_item, max_content);
            item_vis = visible_len(&proc_item);
        }

        if cur_vis + item_vis <= max_content {
            cur_items.push(proc_item);
            cur_vis += item_vis;
        } else {
            if !cur_items.is_empty() {
                let pad = max_content.saturating_sub(cur_vis);
                let spaces = " ".repeat(pad);
                let line_content = cur_items.join("");
                lines.push(format!(
                    "{}│{} {}{}{} │{}",
                    FG_GRAY, R, line_content, spaces, FG_GRAY, R
                ));
            }
            let mut stripped = strip_separator(&proc_item);
            let mut stripped_vis = visible_len(&stripped);
            if stripped_vis > max_content {
                stripped = truncate_string(&stripped, max_content);
                stripped_vis = visible_len(&stripped);
            }
            cur_vis = stripped_vis;
            cur_items = vec![stripped];
        }
    }

    if !cur_items.is_empty() {
        let pad = max_content.saturating_sub(cur_vis);
        let spaces = " ".repeat(pad);
        let line_content = cur_items.join("");
        lines.push(format!(
            "{}│{} {}{}{} │{}",
            FG_GRAY, R, line_content, spaces, FG_GRAY, R
        ));
    }

    lines
}

fn build_context_bar(used_pct: f64, classic: bool) -> String {
    let bar_len = 10;
    let pct_int = used_pct.floor() as usize;
    let filled = (pct_int * bar_len) / 100;
    let remainder = (pct_int * bar_len) % 100;

    let fill_color = if pct_int >= 90 {
        FG_TERRACOTTA
    } else if pct_int >= 60 {
        FG_OAT
    } else {
        FG_MUTED_CYAN
    };

    let icon = if classic { "ctx" } else { "󱍏" };
    let pct_fmt = format!("{:.1}%", used_pct);

    let mut bar = String::new();
    if classic {
        for i in 0..bar_len {
            if i < filled {
                bar.push('#');
            } else if i == filled && remainder >= 50 {
                bar.push('+');
            } else {
                bar.push('-');
            }
        }
        format!(
            "{}{}{} [{}{}{}] {}{}{}{}",
            FG_OAT, icon, R, fill_color, bar, R, FG_BRIGHT_WHITE, B, pct_fmt, R
        )
    } else {
        for i in 0..bar_len {
            if i < filled {
                bar.push_str(&format!("{}█{}", fill_color, R));
            } else if i == filled {
                if remainder >= 75 {
                    bar.push_str(&format!("{}▓{}{}", fill_color, R, FG_DIM_GRAY));
                } else if remainder >= 50 {
                    bar.push_str(&format!("{}▒{}{}", fill_color, R, FG_DIM_GRAY));
                } else {
                    bar.push_str(&format!("{}░{}{}", fill_color, R, FG_DIM_GRAY));
                }
            } else {
                bar.push_str(&format!("{}░{}", FG_DIM_GRAY, R));
            }
        }
        format!(
            "{}{} {}{} {}{}{}{}",
            FG_OAT, icon, R, bar, FG_BRIGHT_WHITE, B, pct_fmt, R
        )
    }
}

fn build_quota_bar(
    val: f64,
    label: &str,
    bar_color: &str,
    reset_sec: i64,
    target_bar_len: usize,
    compact: bool,
    classic: bool,
) -> String {
    let sep = if classic {
        format!("{} · {}", FG_GRAY, R)
    } else {
        format!("{} | {}", FG_GRAY, R)
    };

    if val < 0.0 {
        let mut bar = String::new();
        for _ in 0..target_bar_len {
            if classic {
                bar.push('·');
            } else {
                bar.push('░');
            }
        }
        return format!(
            "{}{}{}{}{} {}{}{} N/A{}",
            sep, FG_BRIGHT_WHITE, B, label, R, FG_GRAY, bar, R, R
        );
    }

    let val_int = val.floor() as usize;
    let text_color = if val_int < 20 {
        FG_TERRACOTTA
    } else if val_int < 50 {
        FG_OAT
    } else {
        FG_SAGE
    };

    let filled = (val_int * target_bar_len) / 100;
    let remainder = (val_int * target_bar_len) % 100;

    let mut bar = String::new();
    if classic {
        for i in 0..target_bar_len {
            if i < filled {
                bar.push('█');
            } else if i == filled {
                if remainder >= 75 {
                    bar.push('▓');
                } else if remainder >= 50 {
                    bar.push('▒');
                } else if remainder >= 25 {
                    bar.push('░');
                } else {
                    bar.push('·');
                }
            } else {
                bar.push('·');
            }
        }
    } else {
        for i in 0..target_bar_len {
            if i < filled {
                bar.push_str(&format!("{}█{}", bar_color, R));
            } else if i == filled {
                if remainder >= 75 {
                    bar.push_str(&format!("{}▓{}{}", bar_color, R, FG_DIM_GRAY));
                } else if remainder >= 50 {
                    bar.push_str(&format!("{}▒{}{}", bar_color, R, FG_DIM_GRAY));
                } else if remainder >= 25 {
                    bar.push_str(&format!("{}░{}{}", bar_color, R, FG_DIM_GRAY));
                } else {
                    bar.push_str(&format!("{}░{}", FG_DIM_GRAY, R));
                }
            } else {
                bar.push_str(&format!("{}░{}", FG_DIM_GRAY, R));
            }
        }
    }

    let val_fmt = if compact {
        format!("{}", val_int)
    } else {
        format!("{:.1}", val)
    };

    let reset_str = if reset_sec > 0 {
        let t = format_reset_time(reset_sec, compact);
        if !t.is_empty() {
            if classic {
                format!(" ~{}", t)
            } else if compact {
                format!(" 󰔟{}", t)
            } else {
                format!(" 󰔟 {}", t)
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if classic {
        format!(
            "{}{}{}{}{} {}{}{} {}{}%{}{}",
            sep, FG_BRIGHT_WHITE, B, label, R, bar_color, bar, R, text_color, val_fmt, R, reset_str
        )
    } else {
        format!(
            "{}{}{}{}{} {} {}{}%{}{}",
            sep, FG_BRIGHT_WHITE, B, label, R, bar, text_color, val_fmt, R, reset_str
        )
    }
}

// ─── Main Entry Point ────────────────────────────────────────────────────────
fn main() {
    let args: Vec<String> = env::args().collect();

    // Check legend flag
    if args.iter().any(|a| a == "--legend" || a == "-l" || a == "legend") {
        println!("Antigravity Statusline Legend");
        println!("READY     Agent is idle");
        println!("THINKING  Agent is reasoning");
        println!("WORKING   Agent is executing");
        println!("TOOL      Tool call is active");
        println!("CTX       Context window usage");
        println!("5H / 7D   Quota remaining and reset time");
        return;
    }

    let raw_input = read_stdin_safely();
    let cleaned_input = raw_input.trim().trim_start_matches('\u{feff}');

    let payload: Payload = if !cleaned_input.is_empty() {
        match serde_json::from_str(cleaned_input) {
            Ok(p) => p,
            Err(e) => {
                log_diagnostic(&format!("JSON deserialization error: {}", e));
                Payload::default()
            }
        }
    } else {
        Payload::default()
    };

    let use_classic = args.iter().any(|a| {
        a == "--classic" || a == "--no-nerdfont" || a == "--compatibility"
    }) || payload.classic.unwrap_or(false)
        || payload.nerdfont == Some(false)
        || payload.theme.as_deref() == Some("classic");

    let dot_l1 = if use_classic {
        format!("{} ╱ {}", FG_GRAY, R)
    } else {
        format!("{} | {}", FG_GRAY, R)
    };
    let dot_l2 = if use_classic {
        format!("{} · {}", FG_GRAY, R)
    } else {
        format!("{} | {}", FG_GRAY, R)
    };

    let cols = payload.terminal_width.unwrap_or(80).clamp(20, 400) as usize;
    let width = cols;

    let (max_user_len, max_model_len, max_branch_len): (usize, usize, usize) = if cols < 65 {
        (12, 14, 14)
    } else if cols < 85 {
        (18, 20, 20)
    } else if cols < 110 {
        (24, 28, 25)
    } else {
        (35, 35, 30)
    };

    // State
    let raw_state = payload.agent_state.as_deref().unwrap_or("idle");
    let state_icon = match raw_state {
        "idle" => if use_classic { "●" } else { "" },
        "thinking" => if use_classic { "◆" } else { "󰟷" },
        "working" => if use_classic { "⚙" } else { "" },
        "tool_use" => if use_classic { "🔧" } else { "" },
        _ => if use_classic { "⏳" } else { "" },
    };

    let state_display = match raw_state {
        "idle" => format!("{}{}{} READY{}", FG_SAGE, B, state_icon, R),
        "thinking" => format!("{}{}{} THINKING{}", FG_OAT, B, state_icon, R),
        "working" => format!("{}{}{} WORKING{}", FG_FOG_BLUE, B, state_icon, R),
        "tool_use" => format!("{}{}{} TOOL{}", FG_DUSTY_ROSE, B, state_icon, R),
        s => format!("{}{}{} {}{}", FG_BRIGHT_WHITE, B, state_icon, s.to_uppercase(), R),
    };

    // Git Branch
    let cwd = payload.cwd.as_deref().unwrap_or("");
    let branch = get_git_branch(cwd);
    let vcs_fmt = if !branch.is_empty() {
        let trunc_b = truncate_string(&branch, max_branch_len);
        if use_classic {
            format!("{}{}{}{}", dot_l1, FG_MUTED_CYAN, trunc_b, R)
        } else {
            let icon_vcs = "";
            format!("{}{}{}{} {}{}", dot_l1, FG_MUTED_CYAN, icon_vcs, R, trunc_b, R)
        }
    } else {
        String::new()
    };

    // Model
    let model_disp = payload
        .model
        .as_ref()
        .and_then(|m| m.display_name.clone().or_else(|| m.id.clone()))
        .unwrap_or_default();
    let model_effort = payload.model.as_ref().and_then(|m| m.effort.clone()).unwrap_or_default();
    let model_fmt = if !model_disp.is_empty() {
        let disp = if !model_effort.is_empty() && !model_disp.to_lowercase().contains(&model_effort.to_lowercase()) {
            let cap_effort = format!(" ({})", model_effort);
            format!(
                "{}{}",
                truncate_string(&model_disp, max_model_len.saturating_sub(cap_effort.len()).max(1)),
                cap_effort
            )
        } else {
            truncate_string(&model_disp, max_model_len)
        };
        let icon_model = if use_classic { "" } else { " " };
        format!("{}{}{}{}{}{}", dot_l1, FG_LAVENDER, icon_model, disp, R, "")
    } else {
        String::new()
    };

    // Account / Plan tier
    let plan = payload.plan_tier.as_deref().unwrap_or("");
    let email = payload.email.as_deref().unwrap_or("");
    let user_fmt = if !plan.is_empty() || !email.is_empty() {
        let user_info = if !plan.is_empty() && !email.is_empty() {
            format!("{} ({})", plan, email)
        } else if !plan.is_empty() {
            plan.to_string()
        } else {
            email.to_string()
        };
        let trunc_u = truncate_string(&user_info, max_user_len);
        if use_classic {
            format!("{}{}{}{}", dot_l1, FG_GRAY, trunc_u, R)
        } else {
            let icon_email = "󰇮 ";
            format!("{}{}{}{}{}", dot_l1, FG_GRAY, icon_email, trunc_u, R)
        }
    } else {
        String::new()
    };

    // Context Window
    let used_pct = payload
        .context_window
        .as_ref()
        .and_then(|c| c.used_percentage)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    let ctx_limit = payload
        .context_window
        .as_ref()
        .and_then(|c| c.context_window_size)
        .unwrap_or(0);
    let ctx_bar = build_context_bar(used_pct, use_classic);

    let tok_details_wide = if ctx_limit > 0 {
        let ctx_used = ((ctx_limit as f64) * used_pct / 100.0).round() as u64;
        format!(
            "{}~{}{}{} / {} tokens{}",
            FG_GRAY,
            FG_BRIGHT_WHITE,
            human_format(ctx_used),
            R,
            human_format(ctx_limit),
            R
        )
    } else {
        String::new()
    };

    // Quotas
    let is_gemini_model = model_disp.to_lowercase().contains("gemini")
        || payload
            .model
            .as_ref()
            .and_then(|m| m.id.as_ref())
            .map(|id| id.to_lowercase().contains("gemini"))
            .unwrap_or(false);

    let q_gem_5h = payload.quota.as_ref().and_then(|q| q.gemini_5h.as_ref());
    let q_gem_wk = payload.quota.as_ref().and_then(|q| q.gemini_weekly.as_ref());
    let has_gemini_quota = q_gem_5h.and_then(|q| q.remaining_fraction).is_some()
        || q_gem_wk.and_then(|q| q.remaining_fraction).is_some();

    let q_tp_5h = payload.quota.as_ref().and_then(|q| q.tp_5h.as_ref());
    let q_tp_wk = payload.quota.as_ref().and_then(|q| q.tp_weekly.as_ref());
    let has_tp_quota = q_tp_5h.and_then(|q| q.remaining_fraction).is_some()
        || q_tp_wk.and_then(|q| q.remaining_fraction).is_some();

    let (active_5h, active_wk) = if is_gemini_model {
        if has_gemini_quota {
            (q_gem_5h, q_gem_wk)
        } else if has_tp_quota {
            (q_tp_5h, q_tp_wk)
        } else {
            (None, None)
        }
    } else if has_tp_quota {
        (q_tp_5h, q_tp_wk)
    } else if has_gemini_quota {
        (q_gem_5h, q_gem_wk)
    } else {
        (None, None)
    };

    let is_compact_bars = cols < 95;
    let q_bar_len = if cols < 95 { 5 } else { 8 };

    let q_5h_fmt = if let Some(q) = active_5h {
        if let Some(frac) = q.remaining_fraction {
            let val = (frac * 100.0).clamp(0.0, 100.0);
            let reset_sec = q.reset_in_seconds.unwrap_or(0);
            build_quota_bar(val, "5H", FG_MUTED_CYAN, reset_sec, q_bar_len, is_compact_bars, use_classic)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let q_wk_fmt = if cols < 75 && !q_5h_fmt.is_empty() {
        String::new()
    } else if let Some(q) = active_wk {
        if let Some(frac) = q.remaining_fraction {
            let val = (frac * 100.0).clamp(0.0, 100.0);
            let reset_sec = q.reset_in_seconds.unwrap_or(0);
            build_quota_bar(val, "7D", FG_DUSTY_ROSE, reset_sec, q_bar_len, is_compact_bars, use_classic)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Power
    let power_fmt = if let Some(power) = get_power_status() {
        if power.is_ac {
            let icon_ac = if use_classic { "AC" } else { "󰚥 AC" };
            format!("{}{}{}{}", dot_l2, FG_SAGE, icon_ac, R)
        } else {
            let icon_bat = if use_classic { "BAT" } else { "🔋" };
            format!("{}{}{}{}: {}%{}", dot_l2, FG_OAT, icon_bat, R, power.percent, R)
        }
    } else {
        String::new()
    };

    // Sandbox
    let sb_fmt = if let Some(ref sb) = payload.sandbox {
        if sb.enabled.unwrap_or(false) {
            let net = sb.allow_network.unwrap_or(false);
            if use_classic {
                format!("{}sandbox {}{}", FG_SAGE, if net { "net" } else { "on" }, R)
            } else {
                let icon_sb = if net { "󰒙" } else { "󰴴" };
                format!("{}{} {}{}", FG_SAGE, icon_sb, if net { "net" } else { "on" }, R)
            }
        } else if use_classic {
            format!("{}sandbox off{}", FG_GRAY, R)
        } else {
            let icon_off = "󰦜";
            format!("{}{} off{}", FG_TERRACOTTA, icon_off, R)
        }
    } else if use_classic {
        format!("{}sandbox off{}", FG_GRAY, R)
    } else {
        let icon_off = "󰦜";
        format!("{}{} off{}", FG_TERRACOTTA, icon_off, R)
    };

    // Artifacts, Subagents, Tasks
    let item_sep = if cols < 90 { " ".to_string() } else { dot_l2.clone() };

    let artifacts = payload.artifact_count.unwrap_or(0);
    let art_fmt = if artifacts > 0 {
        if use_classic {
            let label = if cols < 90 { "art:" } else { "artifacts " };
            format!("{}{}{}{}{}{}", item_sep, FG_GRAY, label, FG_BRIGHT_WHITE, artifacts, R)
        } else {
            let icon_art = "";
            format!("{}{}{}{} {}{}{}{}", item_sep, FG_FOG_BLUE, icon_art, R, FG_BRIGHT_WHITE, B, artifacts, R)
        }
    } else {
        String::new()
    };

    let subagents = match &payload.subagents {
        Some(serde_json::Value::Array(arr)) => arr.len() as u32,
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
        _ => 0,
    };
    let sub_fmt = if subagents > 0 {
        if use_classic {
            let label = if cols < 90 { "sub:" } else { "subagents " };
            format!("{}{}{}{}{}{}", item_sep, FG_GRAY, label, FG_BRIGHT_WHITE, subagents, R)
        } else {
            let icon_sub = "󱙺";
            format!("{}{}{}{} {}{}{}{}", item_sep, FG_MUTED_CYAN, icon_sub, R, FG_BRIGHT_WHITE, B, subagents, R)
        }
    } else {
        String::new()
    };

    let tasks = payload.task_count.unwrap_or(0);
    let task_fmt = if tasks > 0 {
        if use_classic {
            let label = if cols < 90 { "task:" } else { "tasks " };
            format!("{}{}{}{}{}{}", item_sep, FG_GRAY, label, FG_BRIGHT_WHITE, tasks, R)
        } else {
            let icon_task = "";
            format!("{}{}{}{} {}{}{}{}", item_sep, FG_DUSTY_ROSE, icon_task, R, FG_BRIGHT_WHITE, B, tasks, R)
        }
    } else {
        String::new()
    };

    // Assemble Rows
    let l1_left = vec![state_display, model_fmt, vcs_fmt];
    let l1_right = vec![user_fmt];

    let l2_left = vec![ctx_bar];
    let l2_right = vec![tok_details_wide];

    let mut l3_left = Vec::new();
    if !q_5h_fmt.is_empty() { l3_left.push(q_5h_fmt); }
    if !q_wk_fmt.is_empty() { l3_left.push(q_wk_fmt); }
    if !power_fmt.is_empty() { l3_left.push(power_fmt); }

    let mut l3_right = vec![sb_fmt];
    if !art_fmt.is_empty() { l3_right.push(art_fmt); }
    if !sub_fmt.is_empty() { l3_right.push(sub_fmt); }
    if !task_fmt.is_empty() { l3_right.push(task_fmt); }

    let title = if width < 40 { " Status " } else if width < 70 { " Antigravity " } else { " Antigravity Dashboard " };
    let border_dash_len = width.saturating_sub(4 + visible_len(title));
    let top_border = format!(
        "{}╭─{}{}{}─{}╮{}",
        FG_GRAY, R, title, FG_GRAY, "─".repeat(border_dash_len), R
    );
    let bot_border = format!(
        "{}╰─{}─╯{}",
        FG_GRAY, "─".repeat(width.saturating_sub(4)), R
    );

    let row1 = format_flex_wrap_line(&l1_left, &l1_right, width, &dot_l1).join("\n");
    let row2 = format_flex_wrap_line(&l2_left, &l2_right, width, &dot_l2).join("\n");
    let row3 = format_flex_wrap_line(&l3_left, &l3_right, width, &dot_l2).join("\n");

    println!("{}\n{}\n{}\n{}\n{}", top_border, row1, row2, row3, bot_border);
}
