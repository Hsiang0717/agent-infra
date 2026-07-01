---
name: powershell-helper
description: Maps POSIX commands to native PowerShell cmdlets and validates command arguments.
---

# PowerShell Helper Skill

## Command Mappings
- Always translate Unix-style commands (`ls`, `cat`, `rm -rf`, `cp`, `mv`) to native cmdlets (`Get-ChildItem`, `Get-Content`, `Remove-Item`, `Copy-Item`, `Move-Item`).
- Ensure all execution paths use Windows backslashes `\` for local execution.
- Use backtick (`` ` ``) for line continuations instead of backslash (`\`).

## Validation
- Execute `Test-SafeCommand.ps1` to perform syntax validation before running any command in the shell.
