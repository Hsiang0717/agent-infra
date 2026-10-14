[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [switch]$Copy,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'

$pluginRoot = $PSScriptRoot
$globalPluginDir = Join-Path $env:USERPROFILE '.gemini\config\plugins\aaep'

if ($Uninstall) {
    if (Get-Command agy -ErrorAction SilentlyContinue) {
        agy plugin uninstall aaep
    }
    if (Test-Path -LiteralPath $globalPluginDir) {
        if ($PSCmdlet.ShouldProcess($globalPluginDir, 'Remove aaep plugin')) {
            $item = Get-Item -LiteralPath $globalPluginDir -Force
            if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                [System.IO.Directory]::Delete($globalPluginDir, $false)
            } else {
                Remove-Item -LiteralPath $globalPluginDir -Recurse -Force
            }
            Write-Host "[Uninstall] Removed aaep plugin from: $globalPluginDir" -ForegroundColor Green
        }
    }
    return
}

# 優先使用 agy 原生 plugin install 指令註冊
if (Get-Command agy -ErrorAction SilentlyContinue) {
    Write-Host "[Install] Registering plugin via agy CLI..." -ForegroundColor Cyan
    agy plugin install $pluginRoot
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[Install] Successfully registered aaep with agy CLI!" -ForegroundColor Green
        return
    }
}

# 降級備用方案：手動檔案部署
$pluginsParent = Split-Path $globalPluginDir -Parent
if (-not (Test-Path -LiteralPath $pluginsParent)) {
    New-Item -ItemType Directory -Path $pluginsParent -Force | Out-Null
}

if (Test-Path -LiteralPath $globalPluginDir) {
    Remove-Item -LiteralPath $globalPluginDir -Recurse -Force
}

Copy-Item -LiteralPath $pluginRoot -Destination $globalPluginDir -Recurse -Force
Write-Host "[Install] Successfully deployed aaep plugin to: $globalPluginDir" -ForegroundColor Green

