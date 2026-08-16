# Git WIP Workflow & Transaction Boundary Guidelines

## Core Principles
- **HEAD is human-accepted history**: `git log` must only contain clean, semantic, human-approved Conventional Commits.
- **`refs/wip/*` is Agent memory**: Granular snapshots are automatically recorded in background shadow refs (`refs/wip/<branch>/current`) on every turn without modifying `HEAD` or interfering with user staging (`git add`).

## Workflow Directives
1. **No Manual Intermediate Commits**: Do not spam `git commit` to `HEAD` during ongoing multi-turn tasks. The Stop Hook automatically updates the isolated Shadow WIP ref upon every turn.
2. **Transaction Commit Boundary (`/git-wip-done`)**:
   - When a clear milestone or task is completed, synthesize all changes into a single Conventional Commit.
   - Run `/git-wip-done -Message "<type>(<scope>): <clear message>"` or draft the commit message and ask user confirmation.
   - `/git-wip-done` creates exactly 1 formal commit on `HEAD` and archives the turn snapshots into `refs/wip/<branch>/archive/<timestamp>`.