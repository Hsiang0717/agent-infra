# Simple health check script for the agent harness setup
$dossierPath = Join-Path $PSScriptRoot "..\..\.agents\ENGINEERING_DOSSIER.md"
$hookPath = Join-Path $PSScriptRoot "..\agy_hooks\hooks\inject-plan.ps1"

Write-Host "--- Harness Engineering Health Check ---" -ForegroundColor Cyan

# 1. Check Execution Policy
$policy = Get-ExecutionPolicy
Write-Host "Execution Policy: $policy"
if ($policy -in @("Restricted", "AllSigned")) {
    Write-Host "[-] WARNING: Execution policy might block local scripts." -ForegroundColor Yellow
} else {
    Write-Host "[+] OK: Execution policy allows script run." -ForegroundColor Green
}

# 2. Check Dossier File
if (Test-Path $dossierPath) {
    Write-Host "[+] OK: ENGINEERING_DOSSIER.md detected." -ForegroundColor Green
} else {
    Write-Host "[-] ERROR: ENGINEERING_DOSSIER.md missing at $dossierPath" -ForegroundColor Red
}

# 3. Check Hook File
if (Test-Path $hookPath) {
    Write-Host "[+] OK: Hook script inject-plan.ps1 detected." -ForegroundColor Green
} else {
    Write-Host "[-] ERROR: Hook script missing at $hookPath" -ForegroundColor Red
}
