---
name: smart-commit
description: "Inspects workspace modifications under AAEP guidelines: enforces main branch protection, detects repo commit style, clusters changes by intent/module, and creates atomic Conventional Commits using native Git CLI."
usage: "/smart-commit [commit_message]"
---

# /smart-commit Command Handler (AAEP Edition)

The `/smart-commit` command guides the agent through an empirical, surgical, and atomic commit sequence under the Autonomous Agent Engineering Protocol (AAEP).

---

## Execution Protocol

### Phase 1: Pre-Flight Radar & Branch Protection (Guardrail)
Inspect the active branch before modifying the index:
```powershell
git branch --show-current
```
- **If on `main` or `master`**: **Zero direct commits on main**. Immediately branch off:
  ```powershell
  git checkout -b <type>/<short-description>
  # Examples: feat/user-auth, fix/null-pointer, refactor/git-agent-flow
  ```
- **If already on a dedicated branch**: Proceed.

---

### Phase 2: Empirical Discovery & Intent Clustering
1. **Style Grounding**: Inspect the repository's existing style:
   ```powershell
   git log -n 5 --oneline
   ```
   - Match existing convention: scoped `<type>(<scope>): <msg>` vs. flat `<type>: <msg>`.
2. **Working Tree Radar**: Scan modified and untracked artifacts:
   ```powershell
   git status -s
   ```
3. **Intent Partitioning**: Cluster files logically by architectural boundaries:
   - `feat[scope]`: User-facing capability
   - `fix[scope]`: Bug or error fix
   - `refactor[scope]`: Code reorganization without behavioral change
   - `perf[scope]`: Performance enhancement
   - `docs[scope]`: Documentation or comments
   - `test[scope]`: Tests or test suites
   - `chore[scope]`: Tooling, build, or dependency updates
4. **History Consolidation (Optional Pre-Step)**:
   - If the active branch accumulated messy WIP or fragmented commits and needs clean squashing before merging to main, run a non-interactive soft reset:
     ```powershell
     git reset --soft origin/main
     ```
     This safely collapses all branch modifications back into staged changes for clean Conventional Commit re-clustering without TTY editor deadlocks.

---

### Phase 3: Surgical Atomic Execution (Native Git)
- **Pre-Commit Verification**: Run fast syntax/compile checks (e.g. `py_compile`, `tsc --noEmit`) before staging.
- **Surgical Staging**:

#### Scenario A: Multi-Concern or Multi-Module Changes (Atomic Clustering)
Stage and commit each logical group independently:
```powershell
# Group 1: Core functionality
git add <path/to/module1> <path/to/module2>
git commit -m "feat[scope]: <imperative description of group 1>"

# Group 2: Unit tests
git add <path/to/tests>
git commit -m "test[scope]: add tests for <feature>"

# Group 3: Documentation
git add README.md docs/
git commit -m "docs[scope]: update documentation"
```

#### Scenario B: Single-Concern Modification
When all changes belong to a single logical scope:
```powershell
git add -A
git commit -m "<type>[optional scope]: <imperative message>"
```

---

### Phase 4: Empirical Verification & Clean Closure
Confirm clean workspace closure:
```powershell
git status -s
git log -n 3 --oneline
```
- Verify zero residual scratch files or unintended modified files remain.
- Verify commit message follows Conventional Commit format and imperative tone.
