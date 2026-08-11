Set-StrictMode -Version 2.0

function Get-PowerStatus {
    if ($env:ANTIGRAVITY_STATUS_NO_POWER -eq '1') { return $null }
    if ($env:OS -notlike '*Windows*' -and -not $env:COMPUTERNAME) { return $null }

    try {
        Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
        $status = [System.Windows.Forms.SystemInformation]::PowerStatus
        if (-not $status) { return $null }

        $pct = [int][Math]::Round((100 * [double]$status.BatteryLifePercent))
        $pct = [Math]::Max(0, [Math]::Min(100, $pct))
        return [pscustomobject]@{
            Percent = $pct
            LineStatus = [string]$status.PowerLineStatus
        }
    } catch {
        return $null
    }
}

Export-ModuleMember -Function Get-PowerStatus
