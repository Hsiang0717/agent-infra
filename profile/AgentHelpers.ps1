# =====================================================================
# 🤖 AGENT-NATIVE HIGH-DENSITY CLI HELPERS (V3 - Lean & Hardened)
# =====================================================================
# 本腳本提供專為 AI Agent 設計的低 Token、無重複功能 PowerShell 輔助函數。
# 可直接 dot-source: . ./profile/AgentHelpers.ps1
# 或由系統 PowerShell Profile ($PROFILE) 自動載入。
# =====================================================================

# 1. 結構大綱提取 (副檔名感知之多語言結構、類別、函式與標題索引)
function outline {
    param([Parameter(Mandatory=$true, ValueFromPipeline=$true)][string]$File)
    
    $resolved = (Resolve-Path -LiteralPath $File -ErrorAction Stop).Path
    $ext = [System.IO.Path]::GetExtension($resolved).ToLowerInvariant()

    switch ($ext) {
        # 1. Markdown / 文件綱要 (提取標題 #, ##, ###, ####)
        { $_ -in '.md', '.markdown', '.mdown', '.mkd' } {
            rg -n '^\s*#{1,4}\s+' $resolved
        }
        # 2. TypeScript / JavaScript / JSX / TSX (支援 function, 箭頭函式, Class 方法, type/interface)
        { $_ -in '.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs' } {
            $p = '^\s*(export\s+(default\s+)?)?((async\s+)?function\b|(const|let|var)\s+\w+\s*=\s*(async\s+)?(\([^)]*\)|[a-zA-Z0-9_$]+)\s*=>|(class|interface|type|enum)\b|(public|private|protected|static|async|override|get|set)\s+(\w+)\s*\([^)]*\)\s*[:{])'
            rg -n $p $resolved
        }
        # 3. Python (捕捉 class 與 def，保留縮排結構)
        { $_ -in '.py', '.pyi' } {
            rg -n '^\s*(async\s+)?(def|class)\b' $resolved
        }
        # 4. Rust (fn, struct, enum, trait, impl, mod, type, macro_rules!)
        { $_ -in '.rs' } {
            $p = '^\s*(pub(\([^)]+\))?\s+)?(async\s+)?(fn|struct|enum|trait|impl|mod|type|macro_rules!)\b'
            rg -n $p $resolved
        }
        # 5. C# / Java / Kotlin / Swift / Scala
        { $_ -in '.cs', '.java', '.kt', '.swift', '.scala' } {
            $p = '^\s*(public|protected|private|static|async|override|virtual|final|abstract|class|interface|record|enum|fun|func)\b'
            rg -n $p $resolved
        }
        # 6. Go (func, type)
        { $_ -in '.go' } {
            rg -n '^\s*(func(\s+\([^)]+\))?|type)\s+\w+' $resolved
        }
        # 7. C / C++ / Header
        { $_ -in '.c', '.cpp', '.cxx', '.cc', '.h', '.hpp' } {
            $p = '^\s*(((class|struct|enum|union|namespace)\s+\w+)|((inline|static|virtual|explicit|friend|constexpr)\s+)*[\w:<>&*]+\s+\w+\s*\([^;]*\)\s*(\{|const|\/\/))'
            rg -n $p $resolved
        }
        # 8. SQL
        { $_ -in '.sql' } {
            rg -n -i '^\s*create\s+(or\s+replace\s+)?(table|view|procedure|function|trigger|index)\b' $resolved
        }
        # 9. PowerShell
        { $_ -in '.ps1', '.psm1' } {
            rg -n '^\s*(function|filter|configuration|class|enum)\s+\w+' $resolved
        }
        # 通用 Fallback (經典多語言正規式)
        default {
            $p = '^\s*(@\w+|(export|default|pub(\([^)]+\))?|async|public|private|protected|static|final|abstract|override)\s+)*(fn|func|function|def|class|interface|type|struct|enum|impl|trait|mod)\b'
            rg -n $p $resolved
        }
    }
}

# 2. 壓縮目錄清單 (解決 Get-ChildItem 肥大表格，保證輸出 Array，短鍵名 n, t, b)
function lsj {
    param([string]$Path = ".")
    $items = Get-ChildItem -Path $Path -Force -ErrorAction SilentlyContinue
    if (-not $items) { return "[]" }
    @($items | ForEach-Object {
        [PSCustomObject]@{
            n = $_.Name
            t = if ($_.PSIsContainer) { "d" } else { "f" }
            b = if ($_.PSIsContainer) { 0 } else { $_.Length }
        }
    }) | ConvertTo-Json -Compress
}

