---
name: agent-infra
description: Antigravity (AGY) Agent 模組化基礎設施，包含系統提示詞、功能擴充 Skill、VS Code 擴充套件與狀態列工具。
version: 0.1.0
author: Hsiang0717
license: MIT
---

# Agent Infra (AGY 基礎設施)

本專案提供 Antigravity (AGY) Agent 的模組化基礎設施，包含系統提示詞、功能擴充 Skill、VS Code 擴充套件與狀態列工具。

## 📂 專案架構

* **[agents/](agents/)**: Agent 系統提示詞（System Prompts）與自主工程生命週期準則。
  * [GEMINI.md](agents/GEMINI.md): **自主工程代理協議 (AAEP - Autonomous Agent Engineering Protocol)**。基於「自主掌舵、崇尚實證、極致克制」四大生命週期階段（事實查證、審慎設計與先抽後建、低開銷外科手術修改、真機與多模態實證驗證）。
  * [AGENTS.md](agents/AGENTS.md): 跨階段通用 AI Coding Agent 執行準則。
  * [Karpathy.md](agents/Karpathy.md): 通用 LLM 編碼行為準則（Claude / Karpathy 風格）。
* **[rules/](rules/)**: 全域與環境硬規則（`trigger: always_on`）。
  * [cli_toolchain.md](rules/cli_toolchain.md): Windows pwsh 執行規範與 Agent-Native CLI 工具鏈（`rg`, `fd`, `jq`, `lsj`, `outline`）。
  * [python_uv.md](rules/python_uv.md): 系統 Python 與 `uv` 工具鏈強制規範。
* **[docs/](docs/)**: 設計哲學與架構演進文檔。
  * [GEMINI_DESIGN_RATIONALE.md](docs/GEMINI_DESIGN_RATIONALE.md): **GEMINI.md 設計哲學與架構演進全紀錄**。詳細記錄為何拋棄「防守拘束器」、Context 經濟學、多模態閉環與各極端情境決策邊界，供後續迭代延續理念。
* **[skills/](skills/)**: 功能擴充模組（Antigravity Skills）。
  * [pi-agent-delegate](skills/pi-agent-delegate/): 將任務委派給本機 `pi` CLI Agent。
  * [skill-creator](skills/skill-creator/): Skill 建立、評測與效能最佳化工具。
* **[plugins/](plugins/)**: 外掛整合套件（Antigravity Plugins）。
  * [agy-context-flow](plugins/agy-context-flow/): 自主情境流動與檔案推薦引擎（Context Flow Engine），支援 5 維因果行為追蹤、雙先驗貝氏融合（Bayesian Prior Fusion）、PPMI-SVD 向量語義檢索與跨 Session 動態 Handover。
  * [git-agent-flow](plugins/git-agent-flow/): Git Agent Flow，包含影子 WIP 快照、極速回滾與 `/smart-commit` 語義化原子 Conventional Commit 管理。
* **[extensions/](extensions/)**: 編輯器擴充套件。
  * [agy-vscode](extensions/agy-vscode/): VS Code 擴充套件，提供一鍵啟動 AGY CLI 的快捷圖示與環境隔離。
* **[status/](status/)**: 狀態列遙測儀表板。
  * [statusline.ps1](status/statusline.ps1): PowerShell 自訂狀態列腳本（固定 3 行自適應佈局、按需延遲載入、.NET 反射加速、Morandi 莫蘭迪低飽和配色）。
  * [install.ps1](status/install.ps1): 一鍵安裝與更新狀態列套件至 `~/.antigravity/`。
  * [install.md](status/install.md): 狀態列安裝、設定與參數說明指南。
