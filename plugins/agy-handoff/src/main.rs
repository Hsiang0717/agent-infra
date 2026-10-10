mod clipboard;
mod session;
mod transcript;

use std::io::{self, Read};

fn print_help() {
    println!(
        r#"agy-handoff: Antigravity Session Handoff & Clipboard Tool

USAGE:
    agy-handoff [OPTIONS]

MODES:
    (default)               Extract the latest assistant response from current session
                            and copy directly to the system clipboard.

    --set <TEXT>, -c <TEXT> Write the provided text directly to the clipboard.
                            (Used by /handoff dynamic agent summary)

    --stdin                 Read text from standard input and write to clipboard.
                            Example: cat memo.md | agy-handoff --stdin

OPTIONS:
    --session, -s <ID>      Specify an explicit session ID instead of auto-locating.
    --print, -p             Print the content to stdout in addition to clipboard.
    --help, -h              Show this help message.
"#
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut explicit_session: Option<String> = None;
    let mut explicit_text: Option<String> = None;
    let mut read_stdin = false;
    let mut print_stdout = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--session" | "-s" => {
                if i + 1 < args.len() {
                    explicit_session = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: --session requires a session UUID argument.");
                    std::process::exit(1);
                }
            }
            "--set" | "-c" => {
                if i + 1 < args.len() {
                    explicit_text = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: --set requires a text argument.");
                    std::process::exit(1);
                }
            }
            "--stdin" => {
                read_stdin = true;
            }
            "--print" | "-p" => {
                print_stdout = true;
            }
            unknown => {
                eprintln!("Unknown argument: {}", unknown);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Branch 1: Stdin input
    if read_stdin {
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        }

        if let Err(e) = clipboard::copy_to_clipboard(&buffer) {
            eprintln!("Error copying to clipboard: {}", e);
            std::process::exit(1);
        }

        if print_stdout {
            print!("{}", buffer);
        }

        println!(
            "✓ [agy-handoff] Copied stdin text to clipboard ({} chars)",
            buffer.chars().count()
        );
        return;
    }

    // Branch 2: Explicit text provided via --set
    if let Some(text) = explicit_text {
        if let Err(e) = clipboard::copy_to_clipboard(&text) {
            eprintln!("Error copying to clipboard: {}", e);
            std::process::exit(1);
        }

        if print_stdout {
            print!("{}", text);
        }

        println!(
            "✓ [agy-handoff] Copied provided text to clipboard ({} chars)",
            text.chars().count()
        );
        return;
    }

    // Branch 3: Extract from session transcript (Default Mode)
    let session_res = match session::resolve_session_id(explicit_session) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error resolving session ID: {}", err);
            std::process::exit(1);
        }
    };

    let extracted = match transcript::get_last_planner_response(
        &session_res.app_data_dir,
        &session_res.session_id,
    ) {
        Ok(ext) => ext,
        Err(err) => {
            eprintln!("Error reading transcript: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(e) = clipboard::copy_to_clipboard(&extracted.content) {
        eprintln!("Error writing to clipboard: {}", e);
        std::process::exit(1);
    }

    if print_stdout {
        print!("{}", extracted.content);
    }

    println!(
        "✓ [agy-handoff] Copied raw response to clipboard!\n  Session: {} ({})\n  Step: #{} | Size: {} chars",
        session_res.session_id,
        session_res.source,
        extracted.step_index,
        extracted.content.chars().count()
    );
}
