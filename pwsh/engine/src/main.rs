use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

mod db;
mod matcher;
mod predictor;
mod prober;
mod tui;

use db::Database;
use predictor::PredictorEngine;
use prober::Prober;

#[derive(Parser)]
#[command(
    name = "smart-hub",
    about = "Context-Aware 3-Tier Smart Command Predictor & History Hub for PowerShell",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive Master-Detail TUI Search (Ctrl+R)
    Search {
        /// Working directory to contextualize search
        #[arg(short, long)]
        cwd: Option<String>,

        /// Pre-filled query string
        #[arg(short, long)]
        query: Option<String>,

        /// Output file path to store selected command and action
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Run one-shot 3-tier prediction ([★ Smart], [● Local], [○ Global])
    Predict {
        /// Working directory to contextualize predictions
        #[arg(short, long)]
        cwd: Option<String>,

        /// Input query string typed by user
        #[arg(short, long)]
        query: String,
    },

    /// Run persistent low-latency stdin/stdout worker for PowerShell ICommandPredictor
    Serve,

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

    /// Print C# ICommandPredictor adapter source code
    CsharpSource,

    /// Print PowerShell integration script (PSReadLine + ICommandPredictor)
    Init,
}

#[derive(Subcommand)]
enum AliasCommands {
    /// Add or update a virtual alias
    Add {
        /// Trigger word/keyword (e.g. 'g', 'dcup', 'skill開發')
        trigger: String,

        /// Expanded command (e.g. 'git status -s', 'docker compose up -d')
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

const CSHARP_PREDICTOR_SOURCE: &str = r#"using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Management.Automation;
using System.Management.Automation.Subsystem;
using System.Management.Automation.Subsystem.Prediction;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace SmartHub
{
    public sealed class TierBuckets
    {
        public readonly List<PredictiveSuggestion> Smart = new List<PredictiveSuggestion>();
        public readonly List<PredictiveSuggestion> Local = new List<PredictiveSuggestion>();
        public readonly List<PredictiveSuggestion> Global = new List<PredictiveSuggestion>();
    }

    public static class SmartHubBridge
    {
        private static readonly object _lock = new object();
        private static Process _worker;
        private static StreamWriter _stdin;
        private static StreamReader _stdout;
        private static bool _registered;

        public static string ExePath;
        public static PathIntrinsics SessionPath;
        public static string FallbackCwd = Environment.CurrentDirectory;

        private static string _lastCwd;
        private static string _lastQuery;
        private static long _lastTick;
        private static TierBuckets _lastBuckets;

        public static readonly Guid SmartId = new Guid("a1b2c3d4-1111-4000-8000-000000000001");
        public static readonly Guid LocalId = new Guid("a1b2c3d4-2222-4000-8000-000000000002");
        public static readonly Guid GlobalId = new Guid("a1b2c3d4-3333-4000-8000-000000000003");

        public static void RegisterAll(string exePath, PathIntrinsics sessionPath)
        {
            ExePath = exePath;
            SessionPath = sessionPath;
            InvalidateCache();

            if (!_registered)
            {
                TryUnregister(SmartId);
                TryUnregister(LocalId);
                TryUnregister(GlobalId);

                SubsystemManager.RegisterSubsystem(
                    SubsystemKind.CommandPredictor,
                    new TierCommandPredictor(SmartId, "★ Smart", "Smart-Hub Project & Virtual Alias Predictor", 'S', true)
                );
                SubsystemManager.RegisterSubsystem(
                    SubsystemKind.CommandPredictor,
                    new TierCommandPredictor(LocalId, "● Local", "Smart-Hub Current Directory & Git Repo History Predictor", 'L', false)
                );
                SubsystemManager.RegisterSubsystem(
                    SubsystemKind.CommandPredictor,
                    new TierCommandPredictor(GlobalId, "○ Global", "Smart-Hub Global SQLite History Predictor", 'G', false)
                );
                _registered = true;
            }

            Task.Run((Action)WarmUp);
        }

        private static void TryUnregister(Guid id)
        {
            try { SubsystemManager.UnregisterSubsystem(SubsystemKind.CommandPredictor, id); }
            catch { }
        }

        public static void WarmUp()
        {
            try
            {
                QueryTier('S', "_warmup_", CancellationToken.None);
            }
            catch { }
        }

        public static void InvalidateCache()
        {
            lock (_lock)
            {
                _lastTick = 0;
                _lastBuckets = null;
            }
        }

        private static string GetCurrentCwd()
        {
            try
            {
                var p = SessionPath?.CurrentFileSystemLocation?.ProviderPath;
                if (!string.IsNullOrEmpty(p)) return p;
            }
            catch { }
            return FallbackCwd ?? Environment.CurrentDirectory;
        }

        private static bool EnsureWorkerLocked()
        {
            if (_worker != null && !_worker.HasExited && _stdin != null && _stdout != null)
            {
                return true;
            }
            KillWorkerLocked();
            if (string.IsNullOrEmpty(ExePath) || !File.Exists(ExePath))
            {
                return false;
            }

            var utf8NoBom = new UTF8Encoding(false);
            var psi = new ProcessStartInfo
            {
                FileName = ExePath,
                Arguments = "serve",
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = false,
                StandardInputEncoding = utf8NoBom,
                StandardOutputEncoding = utf8NoBom
            };

            _worker = Process.Start(psi);
            if (_worker == null) return false;
            _stdin = _worker.StandardInput;
            _stdout = _worker.StandardOutput;
            return true;
        }

        private static void KillWorkerLocked()
        {
            try { _stdin?.Dispose(); } catch { }
            try { _stdout?.Dispose(); } catch { }
            try
            {
                if (_worker != null && !_worker.HasExited) _worker.Kill();
                _worker?.Dispose();
            }
            catch { }
            _stdin = null;
            _stdout = null;
            _worker = null;
        }

        private static string EscapeField(string s)
        {
            if (string.IsNullOrEmpty(s)) return "";
            var sb = new StringBuilder(s.Length);
            foreach (char c in s)
            {
                switch (c)
                {
                    case '\\': sb.Append("\\\\"); break;
                    case '\t': sb.Append("\\t"); break;
                    case '\n': sb.Append("\\n"); break;
                    case '\r': sb.Append("\\r"); break;
                    default: sb.Append(c); break;
                }
            }
            return sb.ToString();
        }

        private static string UnescapeField(string s)
        {
            if (string.IsNullOrEmpty(s) || s.IndexOf('\\') < 0) return s ?? "";
            var sb = new StringBuilder(s.Length);
            for (int i = 0; i < s.Length; i++)
            {
                char c = s[i];
                if (c == '\\' && i + 1 < s.Length)
                {
                    char next = s[++i];
                    switch (next)
                    {
                        case '\\': sb.Append('\\'); break;
                        case 't': sb.Append('\t'); break;
                        case 'n': sb.Append('\n'); break;
                        case 'r': sb.Append('\r'); break;
                        default: sb.Append('\\'); sb.Append(next); break;
                    }
                }
                else
                {
                    sb.Append(c);
                }
            }
            return sb.ToString();
        }

        public static List<PredictiveSuggestion> QueryTier(char tier, string input, CancellationToken ct)
        {
            if (string.IsNullOrWhiteSpace(input) || ct.IsCancellationRequested)
                return null;

            string cwd = GetCurrentCwd();
            TierBuckets buckets;

            lock (_lock)
            {
                long now = Environment.TickCount64;
                if (_lastBuckets != null &&
                    now - _lastTick < 250 &&
                    string.Equals(_lastQuery, input, StringComparison.Ordinal) &&
                    string.Equals(_lastCwd, cwd, StringComparison.OrdinalIgnoreCase))
                {
                    buckets = _lastBuckets;
                }
                else
                {
                    buckets = FetchBucketsLocked(cwd, input);
                    _lastCwd = cwd;
                    _lastQuery = input;
                    _lastTick = Environment.TickCount64;
                    _lastBuckets = buckets;
                }
            }

            if (buckets == null) return null;
            List<PredictiveSuggestion> list = tier switch
            {
                'S' => buckets.Smart,
                'L' => buckets.Local,
                'G' => buckets.Global,
                _ => null
            };
            return (list != null && list.Count > 0) ? list : null;
        }

        private static TierBuckets FetchBucketsLocked(string cwd, string query)
        {
            try
            {
                if (!EnsureWorkerLocked()) return null;
                _stdin.Write("PREDICT\t");
                _stdin.Write(EscapeField(cwd));
                _stdin.Write('\t');
                _stdin.WriteLine(EscapeField(query));
                _stdin.Flush();

                string header = _stdout.ReadLine();
                if (header == null || !header.StartsWith("OK\t", StringComparison.Ordinal))
                {
                    KillWorkerLocked();
                    return null;
                }

                if (!int.TryParse(header.Substring(3), out int count) || count < 0)
                {
                    return null;
                }

                var buckets = new TierBuckets();
                for (int i = 0; i < count; i++)
                {
                    string row = _stdout.ReadLine();
                    if (row == null)
                    {
                        KillWorkerLocked();
                        return null;
                    }
                    int t1 = row.IndexOf('\t');
                    if (t1 <= 0) continue;
                    int t2 = row.IndexOf('\t', t1 + 1);
                    char tier = row[0];
                    string cmd = t2 > t1
                        ? UnescapeField(row.Substring(t1 + 1, t2 - t1 - 1))
                        : UnescapeField(row.Substring(t1 + 1));
                    string tip = t2 > t1
                        ? UnescapeField(row.Substring(t2 + 1))
                        : null;

                    var suggestion = string.IsNullOrEmpty(tip)
                        ? new PredictiveSuggestion(cmd)
                        : new PredictiveSuggestion(cmd, tip);

                    switch (tier)
                    {
                        case 'S': buckets.Smart.Add(suggestion); break;
                        case 'L': buckets.Local.Add(suggestion); break;
                        case 'G': buckets.Global.Add(suggestion); break;
                    }
                }
                return buckets;
            }
            catch
            {
                KillWorkerLocked();
                return null;
            }
        }

        public static void RecordExecutionAsync(string commandLine, bool success)
        {
            if (string.IsNullOrWhiteSpace(commandLine)) return;
            string cwd = GetCurrentCwd();
            int exitCode = success ? 0 : 1;

            Task.Run(() =>
            {
                lock (_lock)
                {
                    _lastTick = 0;
                    _lastBuckets = null;
                    try
                    {
                        if (!EnsureWorkerLocked()) return;
                        _stdin.Write("RECORD\t");
                        _stdin.Write(EscapeField(cwd));
                        _stdin.Write('\t');
                        _stdin.Write(exitCode);
                        _stdin.Write('\t');
                        _stdin.WriteLine(EscapeField(commandLine.Trim()));
                        _stdin.Flush();
                        _stdout.ReadLine();
                    }
                    catch
                    {
                        KillWorkerLocked();
                    }
                }
            });
        }
    }

    public sealed class TierCommandPredictor : ICommandPredictor
    {
        private readonly Guid _id;
        private readonly string _name;
        private readonly string _description;
        private readonly char _tier;
        private readonly bool _recordHistory;

        public TierCommandPredictor(Guid id, string name, string description, char tier, bool recordHistory)
        {
            _id = id;
            _name = name;
            _description = description;
            _tier = tier;
            _recordHistory = recordHistory;
        }

        public Guid Id => _id;
        public string Name => _name;
        public string Description => _description;

        public SuggestionPackage GetSuggestion(PredictionClient client, PredictionContext context, CancellationToken cancellationToken)
        {
            string input = context.InputAst?.Extent?.Text;
            if (string.IsNullOrWhiteSpace(input)) return default;

            var list = SmartHubBridge.QueryTier(_tier, input, cancellationToken);
            if (list != null && list.Count > 0)
            {
                return new SuggestionPackage(list);
            }
            return default;
        }

        public bool CanAcceptFeedback(PredictionClient client, PredictorFeedbackKind feedback)
        {
            return _recordHistory && feedback == PredictorFeedbackKind.CommandLineExecuted;
        }

        public void OnSuggestionDisplayed(PredictionClient client, uint session, int countOrIndex) { }
        public void OnSuggestionAccepted(PredictionClient client, uint session, string acceptedSuggestion) { }
        public void OnCommandLineAccepted(PredictionClient client, IReadOnlyList<string> history) { }

        public void OnCommandLineExecuted(PredictionClient client, string commandLine, bool success)
        {
            if (_recordHistory)
            {
                SmartHubBridge.RecordExecutionAsync(commandLine, success);
            }
        }
    }
}
"#;

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
        Commands::Search { cwd, query, output_file } => {
            let current_dir = match cwd {
                Some(c) => PathBuf::from(c),
                None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            };
            let q = query.unwrap_or_default();
            let out = output_file.as_ref().map(std::path::Path::new);
            if let Err(e) = tui::run(&current_dir, &q, out) {
                eprintln!("Smart-Hub TUI error: {e}");
            }
        }

        Commands::Predict { cwd, query } => {
            let current_dir = cwd.unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .to_string_lossy()
                    .to_string()
            });
            let mut engine = PredictorEngine::new();
            let items = engine.predict(&db, &current_dir, &query);
            for it in items {
                let badge = match it.tier {
                    'S' => "[★ Smart]",
                    'L' => "[● Local]",
                    'G' => "[○ Global]",
                    _ => "[?]",
                };
                println!("{:10} {:45} # {}", badge, it.command, it.tooltip);
            }
        }

        Commands::Serve => {
            predictor::run_serve(&db);
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
                println!("✓ 已從 SQLite 歷史刪除上一條指令: {}", cmd);
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

        Commands::CsharpSource => {
            print!("{}", CSHARP_PREDICTOR_SOURCE);
        }

        Commands::Init => {
            let exe_path = env::current_exe()
                .unwrap_or_else(|_| PathBuf::from("smart-hub.exe"))
                .to_string_lossy()
                .to_string();

            print!(
                r#"if ([System.Environment]::UserInteractive -and -not [Console]::IsOutputRedirected) {{
    $global:SmartHubExe = '{exe_path}'

    # 1. PSReadLine 基礎體驗最佳化 (原生歷史預測，極致流暢，零凍結)
    Set-PSReadLineOption -PredictionSource History
    Set-PSReadLineOption -PredictionViewStyle ListView
    Set-PSReadLineOption -Colors @{{ InlinePrediction = "$([char]0x1b)[38;5;245m" }}
    Set-PSReadLineOption -MaximumHistoryCount 30000
    Set-PSReadLineOption -HistoryNoDuplicates
    Set-PSReadLineOption -HistorySearchCursorMovesToEnd

    Set-PSReadLineKeyHandler -Chord "UpArrow" -Function HistorySearchBackward
    Set-PSReadLineKeyHandler -Chord "DownArrow" -Function HistorySearchForward
    Set-PSReadLineKeyHandler -Chord "Tab" -Function TabCompleteNext
    Set-PSReadLineKeyHandler -Chord "Shift+Tab" -Function TabCompletePrevious

    # 2. Ctrl+R: 喚起 Smart-Hub 雙欄 Master-Detail TUI (提案二)
    function global:Invoke-SmartHubSearch {{
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        $cwd = (Get-Location).Path
        $tmpFile = [System.IO.Path]::GetTempFileName()
        try {{
            & $global:SmartHubExe search --cwd "$cwd" --query "$line" --output-file "$tmpFile"
            if (Test-Path -LiteralPath $tmpFile) {{
                $raw = Get-Content -Raw -Path $tmpFile -Encoding utf8 -ErrorAction SilentlyContinue
                if ($raw) {{
                    $res = $raw | ConvertFrom-Json -ErrorAction SilentlyContinue
                    if ($res -and $res.command) {{
                        [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
                        [Microsoft.PowerShell.PSConsoleReadLine]::Insert($res.command)
                        if ($res.action -eq 'execute') {{
                            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
                        }}
                    }}
                }}
            }}
        }} finally {{
            Remove-Item -LiteralPath $tmpFile -Force -ErrorAction SilentlyContinue
        }}
    }}

    Set-PSReadLineKeyHandler -Chord "Ctrl+r" -ScriptBlock {{
        param($key, $arg)
        global:Invoke-SmartHubSearch
    }}

    # 3. Alt+Backspace: 即刻從 SQLite 歷史中刪除上一筆打錯的指令
    Set-PSReadLineKeyHandler -Chord "Alt+Backspace" -ScriptBlock {{
        param($key, $arg)
        & $global:SmartHubExe delete-last
        [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
    }}

    # 4. 非阻塞非同步寫入 SQLite 歷史 (新指令自動歸檔)
    Set-PSReadLineOption -AddToHistoryHandler {{
        param($cmd)
        if ($cmd -and -not [string]::IsNullOrWhiteSpace($cmd)) {{
            try {{
                $psi = [System.Diagnostics.ProcessStartInfo]::new($global:SmartHubExe)
                $safeCmd = $cmd.Replace('"', '\"')
                $psi.Arguments = "record --command `"$safeCmd`" --cwd `"$((Get-Location).Path)`""
                $psi.CreateNoWindow = $true
                $psi.UseShellExecute = $false
                [System.Diagnostics.Process]::Start($psi) | Out-Null
            }} catch {{ }}
        }}
        return $true
    }}

    # 5. 右方向鍵快速補全輔助
    Set-PSReadLineKeyHandler -Chord "RightArrow" -ScriptBlock {{
        param($key, $arg)
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        if ($cursor -lt $line.Length) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::ForwardChar()
        }} else {{
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptNextSuggestionWord()
        }}
    }}

    Set-PSReadLineKeyHandler -Chord "Ctrl+RightArrow" -ScriptBlock {{
        param($key, $arg)
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        if ($cursor -lt $line.Length) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::NextWord()
        }} else {{
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptSuggestion()
        }}
    }}
}}
"#
            );
        }
    }
}
