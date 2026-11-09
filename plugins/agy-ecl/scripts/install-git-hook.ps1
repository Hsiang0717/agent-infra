<#
.SYNOPSIS
Installs the ECL Zero-Token Pre-Commit Hook into .git/hooks/pre-commit
#>

param(
    [string]$TargetRepo = "."
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path $TargetRepo).Path
$gitDir = Join-Path $repoRoot ".git"

if (-not (Test-Path $gitDir)) {
    Write-Error "Target directory is not a Git repository: $repoRoot"
    exit 1
}

$hooksDir = Join-Path $gitDir "hooks"
if (-not (Test-Path $hooksDir)) {
    New-Item -ItemType Directory -Path $hooksDir -Force | Out-Null
}

$preCommitHookPath = Join-Path $hooksDir "pre-commit"

# Locate the hook script relative to plugins/agy-ecl
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$hookScriptPath = Join-Path $scriptDir "pre_commit_hook.py"

# Normalize path with forward slashes for sh compatibility
$hookScriptPathUnix = $hookScriptPath.Replace('\', '/')

$hookContent = @"
#!/bin/sh
# ECL Zero-Token Coordinate Synchronizer & Causal Audit Hook
if command -v uv >/dev/null 2>&1; then
    uv run python "$hookScriptPathUnix"
elif command -v python3 >/dev/null 2>&1; then
    python3 "$hookScriptPathUnix"
elif command -v python >/dev/null 2>&1; then
    python "$hookScriptPathUnix"
fi
"@

# Write pre-commit hook file with LF line endings and NO BOM
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($preCommitHookPath, $hookContent.Replace("`r`n", "`n"), $utf8NoBom)

Write-Host "[ECL] Pre-commit hook successfully installed at: $preCommitHookPath" -ForegroundColor Green
Write-Host "      Every git commit will now auto-calibrate .agent/semantics coordinates with 0 tokens." -ForegroundColor Cyan
