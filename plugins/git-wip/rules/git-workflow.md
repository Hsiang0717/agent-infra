# Git WIP Workflow & Agent Memory Guidelines

## Core Principles
- **HEAD is human-accepted history**: `git log` must only contain clean, semantic, human-approved Conventional Commits.
- **`refs/wip/*` is Agent memory**: Granular snapshots are automatically recorded in background shadow refs (`refs/wip/<branch>/current`) on every turn without modifying `HEAD` or interfering with user staging (`git add`).

## Workflow Directives
1. **No Manual Intermediate Commits**: Do not spam `git commit` to `HEAD` during ongoing multi-turn tasks. The Stop Hook automatically updates the isolated Shadow WIP ref upon every turn.
2. **Transaction Commit Boundary (`/git-wip-done`)**:
   - When a clear milestone or task is completed, synthesize all changes into a single Conventional Commit.
   - Run `/git-wip-done -Message "<type>(<scope>): <clear message>"` or draft the commit message and ask user confirmation.
   - `/git-wip-done` creates exactly 1 formal commit on `HEAD` and archives the turn snapshots into `refs/wip/<branch>/archive/<timestamp>`.

---

## Agent Memory Operational Recipes (指令與語法指南)

When inspecting history, reviewing diffs, or performing rollbacks, the Agent should use the following standard Git commands:

### 1. 檢視 Agent 記憶與歷程 (Inspection)
- **查看最近的回合快照列表**：
  ```bash
  git log --oneline -n 10 refs/wip/<branch>/current
  ```
- **查看本階段所有回合相對於 HEAD 的變更清單**：
  ```bash
  git diff --stat HEAD..refs/wip/<branch>/current
  ```
- **比對特定兩個 Turn 之間的差異**：
  ```bash
  git diff refs/wip/<branch>/current~1 refs/wip/<branch>/current
  ```
- **讀取特定快照中的檔案內容（不影響工作區）**：
  ```bash
  git show refs/wip/<branch>/current:<file_path>
  ```

### 2. 回退與復原操作 (Rollback & Recovery)
- **單檔回退至上一個回合 (Turn - 1)**：
  ```bash
  git checkout refs/wip/<branch>/current~1 -- <file_path>
  ```
- **單檔回退至指定快照雜湊 (Snapshot Hash)**：
  ```bash
  git checkout <snapshot_hash> -- <file_path>
  ```
- **全工作區回退至上一個回合**：
  ```bash
  git checkout refs/wip/<branch>/current~1 -- .
  ```
- **放棄所有回合，完全退回正式 HEAD 狀態**：
  ```bash
  git checkout HEAD -- .
  git clean -fd
  ```