[Console]::InputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$payload = $null
try {
    $builder = [System.Text.StringBuilder]::new()
    while ($true) {
        $line = [Console]::In.ReadLine()
        if ($null -eq $line) { break }
        $null = $builder.AppendLine($line)
        $text = $builder.ToString().Trim().TrimStart([char]0xFEFF)
        if ($text.StartsWith("{") -and $text.EndsWith("}")) {
            try {
                $payload = $text | ConvertFrom-Json
                if ($null -ne $payload) { break }
            } catch {}
        }
    }
} catch {}

if ($null -eq $payload -and $input) {
    try {
        $text = ($input | Out-String).Trim().TrimStart([char]0xFEFF)
        $payload = $text | ConvertFrom-Json
    } catch {}
}

$response = @{ decision = "allow" }
if (-not $payload) {
    $response | ConvertTo-Json -Compress
    exit 0
}

$workspaces = $payload.workspacePaths
$rootDirs = if ($workspaces -and $workspaces.Count -gt 0) { $workspaces } else { @((Get-Location).Path) }
$reason = $payload.terminationReason
$fullyIdle = [bool]$payload.fullyIdle

# Helper function to create a Pure Shadow WIP snapshot (refs/wip/<branch>/current)
# HEAD remains clean and unmodified. Staging area is isolated from user's index.
function Create-ShadowWipSnapshot($repoDir) {
    if (-not (Test-Path $repoDir)) { return }
    Push-Location $repoDir
    try {
        $null = git rev-parse --is-inside-work-tree 2>$null
        if ($LASTEXITCODE -ne 0) { return }

        $status = git status --porcelain 2>$null
        if (-not $status) { return }

        # 1. Parse changed file names accurately without trimming bug
        $changedFiles = $status | ForEach-Object {
            $raw = $_
            if ($raw.Length -ge 4) {
                $pathPart = $raw.Substring(3).Trim()
                # Handle renamed files: "R  orig.txt -> new.txt"
                if ($pathPart -match '->\s*(.+)$') {
                    $pathPart = $matches[1].Trim()
                }
                $pathPart = $pathPart.Trim('"')
                [System.IO.Path]::GetFileName($pathPart)
            }
        } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique -First 3

        $isoTime = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        $fileSummary = if ($changedFiles) { " (" + ($changedFiles -join ", ") + ")" } else { "" }
        $subject = "WIP: [$isoTime]$fileSummary"

        # 2. Get current branch name safely
        $branchName = (git symbolic-ref --short -q HEAD 2>$null)
        if (-not $branchName) {
            $branchName = (git rev-parse --short HEAD 2>$null)
            if (-not $branchName) { $branchName = "main" }
        }
        # Sanitize ref name
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
                # Determine parent: previous shadow WIP commit, or current HEAD
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
                    git update-ref "$wipRef" $wipCommit.Trim() 2>$null
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

# Create WIP snapshot when model stops or task ends
$shouldCommit = $true
if ($reason) {
    $shouldCommit = ($reason -eq "model_stop" -and $fullyIdle)
}

if ($shouldCommit) {
    $targetRepos = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)

    foreach ($rootDir in $rootDirs) {
        if (-not (Test-Path $rootDir)) { continue }

        Push-Location $rootDir
        $null = git rev-parse --is-inside-work-tree 2>$null
        $isRootGit = ($LASTEXITCODE -eq 0)
        Pop-Location

        if ($isRootGit) {
            $null = $targetRepos.Add((Resolve-Path $rootDir).Path)
        } else {
            $gitDirs = Get-ChildItem -Path $rootDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
                Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

            foreach ($g in $gitDirs) {
                $parentDir = (Resolve-Path $g.Parent.FullName).Path
                $null = $targetRepos.Add($parentDir)
            }
        }
    }

    foreach ($repo in $targetRepos) {
        Create-ShadowWipSnapshot $repo
    }
}

$response | ConvertTo-Json -Compress