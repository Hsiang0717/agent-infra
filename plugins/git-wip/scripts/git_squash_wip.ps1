param(
    [Parameter(Mandatory=$false)]
    [string]$Message = "chore: finalize agent modifications"
)

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Finalize-RepoWip($repoDir) {
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

        # Check if there are legacy WIP commits directly on HEAD
        $logLines = git log --format="%s" -n 50 2>$null
        $legacyWipCount = 0
        foreach ($subject in $logLines) {
            if ($subject -match "^WIP(\b|:|\s).*$") {
                $legacyWipCount++
            } else {
                break
            }
        }

        # Check shadow snapshots
        $hasShadowWip = [bool](git rev-parse -q --verify "$wipRef" 2>$null)
        $shadowSnapshotCount = 0
        if ($hasShadowWip) {
            $headCommit = (git rev-parse -q --verify HEAD 2>$null)
            if ($headCommit) {
                $shadowList = git rev-list "$headCommit..$wipRef" 2>$null
                if ($shadowList) {
                    $shadowSnapshotCount = @($shadowList).Count
                }
            } else {
                $shadowList = git rev-list "$wipRef" 2>$null
                if ($shadowList) {
                    $shadowSnapshotCount = @($shadowList).Count
                }
            }
        }

        $status = git status --porcelain 2>$null

        if ($legacyWipCount -eq 0 -and -not $status -and $shadowSnapshotCount -eq 0) {
            Write-Host "[Info] [$repoName] Working tree is clean and no pending WIP snapshots found." -ForegroundColor Yellow
            return
        }

        Write-Host "[Info] [$repoName] Finalizing transaction on branch [$branchName]..." -ForegroundColor Cyan
        if ($shadowSnapshotCount -gt 0) {
            Write-Host "       Shadow Turn Snapshots: $shadowSnapshotCount" -ForegroundColor DarkGray
        }
        if ($legacyWipCount -gt 0) {
            Write-Host "       Legacy HEAD WIP Commits: $legacyWipCount" -ForegroundColor DarkGray
        }

        # 1. If legacy WIP commits exist on HEAD, soft-reset them first
        if ($legacyWipCount -gt 0) {
            $totalCommits = (git rev-list --count HEAD 2>$null)
            if ([int]$totalCommits -le $legacyWipCount) {
                $rootCommit = (git rev-list --max-parents=0 HEAD 2>$null).Trim()
                git reset --soft $rootCommit 2>$null
                git commit --amend -m "$Message" --no-verify 2>$null
            } else {
                git reset --soft "HEAD~$legacyWipCount" 2>$null
                git commit -m "$Message" --no-verify 2>$null
            }
        } else {
            # 2. Pure Shadow mode or dirty tree: Stage and create single clean formal commit
            git add -A 2>$null
            git commit -m "$Message" --no-verify 2>$null
        }

        if ($LASTEXITCODE -eq 0) {
            $finalHash = (git rev-parse --short HEAD 2>$null).Trim()
            Write-Host "[Success] [$repoName] Formal Commit: [$finalHash] $Message" -ForegroundColor Green

            # 3. Archive Shadow WIP history and align current shadow ref to new HEAD
            if ($hasShadowWip) {
                $archiveTime = Get-Date -Format "yyyyMMdd_HHmmss"
                $archiveRef = "refs/wip/$safeBranch/archive/$archiveTime"
                $currentWipHash = (git rev-parse "$wipRef" 2>$null).Trim()
                if ($currentWipHash) {
                    git update-ref "$archiveRef" "$currentWipHash" 2>$null
                    git update-ref "$wipRef" "$finalHash" 2>$null
                    Write-Host "          Archived WIP memory to $archiveRef" -ForegroundColor DarkGray
                }
            }
        } else {
            Write-Host "[Warning] [$repoName] No changes were committed or git commit returned an error." -ForegroundColor Yellow
        }
    } finally {
        Pop-Location
    }
}

$currentDir = (Get-Location).Path
$null = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -eq 0) {
    Finalize-RepoWip $currentDir
} else {
    $gitDirs = Get-ChildItem -Path $currentDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

    if (-not $gitDirs) {
        Write-Host "[Error] Current directory is not a Git repository and no nested Git repositories were found." -ForegroundColor Red
        exit 1
    }

    foreach ($g in $gitDirs) {
        Finalize-RepoWip $g.Parent.FullName
    }
}