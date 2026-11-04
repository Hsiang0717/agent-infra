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
