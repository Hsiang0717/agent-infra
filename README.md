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

* **[agents/](agents/)**: Agent 系統提示詞（System Prompts）與行為準則。
  * [GEMINI.md](agents/GEMINI.md): Gemini 模型適用的行為準則。
  * [Karpathy.md](agents/Karpathy.md): 通用 LLM 編碼行為準則（Claude 風格）。
* **[skills/](skills/)**: 功能擴充模組（Antigravity Skills）。
  * [git-mr-flow](skills/git-mr-flow/): 基於 Merge Request 的 Git 工作流引導。
  * [pi-agent-delegate](skills/pi-agent-delegate/): 將任務委派給本機 `pi` CLI Agent。
  * [skill-creator](skills/skill-creator/): Skill 建立、評測與效能最佳化工具。
* **[extensions/](extensions/)**: 編輯器擴充套件。
  * [agy-vscode](extensions/agy-vscode/): VS Code 擴充套件，提供一鍵啟動 AGY CLI 的快捷圖示。
* **[status/](status/)**: 狀態列遙測儀表板。
  * [statusline.ps1](status/statusline.ps1): PowerShell 自訂狀態列腳本。
  * [install.md](status/install.md): 安裝與設定指南。
