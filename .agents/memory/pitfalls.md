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