# 3. 大型檔案雷達 (事前防範讀檔超限，自動排除雜訊目錄)
function topj {
    param([int]$Count = 10, [string]$Path = ".")
    $root = (Resolve-Path $Path).Path
    $files = fd --type f --hidden --exclude .git --exclude node_modules --exclude .venv . $root
    if (-not $files) { return "[]" }
    $res = @($files | Get-Item -ErrorAction SilentlyContinue |
        Sort-Object Length -Descending |
        Select-Object -First $Count |
        ForEach-Object {
            [PSCustomObject]@{
                kb = [math]::Round($_.Length / 1KB, 1)
                p  = $_.FullName.Replace($root + "\", "").Replace("\", "/")
            }
        })
    if (-not $res) { return "[]" }
    $res | ConvertTo-Json -Compress
}

# 4. 靜默 HTTP 伺服器探測 (防範 Dev Server 探測時噴出幾萬 Token HTML 或錯誤堆疊)
function pingj {
    param([string]$Url = "http://localhost:3000", [int]$TimeoutSec = 2)
    try {
        $resp = Invoke-WebRequest -Uri $Url -TimeoutSec $TimeoutSec -UseBasicParsing -ErrorAction Stop
        [PSCustomObject]@{ ok = $true; code = [int]$resp.StatusCode } | ConvertTo-Json -Compress
    } catch {
        $code = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { $null }
        if ($code) {
            [PSCustomObject]@{ ok = $true; code = $code } | ConvertTo-Json -Compress
        } else {
            [PSCustomObject]@{ ok = $false; err = "unreachable" } | ConvertTo-Json -Compress
        }
    }
}

# 5. 複合連接埠探測 (秒查 Port 狀態與對應程序名稱，補足 Windows 二段式查詢痛點)
function portj {
    param([Parameter(Mandatory=$true)][int]$Port)
    $conns = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
    if (-not $conns) { return "[]" }
    @($conns | ForEach-Object {
        $pidNum = $_.OwningProcess
        $pName = if ($pidNum -gt 0) {
            (Get-Process -Id $pidNum -ErrorAction SilentlyContinue).ProcessName 
        } else { "System" }
        [PSCustomObject]@{
            port  = $_.LocalPort
            state = $_.State.ToString()
            pid   = $pidNum
            name  = if ($pName) { $pName } else { "unknown" }
        }
    }) | ConvertTo-Json -Compress
}

# 6. 結構化進程探測 (依記憶體排序之壓縮 JSON)
function psj {
    param([string]$Name = "*", [int]$Top = 20)
    $procs = Get-Process -Name $Name -ErrorAction SilentlyContinue
    if (-not $procs) { return "[]" }
    @($procs |
        Sort-Object WorkingSet64 -Descending |
        Select-Object -First $Top |
        ForEach-Object {
            [PSCustomObject]@{
                pid  = $_.Id
                name = $_.ProcessName
                mb   = [math]::Round($_.WorkingSet64 / 1MB, 1)
            }
        }) | ConvertTo-Json -Compress
}

# 7. 安靜強制終止進程 (支援 PID 或名稱，不噴錯)
function killp {
    param([Parameter(Mandatory=$true)][string]$Target)
    if ($Target -match '^\d+$') {
        Stop-Process -Id ([int]$Target) -Force -ErrorAction SilentlyContinue
        return "Killed PID: $Target"
    } else {
        $targets = Get-Process -Name $Target -ErrorAction SilentlyContinue
        if ($targets) {
            $pids = $targets.Id
            $targets | Stop-Process -Force -ErrorAction SilentlyContinue
            return "Killed '$Target' (PIDs: $($pids -join ', '))"
        }
        return "No matching process: $Target"
    }
}

# 8. 穿透 Session 隔離啟動互動 GUI 程式 (支援原生與非原生 Python/Node/Electron/Java 等框架)
function start-gui {
    param(
        [Parameter(Mandatory=$true)][string]$FilePath,
        [string]$ArgumentList = "",
        [string]$WorkingDirectory = ""
    )
    $workDir = if ($WorkingDirectory) { (Resolve-Path $WorkingDirectory).Path } else { (Get-Location).Path }
    $taskName = "AGY_Launch_$(Get-Random)"
    $actionParams = @{ Execute = $FilePath }
    if (-not [string]::IsNullOrWhiteSpace($ArgumentList)) {
        $actionParams['Argument'] = $ArgumentList
    }
    if (-not [string]::IsNullOrWhiteSpace($workDir)) {
        $actionParams['WorkingDirectory'] = $workDir
    }
    $action = New-ScheduledTaskAction @actionParams
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive
    Register-ScheduledTask -TaskName $taskName -Action $action -Settings $settings -Principal $principal -Force | Out-Null
    Start-ScheduledTask -TaskName $taskName | Out-Null
    Start-Sleep -Milliseconds 600
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false | Out-Null
}

# 9. 正規化檔案換行符號為純 LF (無 BOM UTF-8)，徹底消除 Windows CRLF 對 edit_file 的干擾
function lf {
    param([Parameter(Mandatory=$true, ValueFromPipeline=$true)][string[]]$Path)
    process {
        foreach ($p in $Path) {
            $resolved = (Resolve-Path -LiteralPath $p -ErrorAction Stop).Path
            $raw = [System.IO.File]::ReadAllText($resolved)
            if ($raw.Contains("`r`n")) {
                [System.IO.File]::WriteAllText($resolved, $raw.Replace("`r`n", "`n"), (New-Object System.Text.UTF8Encoding($false)))
            }
        }
    }
}


