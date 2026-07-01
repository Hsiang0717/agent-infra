param (
    [Parameter(Mandatory=$true)]
    [string]$ConversationId,
    [string]$Path = ".agents/session-state.json"
)

if (-not (Test-Path $Path)) {
    Write-Error "Session state file not found at $Path"
    exit 1
}

try {
    # Read the file content
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

    # Write it back to file
    $updatedJson = $jsonContent | ConvertTo-Json -Depth 100
    Set-Content -Path $Path -Value $updatedJson -Encoding utf8

    Write-Host "Successfully saved state: Session ID updated to $ConversationId" -ForegroundColor Green
    exit 0
}
catch {
    Write-Error "Failed to update session state: $_"
    exit 1
}
