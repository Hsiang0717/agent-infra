param (
    [Parameter(Mandatory=$true)]
    [string]$ConversationId,
    [Parameter(Mandatory=$true)]
    [string]$CompletedMilestonesJson,
    [Parameter(Mandatory=$true)]
    [string]$OpenProblemsJson,
    [string]$Path = ".agents/session-state.json"
)

# 1. Validate Target Path
if (-not (Test-Path $Path)) {
    Write-Error "Session state file not found at $Path"
    exit 1
}

try {
    # Parse target JSON arrays from parameters
    $completedMilestones = $CompletedMilestonesJson | ConvertFrom-Json
    $openProblems = $OpenProblemsJson | ConvertFrom-Json

    # Read the existing file content
    $jsonContent = Get-Content -Path $Path -Raw | ConvertFrom-Json

    # Update metadata
    if (-not $jsonContent.session_metadata) {
        $jsonContent | Add-Member -MemberType NoteProperty -Name "session_metadata" -Value ([PSCustomObject]@{
            "session_id" = $ConversationId
            "timestamp_last_update" = (Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz")
        })
    } else {
        $jsonContent.session_metadata.session_id = $ConversationId
        $jsonContent.session_metadata.timestamp_last_update = (Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz")
    }

    # Update distilled context
    if (-not $jsonContent.distilled_context) {
        $jsonContent | Add-Member -MemberType NoteProperty -Name "distilled_context" -Value ([PSCustomObject]@{
            "completed_milestones" = $completedMilestones
            "open_problems" = $openProblems
        })
    } else {
        $jsonContent.distilled_context.completed_milestones = $completedMilestones
        $jsonContent.distilled_context.open_problems = $openProblems
    }

    # Write it back to file with proper formatting
    $updatedJson = $jsonContent | ConvertTo-Json -Depth 100
    Set-Content -Path $Path -Value $updatedJson -Encoding utf8

    Write-Host "Successfully saved CSD state: Session ID updated to $ConversationId, milestones and open problems refreshed." -ForegroundColor Green
    exit 0
}
catch {
    Write-Error "Failed to update session state with CSD: $_"
    exit 1
}
