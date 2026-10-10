---
trigger: always_on
description: "Enforce Windows pwsh environment and agent-native CLI tools (rg, fd, jq, lsj, outline)"
---

# CLI Toolchain & Agent-Native Environment Protocol

- **Operating System & Shell:**
  - The runtime environment is Windows with PowerShell 7 (`pwsh`).
  - Always use PowerShell-compatible syntax for commands:
    - Environment variables: `$env:VAR = "value"` (never `export VAR=value`).
    - No Bash-only idioms (`VAR=val cmd`, `which`). Use `Get-Command` or `where.exe`.

- **Agent-Native CLI Tools (Preferred over raw OS commands):**
  - **Code Search:** Always prefer `ripgrep` (`rg`). Use `rg --json` or `rg -n` with scoped paths. Never use Windows `find.exe` or unindexed `grep`.
  - **File Discovery:** Always prefer `fd` for filename/path searches. Automatically ignores `.git` and `node_modules`.
  - **JSON Processing:** Use `jq` for streaming and filtering structured JSON output to minimize context consumption.
  - **Compressed Directory Inspection:** Use `lsj [path]` (PowerShell profile helper) to get compressed JSON containing file names, lengths (bytes), and timestamps, saving up to 70% tokens over traditional directory dumps.
  - **Structural Outline Extraction:** Use `outline <file>` (PowerShell profile helper) to extract classes, functions, and exports with line numbers before reading large modules.
