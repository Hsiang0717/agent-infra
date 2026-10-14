---
name: agent-infra
description: Antigravity (AGY) Agent 模組化基礎設施，包含自主工程代理協議、系統提示詞、全域規則、外掛套件、Agent-Native CLI 工具鏈與 Rust 原生狀態列。
version: 0.2.0
author: Hsiang0717
license: MIT
---

# Agent Infra (AGY 基礎設施)

本專案提供 Antigravity (AGY) Agent 的模組化基礎設施，包含自主工程代理協議（AAEP）、系統提示詞、全域環境硬規則、功能外掛套件（Plugins）、Agent-Native CLI 工具鏈與原生高效能狀態列。

## 📂 專案架構

* **[agents/](agents/)**: Agent 系統提示詞（System Prompts）與自主工程生命週期準則。
  * [GEMINI.md](agents/GEMINI.md): **自主工程代理協議 (AAEP - Autonomous Agent Engineering Protocol)**。基於「自主掌舵、崇尚實證、極致克制」四大生命週期階段（事實查證與層級探索、審慎設計與先抽後建、低開銷外科手術修改、真機與多模態實證驗證），並深度整合 Context 語意動態姿態（NOMINAL / TIGHT / CRITICAL）。
  * [Karpathy.md](agents/Karpathy.md): 通用 LLM 編碼行為準則（Claude / Karpathy 風格）。
* **[rules/](rules/)**: 全域與環境硬規則（`trigger: always_on`）。
  * [cli_toolchain.md](rules/cli_toolchain.md): Windows pwsh 執行規範、Agent-Native CLI 工具鏈（`rg`, `fd`, `ast-grep`, `jq`, `outline`）與高密度壓縮輔助指令（`lsj`, `topfiles`, `pingj`, `portj`, `psj`, `killp`）。
  * [python_uv.md](rules/python_uv.md): 系統 Python 與 `uv` 工具鏈強制規範。
* **[docs/](docs/)**: 設計哲學與架構演進文檔。
  * [GEMINI_DESIGN_RATIONALE.md](docs/GEMINI_DESIGN_RATIONALE.md): **GEMINI.md 設計哲學與架構演進全紀錄**。詳細記錄為何拋棄「防守拘束器」、Context 經濟學、多模態閉環與各極端情境決策邊界，供後續迭代延續理念。
* **[plugins/](plugins/)**: 外掛整合套件（Antigravity Plugins）。
  * [aaep](plugins/aaep/): 核心自主工程代理協議提醒與自我校準 Skill（`/aaep`），無額外環境與二進位相依，提供純粹心智重錨、四大生命週期檢核與防漂移自省清單。
  * [agy-context-monitor](plugins/agy-context-monitor/): Rust 原生開發之極速 Context 監控 Hook（`PreInvocation`），於推論前反向探測 Transcript Token 吞吐（< 1ms），並依據 256k 額度注入三級語意姿態提示（NOMINAL / TIGHT / CRITICAL），實現零數字干擾的動態收斂引導。
  * [agy-handoff](plugins/agy-handoff/): Rust 原生開發的高密度 Session 軌跡交接外掛，自動打包當前上下文並寫入系統剪貼簿。
  * [git-agent-flow](plugins/git-agent-flow/): 智慧型原子化 Conventional Commit 工作流外掛，提供 `/smart-commit` 意圖聚類、多模組分組提交與標準版控規範。
  * [screen-capture](plugins/screen-capture/): Rust + Windows Graphics Capture (WGC) 原生高解析螢幕與視窗擷取工具，專供 Agent 進行多模態 UI 視覺驗證閉環。
* **[profile/](profile/)**: Agent-Native CLI 工具鏈與 PowerShell Profile 擴充。
  * [README.md](profile/README.md): 外部相依清單（winget）、原生 CLI 分工矩陣與 Helper 說明。
  * 提供 `AgentHelpers.ps1` 與模組化包裝（`.psd1` / `.psm1`），支援宣告式 Dot-source 引用與標準 PowerShell 模組 Auto-loading 雙安裝架構。
* **[pwsh/](pwsh/)**: PowerShell 互動終端鍵盤操作與 PSReadLine 調校。
  * [README.md](pwsh/README.md): 上下文感知右方向鍵（單字/整行補全）、智慧歷史前綴搜尋、行內快速補全說明。
  * 提供 `KeyBindings.ps1` 與模組封裝（`.psd1` / `.psm1`），支援本機 Dot-source 與跨環境模組部署。
* **[status/](status/)**: 狀態列遙測儀表板。
  * [statusline-rs](status/statusline-rs/): Rust 原生高效能低延遲狀態列（冷啟動 < 20ms），具備固定 3 行自適應防跳動儀表板（Zero-Jitter Dashboard）、按需延遲載入（Lazy Loading）與 Morandi 莫蘭迪低飽和配色。
  * [install.ps1](status/install.ps1): 一鍵編譯與部署 `statusline.exe` 至 `~/.antigravity/`。
  * [install.md](status/install.md): 狀態列安裝、設定與參數說明指南。
