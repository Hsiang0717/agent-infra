# =====================================================================
# 🤖 AGENT-NATIVE HIGH-DENSITY CLI HELPERS (Low-Token, Structured Output)
# =====================================================================
# 本腳本封裝了專為 AI Agent 設計的 PowerShell 高密度、低 Token 輔助函數。
# 可以直接 dot-source: . ./profile/AgentHelpers.ps1
# 或由全域 PowerShell Profile ($PROFILE) 自動載入。
# =====================================================================

# 1. 以壓縮 JSON 輸出檔案與目錄清單 (支援隱藏檔 -Force，取代傳統 Get-ChildItem/ls/dir)
function lsj {
    param([string]$Path = ".")
    Get-ChildItem -Path $Path -Force -ErrorAction SilentlyContinue |
        ForEach-Object {
            [PSCustomObject]@{
                name = $_.Name
                type = if ($_.PSIsContainer) { "dir" } else { "file" }
                bytes = if ($_.PSIsContainer) { 0 } else { $_.Length }
            }
        } | ConvertTo-Json -Compress
}


# 2. 秒級提取程式碼結構大綱與行號 (千行大檔必備)
function outline {
    param([Parameter(Mandatory=$true)][string]$File)
    rg -n "^(export|function|class|def|const \w+ =|interface|type \w+ =|pub fn|impl)" $File
}

# 3. 結構化代碼搜尋 (支援限制匹配數量)
function rgj {
    param(
        [Parameter(Mandatory=$true)][string]$Pattern,
        [string]$Path = ".",
        [int]$Max = 10
    )
    rg --json -m $Max $Pattern $Path
}

# 4. 快速精準檔案搜尋 (基於 fd，自動忽略 node_modules/.git)
function findf {
    param(
        [Parameter(Mandatory=$true)][string]$Pattern,
        [string]$Path = "."
    )
    fd $Pattern $Path
}

# 5. 極速大型檔案雷達 (基於 fd，自動排除雜訊目錄，壓縮 JSON 輸出 Top-N 大檔案)
function topfiles {
    param(
        [int]$Count = 10,
        [string]$Path = "."
    )
    $root = (Resolve-Path $Path).Path
    $files = fd --type f --hidden --exclude .git --exclude node_modules --exclude .venv . $root
    if (-not $files) { return "[]" }
    $files | Get-Item -ErrorAction SilentlyContinue |
        Sort-Object Length -Descending |
        Select-Object -First $Count |
        ForEach-Object {
            [PSCustomObject]@{
                kb = [math]::Round($_.Length / 1KB, 1)
                path = $_.FullName.Replace($root + "\", "").Replace("\", "/")
            }
        } | ConvertTo-Json -Compress
}

# 6. 結構化進程探測 (低 Token 壓縮 JSON，支援依名稱篩選)
function psj {
    param(
        [string]$Name = "*",
        [int]$Top = 20
    )
    $procs = Get-Process -Name $Name -ErrorAction SilentlyContinue |
        Sort-Object WorkingSet64 -Descending |
        Select-Object -First $Top |
        ForEach-Object {
            [PSCustomObject]@{
                pid = $_.Id
                name = $_.ProcessName
                mem_mb = [math]::Round($_.WorkingSet64 / 1MB, 1)
                title = if ($_.MainWindowTitle) { $_.MainWindowTitle } else { $null }
            }
        }
    if (-not $procs) { return "[]" }
    $procs | ConvertTo-Json -Compress
}

# 7. 安靜安全強制殺進程 (支援 PID 或名稱)
function killp {
    param(
        [Parameter(Mandatory=$true)][string]$Target
    )
    if ($Target -match '^\d+$') {
        Stop-Process -Id ([int]$Target) -Force -ErrorAction SilentlyContinue
        return "Killed PID: $Target"
    } else {
        $targets = Get-Process -Name $Target -ErrorAction SilentlyContinue
        if ($targets) {
            $pids = $targets.Id
            $targets | Stop-Process -Force -ErrorAction SilentlyContinue
            return "Killed process '$Target' (PIDs: $($pids -join ', '))"
        } else {
            return "No matching process found for: $Target"
        }
    }
}

# 8. 連接埠佔用探測 (秒查 Port 狀態與佔用 PID/程序名)
function portj {
    param(
        [Parameter(Mandatory=$true)][int]$Port
    )
    $conns = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
    if (-not $conns) { return "[]" }
    $conns | ForEach-Object {
        $pidNum = $_.OwningProcess
        $pName = (Get-Process -Id $pidNum -ErrorAction SilentlyContinue).ProcessName
        [PSCustomObject]@{
            port  = $_.LocalPort
            state = $_.State.ToString()
            pid   = $pidNum
            name  = if ($pName) { $pName } else { "unknown" }
        }
    } | ConvertTo-Json -Compress
}

# 9. 應用程式探測 (秒查已安裝 Windows App 之名稱與 AppID/AUMID)
function appj {
    param(
        [string]$Query = "*"
    )
    $apps = Get-StartApps | Where-Object { $_.Name -like "*$Query*" -or $_.AppID -like "*$Query*" }
    if (-not $apps) { return "[]" }
    $apps | Select-Object Name, AppID | ConvertTo-Json -Compress
}


