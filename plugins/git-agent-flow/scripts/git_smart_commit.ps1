param(
    [Parameter(Mandatory=$false, Position=0)]
    [string]$Message,

    [Parameter(Mandatory=$false)]
    [string]$PlanJson
)

[Console]::InputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Execute-SmartCommit($repoDir, $commitMessage, $jsonPlan) {
    if (-not (Test-Path $repoDir)) { return }
    Push-Location $repoDir
    try {
        $null = git rev-parse --is-inside-work-tree 2>$null
        if ($LASTEXITCODE -ne 0) { return }

        $repoName = Split-Path $repoDir -Leaf
        $branchName = (git symbolic-ref --short -q HEAD 2>$null)
        if (-not $branchName) {
            $branchName = (git rev-parse --short HEAD 2>$null)
            if (-not $branchName) { $branchName = "HEAD" }
        }

        $status = git status --porcelain 2>$null
        if (-not $status) {
            Write-Host "[Info] [$repoName] Working tree is clean. No modifications to commit." -ForegroundColor Yellow
            return
        }

        # Parse commit groups
        $commitGroups = @()
        if ($jsonPlan -and $jsonPlan.Trim()) {
            try {
                $parsed = $jsonPlan | ConvertFrom-Json
                $commitGroups = @($parsed)
            } catch {
                Write-Host "[Warning] [$repoName] Failed to parse PlanJson, falling back to single commit. Error: $_" -ForegroundColor Yellow
            }
        }

        if (-not $commitGroups -or $commitGroups.Count -eq 0) {
            $finalMsg = if ($commitMessage -and $commitMessage.Trim()) { $commitMessage } else { "chore: apply agent modifications" }
            $commitGroups = @(
                @{
                    message = $finalMsg
                    files = @()
                }
            )
        }

        Write-Host "[Info] [$repoName] Executing Smart Commit(s) on branch [$branchName]..." -ForegroundColor Cyan

        $createdCommits = 0
        foreach ($group in $commitGroups) {
            $msg = if ($group.message) { $group.message } elseif ($group.msg) { $group.msg } else { "chore: update files" }
            $files = if ($group.files) { @($group.files) } else { @() }

            if ($files.Count -gt 0) {
                foreach ($f in $files) {
                    git add -- "$f"
                }
            } else {
                git add -A
            }

            # Check if index has staged changes
            $stagedDiff = git diff --cached --name-only
            if ($stagedDiff) {
                $commitOutput = git commit -m "$msg" --no-verify 2>&1
                if ($LASTEXITCODE -eq 0) {
                    $commitHash = (git rev-parse --short HEAD 2>$null).Trim()
                    Write-Host "  [Commit] [$commitHash] $msg" -ForegroundColor Green
                    $createdCommits++
                } else {
                    Write-Host "  [Error] Failed to commit: $msg" -ForegroundColor Red
                    Write-Host "          $commitOutput" -ForegroundColor DarkRed
                }
            }
        }

        # Check remaining working tree status
        $remainingStatus = git status --porcelain 2>$null
        if ($remainingStatus) {
            Write-Host "  [Note] Some unstaged or untracked changes remain in the working directory." -ForegroundColor DarkYellow
        } else {
            Write-Host "  [Success] All changes committed cleanly." -ForegroundColor Green
        }

    } finally {
        Pop-Location
    }
}

$currentDir = (Get-Location).Path
$null = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -eq 0) {
    Execute-SmartCommit $currentDir $Message $PlanJson
} else {
    $gitDirs = Get-ChildItem -Path $currentDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

    if (-not $gitDirs) {
        Write-Host "[Error] Current directory is not a Git repository." -ForegroundColor Red
        exit 1
    }

    foreach ($g in $gitDirs) {
        Execute-SmartCommit $g.Parent.FullName $Message $PlanJson
    }
}
