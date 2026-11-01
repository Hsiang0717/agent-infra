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

## 2. Diff Hygiene & Surgical Change Boundaries
- **Minimal Diff Footprint**: Never introduce speculative refactoring, bulk whitespace changes, or whole-file reformatting in the same commit as functional logic or bug fixes.
- **Extract-Before-Extend**: In tangled or overgrown modules, safely extract pure helpers/submodules in an isolated commit first before extending functionality.
- **Secret & Residue Hygiene**: Never commit `.env`, credentials, tokens, or temporary scratch files. Purge debugging statements before staging.

## 3. Atomic Conventional Commits & Empirical Style Alignment
- **Atomic Principle**: One commit = one logical change. Never bundle unrelated modules or mixed concerns into a single massive commit.
- **Empirical Style Detection**: Inspect existing repo history via `git log -n 5 --oneline` to match project conventions:
  - Format: `<type>[optional scope]: <imperative-subject>`
  - Use `<type>(<scope>): <msg>` if repository history uses scopes (e.g. monorepos or modular packages).
  - Use `<type>: <msg>` if repository uses flat commits or scope is not applicable.
- **Semantic Classification**:
  - `feat`: New user-facing capability or system feature
  - `fix`: Bug fix or error resolution
  - `refactor`: Structural reorganization without behavioral change
  - `perf`: Performance optimization
  - `docs`: Documentation, README, or API reference updates
  - `test`: Test suite, fixture, or mock additions/updates
  - `chore`: Tooling, build configuration, or dependency updates

## 4. Pre-Commit Verification Gate & Clean Closure
- **Tier 1 Syntax Gate**: Verify parse/AST validity before staging (`python -m py_compile`, `tsc --noEmit`, `cargo check`).
- **Native Execution**: Stage files surgically via `git add <files>` and commit natively via `git commit -m "..."`.
- **End-to-End Closure**: Verify post-commit state with `git status -s` (guaranteeing zero leftover untracked/unstaged debris) and `git log -n 3 --oneline`.

## 5. Linear History & Non-Interactive Consolidation
- **Zero Interactive Rebase**: Never execute interactive `git rebase -i` in autonomous sessions (prevents TTY editor hanging and subshell deadlocks).
- **Soft-Reset Consolidation**: To squash or re-cluster fragmented WIP commits before merging into main, use non-interactive soft reset:
  `git reset --soft origin/main`
  and re-commit atomically via Conventional Commits.
- **Linear Upstream Rebase**: Rebase topic branches non-interactively against upstream: `git fetch origin && git rebase origin/main`.
