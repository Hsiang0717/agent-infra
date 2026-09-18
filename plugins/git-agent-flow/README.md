# Git Agent Flow (`git-agent-flow`)

> **維護純影子 WIP 快照（`refs/wip/*`）與原子化 Conventional Commit 工作流的 Antigravity 外掛。**

---

## 🌟 核心理念（Core Concepts）

1. **HEAD 是人類認可的乾淨歷史**：
   - 官方 `git log` 僅包含語義清晰、人類批准的 Conventional Commits。
2. **`refs/wip/*` 影子快照與記憶體**：
   - 隨時以隔離的影子 Ref（`refs/wip/<branch>/current`）按需儲存工作階段快照，不干擾暫存區（Staging Area / `git add`）與 `HEAD`。
3. **原子化 Conventional Commit 聚類（`/smart-commit`）**：
   - 自動審視工作區所有變更，依據模組與意圖自動分組提交（`feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `chore`），並於提交後自動重設與歸檔影子快照。
4. **極速回滾（Fast Rollback）**：
   - 支援回滾單一檔案或整個工作區至上一個 Turn (`~1`) 或指定快照雜湊。

---

## ⚡ 指令與操作（Usage）

### 1. 手動建立影子快照
```powershell
# Slash command
/smart-commit snapshot "暫存實作進度"

# 或直接執行腳本
pwsh -NoProfile -File scripts/git_snapshot.ps1 -Message "暫存實作進度"
```

### 2. 檢視快照清單
```powershell
pwsh -NoProfile -File scripts/git_rollback.ps1 -List
```

### 3. 極速回滾
```powershell
# 回滾整個工作區至上一回合
/smart-commit rollback

# 回滾單一檔案
/smart-commit rollback ~1 path/to/file.ts
```

### 4. 語義化原子 Commit
```powershell
# 單一原子 Commit
/smart-commit "feat(core): implement feature X"

# 多模組計畫式提交
pwsh -NoProfile -File scripts/git_smart_commit.ps1 -PlanJson '[{"message":"feat(core): add feature","files":["src/core.ts"]},{"message":"docs: update guide","files":["README.md"]}]'
```

---

## 📂 外掛目錄結構

```text
git-agent-flow/
├── plugin.json                   # 外掛清單
├── hooks.json                    # 生命週期 Hook 配置
├── rules/
│   └── git-workflow.md           # Git 記憶與工作流通用準則
├── scripts/                      # PowerShell 輔助腳本
│   ├── git_snapshot.ps1          # 影子快照建立
│   ├── git_rollback.ps1          # 極速回滾核心
│   ├── git_smart_commit.ps1      # Conventional Commit 聚類與執行
│   ├── git_squash_wip.ps1        # WIP Squash 壓縮
│   └── git_on_stop.ps1           # 任務停止回呼處理
└── skills/
    └── smart-commit/
        └── SKILL.md              # /smart-commit 技能定義
```
