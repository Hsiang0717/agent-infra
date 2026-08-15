param(
    [Parameter(Mandatory=$false)]
    [string]$Message = "chore: finalize agent modifications"
)

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Squash-RepoWip($repoDir) {
    if (-not (Test-Path $repoDir)) { return }
    Push-Location $repoDir
    try {
        $null = git rev-parse --is-inside-work-tree 2>$null
        if ($LASTEXITCODE -ne 0) { return }

        $repoName = Split-Path $repoDir -Leaf
        $logLines = git log --format="%s" 2>$null
        $wipCount = 0

        foreach ($subject in $logLines) {
            if ($subject -match "^WIP(\s.*)?$") {
                $wipCount++
            } else {
                break
            }
        }

        if ($wipCount -eq 0) {
            Write-Host "[Info] [$repoName] No consecutive WIP commits found at HEAD." -ForegroundColor Yellow
            return
        }

        Write-Host "[Info] [$repoName] Found $wipCount consecutive WIP commit(s). Squashing..." -ForegroundColor Cyan

        # 3. Soft reset and squash
        $totalCommits = (git rev-list --count HEAD 2>$null)
        if ([int]$totalCommits -le $wipCount) {
            $rootCommit = (git rev-list --max-parents=0 HEAD 2>$null).Trim()
            git reset --soft $rootCommit
            git commit --amend -m "$Message" --no-verify
        } else {
            git reset --soft "HEAD~$wipCount"
            git commit -m "$Message" --no-verify
        }

        if ($LASTEXITCODE -eq 0) {
            $finalHash = (git rev-parse --short HEAD).Trim()
            Write-Host "[Success] [$repoName] Successfully squashed $wipCount WIP commit(s) into [$finalHash] $Message" -ForegroundColor Green
        }
    } finally {
        Pop-Location
    }
}

$currentDir = (Get-Location).Path
$null = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -eq 0) {
    Squash-RepoWip $currentDir
} else {
    # Scan for nested Git repositories (up to 3 levels deep)
    $gitDirs = Get-ChildItem -Path $currentDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

    if (-not $gitDirs) {
        Write-Host "[Error] Current directory is not a Git repository and no nested Git repositories were found." -ForegroundColor Red
        exit 1
    }

    foreach ($g in $gitDirs) {
        Squash-RepoWip $g.Parent.FullName
    }
}