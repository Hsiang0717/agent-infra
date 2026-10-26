---
trigger: always_on
description: "Enforce Windows pwsh environment and agent-native CLI tools (rg, fd, jq, lsj, outline)"
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
  - **4. File Editing & Windows Line-Ending Hygiene:**
    - Use `edit_file` for surgical replacements and `write_file` only for new files or full rewrites. Always copy `Target` verbatim from a single contiguous `read_file` slice. If `edit_file` reports target not found on a Windows file despite exact visible text, run `lf <file>` once to normalize `\r\n` to `\n` rather than repeating blind retries.
  - **5. VCS Status:** Use native `git status -s` (minimal 2-char status prefix).

- **Agent-Native CLI Helpers (Essential pwsh extensions):**
  - **Topology & Large Files:**
    - `lsj [path]`: Compact directory JSON (`[{"n":"file.ts","t":"f","b":1024}]`). Guaranteed Array.
    - `topj [count] [path]`: Large file radar (`[{"kb":120.5,"p":"bundle.js"}]`), auto-excludes `.git`/`node_modules`/.venv.
    - `lf <path>`: In-place CRLF (`\r\n`) to UTF-8 no-BOM LF (`\n`) normalizer for external Windows files.
  - **Runtime & Diagnostics:**
    - `pingj [url]`: Silent local HTTP server probe (`{"ok":true,"code":200}`). Prevents catastrophic HTML dumps on dev-server checks.
    - `portj <port>`: Port status and owning process name/PID in compact JSON.
    - `psj [name] [top]`: Memory-ranked process probe in compact JSON.
    - `killp <target>`: Safe and quiet termination by PID or process name.
    - `start-gui <path> [args] [workdir]`: Launch interactive GUI applications (native Win32/Qt/Slint or web/Python/Electron) into active user desktop (`WinSta0\Default`), bypassing Windows background session isolation.
