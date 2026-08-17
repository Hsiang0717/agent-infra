---
name: smart-commit
description: "Groups workspace changes by intent/module and generates clean, atomic Conventional Commits while keeping Agent shadow memory in sync."
usage: "/smart-commit [commit_message]"
---

# /smart-commit Command Handler

The `/smart-commit` command analyzes current working directory modifications, clusters them by intent/architectural module, and generates one or more atomic **Conventional Commits** on `HEAD`.

## Workflow Execution Steps:

1. **Analyze Working Tree Changes**:
   - Inspect modified and untracked files using `git status --porcelain`.
   - Read relevant diffs to understand the purpose of each change.

2. **Formulate Semantic Commit Groups (Atomic Commits)**:
   - Group files based on their functional context and modification intent (e.g., `feat`, `fix`, `refactor`, `docs`, `test`, `chore`).
   - If user provided an explicit commit message:
     - If all changes belong to that intent, use it as a single commit.
     - If changes span multiple distinct areas, suggest/split into logical atomic commits.
   - For each group, compose a standard **Conventional Commit** message: `<type>(<scope>): <clear descriptive summary>`.

3. **Execute Grouped Commit Execution**:
   - For multiple commit groups, construct a JSON array and run:
     ```powershell
     pwsh -NoProfile -File ./scripts/git_smart_commit.ps1 -PlanJson '[{"message":"feat(core): ...","files":["file1","file2"]},{"message":"docs: ...","files":["README.md"]}]'
     ```
   - For a single atomic commit covering all changes:
     ```powershell
     pwsh -NoProfile -File ./scripts/git_smart_commit.ps1 -Message "<type>(<scope>): <message>"
     ```

4. **Synchronize & Report**:
   - The script automatically updates `refs/wip/<branch>/current` to align with the new `HEAD` and archives previous shadow snapshots.
   - Report the created commits (`git log -n <count> --oneline`) and confirm clean working directory status.
