---
trigger: always_on
description: "Enforce Windows pwsh environment, agent-native CLI tools (rg, fd, jq, lsj, outline), and autonomous helper bootstrap protocol"
---

# CLI Toolchain & Agent-Native Environment Protocol

- **Operating System & Shell:**
  - The runtime environment is Windows with PowerShell 7 (`pwsh`).
  - Always use PowerShell syntax: `$env:VAR = "value"`, `Get-Command` (never Bash idioms like `export` or `which`).

- **Shell Command Boundaries (Native Tools First):**
  - **Directory & File Discovery:** Use `lsj [path]`, `topj [count] [path]`, or `fd [pattern]`. Never run raw shell listing commands (`Get-ChildItem`, `dir`, `ls`, `tree`).
  - **File Reading & Code Search:** Use native `read_file` (`outline <file>` first if >300 lines) and `rg -n <pattern> <path>` (always pass an explicit `<path>` like `src/` or `.` so `rg` never blocks on non-TTY stdin). Never run raw shell dump commands (`cat`, `type`, `Get-Content`, `Select-String`).
  - **VCS Status:** Use `git status -s` (minimal 2-char status prefix).

- **Agent-Native CLI Helpers (Essential pwsh extensions):**
  - **Topology & Large Files:**
    - `lsj [path]`: Compact directory JSON (`[{"n":"file.ts","t":"f","b":1024}]`). Guaranteed Array.
    - `topj [count] [path]`: Large file radar (`[{"kb":120.5,"p":"bundle.js"}]`), auto-excludes `.git`/`node_modules`/.venv.
  - **Runtime & Diagnostics:**
    - `pingj [url]`: Silent local HTTP server probe (`{"ok":true,"code":200}`). Prevents catastrophic HTML dumps on dev-server checks.
    - `portj <port>`: Port status and owning process name/PID in compact JSON.
    - `psj [name] [top]`: Memory-ranked process probe in compact JSON.
    - `killp <target>`: Safe and quiet termination by PID or process name.
    - `start-gui <path> [args] [workdir]`: Launch interactive GUI applications (native Win32/Qt/Slint or web/Python/Electron) into active user desktop (`WinSta0\Default`), bypassing Windows background session isolation.

- **Autonomous Helper Bootstrap Protocol (Active Self-Healing Installation):**
  - When calling an agent-native helper (`lsj`, `topj`, `outline`, `pingj`, `portj`, `killp`) results in `CommandNotFoundException` or indicates the helper is missing in the current session:
    - **NEVER degrade to noisy, token-bloating shell commands** (`Get-ChildItem`, `dir`, `tree`).
    - **Immediately execute autonomous self-installation via `run_command`**:
      ```powershell
      pwsh -NoProfile -ExecutionPolicy Bypass -File .\profile\install-module.ps1 -Copy
      ```
      or import the module into the active user session:
      ```powershell
      Import-Module AgentHelpers
      ```
    - Once bootstrapped, immediately resume execution using the native helpers.
