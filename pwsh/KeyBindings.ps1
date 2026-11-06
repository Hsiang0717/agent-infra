# =====================================================================
# ⌨️ POWERSHELL INTERACTIVE UX & PSREADLINE CONFIGURATION
# =====================================================================
# 本腳本提供針對 PSReadLine 深度調校的流暢終端體驗：
# 1. 浮動清單預測 (ListView) + 歷史紀錄/外掛來源
# 2. 上下鍵歷史搜尋 (自動定位至行尾)
# 3. 行內無阻輪替 Tab 補全 (Tab / Shift+Tab)
# 4. 上下文感知右鍵補全 (行內移動 / 行尾採納單字 / Ctrl+右鍵採納整行)
# =====================================================================

if ([System.Environment]::UserInteractive -and -not [Console]::IsOutputRedirected) {
    # 預測引擎：啟用下方清單式浮動提示 (ListView)，隨打隨提示，不搶焦點
    Set-PSReadLineOption -PredictionSource HistoryAndPlugin
    Set-PSReadLineOption -PredictionViewStyle ListView
    Set-PSReadLineOption -Colors @{ InlinePrediction = "$([char]0x1b)[38;5;245m" }

    # 歷史紀錄管理 (加大上限至 30,000 筆，並自動過濾連續重複指令)
    Set-PSReadLineOption -MaximumHistoryCount 30000
    Set-PSReadLineOption -HistoryNoDuplicates

    # 歷史紀錄搜尋 (上下鍵過濾開頭，游標自動置於最末端)
    Set-PSReadLineOption -HistorySearchCursorMovesToEnd
    Set-PSReadLineKeyHandler -Chord "UpArrow" -Function HistorySearchBackward
    Set-PSReadLineKeyHandler -Chord "DownArrow" -Function HistorySearchForward

    # =====================================================================
    # 🚀 Smart-Hub 智慧歷史、環境感知與虛別名引擎 (Rust + SQLite)
    # =====================================================================
    $smartHubExe = (Get-Command smart-hub -ErrorAction SilentlyContinue)?.Source
    if (-not $smartHubExe) {
        $candidate = Join-Path $PSScriptRoot "engine\target\release\smart-hub.exe"
        if (Test-Path $candidate) {
            $smartHubExe = $candidate
        }
    }

    if ($smartHubExe) {
        # Ctrl+R: 喚起 Smart-Hub 三層互動清單 (Smart/Alias, Local, Global)
        Set-PSReadLineKeyHandler -Chord "Ctrl+r" -ScriptBlock {
            param($key, $arg)
            $line = $null
            $cursor = $null
            [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

            $cwd = (Get-Location).Path
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = $smartHubExe
            $psi.Arguments = "search --cwd `"$cwd`" --query `"$line`""
            $psi.UseShellExecute = $false
            $psi.RedirectStandardOutput = $true
            $psi.StandardOutputEncoding = [System.Text.Encoding]::UTF8

            $p = [System.Diagnostics.Process]::Start($psi)
            $output = $p.StandardOutput.ReadToEnd()
            $p.WaitForExit()

            if ($p.ExitCode -eq 0 -and $output) {
                [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
                [Microsoft.PowerShell.PSConsoleReadLine]::Insert($output)
            }
        }

        # Alt+Backspace: 即刻從 SQLite 歷史中刪除上一筆打錯的指令
        Set-PSReadLineKeyHandler -Chord "Alt+Backspace" -ScriptBlock {
            param($key, $arg)
            & $smartHubExe delete-last
        }

        # 自動非同步寫入指令歷史至 SQLite
        $existingHandler = (Get-PSReadLineOption).AddToHistoryHandler
        Set-PSReadLineOption -AddToHistoryHandler {
            param($command)
            if ($command -and -not [string]::IsNullOrWhiteSpace($command)) {
                $cwd = (Get-Location).Path
                Start-Process -FilePath $smartHubExe -ArgumentList @("record", "--command", $command, "--cwd", $cwd) -WindowStyle Hidden
            }
            if ($existingHandler) {
                return & $existingHandler $command
            }
            return $true
        }
    }

    # Tab 鍵：全面回歸最順暢的「行內逐項輪替」，絕不彈出干擾輸入的選單
    # (連按 Tab 換下一個，Shift+Tab 換上一個)
    Set-PSReadLineKeyHandler -Chord "Tab" -Function TabCompleteNext
    Set-PSReadLineKeyHandler -Chord "Shift+Tab" -Function TabCompletePrevious

    # F2 鍵：可隨時在「行內灰色預測 (Inline)」與「下方清單預測 (ListView)」之間快速切換
    # (PSReadLine 原生支援，保留此習慣即可)

    # 右方向鍵：行內移動一格，行尾補全單字
    Set-PSReadLineKeyHandler -Chord "RightArrow" -ScriptBlock {
        param($key, $arg)
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        if ($cursor -lt $line.Length) {
            [Microsoft.PowerShell.PSConsoleReadLine]::ForwardChar()
        } else {
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptNextSuggestionWord()
        }
    }

    # Ctrl + 右方向鍵：行內跳至下一個單字，行尾接受整行預測
    Set-PSReadLineKeyHandler -Chord "Ctrl+RightArrow" -ScriptBlock {
        param($key, $arg)
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        if ($cursor -lt $line.Length) {
            [Microsoft.PowerShell.PSConsoleReadLine]::NextWord()
        } else {
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptSuggestion()
        }
    }
}
