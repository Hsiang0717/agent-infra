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
  - **Code Search:** Always prefer `ripgrep` (`rg`). Use `rg --json` or `rg -n` with scoped paths. Never use Windows `find.exe`, unindexed `grep`, or PowerShell `Select-String`.
  - **File Discovery:** Always prefer `fd` for filename/path searches. Automatically ignores `.git` and `node_modules`.
  - **JSON Processing:** Use `jq` for streaming and filtering structured JSON output to minimize context consumption.
  - **Compressed Directory Inspection:** Use `lsj [path]` (PowerShell profile helper) to get compressed JSON containing file and directory entries (supports `-Force`, includes `name`, `type: dir/file`, `bytes`). Saves up to 80% tokens over traditional `dir`, `ls`, or unconstrained `Get-ChildItem`.
  - **Large File Radar (Pre-Flight Scale Awareness):** Use `topfiles [count] [path]` (PowerShell profile helper) to inspect the largest files in compressed JSON format (auto-ignores `.git`, `node_modules`, `.venv`). Never run unconstrained `Get-ChildItem -Recurse -File | Sort-Object`.
  - **Process Inspection & Management:** Use `psj [name]` for structured process inspection (PID, memory MB, window title) in compressed JSON. Use `killp <name|pid>` for quiet, safe termination. Never dump unconstrained `Get-Process`.
  - **Port / Network Inspection:** Use `portj <port>` to instantly inspect listening port status, PID, and process name in compressed JSON. Never run unconstrained `netstat` or verbose `Get-NetTCPConnection`.
  - **Installed Apps Discovery:** Use `appj [query]` (PowerShell profile helper) to search installed Windows applications and get Name/AppID in compressed JSON. Never dump unconstrained `Get-StartApps`.
  - **Structural Outline Extraction:** Use `outline <file>` (PowerShell profile helper) to extract classes, functions, and exports with line numbers before reading large modules.
  - **File Reading & Inspection (Strict Tool Hierarchy):**
    - **Always use `view_file` tool:** Use the native `view_file` tool with scoped `StartLine` and `EndLine` slices for inspecting source code.
    - **Forbidden CLI Commands:** Never execute raw dump commands via `run_command` (e.g. `cat`, `type`, `Get-Content`, `gc`, `more`). Dumping raw files to stdout wastes context tokens and lacks line-number anchors required for surgical editing.
    - **Structure First:** If you only need to explore symbols or structure, use `outline <file>` or `rg -n` instead of reading raw files.



