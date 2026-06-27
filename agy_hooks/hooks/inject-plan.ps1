# Read the incoming JSON from stdin
$inputJsonText = $Input | Out-String

if ([string]::IsNullOrWhiteSpace($inputJsonText)) {
    # If stdin is empty, just exit safely
    exit 0
}

try {
    $inputJson = ConvertFrom-Json $inputJsonText -ErrorAction Stop
} catch {
    # If JSON parsing fails, output the original input to avoid breaking the execution pipeline
    Write-Output $inputJsonText
    exit 0
}

# Define the path to ENGINEERING_DOSSIER.md
# Search upwards from the current working directory for .agents\ENGINEERING_DOSSIER.md
$currentDir = Get-Location
$planPath = $null
while ($currentDir -ne $null -and $currentDir -ne "") {
    $candidate = Join-Path $currentDir ".agents\ENGINEERING_DOSSIER.md"
    if (Test-Path $candidate) {
        $planPath = $candidate
        break
    }
    $parent = Split-Path $currentDir -Parent
    if ($parent -eq $currentDir) { break }
    $currentDir = $parent
}

# Fallback to relative to hook script if not found in parent directories
if (-not $planPath) {
    $planPath = Join-Path $PSScriptRoot "..\ENGINEERING_DOSSIER.md"
}

if ($planPath -and (Test-Path $planPath)) {
    try {
        $planContent = Get-Content $planPath -Raw -ErrorAction Stop
        if ($planContent -and $planContent.Trim() -ne "") {
            # Detect plan lifecycle states
            $hasGate = $planContent -match "\[GATE_CONFIRMATION\]"
            $hasDefaultPlaceholder = $planContent -match "Final Scope:\s*\[Agreed task\]"
            $isCompleted = $planContent -match "\[POST_EXECUTION_REVIEW\]"

            # Only inject if plan is confirmed (has gate & template replaced) and not yet finished (no review)
            if ($hasGate -and -not $hasDefaultPlaceholder -and -not $isCompleted) {
                $injection = "`n`n[ACTIVE_PLAN_STATE]`n以下為目前規劃與執行狀態，請以此為準進行開發：`n$planContent"
                
                # Inject into systemInstruction if it exists, or append to the first message part
                if ($inputJson.systemInstruction) {
                    if ($inputJson.systemInstruction.parts) {
                        $inputJson.systemInstruction.parts[0].text += $injection
                    } elseif ($inputJson.systemInstruction.text) {
                        $inputJson.systemInstruction.text += $injection
                    } else {
                        $inputJson.systemInstruction += $injection
                    }
                } elseif ($inputJson.contents) {
                    # Fallback to inject into the first message content part
                    $firstContent = $inputJson.contents[0]
                    if ($firstContent.parts) {
                        $firstContent.parts[0].text = $injection + "`n`n" + $firstContent.parts[0].text
                    }
                }
            }
        }
    } catch {
        # Silent fail on read issues to not block agent invocation
    }
}

# Output the modified JSON back to stdout
$outputJsonText = ConvertTo-Json $inputJson -Depth 10 -Compress
Write-Output $outputJsonText
