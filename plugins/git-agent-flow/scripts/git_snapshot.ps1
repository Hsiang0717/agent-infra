param(
    [Parameter(Mandatory=$false, Position=0)]
    [string]$Message = ""
)

[Console]::InputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Create-ShadowSnapshot($repoDir, $customMsg) {
    if (-not (Test-Path $repoDir)) { return }
    Push-Location $repoDir
    try {
        $null = git rev-parse --is-inside-work-tree 2>$null
        if ($LASTEXITCODE -ne 0) { return }

        $repoName = Split-Path $repoDir -Leaf
        $status = git status --porcelain 2>$null
        if (-not $status) {
            Write-Host "[Info] [$repoName] Working tree is clean. No snapshot needed." -ForegroundColor Yellow
            return
        }

        # 1. Parse changed file names accurately
        $changedFiles = $status | ForEach-Object {
            $raw = $_
            if ($raw.Length -ge 4) {
                $pathPart = $raw.Substring(3).Trim()
                if ($pathPart -match '->\s*(.+)$') {
                    $pathPart = $matches[1].Trim()
                }
                $pathPart = $pathPart.Trim('"')
                [System.IO.Path]::GetFileName($pathPart)
            }
        } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique -First 4

        $isoTime = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        $fileSummary = if ($changedFiles) { " (" + ($changedFiles -join ", ") + ")" } else { "" }
        
        $subject = if ($customMsg -and $customMsg.Trim()) {
            "WIP: [$isoTime] $($customMsg.Trim())$fileSummary"
        } else {
            "WIP: [$isoTime]$fileSummary"
        }

        # 2. Get current branch name safely
        $branchName = (git symbolic-ref --short -q HEAD 2>$null)
        if (-not $branchName) {
            $branchName = (git rev-parse --short HEAD 2>$null)
            if (-not $branchName) { $branchName = "main" }
        }
        $safeBranch = $branchName -replace '[^\w\.\-\/]', '_'
        $wipRef = "refs/wip/$safeBranch/current"

        # 3. Use an isolated index to prevent race conditions with user's manual staging
        $gitDir = (git rev-parse --git-dir 2>$null).Trim()
        if (-not $gitDir) { return }

        $tempIndex = Join-Path $gitDir ("index_wip_shadow_" + [System.Guid]::NewGuid().ToString("N"))
        try {
            $realIndex = Join-Path $gitDir "index"
            if (Test-Path $realIndex) {
                Copy-Item -Path $realIndex -Destination $tempIndex -Force -ErrorAction SilentlyContinue
            }

            $env:GIT_INDEX_FILE = $tempIndex
            git add -A 2>$null
            $treeHash = (git write-tree 2>$null)
            if ($treeHash) {
                $treeHash = $treeHash.Trim()
            }

            if ($treeHash -and $LASTEXITCODE -eq 0) {
                $parentWip = (git rev-parse -q --verify "$wipRef" 2>$null)
                $headHash = (git rev-parse -q --verify HEAD 2>$null)

                $parentArgs = @()
                if ($parentWip -and $parentWip.Trim()) {
                    $parentArgs += "-p"
                    $parentArgs += $parentWip.Trim()
                } elseif ($headHash -and $headHash.Trim()) {
                    $parentArgs += "-p"
                    $parentArgs += $headHash.Trim()
                }

                $wipCommit = if ($parentArgs.Count -gt 0) {
                    (git commit-tree $treeHash @parentArgs -m "$subject" 2>$null)
                } else {
                    (git commit-tree $treeHash -m "$subject" 2>$null)
                }

                if ($wipCommit -and $wipCommit.Trim()) {
                    $snapHash = $wipCommit.Trim()
                    git update-ref "$wipRef" $snapHash 2>$null
                    $shortHash = $snapHash.Substring(0, [Math]::Min(7, $snapHash.Length))
                    Write-Host "[Snapshot Created] [$repoName] $shortHash on $wipRef" -ForegroundColor Green
                    Write-Host "                   Message: $subject" -ForegroundColor Gray
                }
            }
        } finally {
            Remove-Item Env:\GIT_INDEX_FILE -ErrorAction SilentlyContinue
            if (Test-Path $tempIndex) {
                Remove-Item -Path $tempIndex -Force -ErrorAction SilentlyContinue
            }
        }
    } finally {
        Pop-Location
    }
}

$currentDir = (Get-Location).Path
$null = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -eq 0) {
    Create-ShadowSnapshot $currentDir $Message
} else {
    $gitDirs = Get-ChildItem -Path $currentDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

    if (-not $gitDirs) {
        Write-Host "[Error] Current directory is not a Git repository." -ForegroundColor Red
        exit 1
    }

    foreach ($g in $gitDirs) {
        Create-ShadowSnapshot $g.Parent.FullName $Message
    }
}
