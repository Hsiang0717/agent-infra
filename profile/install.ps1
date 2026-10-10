# =====================================================================
# 安裝 / 同步 Agent-Native Helpers 至系統 PowerShell Profile
# =====================================================================
param(
    [switch]$WhatIf
)

$profilePath = $PROFILE.CurrentUserCurrentHost
if (-not (Test-Path $profilePath)) {
    $parentDir = Split-Path $profilePath
    if (-not (Test-Path $parentDir)) {
        New-Item -ItemType Directory -Path $parentDir -Force | Out-Null
    }
    New-Item -ItemType File -Path $profilePath -Force | Out-Null
}

$helperContent = Get-Content -Raw -Path "$PSScriptRoot\AgentHelpers.ps1"
$existingProfile = Get-Content -Raw -Path $profilePath -ErrorAction SilentlyContinue

$startTag = "# >>> AGENT-NATIVE HELPERS (START) >>>"
$endTag   = "# <<< AGENT-NATIVE HELPERS (END) <<<"

$block = @"
$startTag
$helperContent
$endTag
"@

if ($existingProfile -match [regex]::Escape($startTag)) {
    Write-Host "[Update] Replacing existing Agent-Native helpers in $profilePath..." -ForegroundColor Cyan
    $regex = "(?s)" + [regex]::Escape($startTag) + ".*?" + [regex]::Escape($endTag)
    $newProfile = [regex]::Replace($existingProfile, $regex, $block)
} else {
    Write-Host "[Install] Appending Agent-Native helpers to $profilePath..." -ForegroundColor Green
    $newProfile = $existingProfile + "`r`n`r`n" + $block
}

if ($WhatIf) {
    Write-Host "[WhatIf] Would update profile at: $profilePath" -ForegroundColor Yellow
} else {
    Set-Content -Path $profilePath -Value $newProfile -Encoding utf8
    Write-Host "[Done] Successfully synchronized Agent-Native helpers to $profilePath!" -ForegroundColor Green
}
