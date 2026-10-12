# =====================================================================
# 安裝 / 同步 Agent-Native Helpers 與 CLI 工具
# 用法:
#   .\profile\install.ps1                 # 僅同步 Helper 至 PowerShell Profile
#   .\profile\install.ps1 -InstallTools   # 自動檢測並安裝缺少之 winget 工具
# =====================================================================
param(
    [switch]$InstallTools,
    [switch]$WhatIf
)

# 1. 檢查並安裝 winget 套件依賴
$requiredTools = @(
    @{ Name = "ripgrep";  Cmd = "rg";       Id = "BurntSushi.ripgrep.MSVC" },
    @{ Name = "fd";       Cmd = "fd";       Id = "sharkdp.fd" },
    @{ Name = "ast-grep"; Cmd = "ast-grep"; Id = "ast-grep.ast-grep" },
    @{ Name = "jq";       Cmd = "jq";       Id = "jqlang.jq" },
    @{ Name = "uv";       Cmd = "uv";       Id = "astral-sh.uv" }
)

if ($InstallTools) {
    Write-Host "==> Checking Agent-Native CLI dependencies..." -ForegroundColor Cyan
    foreach ($tool in $requiredTools) {
        if (-not (Get-Command $tool.Cmd -ErrorAction SilentlyContinue)) {
            Write-Host "[Missing] $($tool.Name) not found. Installing via winget ($($tool.Id))..." -ForegroundColor Yellow
            if (-not $WhatIf) {
                winget install --id $tool.Id -e --accept-source-agreements --accept-package-agreements
            }
        } else {
            Write-Host "[OK] $($tool.Name) ($($tool.Cmd)) is already installed." -ForegroundColor Green
        }
    }
}

# 2. 同步 AgentHelpers 至系統 PowerShell Profile
$profilePath = $PROFILE.CurrentUserCurrentHost
if (-not (Test-Path $profilePath)) {
    $parentDir = Split-Path $profilePath
    if (-not (Test-Path $parentDir)) {
        New-Item -ItemType Directory -Path $parentDir -Force | Out-Null
    }
    New-Item -ItemType File -Path $profilePath -Force | Out-Null
}

$helperScriptPath = (Resolve-Path "$PSScriptRoot\AgentHelpers.ps1").Path
$existingProfile = Get-Content -Raw -Path $profilePath -ErrorAction SilentlyContinue

$startTag = "# >>> AGENT-NATIVE HELPERS (START) >>>"
$endTag   = "# <<< AGENT-NATIVE HELPERS (END) <<<"

$block = @"
$startTag
if (Test-Path "$helperScriptPath") {
    . "$helperScriptPath"
}
$endTag
"@

if ($existingProfile -match [regex]::Escape($startTag)) {
    Write-Host "==> [Update] Replacing existing Agent-Native helpers in $profilePath..." -ForegroundColor Cyan
    $regex = "(?s)" + [regex]::Escape($startTag) + ".*?" + [regex]::Escape($endTag)
    $newProfile = [regex]::Replace($existingProfile, $regex, $block)
} else {
    Write-Host "==> [Install] Appending Agent-Native helpers to $profilePath..." -ForegroundColor Green
    $newProfile = $existingProfile + "`r`n`r`n" + $block
}

if ($WhatIf) {
    Write-Host "[WhatIf] Target profile: $profilePath" -ForegroundColor Yellow
} else {
    Set-Content -Path $profilePath -Value $newProfile -Encoding utf8
    Write-Host "[Done] Successfully configured Agent-Native environment!" -ForegroundColor Green
}
