# =====================================================================
# 安裝 / 同步 pwsh KeyBindings 設定至系統 PowerShell Profile
# 用法:
#   .\pwsh\install.ps1
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

$keyBindingsPath = (Resolve-Path "$PSScriptRoot\KeyBindings.ps1").Path
$existingProfile = Get-Content -Raw -Path $profilePath -ErrorAction SilentlyContinue

$startTag = "# >>> POWERSHELL KEYBINDINGS (START) >>>"
$endTag   = "# <<< POWERSHELL KEYBINDINGS (END) <<<"

$block = @"
$startTag
if (Test-Path "$keyBindingsPath") {
    . "$keyBindingsPath"
}
$endTag
"@

if ($existingProfile -match [regex]::Escape($startTag)) {
    Write-Host "==> [Update] Replacing existing KeyBindings in $profilePath..." -ForegroundColor Cyan
    $regex = "(?s)" + [regex]::Escape($startTag) + ".*?" + [regex]::Escape($endTag)
    $newProfile = [regex]::Replace($existingProfile, $regex, $block)
} else {
    Write-Host "==> [Install] Appending KeyBindings to $profilePath..." -ForegroundColor Green
    $newProfile = $existingProfile + "`r`n`r`n" + $block
}

if ($WhatIf) {
    Write-Host "[WhatIf] Target profile: $profilePath" -ForegroundColor Yellow
} else {
    Set-Content -Path $profilePath -Value $newProfile -Encoding utf8
    Write-Host "[Done] Successfully configured PowerShell KeyBindings!" -ForegroundColor Green
}
