# Git Engineering & Conventional Commit Protocol (AAEP Edition)

This protocol governs version control operations, branch lifecycles, and commit hygiene under the Autonomous Agent Engineering Protocol (AAEP).

## 1. Branch Protection & Semantic Isolation (Strict Guardrail)
- **Zero Direct Commits on Main**: `main` and `master` represent production ground truth. Direct commits or unstaged experiments on main/master are strictly forbidden.
- **Autonomous Branch-Off**: Before modifying code or staging commits, empirically check current branch:
  ```powershell
  git branch --show-current
  ```
  If on `main` or `master`, immediately branch off to a dedicated semantic topic branch:
  `git checkout -b <type>/<short-description>`
  - Types: `feat/*`, `fix/*`, `refactor/*`, `perf/*`, `docs/*`, `chore/*`.
- **Short-Lived Topic Branches**: Scope branches strictly to the active task boundary; rebase against upstream (`git fetch && git rebase origin/main`) to maintain clean linear history.

## 2. Diff Hygiene & Atomic Conventional Commits
- **Minimal Diff & Secret Hygiene**: Never bundle unrelated modules, speculative refactoring, bulk formatting, `.env`/credentials, or temporary scratch files into commits.
- **Empirical Style Alignment**: Inspect `git log -n 5 --oneline` to match repository convention (`<type>(<scope>): <imperative-subject>` vs `<type>: <imperative-subject>`), using standard types (`feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`).
- **Pre-Commit Verification & Clean Closure**: Verify syntax/AST validity before staging (`git add <files>` ➔ `git commit -m "..."`), then confirm clean working tree via `git status -s`.

## 3. Linear History & Non-Interactive Consolidation
- **Zero Interactive Rebase**: Never execute interactive `git rebase -i` or `git commit` without `-m` in autonomous sessions (prevents TTY editor hanging).
- **Soft-Reset Consolidation**: To squash fragmented WIP commits before merging, run `git reset --soft origin/main` and re-commit atomically.
