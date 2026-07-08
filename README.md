# Agent Infra (AGY 基礎設施)

本專案提供基於 PowerShell 7 運行的 Agent 模組化框架與管理工具，旨在協助 Agent 在開發期達成精準的任務規劃（Minimizing Technical Debt）與指令安全管理。

## 📂 核心架構

* **[agents/](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/agents/)**: Agent 的系統提示詞（System Prompts）配置。
  * [pwsh7-config-modular.md](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/agents/pwsh7-config-modular.md): 模組化動態載入的系統提示詞。
* **[skills/](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/skills/)**: 專屬功能擴充模組。
  * [tbp-engineering-protocol](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/skills/tbp-engineering-protocol/): 工程規劃與 Gate 審核協議。
  * [powershell-helper](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/skills/powershell-helper/): PowerShell 指令映射與安全性校驗。
  * [git-mr-flow](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/skills/git-mr-flow/): 基於 MR 的 Git 工作流引導。
* **[status/](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/status/)**: 系統狀態與快取健全度檢查工具。

---

## 🚀 如何使用與運作流程

本系統基於**四階段狀態機**進行任務生命週期管理：

### 1. 狀態載入 (Startup)
Agent 啟動時會讀取並載入目前的環境配置與記憶狀態：
* 讀取狀態檔：[.agents/session-state.json](file:///C:/Users/Administrator/Desktop/skill開發/.agents/session-state.json)
* 讀取規劃日誌：[.agents/ENGINEERING_DOSSIER.md](file:///C:/Users/Administrator/Desktop/skill開發/.agents/ENGINEERING_DOSSIER.md)

### 2. 工程規劃與 Gate 審核 (TBP Protocol)
非瑣碎（Non-trivial）任務必須在 [.agents/ENGINEERING_DOSSIER.md](file:///C:/Users/Administrator/Desktop/skill開發/.agents/ENGINEERING_DOSSIER.md) 中填寫規劃，並受以下安全規則約束：
* **字數約束**：所有規劃項目每行 Bullet Point 不得超過 30 個單字（約 1-2 句短句）。
* **規劃校驗**：使用腳本 [Invoke-DossierLinter.ps1](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/skills/tbp-engineering-protocol/scripts/Invoke-DossierLinter.ps1) 自動進行格式與字數檢查。
* **安全校驗**：所有執行的 PowerShell 命令皆經由 [Test-SafeCommand.ps1](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/skills/powershell-helper/scripts/Test-SafeCommand.ps1) 過濾危險別名與 Unix 命令。

### 3. 進度與記憶存檔 (Save State)
當您在對話中說出 **「存檔」**、**「保存進度」** 等關鍵字時，Agent 會主動執行 CSD (Consented State Distillation) 協定，從當前上下文蒸餾出已完成里程碑與未解決問題，並寫入 [.agents/session-state.json](file:///C:/Users/Administrator/Desktop/skill開發/.agents/session-state.json)：
```powershell
powershell -ExecutionPolicy Bypass -File .agents/skills/tbp-engineering-protocol/scripts/Save-SessionStateWithCSD.ps1 -ConversationId <當前對話ID> -CompletedMilestonesJson '<milestones_json>' -OpenProblemsJson '<problems_json>'
```
（亦可降級為不含 CSD 蒸餾的快速存檔：使用 `Save-SessionState.ps1` 僅更新會話 ID 與時間戳）。

