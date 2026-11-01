# Git Agent Flow (`git-agent-flow`)

> **AAEP 原生輕量化 Conventional Commit 與分支保護工作流外掛。**

---

## 🌟 核心理念（AAEP Core Principles）

1. **主幹保護與語義隔離（Branch Protection & Isolation）**：
   - `main` / `master` 代表生產環境地面真理（Ground Truth），嚴禁直接於主幹進行 Commit 或未隔離實驗。
   - 任何變更必須自主切換至專屬主題分支（`feat/*`、`fix/*`、`refactor/*`）。
2. **外科手術式變更邊界（Surgical Diff Hygiene）**：
   - 拒絕大量無關格式化污染，堅守「先提後擴（Extract-Before-Extend）」原則。
   - 提交前清理臨時檔案（Scratch files）與除錯語句，嚴防機密洩漏。
3. **自適應原子化 Conventional Commit（Adaptive Atomic Commits）**：
   - 一個 Commit 僅代表單一邏輯關注點。
   - 實證探測儲存庫歷史風格（`git log -n 5`），智慧自適應是否帶有範疇括號（`feat(scope):` vs `feat:`）。
4. **驗證門禁與乾淨閉環（Verification Gate & Clean Closure）**：
   - 提交前先通過語法與編譯門禁（Syntax Gate）。
   - 拋棄中介腳本與 JSON 轉義負擔，直接由 Agent 運用原生 Git CLI 執行提交，提交後保證工作區完全乾淨。

---

## ⚡ 指令與操作（Usage）

### 1. 單一原子提交
```powershell
/smart-commit "feat: implement feature X"
# 或具備模組 Scope：
/smart-commit "feat(core): implement feature X"
```

### 2. 多模組自動意圖聚類
當改動橫跨多個模組時，直接呼叫 `/smart-commit`，Agent 將依循 AAEP 四階段流程自主執行：
1. **Phase 1（Pre-Flight Radar）**：檢查分支，若在 `main` 則強制先切出獨立主題分支。
2. **Phase 2（Empirical Discovery）**：以 `git log` 探測專案風格，以 `git status -s` 審視變更並劃分意圖聚類。
3. **Phase 3（Surgical Execution）**：針對各模組獨立執行 `git add <files>` 與 `git commit -m "<type>[scope]: <msg>"`。
4. **Phase 4（Empirical Closure）**：檢查 `git status -s` 與 `git log` 確認無遺留垃圾與殘留變更。

### 3. 分支歷史整頓（History Consolidation）
當主題分支累積了零散的 WIP 或修補提交，在合回主幹前建議整頓歷史：
- **人類開發者**：可使用 `git rebase -i origin/main` 進行可視化 squash 與 reword。
- **Agent 自動化**：採用非交互式的 `git reset --soft origin/main` 將改動安全退回暫存區，再呼叫 `/smart-commit` 重新進行乾淨的原子化提交，避免彈出編輯器導致終端阻塞。

---

## 📂 外掛目錄結構

```text
git-agent-flow/
├── plugin.json                   # 外掛清單
├── rules/
│   └── AGENTS.md                 # 規範：AAEP 分支保護、Diff 衛生與 Conventional Commit 準則
└── skills/
    └── smart-commit/
        └── SKILL.md              # /smart-commit 技能定義（AAEP 四階段原生 Git SOP）
```
