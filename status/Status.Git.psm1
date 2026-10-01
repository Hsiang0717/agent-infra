Set-StrictMode -Version 2.0

function Find-GitDir {
    param([Parameter(Mandatory = $true)][string]$Path)

    $current = $Path
    while ($current) {
        $candidate = Join-Path $current '.git'
        if (Test-Path -LiteralPath $candidate -PathType Container) {
            return $candidate
        }

        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            try {
                $content = (Get-Content -LiteralPath $candidate -Raw -ErrorAction Stop).Trim()
                if ($content -match '^gitdir:\s*(.+)$') {
                    $gitDirTarget = $Matches[1].Trim()
                    if (-not [System.IO.Path]::IsPathRooted($gitDirTarget)) {
                        $gitDirTarget = [System.IO.Path]::GetFullPath((Join-Path $current $gitDirTarget))
                    }
                    if (Test-Path -LiteralPath $gitDirTarget -PathType Container) {
                        return $gitDirTarget
                    }
                }
            } catch {}
        }

        $parent = Split-Path $current -Parent
        if (-not $parent -or $parent -eq $current) { break }
        $current = $parent
    }

    return $null
}

function Get-GitBranch {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Container)) { return '' }

    try {
        $resolvedPath = (Resolve-Path -LiteralPath $Path -ErrorAction Stop).Path
        $gitDir = Find-GitDir -Path $resolvedPath
        if (-not $gitDir) { return '' }

        $headFile = Join-Path $gitDir 'HEAD'
        if (-not (Test-Path -LiteralPath $headFile -PathType Leaf)) { return '' }

        $head = (Get-Content -LiteralPath $headFile -Raw -ErrorAction Stop).Trim()
        if ($head -match '^ref:\s*refs/heads/(.+)$') {
            return $Matches[1].Trim()
        }
        if ($head.Length -ge 7) {
            return $head.Substring(0, 7)
        }
    } catch {}

    return ''
}

Export-ModuleMember -Function Get-GitBranch
