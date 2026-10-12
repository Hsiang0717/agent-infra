---
name: smart-commit
description: "Inspects workspace modifications, groups changes by intent/module, and generates atomic Conventional Commits on HEAD."
usage: "/smart-commit [commit_message] | /smart-commit -PlanJson '<json>'"
---

# /smart-commit Command Handler

The `/smart-commit` command analyzes working directory changes, clusters them by architectural module and intent, and generates clean, atomic **Conventional Commits** directly on `HEAD`.

## Workflow:

### 1. Analyze Working Tree Changes
- Inspect modified and untracked files using `git status -s`.
- Identify distinct logical boundaries across components (e.g. `feat`, `fix`, `refactor`, `docs`, `test`, `chore`).

### 2. Execute Atomic Commit(s)

#### Option A: Multi-group Commit via Plan JSON
When changes span multiple scopes or concerns, group files into a JSON array:
```powershell
$scriptPath = if (Test-Path "plugins/git-agent-flow/scripts/git_smart_commit.ps1") { "plugins/git-agent-flow/scripts/git_smart_commit.ps1" } else { "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_smart_commit.ps1" }
pwsh -NoProfile -File $scriptPath -PlanJson '[{"message":"feat(core): implement feature X","files":["src/core.ts"]},{"message":"docs: update documentation","files":["README.md"]}]'
```

#### Option B: Single Atomic Commit
When all current modifications belong to a single logical change:
```powershell
$scriptPath = if (Test-Path "plugins/git-agent-flow/scripts/git_smart_commit.ps1") { "plugins/git-agent-flow/scripts/git_smart_commit.ps1" } else { "$HOME/.gemini/config/plugins/git-agent-flow/scripts/git_smart_commit.ps1" }
pwsh -NoProfile -File $scriptPath -Message "<type>(<scope>): <message>"
```

### 3. Verify
- Verify resulting `git log --oneline -n <count>` and ensure working directory is in expected state via `git status -s`.
