param(
    [Parameter(Mandatory=$false)]
    [string]$Message,

    [Parameter(Mandatory=$false)]
    [string]$PlanJson
)

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
            if (-not $branchName) { $branchName = "main" }
        }
        $safeBranch = $branchName -replace '[^\w\.\-\/]', '_'
        $wipRef = "refs/wip/$safeBranch/current"

        $status = git status --porcelain 2>$null
        if (-not $status) {
            Write-Host "[Info] [$repoName] Working tree is clean. No modifications to commit." -ForegroundColor Yellow
            return
        }

        # Parse commit groups
        $commitGroups = @()
        if ($jsonPlan -and $jsonPlan.Trim()) {
            try {
                $commitGroups = $jsonPlan | ConvertFrom-Json
            } catch {
                Write-Host "[Warning] [$repoName] Failed to parse PlanJson, falling back to single commit." -ForegroundColor Yellow
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

        $hasShadowWip = [bool](git rev-parse -q --verify "$wipRef" 2>$null)
        $currentWipHash = if ($hasShadowWip) { (git rev-parse "$wipRef" 2>$null).Trim() } else { $null }

        $createdCommits = 0
        foreach ($group in $commitGroups) {
            $msg = if ($group.message) { $group.message } elseif ($group.msg) { $group.msg } else { "chore: update files" }
            $files = if ($group.files) { @($group.files) } else { @() }

            if ($files.Count -gt 0) {
                foreach ($f in $files) {
                    git add -- "$f" 2>$null
                }
            } else {
                git add -A 2>$null
            }

            # Check if index has staged changes
            $stagedDiff = git diff --cached --name-only 2>$null
            if ($stagedDiff) {
                git commit -m "$msg" --no-verify 2>$null
                if ($LASTEXITCODE -eq 0) {
                    $commitHash = (git rev-parse --short HEAD 2>$null).Trim()
                    Write-Host "  [Commit] [$commitHash] $msg" -ForegroundColor Green
                    $createdCommits++
                }
            }
        }

        # If any unstaged files remain and this was a multi-group commit, check status
        $remainingStatus = git status --porcelain 2>$null
        if ($remainingStatus) {
            Write-Host "  [Note] Some unstaged or untracked changes remain in the working directory." -ForegroundColor DarkYellow
        }

        # Synchronize and Archive Shadow WIP Ref
        if ($createdCommits -gt 0) {
            $newHeadHash = (git rev-parse HEAD 2>$null).Trim()
            $archiveTime = Get-Date -Format "yyyyMMdd_HHmmss"
            $archiveRef = "refs/wip/$safeBranch/archive/$archiveTime"

            if ($currentWipHash) {
                git update-ref "$archiveRef" "$currentWipHash" 2>$null
                Write-Host "  [Shadow] Archived WIP history to $archiveRef" -ForegroundColor DarkGray
            }

            git update-ref "$wipRef" "$newHeadHash" 2>$null
            Write-Host "  [Shadow] Re-aligned $wipRef to HEAD ($newHeadHash)" -ForegroundColor DarkGray
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
