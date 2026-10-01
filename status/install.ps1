[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$Destination = (Join-Path $env:USERPROFILE '.antigravity')
)

$ErrorActionPreference = 'Stop'

$nativeDllPath = Join-Path $PSScriptRoot 'Status.Native.dll'
if (-not (Test-Path -LiteralPath $nativeDllPath)) {
    $code = @"
using System;
using System.Runtime.InteropServices;

public static class StatusPipe {
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr GetStdHandle(int nStdHandle);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool PeekNamedPipe(
        IntPtr hNamedPipe,
        IntPtr lpBuffer,
        uint nBufferSize,
        IntPtr lpBytesRead,
        out uint lpTotalBytesAvail,
        IntPtr lpBytesLeftThisMessage
    );

    public static uint GetAvailableBytes() {
        IntPtr hStdin = GetStdHandle(-10);
        if (hStdin == IntPtr.Zero || hStdin == new IntPtr(-1)) return 0;
        uint avail = 0;
        if (PeekNamedPipe(hStdin, IntPtr.Zero, 0, IntPtr.Zero, out avail, IntPtr.Zero)) {
            return avail;
        }
        return 0;
    }
}
"@
    Add-Type -TypeDefinition $code -OutputAssembly $nativeDllPath
}

$packageFiles = @(
    'statusline.ps1',
    'Status.Git.psm1',
    'Status.Power.psm1',
    'Status.Native.dll'
)

New-Item -ItemType Directory -Path $Destination -Force | Out-Null

foreach ($file in $packageFiles) {
    $sourcePath = Join-Path $PSScriptRoot $file
    $targetPath = Join-Path $Destination $file
    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "Package file not found: $sourcePath"
    }

    if ($PSCmdlet.ShouldProcess($targetPath, 'Install statusline package')) {
        Copy-Item -LiteralPath $sourcePath -Destination $targetPath -Force
    }
}

$rustExe = Join-Path $PSScriptRoot 'statusline-rs\target\release\statusline.exe'
if (Test-Path -LiteralPath $rustExe) {
    $targetExe = Join-Path $Destination 'statusline.exe'
    if ($PSCmdlet.ShouldProcess($targetExe, 'Install Rust statusline.exe')) {
        Copy-Item -LiteralPath $rustExe -Destination $targetExe -Force
    }
}

if ($WhatIfPreference) {
    Write-Output "Preview only; no files changed in: $Destination"
} else {
    Write-Output "Installed Antigravity statusline package to: $Destination"
}
