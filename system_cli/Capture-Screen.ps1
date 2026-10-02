<#
.SYNOPSIS
    Screen capture utility optimized for autonomous AI Agent execution.
.DESCRIPTION
    Captures full screen or a targeted window (by PID, ProcessName, or WindowTitle)
    and outputs the absolute image path for `view_file` inspection.
#>
[CmdletBinding(DefaultParameterSetName = 'FullScreen')]
param(
    [Parameter(ParameterSetName = 'ByProcessName')]
    [string]$ProcessName,

    [Parameter(ParameterSetName = 'ById')]
    [Alias('PID')]
    [int]$Id,

    [Parameter(ParameterSetName = 'ByTitle')]
    [string]$WindowTitle,

    [Parameter(ParameterSetName = 'FullScreen')]
    [switch]$FullScreen,

    [Parameter(Mandatory = $false)]
    [string]$OutputPath
)

# Forward execution to Windows PowerShell 5.1 when running under PowerShell Core (pwsh 7+)
# to support desktop device context capture inside sandbox environments.
if ($PSVersionTable.PSEdition -eq 'Core') {
    $winPS = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    if (Test-Path $winPS) {
        $forwardArgs = @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $PSCommandPath)
        foreach ($key in $PSBoundParameters.Keys) {
            $val = $PSBoundParameters[$key]
            if ($val -is [switch]) {
                if ($val.IsPresent) { $forwardArgs += "-$key" }
            } else {
                $forwardArgs += "-$key"
                $forwardArgs += "$val"
            }
        }
        & $winPS @forwardArgs
        exit $LASTEXITCODE
    }
}

Add-Type -AssemblyName System.Drawing, System.Windows.Forms

# Win32 API for window manipulation and capture
$win32Sig = @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Threading;

public class Win32Window {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBkgnd, uint nFlags);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll", SetLastError = true)] public static extern IntPtr OpenDesktop(string lpszDesktop, uint dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)] public static extern IntPtr OpenInputDesktop(uint dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool SetThreadDesktop(IntPtr hDesktop);
    [DllImport("user32.dll", SetLastError = true)] public static extern bool CloseDesktop(IntPtr hDesktop);
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);
    [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr hObject, int nXDest, int nYDest, int nWidth, int nHeight, IntPtr hObjectSource, int nXSrc, int nYSrc, int dwRop);
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int nIndex);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    public static string CaptureScreenDesktopThread(string outputPath, out int outWidth, out int outHeight) {
        string error = null;
        int w = 0, h = 0;
        Thread t = new Thread(new ThreadStart(delegate() {
            IntPtr hDesk = IntPtr.Zero;
            try {
                IntPtr hIn = OpenInputDesktop(0, false, 0x0100);
                if (hIn == IntPtr.Zero && Marshal.GetLastWin32Error() == 5) {
                    throw new Exception("The Windows desktop session is locked or disconnected (ERROR_ACCESS_DENIED). Screen capture requires an active, unlocked desktop session.");
                }
                if (hIn != IntPtr.Zero) {
                    CloseDesktop(hIn);
                }

                hDesk = OpenDesktop("Default", 0, false, 0x01FF);
                if (hDesk != IntPtr.Zero) {
                    SetThreadDesktop(hDesk);
                }
                int width = GetSystemMetrics(0);
                int height = GetSystemMetrics(1);
                w = width;
                h = height;
                using (Bitmap bmp = new Bitmap(width, height)) {
                    using (Graphics g = Graphics.FromImage(bmp)) {
                        IntPtr hdcDest = g.GetHdc();
                        IntPtr hdcSrc = GetDC(IntPtr.Zero);
                        bool blt = BitBlt(hdcDest, 0, 0, width, height, hdcSrc, 0, 0, 0x00CC0020);
                        ReleaseDC(IntPtr.Zero, hdcSrc);
                        g.ReleaseHdc(hdcDest);
                        if (!blt) throw new Exception("BitBlt failed from desktop DC.");
                        bmp.Save(outputPath, ImageFormat.Png);
                    }
                }
            } catch (Exception ex) {
                error = ex.Message;
            } finally {
                if (hDesk != IntPtr.Zero) CloseDesktop(hDesk);
            }
        }));
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
        outWidth = w;
        outHeight = h;
        return error;
    }
}
"@
Add-Type -TypeDefinition $win32Sig -ReferencedAssemblies System.Drawing -ErrorAction SilentlyContinue

