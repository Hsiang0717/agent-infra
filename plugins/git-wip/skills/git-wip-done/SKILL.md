---
name: git-wip-done
description: "Finalizes Agent memory (refs/wip/*) into a single clean Conventional Commit on HEAD and archives turn snapshots."
usage: "/git-wip-done [commit_message]"
---

# /git-wip-done Command Handler

The `/git-wip-done` command serves as the **Transaction Commit Boundary** for Agent tasks.

## Workflow Execution Steps:

1. **Summarize Changes & Draft Conventional Commit**:
   - Inspect the modified files and recent turn snapshots.
   - If user provided a message (e.g. `/git-wip-done feat(backend): add sse`), use it.
   - If not provided, generate a structured Conventional Commit message (e.g. `feat(scope): ...` or `fix(scope): ...`).

2. **Execute Transaction Finalization Script**:
   ```powershell
   pwsh -NoProfile -File ./scripts/git_squash_wip.ps1 -Message "<Commit Message>"
   ```
   *Action Details*:
   - Stages all working tree modifications.
   - Creates exactly **1 formal commit on `HEAD`**.
   - Archives Agent shadow memory to `refs/wip/<branch>/archive/<timestamp>`.
   - Re-aligns `refs/wip/<branch>/current` to the new `HEAD`.

3. **Report Result**:
   - Output the resulting commit hash, commit message, and archive ref status.