[Console]::InputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$rawInput = ($input | Out-String).TrimStart([char]0xFEFF).Trim()
if (-not $rawInput) {
    $rawInput = [Console]::In.ReadToEnd().TrimStart([char]0xFEFF).Trim()
}

$payload = $null
if ($rawInput) {
    try { $payload = $rawInput | ConvertFrom-Json } catch {}
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

# Helper function to commit WIP in a git repository if it has dirty changes
function Commit-WipIfDirty($repoDir) {
    if (-not (Test-Path $repoDir)) { return }
    Push-Location $repoDir
    try {
        $null = git rev-parse --is-inside-work-tree 2>$null
        if ($LASTEXITCODE -eq 0) {
            $status = git status --porcelain 2>$null
            if ($status) {
                git add -A 2>$null
                git commit -m "WIP" --no-verify 2>$null
            }
        }
    } finally {
        Pop-Location
    }
}

# Create WIP snapshot only when agent finishes turn normally
if ($reason -eq "model_stop" -and $fullyIdle) {
    $targetRepos = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)

    foreach ($rootDir in $rootDirs) {
        if (-not (Test-Path $rootDir)) { continue }

        # 1. Check if the root directory itself is a Git repository
        Push-Location $rootDir
        $null = git rev-parse --is-inside-work-tree 2>$null
        $isRootGit = ($LASTEXITCODE -eq 0)
        Pop-Location

        if ($isRootGit) {
            $null = $targetRepos.Add((Resolve-Path $rootDir).Path)
        } else {
            # 2. Search for nested Git repositories (up to 3 levels deep)
            $gitDirs = Get-ChildItem -Path $rootDir -Directory -Recurse -Depth 3 -Force -ErrorAction SilentlyContinue |
                Where-Object { $_.Name -eq ".git" -and $_.FullName -notmatch "node_modules|\.gemini|\.cache|vendor" }

            foreach ($g in $gitDirs) {
                $parentDir = (Resolve-Path $g.Parent.FullName).Path
                $null = $targetRepos.Add($parentDir)
            }
        }
    }

    foreach ($repo in $targetRepos) {
        Commit-WipIfDirty $repo
    }
}

$response | ConvertTo-Json -Compress