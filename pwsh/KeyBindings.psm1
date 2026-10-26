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

    # Atuin 智慧歷史引擎 (當前目錄優先、SQLite 索引與歷史搜尋)
    if (Get-Command atuin -ErrorAction SilentlyContinue) {
        atuin init powershell | Out-String | Invoke-Expression
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
