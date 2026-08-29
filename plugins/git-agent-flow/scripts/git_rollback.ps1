param(
    [Parameter(Mandatory=$false, Position=0)]
    [string]$Target = "",

    [Parameter(Mandatory=$false)]
    [string]$File = "",

    [Parameter(Mandatory=$false)]
    [switch]$List,

    [Parameter(Mandatory=$false)]
    [switch]$HardResetHead
)

[Console]::InputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Rollback-Repo($repoDir) {
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

        if ($List) {
            Write-Host "=== [$repoName] Shadow Snapshots on $wipRef ===" -ForegroundColor Cyan
            $exists = git rev-parse -q --verify "$wipRef" 2>$null
            if ($exists) {
                git log --oneline --graph -n 15 "$wipRef"
            } else {
                Write-Host "No active shadow snapshot found for branch [$safeBranch]." -ForegroundColor Yellow
            }
            return
        }

        if ($HardResetHead) {
            Write-Host "[Reset] [$repoName] Discarding all WIP and restoring workspace to HEAD..." -ForegroundColor Yellow
            git checkout HEAD -- . 2>$null
            git clean -fd 2>$null
            git update-ref "$wipRef" (git rev-parse HEAD) 2>$null
            Write-Host "[Success] [$repoName] Reset complete." -ForegroundColor Green
            return
        }

        # Check if wipRef exists
        $hasWip = git rev-parse -q --verify "$wipRef" 2>$null
        if (-not $hasWip) {
            Write-Host "[Error] [$repoName] No shadow snapshot ($wipRef) found to restore from." -ForegroundColor Red
            return
        }

        # Determine target ref/commit
        $targetCommit = "$wipRef"
        if ($Target) {
            if ($Target -match '^[~^0-9]+$') {
                $targetCommit = "$wipRef$Target"
            } elseif ($Target -match '^refs\/') {
                $targetCommit = $Target
            } elseif ($Target -match '^[0-9a-fA-F]{6,40}$') {
                $targetCommit = $Target
            } else {
                # Target might be a file path if $File is empty
                if (-not $File) {
                    $File = $Target
                    $targetCommit = "$wipRef~1"
                }
            }
        } else {
            # Default rollback to previous turn
            $targetCommit = "$wipRef~1"
        }

        # Check if targetCommit is valid
        $resolvedHash = git rev-parse -q --verify "$targetCommit" 2>$null
        if (-not $resolvedHash) {
            # If ~1 does not exist, fall back to current wipRef
            $resolvedHash = git rev-parse -q --verify "$wipRef" 2>$null
            $targetCommit = "$wipRef"
        }

        if ($File -and $File.Trim()) {
            Write-Host "[Rollback] [$repoName] Restoring file [$File] from ($targetCommit)..." -ForegroundColor Cyan
            git checkout "$targetCommit" -- "$File" 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "[Success] Restored [$File] from $targetCommit" -ForegroundColor Green
            } else {
                Write-Host "[Error] Failed to restore [$File] from $targetCommit" -ForegroundColor Red
            }
        } else {
            Write-Host "[Rollback] [$repoName] Restoring entire workspace from ($targetCommit)..." -ForegroundColor Cyan
            git checkout "$targetCommit" -- . 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "[Success] Restored entire workspace to $targetCommit" -ForegroundColor Green
            } else {
                Write-Host "[Error] Failed to rollback workspace to $targetCommit" -ForegroundColor Red
            }
        }
    } finally {
        Pop-Location
    }
}

$currentDir = (Get-Location).Path
$null = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -eq 0) {
    Rollback-Repo $currentDir
} else {
    $gitDirs = Get-ChildItem -Path $currentDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

    if (-not $gitDirs) {
        Write-Host "[Error] Current directory is not a Git repository." -ForegroundColor Red
        exit 1
    }

    foreach ($g in $gitDirs) {
        Rollback-Repo $g.Parent.FullName
    }
}
