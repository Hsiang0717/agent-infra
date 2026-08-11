Set-StrictMode -Version 2.0

function Invoke-GitWithTimeout {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [int]$TimeoutMs = 200
    )

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = 'git'
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true

    # ProcessStartInfo.ArgumentList is available in newer .NET versions.
    # Keep a quoted fallback for Windows PowerShell 5.1 compatibility.
    if ($psi.PSObject.Properties.Name -contains 'ArgumentList') {
        foreach ($arg in @('-C', $Path) + $Arguments) {
            [void]$psi.ArgumentList.Add([string]$arg)
        }
    } else {
        $allArgs = @('-C', $Path) + $Arguments
        $psi.Arguments = (($allArgs | ForEach-Object {
            '"' + ([string]$_ -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
        }) -join ' ')
    }

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    try {
        if (-not $proc.Start()) { return $null }

        $stdoutTask = $proc.StandardOutput.ReadToEndAsync()
        $stderrTask = $proc.StandardError.ReadToEndAsync()
        if (-not $proc.WaitForExit($TimeoutMs)) {
            try { $proc.Kill() } catch {}
            return $null
        }
        $proc.WaitForExit()
        if ($proc.ExitCode -ne 0) { return $null }
        return $stdoutTask.Result
    } catch {
        return $null
    } finally {
        try { $proc.Dispose() } catch {}
    }
}

function Get-GitStatus {
    param(
        [string]$Path,
        [int]$TimeoutMs = 200
    )

    if (-not $Path -or -not (Test-Path -LiteralPath $Path -PathType Container)) {
        return [pscustomobject]@{ Branch = ''; Dirty = $false; Available = $false }
    }

    $env:GIT_OPTIONAL_LOCKS = '0'
    $output = Invoke-GitWithTimeout -Path $Path -Arguments @('status', '--porcelain', '--branch') -TimeoutMs $TimeoutMs
    if (-not $output) {
        return [pscustomobject]@{ Branch = ''; Dirty = $false; Available = $false }
    }

    $lines = @($output -split "`r?`n" | Where-Object { $_ })
    $branch = ''
    $dirty = $false
    if ($lines.Count -gt 0 -and $lines[0] -match '^##\s+(.+?)(?:\.\.\S+)?(?:\s|$)') {
        $branch = $Matches[1].Trim()
    }
    if ($lines.Count -gt 1) { $dirty = $true }

    return [pscustomobject]@{ Branch = $branch; Dirty = $dirty; Available = $true }
}

Export-ModuleMember -Function Get-GitStatus
