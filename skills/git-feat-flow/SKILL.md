---
name: git-feat-flow
description: 專為 Feature Branch 設計的 AI 自適應 Git 工作流。支援透過 LLM 自動分析專案結構並生成客製化規約 (.git-feat-flow.md)，實現精準的分批提交與 Rebase 管理。
---

# Git Feature Flow (AI-Powered)

本 Skill 透過 AI 語意分析，自動適應任何專案結構，將 Agent 轉化為具備專案上下文知識的 Git 顧問。

## 核心工作流

### 1. 專案適應 (AI Project Adaptation)
當您在一個新專案中啟用此 Skill，或需要更新規約時：
1. **執行分析**：執行 `scripts/project_initializer.js` 收集專案元數據。
2. **AI 診斷**：Agent 將元數據交由 LLM 分析，產出專案專屬的 `.git-feat-flow.md`。
3. **建立規約**：此檔案包含 YAML Frontmatter（機器讀取）與 Markdown 說明（人類閱讀），定義了目錄與 `scope` 的映射關係。

### 2. 智慧型分批提交 (AI Granular Commits)
1. **讀取規約**：自動載入 `.git-feat-flow.md` 中的規則。
2. **語意分析**：若發生混合變更，Agent 會利用 LLM 閱讀 `git diff`，判斷變更的真實意圖（意圖 > 路徑）。
3. **批次建議**：生成符合專案規約的 Conventional Commits。

### 3. 安全同步 (Safe Rebase)
1. **環境預檢**：執行 `scripts/rebase_helper.js` 確保工作區安全。
2. **衝突建議**：發生衝突時，Agent 分析衝突區塊並給出語意化的合併建議。

### 4. 任務完成清理 (Post-Merge Cleanup)
當您的 Feature Branch 已在遠端（GitHub/GitLab）合併後，執行此流程：
1. **自動歸位**：切換回 `main` 分支。
2. **同步更新**：從遠端 `pull` 最新代碼。
3. **淨化環境**：執行 `fetch --prune` 移除過期的遠端追蹤分支。
4. **安全刪除**：嘗試刪除已合併的本地分支，確保工作區不留殘餘。

## 指令參考

- **"分析專案結構並設定 git-feat-flow"**：啟動 AI 適應流程，生成或更新 `.git-feat-flow.md`。
- **"幫我分批提交此次變更"**：基於規約與 AI 判斷進行智慧提交。
- **"同步變更並處理衝突"**：執行 Rebase 並開啟 AI 衝突輔助模式。
- **"任務完成，幫我清理分支"**：啟動歸位與安全刪除流程。

## 規則檔：.git-feat-flow.md
此檔案位於專案根目錄，包含：
- `mappings`: 定義特定路徑對應的 `scope` 與 `type`。
- `global_rules`: 定義副檔名或特定檔案的歸類邏輯。
- `ai_adaptation`: 是否允許 LLM 在執行時根據內容進行動態微調。
