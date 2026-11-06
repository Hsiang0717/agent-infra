use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

mod db;
mod matcher;
mod prober;
mod tui;

use db::Database;
use prober::Prober;

#[derive(Parser)]
#[command(
    name = "smart-hub",
    about = "Context-Aware Smart Command & History Hub for PowerShell",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI search and return chosen command to stdout
    Search {
        /// Working directory to contextualize search
        #[arg(short, long)]
        cwd: Option<String>,

        /// Initial search query
        #[arg(short, long)]
        query: Option<String>,
    },

    /// Record a command execution into SQLite
    Record {
        /// The command line executed
        #[arg(short, long)]
        command: String,

        /// Working directory where the command was executed
        #[arg(short, long)]
        cwd: Option<String>,

        /// Exit code of the command
        #[arg(short, long)]
        exit_code: Option<i32>,

        /// Duration of execution in milliseconds
        #[arg(short, long)]
        duration_ms: Option<i64>,
    },

    /// Immediately delete the last executed command from history
    DeleteLast,

    /// Probe project context and detected actions in the given directory
    Probe {
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },

    /// Manage custom virtual aliases
    Alias {
        #[command(subcommand)]
        sub: AliasCommands,
    },

    /// Print PowerShell integration script for KeyBindings/PSReadLine
    Init,
}

#[derive(Subcommand)]
enum AliasCommands {
    /// Add or update a virtual alias
    Add {
        /// Trigger word/keyword (e.g. 'g', 'dcup', 'skill開發')
        trigger: String,

        /// Expanded command (e.g. 'git status', 'docker compose up -d')
        expansion: String,

        /// Optional description
        #[arg(short, long)]
        desc: Option<String>,

        /// Optional category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Remove a virtual alias
    Remove {
        /// Trigger word
        trigger: String,
    },

    /// List all registered virtual aliases
    List,
}

fn main() {
    let cli = Cli::parse();
    let db = match Database::open_default() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("smart-hub database error: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Search { cwd, query } => {
            let current_dir = cwd.unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .to_string_lossy()
                    .to_string()
            });

            match tui::run_tui(&db, &current_dir, query.as_deref()) {
                Ok(Some(selected_cmd)) => {
                    // Output the selected command to stdout for PowerShell to capture
                    print!("{}", selected_cmd);
                    std::process::exit(0);
                }
                Ok(None) => {
                    // Cancelled by user (Esc / Ctrl+C)
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("TUI error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Record {
            command,
            cwd,
            exit_code,
            duration_ms,
        } => {
            let current_dir = cwd.unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .to_string_lossy()
                    .to_string()
            });
            let git_root = Prober::probe(std::path::Path::new(&current_dir))
                .git_root
                .map(|p| p.to_string_lossy().to_string());

            if let Err(e) = db.record(
                &command,
                &current_dir,
                git_root.as_deref(),
                exit_code,
                duration_ms,
            ) {
                eprintln!("Failed to record command: {}", e);
            }
        }

        Commands::DeleteLast => match db.delete_last() {
            Ok(Some(cmd)) => {
                println!("✓ 已刪除上一條歷史記錄: {}", cmd);
            }
            Ok(None) => {
                println!("歷史記錄為空");
            }
            Err(e) => {
                eprintln!("刪除失敗: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Probe { cwd } => {
            let current_dir = cwd.unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .to_string_lossy()
                    .to_string()
            });
            let ctx = Prober::probe(std::path::Path::new(&current_dir));
            println!("Project Root: {:?}", ctx.root);
            println!("Git Root: {:?}", ctx.git_root);
            println!("Type: {:?}", ctx.project_type);
            println!("Detected Actions:");
            for act in ctx.actions {
                println!("  - [{}] {} ➔ {}", act.source, act.label, act.command);
            }
        }

        Commands::Alias { sub } => match sub {
            AliasCommands::Add {
                trigger,
                expansion,
                desc,
                category,
            } => {
                if let Err(e) = db.add_alias(
                    &trigger,
                    &expansion,
                    desc.as_deref(),
                    category.as_deref(),
                ) {
                    eprintln!("新增別名失敗: {}", e);
                    std::process::exit(1);
                }
                println!("✓ 已保存虛別名: {} ➔ {}", trigger, expansion);
            }
            AliasCommands::Remove { trigger } => match db.remove_alias(&trigger) {
                Ok(true) => println!("✓ 已刪除別名: {}", trigger),
                Ok(false) => println!("找不到別名: {}", trigger),
                Err(e) => {
                    eprintln!("刪除別名失敗: {}", e);
                    std::process::exit(1);
                }
            },
            AliasCommands::List => match db.get_all_aliases() {
                Ok(list) => {
                    if list.is_empty() {
                        println!("尚未註冊任何虛別名。使用 `smart-hub alias add <trigger> <expansion>` 新增。");
                    } else {
                        println!("=== 註冊的虛別名 (Virtual Aliases) ===");
                        for a in list {
                            let desc = a
                                .description
                                .map(|d| format!(" ({})", d))
                                .unwrap_or_default();
                            println!("  {:15} ➔ {}{}", a.trigger, a.expansion, desc);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("讀取別名失敗: {}", e);
                    std::process::exit(1);
                }
            },
        },

        Commands::Init => {
            print!(
                r#"# Smart-Hub KeyBindings Integration for PowerShell
$script:SmartHubExe = (Get-Command smart-hub -ErrorAction SilentlyContinue)?.Source
if (-not $script:SmartHubExe) {{
    $script:SmartHubExe = "$PSScriptRoot\engine\target\release\smart-hub.exe"
}}

if (Test-Path $script:SmartHubExe) {{
    # 註冊 Ctrl+R 啟動智能分層歷史與情境清單
    Set-PSReadLineKeyHandler -Chord "Ctrl+r" -ScriptBlock {{
        param($key, $arg)
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        $cwd = (Get-Location).Path
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = $script:SmartHubExe
        $psi.Arguments = "search --cwd `"$cwd`" --query `"$line`""
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.StandardOutputEncoding = [System.Text.Encoding]::UTF8

        $p = [System.Diagnostics.Process]::Start($psi)
        $output = $p.StandardOutput.ReadToEnd()
        $p.WaitForExit()

        if ($p.ExitCode -eq 0 -and $output) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($output)
        }}
    }}

    # 註冊 Alt+Backspace 即時刪除上一筆打錯的指令
    Set-PSReadLineKeyHandler -Chord "Alt+Backspace" -ScriptBlock {{
        param($key, $arg)
        & $script:SmartHubExe delete-last
    }}
}}
"#
            );
        }
    }
}
