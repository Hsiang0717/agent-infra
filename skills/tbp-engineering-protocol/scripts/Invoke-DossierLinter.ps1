param (
    [string]$Path = ".agents/ENGINEERING_DOSSIER.md"
)

if (-not (Test-Path $Path)) {
    Write-Error "Dossier file not found at $Path"
    exit 1
}

$content = Get-Content -Path $Path
$errors = @()
$lineNum = 0

foreach ($line in $content) {
    $lineNum++
    # Trim leading space and markdown bullets
    $trimmed = $line.Trim()
    if ($trimmed -match '^\*\s+(.+)$' -or $trimmed -match '^-\s+(.+)$') {
        $bulletText = $Matches[1]
        $words = $bulletText -split '\s+' | Where-Object { $_ -ne "" }
        if ($words.Count -gt 30) {
            $errors += "Line $lineNum exceeds 30 words ($($words.Count) words): '$($bulletText.SubString(0, [Math]::Min(30, $bulletText.Length)))...'"
        }
    }
}

if ($errors.Count -gt 0) {
    Write-Host "Dossier linting failed with $($errors.Count) error(s):" -ForegroundColor Red
    foreach ($err in $errors) {
        Write-Host "  - $err" -ForegroundColor Yellow
    }
    exit 1
} else {
    Write-Host "Dossier linting passed successfully." -ForegroundColor Green
    exit 0
}
