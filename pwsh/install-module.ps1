# =====================================================================
# 📦 KeyBindings PowerShell 模組化安裝/連結腳本 (方案 2)
#
# 原理:
#   將本模組發布或建立符號連結 (Symlink/Junction) 至系統模組路徑：
#   ($HOME\Documents\PowerShell\Modules\KeyBindings)
#
# 說明:
#   由於 KeyBindings 是針對終端互動體驗 (PSReadLine) 進行環境全域設定，
#   若採用模組化形式，可透過本腳本將其部署為標準本機模組。
#
# 用法:
#   .\pwsh\install-module.ps1           # 建立 Junction 連結 (開發模式：源頭修改即時生效)
#   .\pwsh\install-module.ps1 -Copy     # 實體拷貝檔案發布
#   .\pwsh\install-module.ps1 -Uninstall# 移除已安裝之模組
# =====================================================================
param(
    [switch]$Copy,
    [switch]$Uninstall,
    [switch]$WhatIf
)

# 1. 取得使用者預設模組路徑
$userModulesPath = ($env:PSModulePath -split ';') | Where-Object { 
    $_ -match 'Users[\\/][^\\/]+[\\/]Documents[\\/]PowerShell[\\/]Modules' -or 
    $_ -match 'Users[\\/][^\\/]+[\\/]\.local[\\/]share[\\/]powershell[\\/]Modules'
} | Select-Object -First 1

if (-not $userModulesPath) {
    $userModulesPath = Join-Path ([Environment]::GetFolderPath("MyDocuments")) "PowerShell\Modules"
}

$moduleDestDir = Join-Path $userModulesPath "KeyBindings"
$sourceDir = $PSScriptRoot

# 2. 移除模式
if ($Uninstall) {
    if (Test-Path $moduleDestDir) {
        Write-Host "==> [Uninstall] Removing KeyBindings module from $moduleDestDir..." -ForegroundColor Yellow
        if (-not $WhatIf) {
            Remove-Item -Path $moduleDestDir -Recurse -Force
        }
        Write-Host "[Done] Successfully uninstalled KeyBindings module." -ForegroundColor Green
    } else {
        Write-Host "[Skip] KeyBindings module is not installed at $moduleDestDir." -ForegroundColor Cyan
    }
    return
}

# 3. 確保來源檔案齊全
$psm1Path = Join-Path $sourceDir "KeyBindings.psm1"
$psd1Path = Join-Path $sourceDir "KeyBindings.psd1"

if (-not (Test-Path $psm1Path)) {
    Write-Host "==> Generating KeyBindings.psm1 from KeyBindings.ps1..." -ForegroundColor Cyan
    Copy-Item (Join-Path $sourceDir "KeyBindings.ps1") $psm1Path -Force
}

if (-not (Test-Path $psd1Path)) {
    Write-Error "KeyBindings.psd1 manifest not found in $sourceDir!"
    return
}

# 4. 建立模組目錄結構
if (-not (Test-Path $userModulesPath)) {
    if (-not $WhatIf) {
        New-Item -ItemType Directory -Path $userModulesPath -Force | Out-Null
    }
}

if (Test-Path $moduleDestDir) {
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
    Write-Host "🎉 [Done] KeyBindings module is successfully installed!" -ForegroundColor Green
    Write-Host "👉 模組載入說明:" -ForegroundColor Cyan
    Write-Host "   可用 Get-Module -ListAvailable KeyBindings 檢查狀態。" -ForegroundColor White
    Write-Host "   由於本模組屬於全域鍵盤設定，若需在啟動時套用，可在 `$PROFILE 撰寫: Import-Module KeyBindings" -ForegroundColor White
}
