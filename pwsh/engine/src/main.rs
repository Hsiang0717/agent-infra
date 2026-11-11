use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

mod db;
mod inline;
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
        #[arg(short, long, allow_hyphen_values = true)]
        query: Option<String>,

        /// Output file path to store selected command and action
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Interactive Inline Menu with 2-column layout and group tabs
    Inline {
        /// Working directory to contextualize search
        #[arg(short, long)]
        cwd: Option<String>,

        /// Current command-line input to filter suggestions
        #[arg(short, long, allow_hyphen_values = true)]
        query: Option<String>,

        /// Output file path to store selected command and action (TAB / EXEC)
        #[arg(short, long)]
        output_file: Option<String>,
    },

    /// Run one-shot 3-tier prediction ([★ Smart], [● Local], [○ Global])
    Predict {
        /// Working directory to contextualize predictions
        #[arg(short, long)]
        cwd: Option<String>,

        /// Input query string typed by user
        #[arg(short, long, allow_hyphen_values = true)]
        query: String,
    },

    /// Run persistent low-latency stdin/stdout worker for PowerShell ICommandPredictor
    Serve,

    /// Record a command execution into SQLite
    Record {
        /// The command line executed
        #[arg(short, long, allow_hyphen_values = true)]
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
        private static List<PredictiveSuggestion> _lastSuggestions;

        public static readonly Guid PredictorId = new Guid("a1b2c3d4-9999-4000-8000-000000000001");
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
                TryUnregister(PredictorId);
                TryUnregister(SmartId);
                TryUnregister(LocalId);
                TryUnregister(GlobalId);

                SubsystemManager.RegisterSubsystem(
                    SubsystemKind.CommandPredictor,
                    new SmartHubPredictor()
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
                lock (_lock)
                {
                    EnsureWorkerLocked();
                }
                QuerySuggestions("git", CancellationToken.None);
            }
            catch { }
        }

        public static void InvalidateCache()
        {
            lock (_lock)
            {
                _lastTick = 0;
                _lastSuggestions = null;
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

        public static List<PredictiveSuggestion> QuerySuggestions(string input, CancellationToken ct)
        {
            if (string.IsNullOrWhiteSpace(input) || ct.IsCancellationRequested)
                return null;

            string cwd = GetCurrentCwd();
            long now = Environment.TickCount64;

            if (!Monitor.TryEnter(_lock, 12))
            {
                return _lastSuggestions;
            }

            try
            {
                if (_lastSuggestions != null &&
                    now - _lastTick < 300 &&
                    string.Equals(_lastQuery, input, StringComparison.Ordinal) &&
                    string.Equals(_lastCwd, cwd, StringComparison.OrdinalIgnoreCase))
                {
                    return _lastSuggestions;
                }

                var list = FetchSuggestionsLocked(cwd, input);
                _lastCwd = cwd;
                _lastQuery = input;
                _lastTick = Environment.TickCount64;
                _lastSuggestions = list;
                return list;
            }
            finally
            {
                Monitor.Exit(_lock);
            }
        }

        private static List<PredictiveSuggestion> FetchSuggestionsLocked(string cwd, string query)
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

                var list = new List<PredictiveSuggestion>(Math.Min(count, 12));
                for (int i = 0; i < count; i++)
                {
                    string row = _stdout.ReadLine();
                    if (row == null)
                    {
                        KillWorkerLocked();
                        return list;
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

                    list.Add(suggestion);
                }
                return list;
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
                if (!Monitor.TryEnter(_lock, 200)) return;
                try
                {
                    _lastTick = 0;
                    _lastSuggestions = null;
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
                finally
                {
                    Monitor.Exit(_lock);
                }
            });
        }
    }

    public sealed class SmartHubPredictor : ICommandPredictor
    {
        public Guid Id => SmartHubBridge.PredictorId;
        public string Name => "Smart-Hub";
        public string Description => "Context-Aware 3-Tier Smart Command & History Predictor";

        public SuggestionPackage GetSuggestion(PredictionClient client, PredictionContext context, CancellationToken cancellationToken)
        {
            string input = context.InputAst?.Extent?.Text;
            if (string.IsNullOrWhiteSpace(input) || cancellationToken.IsCancellationRequested) return default;

            var list = SmartHubBridge.QuerySuggestions(input, cancellationToken);
            if (list != null && list.Count > 0)
            {
                return new SuggestionPackage(list);
            }
            return default;
        }

        public bool CanAcceptFeedback(PredictionClient client, PredictorFeedbackKind feedback)
        {
            return feedback == PredictorFeedbackKind.CommandLineExecuted;
        }

        public void OnSuggestionDisplayed(PredictionClient client, uint session, int countOrIndex) { }
        public void OnSuggestionAccepted(PredictionClient client, uint session, string acceptedSuggestion) { }
        public void OnCommandLineAccepted(PredictionClient client, IReadOnlyList<string> history) { }

        public void OnCommandLineExecuted(PredictionClient client, string commandLine, bool success)
        {
            SmartHubBridge.RecordExecutionAsync(commandLine, success);
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

        Commands::Inline { cwd, query, output_file } => {
            let current_dir = match cwd {
                Some(c) => PathBuf::from(c),
                None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            };
            let q = query.unwrap_or_default();
            let out = output_file.map(PathBuf::from);
            match inline::InlineRunner::new(&current_dir, &q, out) {
                Ok(mut runner) => {
                    if let Err(e) = runner.run() {
                        eprintln!("Smart-Hub Inline error: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to start Smart-Hub Inline: {e}");
                }
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
            let dll_path = env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|dir| dir.join("SmartHub.Predictor.dll")))
                .unwrap_or_else(|| PathBuf::from("SmartHub.Predictor.dll"))
                .to_string_lossy()
                .to_string();

            print!(
                r#"if ($Host.Name -eq 'ConsoleHost' -or $Host.UI.RawUI) {{
    $global:SmartHubExe = '{exe_path}'
    $smartHubDll = '{dll_path}'

    # 1. 載入極速 Smart-Hub 原生預測適配器
    if (-not ('SmartHub.SmartHubBridge' -as [type])) {{
        if (Test-Path -LiteralPath $smartHubDll) {{
            [System.Reflection.Assembly]::Load([System.IO.File]::ReadAllBytes($smartHubDll)) | Out-Null
        }} else {{
            $cs = & $global:SmartHubExe csharp-source | Out-String
            Add-Type -TypeDefinition $cs -ErrorAction SilentlyContinue
        }}
    }}

    if ('SmartHub.SmartHubBridge' -as [type]) {{
        [SmartHub.SmartHubBridge]::RegisterAll($global:SmartHubExe, $ExecutionContext.SessionState.Path)
        try {{ Set-PSReadLineOption -PredictionSource Plugin -ErrorAction Stop }} catch {{}}
    }} else {{
        try {{ Set-PSReadLineOption -PredictionSource HistoryAndPlugin -ErrorAction Stop }} catch {{}}
    }}

    # 2. 原生內聯預測樣式 (InlineView 微淡幽靈文字，絕不重疊污染下方)
    try {{
        Set-PSReadLineOption -PredictionViewStyle InlineView -ErrorAction Stop
        Set-PSReadLineOption -Colors @{{ InlinePrediction = "$([char]0x1b)[38;5;245m" }} -ErrorAction Stop
        Set-PSReadLineOption -MaximumHistoryCount 30000 -ErrorAction Stop
        Set-PSReadLineOption -HistoryNoDuplicates -ErrorAction Stop
    }} catch {{}}

    # 3. 原生命令列鍵位
    Set-PSReadLineKeyHandler -Chord "UpArrow" -Function HistorySearchBackward
    Set-PSReadLineKeyHandler -Chord "Tab" -Function TabCompleteNext
    Set-PSReadLineKeyHandler -Chord "Shift+Tab" -Function TabCompletePrevious

    # 4. 內聯選單觸發：按 DownArrow 進入內聯選單 (左右排版、禁止原生操作)
    Set-PSReadLineKeyHandler -Chord "DownArrow" -ScriptBlock {{
        param($key, $arg)
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        if ($null -eq $line) {{ $line = "" }}

        $actFile = Join-Path $env:TEMP ("smarthub_act_" + [System.Guid]::NewGuid().ToString("N") + ".tmp")
        $cwd = $ExecutionContext.SessionState.Path.CurrentFileSystemLocation.ProviderPath

        # 啟動內聯雙態選單 (選單期間禁止原生操作)
        & $global:SmartHubExe inline "--cwd=$cwd" "--query=$line" "--output-file=$actFile"

        if ('SmartHub.SmartHubBridge' -as [type]) {{
            [SmartHub.SmartHubBridge]::InvalidateCache()
        }}

        if (Test-Path -LiteralPath $actFile) {{
            $raw = Get-Content -LiteralPath $actFile -Raw -Encoding utf8
            Remove-Item -LiteralPath $actFile -Force -ErrorAction SilentlyContinue
            if ($raw) {{
                $parts = $raw.TrimEnd("`r", "`n").Split("`t", 3)
                if ($parts.Length -ge 2) {{
                    $action = $parts[0]
                    $scrolled = 0
                    [void][int]::TryParse($parts[1], [ref]$scrolled)
                    $cmd = if ($parts.Length -ge 3) {{ $parts[2] }} else {{ "" }}

                    # 若終端機因底部空間不足而向上捲動，同步補償 PSConsoleReadLine 內部 _initialY 座標
                    if ($scrolled -gt 0) {{
                        try {{
                            $flags = [System.Reflection.BindingFlags]'NonPublic,Instance,Static'
                            $singletonField = [Microsoft.PowerShell.PSConsoleReadLine].GetField('_singleton', $flags)
                            $initYField = [Microsoft.PowerShell.PSConsoleReadLine].GetField('_initialY', $flags)
                            if ($singletonField -and $initYField) {{
                                $inst = $singletonField.GetValue($null)
                                if ($inst) {{
                                    $curY = [int]$initYField.GetValue($inst)
                                    $initYField.SetValue($inst, [Math]::Max(0, $curY - $scrolled))
                                }}
                            }}
                        }} catch {{}}
                    }}

                    if ($action -eq "TAB") {{
                        # Tab 回填：回填到終端機命令列，恢復原生操作
                        [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $cmd)
                        [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($cmd.Length)
                    }} elseif ($action -eq "EXEC") {{
                        # Enter 執行：回填並直接執行
                        [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $cmd)
                        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
                    }}
                }}
            }}
        }}
    }}

    # 5. 右方向鍵快速補全幽靈文字
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
