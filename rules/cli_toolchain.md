---
trigger: always_on
description: "Enforce Windows pwsh environment and agent-native CLI tools (ast-grep, rg, fd, jq, lsj, outline)"
---

# CLI Toolchain & Agent-Native Environment Protocol

- **Operating System & Shell:**
  - The runtime environment is Windows with PowerShell 7 (`pwsh`).
  - Always use PowerShell syntax: `$env:VAR = "value"`, `Get-Command` (never Bash idioms like `export` or `which`).

- **Inspection & Search Hierarchy (Native Tools First):**
  - **Directory & Scale Discovery:** Always use `lsj [path]` (compact JSON with exact byte size `"b"`) or `topfiles [count] [path]` to inspect directory topology and file sizes, or `fd [pattern]` for file search (auto-ignores `.git`/`node_modules`). Never run raw shell listing commands (`Get-ChildItem`, `dir`, `ls`, `tree`), which bloat tokens and truncate columns.
  - **File Reading & Pre-Read Outline Gate:**
    - Always use native `read_file` (`view_file`) tool with explicit `StartLine` and `EndLine` slices (30-80 lines). Never run raw shell dump commands (`cat`, `type`, `Get-Content`).
    - **Mandatory Outline Gate (>300 lines / `"b" > 10000` in `lsj`):** NEVER call unbounded `read_file` (`view_file`) without `StartLine` and `EndLine` on large files. You MUST run `outline <file>` (or `rg -n <pattern> <file>`) FIRST to get exact line coordinates, then read only the targeted slice.
  - **Code Search & Structural Precision (Funnel Protocol):**
    - **Selection Rules:**
      - Plain text, error messages, logs, or non-code files (`.env`, `.json`, `.md`): Use `rg -n <pattern> <path>` (always provide an explicit `<path>` such as `src/` or `.` so `rg` never blocks waiting on non-TTY piped stdin).
      - Code syntax (function/method calls, destructuring, decorators): MUST use `ast-grep -p` (never attempt multi-line regex in `rg`).
      - Unknown monolithic files (>300 lines / `"b" > 10000` in `lsj`): NEVER inspect raw slices blindly or call unbounded `read_file` (`view_file`); MUST run `outline <file>` first to obtain the structural index before reading with `StartLine` and `EndLine`.
    - **ast-grep Quick Recipes:**
      - Method/function calls: `ast-grep -p '$OBJ.myMethod($$$ARGS)'`
      - Arbitrary destructuring: `ast-grep -p 'const { $$$PRE, targetKey, $$$POST } = $OBJ'`
      - Class / function declarations: `ast-grep -p 'function $NAME($$$) { $$$ }'`
      - Mass batch refactor across 3+ files: `ast-grep -p 'oldFn($$$A)' -r 'newFn($$$A)' -U` (deterministic zero-token execution; avoids file-by-file LLM rewrite).
    - **Tooling Constraints:**
      - Prohibited: Crafting multi-line regexes in `rg` to capture functions or statements across line breaks.
      - Prohibited: Deep file slicing on unfamiliar code (>300 lines) without a pre-flight `outline <file>`.
  - **VCS Status:** Use native `git status -s` (minimal 2-char status prefix, no wrapper needed).

- **Agent-Native CLI Helpers (Essential pwsh extensions):**
  - **Topology & Large Files:**
    - `lsj [path]`: Compact directory JSON (`[{"n":"file.ts","t":"f","b":1024}]`). Guaranteed Array.
    - `topfiles [count] [path]`: Large file radar (`[{"kb":120.5,"p":"bundle.js"}]`), auto-excludes `.git`/`node_modules`/.venv.
  - **Runtime & Diagnostics:**
    - `pingj [url]`: Silent local HTTP server probe (`{"ok":true,"code":200}`). Prevents catastrophic HTML dumps on dev-server checks.
    - `portj <port>`: Port status and owning process name/PID in compact JSON.
    - `psj [name] [top]`: Memory-ranked process probe in compact JSON.
    - `killp <target>`: Safe and quiet termination by PID or process name.
    - `start-gui <path> [args] [workdir]`: Launch interactive GUI applications (native Win32/Qt/Slint or web/Python/Electron) into active user desktop (`WinSta0\Default`), bypassing Windows background session isolation.
