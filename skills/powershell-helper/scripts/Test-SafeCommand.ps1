param (
    [string]$CommandLine
)

if ([string]::IsNullOrWhiteSpace($CommandLine)) {
    Write-Host "Empty command line provided." -ForegroundColor Yellow
    exit 0
}

$unsafePatterns = @(
    "rm\s+-rf",
    "\bcat\b",
    "\bls\b",
    "\bmv\b",
    "\bcp\b",
    "\bpwd\b",
    "export\s+[A-Za-z0-9_]+="
)

$errors = @()

foreach ($pattern in $unsafePatterns) {
    if ($CommandLine -match $pattern) {
        $errors += "Command contains non-native or unsafe alias/command matching pattern '$pattern'."
    }
}

# Check for Unix line continuations
if ($CommandLine -match "[^\`\s]\s+\\\s*$") {
    $errors += "Command uses Unix line continuation '\' instead of PowerShell backtick '`'."
}

if ($errors.Count -gt 0) {
    Write-Host "Safety check failed:" -ForegroundColor Red
    foreach ($err in $errors) {
        Write-Host "  - $err" -ForegroundColor Yellow
    }
    exit 1
} else {
    Write-Host "Safety check passed." -ForegroundColor Green
    exit 0
}
