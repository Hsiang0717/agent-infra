[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$Destination = (Join-Path $env:USERPROFILE '.antigravity'),
    [switch]$Build,
    [switch]$CleanLegacy
)

$ErrorActionPreference = 'Stop'

$rsDir = Join-Path $PSScriptRoot 'statusline-rs'
$releaseExe = Join-Path $rsDir 'target\release\statusline.exe'

# 1. 如指定 -Build 或 release 執行檔不存在，則自動使用 cargo build --release
if ($Build -or (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf))) {
    Write-Host "[Build] Compiling statusline-rs in release mode..." -ForegroundColor Cyan
    Push-Location $rsDir
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

# 2. 確保目標目錄存在
if (-not (Test-Path -LiteralPath $Destination)) {
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
}

$targetExe = Join-Path $Destination 'statusline.exe'

# 3. 安裝 Rust 原生執行檔
if ($PSCmdlet.ShouldProcess($targetExe, 'Install Rust statusline.exe')) {
    Copy-Item -LiteralPath $releaseExe -Destination $targetExe -Force
    Write-Host "[Installed] Successfully deployed Rust statusline.exe to: $targetExe" -ForegroundColor Green
}

# 4. 清理過時舊版 PowerShell 實作殘留檔 (預設或指定 -CleanLegacy)
$legacyFiles = @(
    'statusline.ps1',
    'Status.Git.psm1',
    'Status.Power.psm1',
    'Status.Native.dll'
)

$legacyFound = @()
foreach ($file in $legacyFiles) {
    $legacyPath = Join-Path $Destination $file
    if (Test-Path -LiteralPath $legacyPath) {
        $legacyFound += $legacyPath
    }
}
# 也一併包含舊版備份檔
Get-ChildItem -Path $Destination -Filter "statusline.ps1*" -File -ErrorAction SilentlyContinue | ForEach-Object {
    if ($_.FullName -notin $legacyFound) { $legacyFound += $_.FullName }
}
Get-ChildItem -Path $Destination -Filter "Status.Git.psm1*" -File -ErrorAction SilentlyContinue | ForEach-Object {
    if ($_.FullName -notin $legacyFound) { $legacyFound += $_.FullName }
}

if ($legacyFound.Count -gt 0) {
    foreach ($item in $legacyFound) {
        if ($PSCmdlet.ShouldProcess($item, 'Remove obsolete statusline file')) {
            Remove-Item -LiteralPath $item -Force -ErrorAction SilentlyContinue
            Write-Host "[Cleaned] Removed legacy file: $item" -ForegroundColor Yellow
        }
    }
}

if ($WhatIfPreference) {
    Write-Output "Preview only; no files changed in: $Destination"
} else {
    Write-Output "Antigravity Rust statusline is ready at: $targetExe"
}
