<#
.SYNOPSIS
Installs and deploys the agy-ecl plugin.
Ensures ast-grep dependency is installed, hooks are registered, and optionally deploys globally.
#>

[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [switch]$Global,
    [switch]$SkipHook
)

$ErrorActionPreference = 'Stop'
$pluginRoot = $PSScriptRoot

Write-Host "===================================================" -ForegroundColor Cyan
Write-Host "      agy-ecl (External Cognitive Layer) Setup      " -ForegroundColor Cyan
Write-Host "===================================================" -ForegroundColor Cyan

# 1. 檢測並自動安裝 ast-grep 依賴
Write-Host "[1/3] Checking ast-grep dependency..." -ForegroundColor Yellow
$astGrepCmd = Get-Command ast-grep -ErrorAction SilentlyContinue
if (-not $astGrepCmd) {
    Write-Host "ast-grep is not detected. Installing via winget..." -ForegroundColor Cyan
    try {
        winget install --id ast-grep.ast-grep --exact --accept-package-agreements --accept-source-agreements
        # Refresh PATH in current session
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
        $astGrepCmd = Get-Command ast-grep -ErrorAction SilentlyContinue
        if ($astGrepCmd) {
            Write-Host "ast-grep installed successfully: $($astGrepCmd.Source)" -ForegroundColor Green
        } else {
            Write-Warning "ast-grep installed but PATH not refreshed in active session. Please restart your terminal."
        }
    }
    catch {
        Write-Error "Failed to install ast-grep via winget. Please run 'winget install --id ast-grep.ast-grep' manually."
        exit 1
    }
} else {
    Write-Host "ast-grep is already installed: $($astGrepCmd.Source)" -ForegroundColor Green
}

# 2. 自動註冊 Git Pre-Commit Hook (若處於 Git 倉庫根目錄)
if (-not $SkipHook) {
    Write-Host "[2/3] Checking Git repository for Pre-Commit Hook..." -ForegroundColor Yellow
    $gitDir = Join-Path (Get-Location) ".git"
    if (Test-Path $gitDir) {
        $hookInstaller = Join-Path $pluginRoot "scripts\install-git-hook.ps1"
        if (Test-Path $hookInstaller) {
            & pwsh -File $hookInstaller -TargetRepo (Get-Location)
        }
    } else {
        Write-Host "Current directory is not a Git repository root. Skipping local hook installation." -ForegroundColor Gray
    }
} else {
    Write-Host "[2/3] Skipping Git Hook registration (-SkipHook)." -ForegroundColor Gray
}

# 3. 部署至全域外掛目錄 (~/.gemini/config/plugins/agy-ecl)
if ($Global) {
    Write-Host "[3/3] Deploying agy-ecl globally to ~/.gemini/config/plugins/agy-ecl..." -ForegroundColor Yellow
    $globalPluginsDir = Join-Path $env:USERPROFILE ".gemini\config\plugins\agy-ecl"
    
    if (-not (Test-Path $globalPluginsDir)) {
        New-Item -ItemType Directory -Path $globalPluginsDir -Force | Out-Null
    }

    # Copy plugin files
    Copy-Item -Path (Join-Path $pluginRoot "plugin.json") -Destination $globalPluginsDir -Force
    Copy-Item -Path (Join-Path $pluginRoot "README.md") -Destination $globalPluginsDir -Force
    if (Test-Path (Join-Path $pluginRoot "hooks.json")) {
        Copy-Item -Path (Join-Path $pluginRoot "hooks.json") -Destination $globalPluginsDir -Force
    }
    
    # Copy directories: bin, rules, scripts, skills
    foreach ($dir in @("bin", "rules", "scripts", "skills")) {
        $srcDir = Join-Path $pluginRoot $dir
        $destDir = Join-Path $globalPluginsDir $dir
        if (Test-Path $destDir) {
            Remove-Item $destDir -Recurse -Force
        }
        if (Test-Path $srcDir) {
            Copy-Item -Path $srcDir -Destination $globalPluginsDir -Recurse -Force
        }
    }
    Write-Host "agy-ecl deployed globally to: $globalPluginsDir" -ForegroundColor Green
} else {
    Write-Host "[3/3] Local installation complete. Use -Global to install for all projects." -ForegroundColor Gray
}

Write-Host "===================================================" -ForegroundColor Cyan
Write-Host "agy-ecl installation completed successfully!" -ForegroundColor Green
Write-Host "===================================================" -ForegroundColor Cyan
