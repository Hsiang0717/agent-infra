# Verified Pitfalls & Non-Obvious Traps

This file persists verified empirical pitfalls and unexpected environment/system behaviors discovered across sessions.
It serves as an ephemeral memory anchor for autonomous agents. Keep entries ultra-compact (< 3 lines per entry).

## Format:
- **`[Tag/Domain]`**: Summary of non-obvious trap ➔ Empirical fix or invariant (`YYYY-MM-DD`).

---

## Verified Traps

- **`[pwsh-cli-pipe]`**: PowerShell 7 pipe to external tools (e.g. `rg -n <pattern>`) without an explicit path argument triggers non-TTY stdin hang. Always pass explicit path (e.g. `rg -n <pattern> .`). (2026-10-25)
- **`[git-rebase-tty]`**: Running interactive `git rebase -i` inside headless agent sessions causes deadlock waiting on `$EDITOR`. Always use non-interactive `git reset --soft origin/main` to consolidate commits. (2026-11-01)
- **`[edit-file-micro-anchor]`**: Oversized `Target` (>50 lines) fails on subtle whitespace/newline mismatches. Always micro-anchor on 2-3 unique lines around the target site. (2026-11-01)
- **`[edit-file-global-replace]`**: `edit_file` replaces ALL occurrences. Never use generic repeated lines (e.g. whitespace + `errors.append(`) without a structurally unique enclosing statement (e.g. `elif <specific_cond>:`), or every identical line across the file is corrupted. (2026-11-01)
- **`[edit-file-slice-boundary]`**: `Target` must never exceed the bounds of the latest `read_file` slice (e.g. reading 401-408 but appending unread 409-415 lines into Target), causing immediate mismatch. (2026-11-01)
- **`[edit-file-brace-closure]`**: Fixing nested brace/bracket syntax errors must anchor to PRECEDING unique lines; never span downward into subsequent function declarations or outer class boundaries, which induces bracket-counting hallucinations. (2026-11-04)
- **`[tikz-nested-braces]`**: In TikZ inline node extraction, naive regex `\{([^}]*)\}` prematurely truncates nested LaTeX macros (`\text{...}`, `\frac{..}{..}`). Always use balanced brace counting (`extractBalancedBraces`). (2026-11-05)
- **`[pydantic-ai-singleton-loop]`**: Invoking `agent.run_sync()` inside `asyncio.to_thread()` while a singleton `OpenAIProvider` (`httpx.AsyncClient`) is active on FastAPI's main event loop causes cross-event-loop `500 Internal Server Error`. Always `await agent.run()` on the main event loop (`generate_async`). (2026-11-05)
- **`[cv-shadow-probe-bleach]`**: Block-mean `illum_std > 16.0` on clean white-background geometry diagrams with gray shaded regions (`鋪色部分`) falsely triggers `BackgroundDivision` (25x25 closing kernel), bleaching interior shading to pure white except in `<25px` corners. Guard `has_severe_shadow` with outer perimeter 90th-percentile whiteness (`min(border_p90) >= 240 and std(border_p90) < 8`). (2026-11-05)
- **`[pwsh-command-predictor]`**: `CommandPrediction.PredictInputAsync` enforces a hard 20ms timeout (too tight for per-keystroke `CreateProcess`), and `Environment.CurrentDirectory` does not track PowerShell `Set-Location`. Always use a persistent UTF8-no-BOM pipe worker (`smart-hub serve`), pre-compile `SmartHub.Predictor.dll` loaded via `Assembly.Load(File.ReadAllBytes(dll))` (zero file lock), and read `$ExecutionContext.SessionState.Path.CurrentFileSystemLocation.ProviderPath`. (2026-11-06)
- **`[windows-rpa-uwp-penetration]`**: CLI/Agent processes run in isolated hidden desktop (`exebox-*`), where `SetCursorPos` fails with `ERROR_INVALID_HANDLE (6)` and `SendInput` to UWP/AppContainer is blocked by UIPI (`ERROR_ACCESS_DENIED 5`). Must switch process/thread to `WinSta0\Default` via `SetProcessWindowStation` + `SetThreadDesktop` and attach input queue with `AttachThreadInput` to target window's thread ID. (2026-11-07)
