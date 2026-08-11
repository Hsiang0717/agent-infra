[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$Destination = (Join-Path $env:USERPROFILE '.antigravity')
)

$ErrorActionPreference = 'Stop'
$packageFiles = @(
    'statusline.ps1',
    'Status.Git.psm1',
    'Status.Power.psm1'
)

New-Item -ItemType Directory -Path $Destination -Force | Out-Null

foreach ($file in $packageFiles) {
    $sourcePath = Join-Path $PSScriptRoot $file
    $targetPath = Join-Path $Destination $file
    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "Package file not found: $sourcePath"
    }

    if ($PSCmdlet.ShouldProcess($targetPath, 'Install statusline package')) {
        Copy-Item -LiteralPath $sourcePath -Destination $targetPath -Force
    }
}

if ($WhatIfPreference) {
    Write-Output "Preview only; no files changed in: $Destination"
} else {
    Write-Output "Installed Antigravity statusline to: $Destination"
}
