---
name: git-feat-flow
description: 專為 Feature Branch 設計的 AI 自適應 Git 工作流。支援透過 LLM 自動分析專案結構並生成客製化規約 (.git-feat-flow.md)，實現精準的分批提交與 Rebase 管理。
---

# Git Feature Flow (AI-Powered)

本 Skill 透過 AI 語意分析，自動適應任何專案結構，將 Agent 轉化為具備專案上下文知識的 Git 顧問。

## 核心工作流

### 1. 專案適應 (AI Project Adaptation)
1. **Pilot (AI) 指令**：呼叫 `scripts/project_initializer.js`。
2. **Executor (Script) 執行**：掃描目錄結構、關鍵檔案與 **Remote 狀態**，輸出 JSON 格式的專案元數據。
3. **Pilot (AI) 推理**：讀取 JSON，判斷專案類型、合適的 Scope 分類與 **Workflow Mode (local/collaborative)**，生成 `.git-feat-flow.md`。

### 2. 分支預檢與智慧分批提交 (Branching Guard & Granular Commits)
1. **Pilot (AI) 預檢**：呼叫 `git branch --show-current`，若在 `main`/`master` 則執行自動導流。
2. **Executor (Script) 分類**：執行 `scripts/git_analyzer.js`，根據規約將變更檔案自動分組。
3. **Pilot (AI) 語意分析**：閱讀 `git diff`，結合分組結果，為每一組生成精準的 Conventional Commits。
4. **Gatekeeper (AI) 守則**：若 `workflow_mode` 為 `collaborative`，在 `git push` 後 **必須停止所有動作**，告知使用者發起 MR/PR，禁止私自在本地合併至受保護分支。

### 3. 安全同步 (Safe Rebase)
1. **Executor (Script) 檢查**：執行 `scripts/rebase_helper.js` 確保工作區乾淨且不在主分支。
2. **Pilot (AI) 同步**：執行 `git rebase`，若發生衝突，由 AI 分析衝突區塊並給出合併建議。

### 4. 任務完成清理 (Post-Merge Cleanup)
1. **Pilot (AI) 驗證**：在執行清理前，必須確認遠端分支已成功合併（經由使用者告知或 API 檢查）。
2. **Executor (Script) 執行**：直接呼叫 `scripts/cleanup_helper.js` 執行標準的「歸位、同步、淨化、刪除」流程。


## 指令參考

- **"分析專案結構並設定 git-feat-flow"**：啟動 AI 適應流程，自動偵測環境並設定 `workflow_mode`。
- **"幫我分批提交此次變更"**：進行智慧提交。若在協作模式，完成 Push 後會自動進入等待狀態。
- **"同步變更並處理衝突"**：執行 Rebase 並開啟 AI 衝突輔助模式。
- **"任務完成，幫我清理分支"**：僅在確認合併後執行，確保本地與遠端同步。

## 規則檔：.git-feat-flow.md
此檔案位於專案根目錄，包含：
- `workflow_mode`: `local` (個人專案) 或 `collaborative` (團隊協作，需 MR)。
- `mappings`: 定義特定路徑對應的 `scope` 與 `type`。
- `global_rules`: 定義副檔名或特定檔案的歸類邏輯。
- `ai_adaptation`: 是否允許 LLM 在執行時根據內容進行動態微調。
