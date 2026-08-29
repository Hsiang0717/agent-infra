---
name: smart-commit
description: "Manages Git Agent Flow: manual shadow snapshots, fast rollback, and grouping workspace changes into atomic Conventional Commits on HEAD."
usage: "/smart-commit [commit_message] | /smart-commit snapshot [msg] | /smart-commit rollback [target] [file] | /smart-commit list"
---

# /smart-commit Command Handler

The `/smart-commit` command manages shadow WIP snapshots, rollbacks, and atomic **Conventional Commits** on `HEAD`.

## Sub-commands & Modes:

### 1. Manual Shadow Snapshot (`/smart-commit snapshot [message]` or `/smart-commit snap`)
Creates an isolated shadow snapshot in `refs/wip/<branch>/current` with an optional custom message without touching `HEAD` or user staging:
```powershell
pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_snapshot.ps1" -Message "<snapshot_description>"
```

### 2. Snapshot List & Inspection (`/smart-commit list`)
Lists recent shadow snapshots:
```powershell
pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_rollback.ps1" -List
```

### 3. Fast Rollback (`/smart-commit rollback [target] [file]`)
- Rollback entire workspace to previous turn (`~1`):
  ```powershell
  pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_rollback.ps1"
  ```
- Rollback a single file to previous turn:
  ```powershell
  pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_rollback.ps1" -File "<file_path>"
  ```
- Rollback to a specific snapshot ref or commit hash:
  ```powershell
  pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_rollback.ps1" -Target "<target_ref_or_hash>" -File "<file_path>"
  ```

### 4. Semantic Atomic Commit on HEAD (`/smart-commit [commit_message]`)
Analyzes current working directory modifications, clusters them by intent/module, and generates atomic Conventional Commits:
1. **Analyze Working Tree Changes**:
   - Inspect modified and untracked files using `git status --porcelain`.
   - Formulate Conventional Commit groups (`feat`, `fix`, `refactor`, `docs`, `test`, `chore`).
2. **Execute Grouped Commit Execution**:
   - For multiple commit groups:
     ```powershell
     pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_smart_commit.ps1" -PlanJson '[{"message":"feat(core): ...","files":["file1","file2"]},{"message":"docs: ...","files":["README.md"]}]'
     ```
   - For a single atomic commit:
     ```powershell
     pwsh -NoProfile -File "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_smart_commit.ps1" -Message "<type>(<scope>): <message>"
     ```
3. **Synchronize & Report**:
   - Automatically aligns `refs/wip/<branch>/current` to the new `HEAD` and archives old snapshots.
   - Report created commits and confirm clean status.
