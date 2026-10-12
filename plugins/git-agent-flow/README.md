# Git Agent Flow (`git-agent-flow`)

> **智慧型原子化 Conventional Commit 工作流外掛。**

---

## 🌟 核心理念（Core Concepts）

1. **HEAD 是人類認可的乾淨歷史**：
   - 官方 `git log` 僅包含語義清晰、人類批准的 Conventional Commits。
2. **原子化 Conventional Commit 聚類（`/smart-commit`）**：
   - 自動審視工作區所有變更，依據模組與意圖自動分組提交（`feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `chore`）。
   - 解決多模組變更時手動逐一暫存與提交的繁瑣操作，保證每個 Commit 的獨立性與可回溯性。

---

## ⚡ 指令與操作（Usage）

### 1. 單一原子提交
```powershell
# Slash command
/smart-commit "feat(core): implement feature X"

# 或直接呼叫腳本
pwsh -NoProfile -File scripts/git_smart_commit.ps1 -Message "feat(core): implement feature X"
```

### 2. 多模組計畫式提交（Plan JSON）
當改動橫跨多個模組時，可將檔案依邏輯劃分提交：
```powershell
pwsh -NoProfile -File scripts/git_smart_commit.ps1 -PlanJson '[
  {"message": "feat(core): add authentication core", "files": ["src/auth.ts", "src/types.ts"]},
  {"message": "docs(auth): update authentication guide", "files": ["README.md"]}
]'
```

---

## 📂 外掛目錄結構

```text
git-agent-flow/
├── plugin.json                   # 外掛清單
├── rules/
│   └── AGENTS.md                 # 規範：Conventional Commit 準則
├── scripts/
│   └── git_smart_commit.ps1      # 核心：Conventional Commit 執行與意圖聚類
└── skills/
    └── smart-commit/
        └── SKILL.md              # /smart-commit 技能定義
```
