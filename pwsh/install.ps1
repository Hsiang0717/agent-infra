# =====================================================================
# 🚀 Smart-Hub Rust Engine & ICommandPredictor 一鍵編譯與安裝腳本
#
# 功能:
#   1. 自動編譯 Rust 引擎 (pwsh/engine -> smart-hub.exe)
#   2. 預編譯超輕量 C# ICommandPredictor 轉接器 (SmartHub.Predictor.dll)
#   3. 部署至 ~/.antigravity/ 並清理舊版 KeyBindings.ps1 / 模組殘留
#   4. 將 `smart-hub init` 注入 $PROFILE，驅動原生下方 ListView 浮動清單
#
# 用法:
#   .\pwsh\install.ps1          # 部署並更新 $PROFILE (若無執行檔則自動編譯)
#   .\pwsh\install.ps1 -Build   # 強制重新編譯 Rust 引擎與 C# Predictor DLL
# =====================================================================
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$Destination = (Join-Path $env:USERPROFILE '.antigravity'),
    [switch]$Build
)

$ErrorActionPreference = 'Stop'

$engineDir = Join-Path $PSScriptRoot 'engine'
$releaseDir = Join-Path $engineDir 'target\release'
$releaseExe = Join-Path $releaseDir 'smart-hub.exe'

# 1. 若指定 -Build 或 release 執行檔不存在，執行 cargo build --release
if ($Build -or (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf))) {
    Write-Host "==> [Build] Compiling smart-hub Rust engine in release mode..." -ForegroundColor Cyan
    Get-Process -Name 'smart-hub' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Push-Location $engineDir
    try {
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

if (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf)) {
    throw "Compiled binary not found: $releaseExe"
}

# 2. 確保目標目錄存在並部署 smart-hub.exe
if (-not (Test-Path -LiteralPath $Destination)) {
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
}

$targetExe = Join-Path $Destination 'smart-hub.exe'

if ($PSCmdlet.ShouldProcess($targetExe, 'Deploy smart-hub.exe')) {
    Get-Process -Name 'smart-hub' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Copy-Item -LiteralPath $releaseExe -Destination $targetExe -Force
    Write-Host "==> [Installed] Deployed smart-hub.exe to: $targetExe" -ForegroundColor Green
}

# 4. 清理舊版 KeyBindings PowerShell 模組與腳本殘留
$userModulesPath = ($env:PSModulePath -split ';') | Where-Object {
    $_ -match 'Users[\\/][^\\/]+[\\/]Documents[\\/]PowerShell[\\/]Modules' -or
    $_ -match 'Users[\\/][^\\/]+[\\/]\.local[\\/]share[\\/]powershell[\\/]Modules'
} | Select-Object -First 1

if (-not $userModulesPath) {
    $userModulesPath = Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'PowerShell\Modules'
}

$legacyModuleDir = Join-Path $userModulesPath 'KeyBindings'
if (Test-Path -LiteralPath $legacyModuleDir) {
    if ($PSCmdlet.ShouldProcess($legacyModuleDir, 'Remove legacy KeyBindings module')) {
        Remove-Item -LiteralPath $legacyModuleDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "==> [Cleaned] Removed obsolete module directory: $legacyModuleDir" -ForegroundColor Yellow
    }
}

# 5. 更新 $PROFILE 注入 `smart-hub init`
$profilePath = $PROFILE.CurrentUserCurrentHost
if (-not (Test-Path -LiteralPath $profilePath)) {
    $parentDir = Split-Path $profilePath
    if (-not (Test-Path -LiteralPath $parentDir)) {
        New-Item -ItemType Directory -Path $parentDir -Force | Out-Null
    }
    New-Item -ItemType File -Path $profilePath -Force | Out-Null
}

$existingProfile = Get-Content -Raw -Path $profilePath -ErrorAction SilentlyContinue
if ($null -eq $existingProfile) { $existingProfile = '' }

$startTag = '# >>> POWERSHELL KEYBINDINGS (START) >>>'
$endTag   = '# <<< POWERSHELL KEYBINDINGS (END) <<<'

$block = @"
$startTag
if (Test-Path "$targetExe") {
    & "$targetExe" init | Out-String | Invoke-Expression
}
$endTag
"@

if ($existingProfile -match [regex]::Escape($startTag)) {
    $regex = '(?s)' + [regex]::Escape($startTag) + '.*?' + [regex]::Escape($endTag)
    $newProfile = [regex]::Replace($existingProfile, $regex, $block)
} else {
    $newProfile = $existingProfile.TrimEnd() + "`r`n`r`n" + $block + "`r`n"
}

if ($PSCmdlet.ShouldProcess($profilePath, 'Configure smart-hub init in PowerShell PROFILE')) {
    Set-Content -Path $profilePath -Value $newProfile -Encoding utf8
    Write-Host "==> [Profile] Updated $profilePath with native Smart-Hub ICommandPredictor!" -ForegroundColor Green
}
