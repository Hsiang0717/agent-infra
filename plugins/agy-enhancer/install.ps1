[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [switch]$Build,
    [switch]$Global,
    [switch]$SetupHelpers
)

$ErrorActionPreference = 'Stop'

$pluginRoot = $PSScriptRoot
$releaseExe = Join-Path $pluginRoot 'target\release\agy-enhancer.exe'
$binDir = Join-Path $pluginRoot 'bin'
$localBinExe = Join-Path $binDir 'agy-enhancer.exe'

# 1. 編譯 (若指定 -Build 或 release 執行檔不存在)
if ($Build -or (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf))) {
    Write-Host "[Build] Compiling agy-enhancer in release mode..." -ForegroundColor Cyan
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

# 3. 若指定 -Global，則同步至全域外掛目錄 (~/.gemini/config/plugins/agy-enhancer)
if ($Global) {
    $globalPluginDir = Join-Path $env:USERPROFILE '.gemini\config\plugins\agy-enhancer'
    $globalBinDir = Join-Path $globalPluginDir 'bin'
    $globalBinExe = Join-Path $globalBinDir 'agy-enhancer.exe'
    $globalRulesDir = Join-Path $globalPluginDir 'rules'

    if ($PSCmdlet.ShouldProcess($globalPluginDir, 'Install plugin globally')) {
        if (-not (Test-Path -LiteralPath $globalBinDir)) {
            New-Item -ItemType Directory -Path $globalBinDir -Force | Out-Null
        }
        Copy-Item -LiteralPath $localBinExe -Destination $globalBinExe -Force
        Copy-Item -LiteralPath (Join-Path $pluginRoot 'plugin.json') -Destination (Join-Path $globalPluginDir 'plugin.json') -Force
        Copy-Item -LiteralPath (Join-Path $pluginRoot 'hooks.json') -Destination (Join-Path $globalPluginDir 'hooks.json') -Force

        $rulesSrc = Join-Path $pluginRoot 'rules'
        if (Test-Path -LiteralPath $rulesSrc) {
            if (-not (Test-Path -LiteralPath $globalRulesDir)) {
                New-Item -ItemType Directory -Path $globalRulesDir -Force | Out-Null
            }
            Copy-Item -Path (Join-Path $rulesSrc '*') -Destination $globalRulesDir -Recurse -Force
            Write-Host "[Global] Synchronized rules (AAEP, CLI_TOOLCHAIN) to: $globalRulesDir" -ForegroundColor Cyan
        }

        $srcConfig = Join-Path $pluginRoot 'config.json'
        $globalConfig = Join-Path $globalPluginDir 'config.json'
        if (-not (Test-Path -LiteralPath $globalConfig)) {
            Copy-Item -LiteralPath $srcConfig -Destination $globalConfig -Force
        }
        elseif (Test-Path -LiteralPath $srcConfig) {
            try {
                $srcObj = Get-Content -LiteralPath $srcConfig -Raw -Encoding UTF8 | ConvertFrom-Json
                $dstObj = Get-Content -LiteralPath $globalConfig -Raw -Encoding UTF8 | ConvertFrom-Json
                $hasProfiles = ($null -ne $dstObj.PSObject.Properties['model_profiles']) -and ($dstObj.model_profiles.Count -gt 0)
                if ((-not $hasProfiles) -and ($null -ne $srcObj.PSObject.Properties['model_profiles']) -and ($srcObj.model_profiles.Count -gt 0)) {
                    $dstObj | Add-Member -NotePropertyName 'model_profiles' -NotePropertyValue $srcObj.model_profiles -Force
                    $dstObj | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $globalConfig -Encoding UTF8
                    Write-Host "[Global] Merged default model_profiles into existing config: $globalConfig" -ForegroundColor Cyan
                }
            }
            catch {
                Write-Warning "[Global] Could not inspect/merge existing config.json ($globalConfig): $_"
            }
        }
        Write-Host "[Global] Successfully deployed plugin globally to: $globalPluginDir" -ForegroundColor Green
    }
}

# 4. 協助安裝 AgentHelpers 模組 (PowerShell CLI 工具鏈)
if ($Global -or $SetupHelpers) {
    $workspaceRoot = Split-Path -Parent (Split-Path -Parent $pluginRoot)
    $moduleInstaller = Join-Path $workspaceRoot 'profile\install-module.ps1'
    if (Test-Path -LiteralPath $moduleInstaller) {
        Write-Host "[Helpers] Bootstrapping AgentHelpers PowerShell module..." -ForegroundColor Cyan
        pwsh -NoProfile -ExecutionPolicy Bypass -File $moduleInstaller -Copy
    }
}

Write-Host "agy-enhancer suite is ready!" -ForegroundColor Green
