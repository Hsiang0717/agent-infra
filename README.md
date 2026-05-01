# Agent Infra

本專案旨在整理與開發 Gemini CLI 相關的基礎設施，包含 Agent 配置、VS Code 擴充套件、功能性 Skills 以及相關技術文件。

## 📂 專案結構

- **[agents/](./agents/)**: 存放不同環境與角色的 System Prompts 配置。
  - `pwsh5-config.md`: PowerShell 5.x 環境配置。
  - `pwsh7-config.md`: PowerShell 7.x 環境配置。
- **[extensions/](./extensions/)**: 開發中的 IDE 擴充套件。
  - `gemini-vscode/`: Gemini CLI 的 VS Code 整合插件。
- **[skills/](./skills/)**: 增強 Agent 能力的工具與規範。
  - `annotation-expert/`: 自動化程式碼註解專家。
- **[docs/](./docs/)**: 系統安裝與環境設定指南。
  - `pwsh7-setup.md`: PowerShell 7 環境安裝教學。

## 🚀 快速開始

請參考 [docs/pwsh7-setup.md](./docs/pwsh7-setup.md) 進行初始環境配置。