# Resolve default output path if not specified
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $PSScriptRoot "screenshot.png"
}
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)
$outDir = [System.IO.Path]::GetDirectoryName($OutputPath)
if (-not [string]::IsNullOrEmpty($outDir) -and -not (Test-Path -Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
}

function Show-AvailableWindows {
    Write-Host "`n--- Running GUI Processes (Reference for Agent) ---" -ForegroundColor Yellow
    Get-Process | Where-Object { $_.MainWindowHandle -ne 0 -and -not [string]::IsNullOrWhiteSpace($_.MainWindowTitle) } |
        Select-Object Id, ProcessName, MainWindowTitle |
        Format-Table -AutoSize | Out-String | Write-Host
}

# 1. Target Process / Window Resolution
$targetProc = $null

if ($PSCmdlet.ParameterSetName -eq 'ByProcessName') {
    $targetProc = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne 0 } |
        Select-Object -First 1
    if (-not $targetProc) {
        Write-Error "No running process with visible window found matching ProcessName: '$ProcessName'"
        Show-AvailableWindows
        exit 1
    }
}
elseif ($PSCmdlet.ParameterSetName -eq 'ById') {
    $targetProc = Get-Process -Id $Id -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne 0 }
    if (-not $targetProc) {
        Write-Error "No process with visible window found matching PID: $Id"
        Show-AvailableWindows
        exit 1
    }
}
elseif ($PSCmdlet.ParameterSetName -eq 'ByTitle') {
    $targetProc = Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -like "*$WindowTitle*" } |
        Select-Object -First 1
    if (-not $targetProc) {
        Write-Error "No window found matching WindowTitle pattern: '*$WindowTitle*'"
        Show-AvailableWindows
        exit 1
    }
}

# 2. Window Capture Mode
if ($targetProc) {
    $hwnd = $targetProc.MainWindowHandle

    # If window is minimized, restore it
    if ([Win32Window]::IsIconic($hwnd)) {
        [Win32Window]::ShowWindowAsync($hwnd, 9) | Out-Null # 9 = SW_RESTORE
        Start-Sleep -Milliseconds 200
    }

    $rect = New-Object Win32Window+RECT
    [Win32Window]::GetWindowRect($hwnd, [ref]$rect) | Out-Null

    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top

    if ($width -le 0 -or $height -le 0) {
        Write-Error "Invalid window dimensions ($($width)x$($height)). The window might be hidden or minimized."
        exit 1
    }

    $bmp = New-Object System.Drawing.Bitmap($width, $height)
    $gfx = [System.Drawing.Graphics]::FromImage($bmp)
    $hdc = $gfx.GetHdc()

    # PW_RENDERFULLCONTENT = 2
    [Win32Window]::PrintWindow($hwnd, $hdc, 2) | Out-Null

    $gfx.ReleaseHdc($hdc)
    $gfx.Dispose()

    $bmp.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()

    Write-Host "[CAPTURED_IMAGE]"
    Write-Host "Mode: Window"
    Write-Host "Target: $($targetProc.ProcessName) (PID: $($targetProc.Id), Title: '$($targetProc.MainWindowTitle)')"
    Write-Host "Resolution: ${width}x${height}"
    Write-Host "Path: $OutputPath"
    exit 0
}

# 3. Full Screen Capture Mode
try {
    $captured = $false
    $capWidth = 0
    $capHeight = 0

    try {
        $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
        $bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
        $gfx = [System.Drawing.Graphics]::FromImage($bmp)
        $gfx.CopyFromScreen([System.Drawing.Point]::Empty, [System.Drawing.Point]::Empty, $bounds.Size)
        $gfx.Dispose()

        $bmp.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()

        $captured = $true
        $capWidth = $bounds.Width
        $capHeight = $bounds.Height
    }
    catch {
        # Fallback for sandbox / non-interactive desktop environments (e.g. Antigravity runner)
        $outW = 0
        $outH = 0
        $err = [Win32Window]::CaptureScreenDesktopThread($OutputPath, [ref]$outW, [ref]$outH)
        if ($err) { throw $err }
        $captured = $true
        $capWidth = $outW
        $capHeight = $outH
    }

    if ($captured) {
        Write-Host "[CAPTURED_IMAGE]"
        Write-Host "Mode: FullScreen"
        Write-Host "Resolution: ${capWidth}x${capHeight}"
        Write-Host "Path: $OutputPath"
        exit 0
    }
}
catch {
    Write-Error "FullScreen capture failed: $_"
    exit 1
}
