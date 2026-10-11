[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [switch]$Build,
    [switch]$Global
)

$ErrorActionPreference = 'Stop'

$pluginRoot = $PSScriptRoot
$releaseExe = Join-Path $pluginRoot 'target\release\agy-context-monitor.exe'
$binDir = Join-Path $pluginRoot 'bin'
$localBinExe = Join-Path $binDir 'agy-context-monitor.exe'

# 1. 編譯 (若指定 -Build 或 release 執行檔不存在)
if ($Build -or (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf))) {
    Write-Host "[Build] Compiling agy-context-monitor in release mode..." -ForegroundColor Cyan
    Push-Location $pluginRoot
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

# 2. 部署至本機 bin/
if (-not (Test-Path -LiteralPath $binDir)) {
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
}

if ($PSCmdlet.ShouldProcess($localBinExe, 'Copy compiled binary to local bin')) {
    Copy-Item -LiteralPath $releaseExe -Destination $localBinExe -Force
    Write-Host "[Local] Deployed binary to $localBinExe" -ForegroundColor Green
}

# 3. 若指定 -Global，則同步至全域外掛目錄 (~/.gemini/config/plugins/agy-context-monitor)
if ($Global) {
    $globalPluginDir = Join-Path $env:USERPROFILE '.gemini\config\plugins\agy-context-monitor'
    $globalBinDir = Join-Path $globalPluginDir 'bin'
    $globalBinExe = Join-Path $globalBinDir 'agy-context-monitor.exe'

    if ($PSCmdlet.ShouldProcess($globalPluginDir, 'Install plugin globally')) {
        if (-not (Test-Path -LiteralPath $globalBinDir)) {
            New-Item -ItemType Directory -Path $globalBinDir -Force | Out-Null
        }
        Copy-Item -LiteralPath $localBinExe -Destination $globalBinExe -Force
        Copy-Item -LiteralPath (Join-Path $pluginRoot 'plugin.json') -Destination (Join-Path $globalPluginDir 'plugin.json') -Force
        Copy-Item -LiteralPath (Join-Path $pluginRoot 'hooks.json') -Destination (Join-Path $globalPluginDir 'hooks.json') -Force
        $globalConfig = Join-Path $globalPluginDir 'config.json'
        if (-not (Test-Path -LiteralPath $globalConfig)) {
            Copy-Item -LiteralPath (Join-Path $pluginRoot 'config.json') -Destination $globalConfig -Force
        }
        Write-Host "[Global] Successfully deployed plugin globally to: $globalPluginDir" -ForegroundColor Green
    }
}

Write-Host "agy-context-monitor is ready!" -ForegroundColor Cyan
