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
  * [AGENTS.md](agents/AGENTS.md): 跨階段通用 AI Coding Agent 執行準則。
  * [GEMINI.md](agents/GEMINI.md): 嚴格關鍵工具守門協議（P0 Critical Tool Gates: Zero-Write on Ambiguity, Strict Scope Boundary, Two-Failure Reset）與執行標準。
  * [Karpathy.md](agents/Karpathy.md): 通用 LLM 編碼行為準則（Claude / Karpathy 風格）。
* **[skills/](skills/)**: 功能擴充模組（Antigravity Skills）。
  * [pi-agent-delegate](skills/pi-agent-delegate/): 將任務委派給本機 `pi` CLI Agent。
  * [skill-creator](skills/skill-creator/): Skill 建立、評測與效能最佳化工具。
* **[plugins/](plugins/)**: 外掛整合套件（Antigravity Plugins）。
  * [agy-observational-memory](plugins/agy-observational-memory/): 極簡 Skill-First 雙層記憶架構（Observations + Reflections），提供確定性 12 碼追溯、零延遲紀錄與對話內反思。
  * [git-agent-flow](plugins/git-agent-flow/): Git Agent Flow，包含影子 WIP 快照、極速回滾與 `/smart-commit` 語義化原子 Conventional Commit 管理。
* **[extensions/](extensions/)**: 編輯器擴充套件。
  * [agy-vscode](extensions/agy-vscode/): VS Code 擴充套件，提供一鍵啟動 AGY CLI 的快捷圖示與環境隔離。
* **[status/](status/)**: 狀態列遙測儀表板。
  * [statusline.ps1](status/statusline.ps1): PowerShell 自訂狀態列腳本（固定 3 行自適應佈局、按需延遲載入、.NET 反射加速、Morandi 莫蘭迪低飽和配色）。
  * [install.ps1](status/install.ps1): 一鍵安裝與更新狀態列套件至 `~/.antigravity/`。
  * [install.md](status/install.md): 狀態列安裝、設定與參數說明指南。
