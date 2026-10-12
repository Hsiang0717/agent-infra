# =====================================================================
# 📦 AgentHelpers PowerShell 模組化安裝/連結腳本 (方案 2)
#
# 原理:
#   將本模組發布或符號連結 (Symlink) 至使用者模組路徑 ($HOME\Documents\PowerShell\Modules\AgentHelpers)
#   藉由 PowerShell 內建的 Module Auto-Loading 機制：
#   👉 完全不需修改或污染使用者 $PROFILE
#   👉 終端機只要輸入 outline、lsj、portj 等指令即自動隨選載入
#
# 用法:
#   .\profile\install-module.ps1           # 建立 Symlink (推薦開發者使用，源頭修改立即生效)
#   .\profile\install-module.ps1 -Copy     # 拷貝檔案發布 (適合分發/無開發需求之機器)
#   .\profile\install-module.ps1 -Uninstall# 移除已安裝的模組
# =====================================================================
param(
    [switch]$Copy,
    [switch]$Uninstall,
    [switch]$WhatIf
)

# 1. 取得使用者預設模組路徑 (跨版本支援 Windows PowerShell 5.1 與 PowerShell 7+)
$userModulesPath = ($env:PSModulePath -split ';') | Where-Object { 
    $_ -match 'Users[\\/][^\\/]+[\\/]Documents[\\/]PowerShell[\\/]Modules' -or 
    $_ -match 'Users[\\/][^\\/]+[\\/]\.local[\\/]share[\\/]powershell[\\/]Modules'
} | Select-Object -First 1

if (-not $userModulesPath) {
    # 預設後備路徑
    $userModulesPath = Join-Path ([Environment]::GetFolderPath("MyDocuments")) "PowerShell\Modules"
}

$moduleDestDir = Join-Path $userModulesPath "AgentHelpers"
$sourceDir = $PSScriptRoot

# 2. 移除模式
if ($Uninstall) {
    if (Test-Path $moduleDestDir) {
        Write-Host "==> [Uninstall] Removing AgentHelpers module from $moduleDestDir..." -ForegroundColor Yellow
        if (-not $WhatIf) {
            Remove-Item -Path $moduleDestDir -Recurse -Force
        }
        Write-Host "[Done] Successfully uninstalled AgentHelpers module." -ForegroundColor Green
    } else {
        Write-Host "[Skip] AgentHelpers module is not installed at $moduleDestDir." -ForegroundColor Cyan
    }
    return
}

# 3. 確保來源檔案齊全
$psm1Path = Join-Path $sourceDir "AgentHelpers.psm1"
$psd1Path = Join-Path $sourceDir "AgentHelpers.psd1"

if (-not (Test-Path $psm1Path)) {
    Write-Host "==> Generating AgentHelpers.psm1 from AgentHelpers.ps1..." -ForegroundColor Cyan
    Copy-Item (Join-Path $sourceDir "AgentHelpers.ps1") $psm1Path -Force
}

if (-not (Test-Path $psd1Path)) {
    Write-Error "AgentHelpers.psd1 manifest not found in $sourceDir!"
    return
}

# 4. 建立模組目錄結構
if (-not (Test-Path $userModulesPath)) {
    if (-not $WhatIf) {
        New-Item -ItemType Directory -Path $userModulesPath -Force | Out-Null
    }
}

if (Test-Path $moduleDestDir) {
    $item = Get-Item $moduleDestDir
    Write-Host "==> Cleaning existing module installation at $moduleDestDir..." -ForegroundColor Yellow
    if (-not $WhatIf) {
        Remove-Item -Path $moduleDestDir -Recurse -Force
    }
}

# 5. 進行安裝 (Symlink 或 Copy)
if ($Copy) {
    Write-Host "==> [Install Mode: Copy] Copying module files to $moduleDestDir..." -ForegroundColor Cyan
    if (-not $WhatIf) {
        New-Item -ItemType Directory -Path $moduleDestDir -Force | Out-Null
        Copy-Item -Path $psm1Path -Destination $moduleDestDir -Force
        Copy-Item -Path $psd1Path -Destination $moduleDestDir -Force
    }
} else {
    Write-Host "==> [Install Mode: Symlink] Linking directory $sourceDir -> $moduleDestDir..." -ForegroundColor Cyan
    if (-not $WhatIf) {
        # 建立目錄符號連結 (若無管理員權限則自動降級為 Directory Junction)
        try {
            New-Item -ItemType SymbolicLink -Path $moduleDestDir -Target $sourceDir -ErrorAction Stop | Out-Null
            Write-Host "    Linked via SymbolicLink successfully." -ForegroundColor Green
        } catch {
            Write-Host "    SymbolicLink failed (requires developer mode or admin). Creating Directory Junction..." -ForegroundColor Yellow
            New-Item -ItemType Junction -Path $moduleDestDir -Target $sourceDir | Out-Null
            Write-Host "    Linked via Junction successfully." -ForegroundColor Green
        }
    }
}

if ($WhatIf) {
    Write-Host "[WhatIf] Target module directory: $moduleDestDir" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "🎉 [Done] AgentHelpers module is successfully installed!" -ForegroundColor Green
    Write-Host "👉 測試方式 (開啟新終端機執行):" -ForegroundColor Cyan
    Write-Host "   Get-Module -ListAvailable AgentHelpers" -ForegroundColor White
    Write-Host "   lsj   # 無需 Import-Module，輸入指令即由 Auto-loading 機制自動載入" -ForegroundColor White
}
