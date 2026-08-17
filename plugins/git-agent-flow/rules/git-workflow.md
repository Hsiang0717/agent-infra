# Git WIP Workflow & Agent Memory Guidelines

## Core Principles
- **HEAD is human-accepted history**: `git log` must strictly contain clean, semantic, human-approved Conventional Commits.
- **`refs/wip/*` is Agent memory**: Granular snapshots are automatically recorded in background shadow refs (`refs/wip/<branch>/current`) on every turn without modifying `HEAD` or interfering with user staging (`git add`).

## Workflow Directives
1. **No Manual Intermediate Commits**: Do not spam `git commit` to `HEAD` during ongoing multi-turn tasks. The Stop Hook automatically updates the isolated Shadow WIP ref upon every turn.
2. **Atomic & Semantic Commit Boundary (`/smart-commit`)**:
   - When a milestone or task is completed, inspect changes across the workspace.
   - Cluster modified files by architectural module and intent (e.g. `feat`, `fix`, `refactor`, `docs`).
   - Execute `/smart-commit` (or provide plan JSON) to stage files in logical groups and generate clean, atomic Conventional Commits on `HEAD`.
   - The shadow memory (`refs/wip/<branch>/current`) is automatically aligned to the latest `HEAD` and snapshots archived.

---

## Agent Memory Operational Recipes (Syntax & Execution Guide)

When inspecting history, reviewing diffs, or performing rollbacks, the Agent should use the following standard Git commands:

### 1. Agent Memory Inspection
- **List recent turn snapshots**:
  ```bash
  git log --oneline -n 10 refs/wip/<branch>/current
  ```
- **List files changed in current session relative to HEAD**:
  ```bash
  git diff --stat HEAD..refs/wip/<branch>/current
  ```
- **Compare differences between consecutive turns**:
  ```bash
  git diff refs/wip/<branch>/current~1 refs/wip/<branch>/current
  ```
- **Read file content from a specific snapshot (without touching workspace)**:
  ```bash
  git show refs/wip/<branch>/current:<file_path>
  ```

### 2. Rollback & Recovery Operations
- **Revert a single file to previous turn (Turn - 1)**:
  ```bash
  git checkout refs/wip/<branch>/current~1 -- <file_path>
  ```
- **Revert a single file to a specific snapshot hash**:
  ```bash
  git checkout <snapshot_hash> -- <file_path>
  ```
- **Revert entire workspace to previous turn**:
  ```bash
  git checkout refs/wip/<branch>/current~1 -- .
  ```
- **Discard all turns and hard reset to official HEAD state**:
  ```bash
  git checkout HEAD -- .
  git clean -fd
  ```