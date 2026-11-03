---
trigger: always_on
description: "Enforce Windows pwsh environment, agent-native CLI tools (rg, fd, jq, lsj, outline), and autonomous helper bootstrap protocol"
---

# CLI Toolchain & Agent-Native Environment Protocol

- **Operating System & Shell:**
  - The runtime environment is Windows with PowerShell 7 (`pwsh`).
  - Always use PowerShell syntax: `$env:VAR = "value"`, `Get-Command` (never Bash idioms like `export` or `which`).

- **Inspection, Search & Editing Hierarchy (Native Tools First):**
  - **1. Directory & Scale Discovery:** Always use `lsj [path]` (compact JSON with exact byte size `"b"`) or `topj [count] [path]` to inspect topology and file sizes, or `fd [pattern]` for file search (auto-ignores `.git`/`node_modules`). Never run raw shell listing commands (`Get-ChildItem`, `dir`, `ls`, `tree`), which bloat tokens and truncate columns.
  - **2. Pre-Read Outline Gate & Scoped Reading:**
    - Use native `read_file` with explicit `StartLine` and `EndLine` slices (30-80 lines). Never run raw shell dump or context-slicing commands (`cat`, `type`, `Get-Content`, `Select-String`).
    - For modules >300 lines (`"b" > 10000` in `lsj`), never call unbounded `read_file` on the whole file: run `outline <file>` (or `rg -n <pattern> <file>`) first to pinpoint exact line coordinates, then read only the targeted slice.
  - **3. Code Search & Structural Precision (Funnel Protocol):**
    - Use `rg -n <pattern> <path>` for fast, targeted code and text searches (always pass an explicit `<path>` such as `src/` or `.` so `rg` never blocks waiting on non-TTY piped stdin). Suppress noise by narrowing query terms or scoping to specific subdirectories.
  - **4. Surgical File Editing Protocol (Minimal Unique Target):**
    - Use `edit_file` for surgical replacements and `write_file` only for new files or full rewrites.
    - **Micro-Anchoring:** Copy `Target` verbatim from a fresh read slice strictly within read boundaries. Keep `Target` scoped to the minimal unique context needed (enclosing signature/condition). Because `edit_file` replaces all occurrences, never target generic repeated statements or bare whitespace alone. Avoid bloated multi-statement targets to eliminate whitespace/drift mismatch risks.
    - **Target Invariant Compliance:** When the system hook injects a `[Target Invariant: ...]` notice (e.g. non-standard 3-space indentation, tabs, trailing whitespace, or double backslashes), match the cited invariant verbatim. Never assume conventional 4-space markdown/PEP8 indentation. Note that in JSON tool calls, source code backslashes must be double-escaped (e.g. source `\\node` must be serialized as `\\\\node` in JSON).
  - **5. VCS Status:** Use native `git status -s` (minimal 2-char status prefix).

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
