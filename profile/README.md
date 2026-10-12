# 🤖 Agent-Native CLI Toolchain & PowerShell Profile

這套工具鏈是專為 AI Agent（如 Claude, Gemini, Antigravity 等）設計的**高密度、低 Token、確定性 Schema** CLI 環境擴充。

---

## 1. 外部相依工具清單 (Winget Packages)

在分享或在全新 Windows 環境中部署時，請確保已安裝以下核心 CLI 工具：

| 工具 | CLI 指令 | Winget Package ID | 核心定位與價值 |
| :--- | :--- | :--- | :--- |
| **Ripgrep** | `rg` | `BurntSushi.ripgrep.MSVC` | 純文字/關鍵字高速檢索（標準 `file:line:text` 輸出） |
| **fd** | `fd` | `sharkdp.fd` | 檔案路徑高速定位（自動忽略 `.git`/`node_modules`） |
| **ast-grep** | `ast-grep` | `ast-grep.ast-grep` | 跨語言語法樹結構搜尋與符號輪廓抽取 |
| **jq** | `jq` | `jqlang.jq` | 結構化 JSON 串流過濾 |
| **uv** | `uv` | `astral-sh.uv` | 極速 Python 執行器與依賴管理 |
| **Git** | `git` | `Git.Git` | 版本控制第一公民 |

### 🚀 一鍵安裝指令 (複製即可在 PowerShell 執行)：

```powershell
winget install --id BurntSushi.ripgrep.MSVC sharkdp.fd ast-grep.ast-grep jqlang.jq astral-sh.uv --accept-source-agreements --accept-package-agreements
```

---

## 2. 專案一鍵配置 (Setup)

你可以直接使用專案內的安裝腳本自動檢查並配置環境：

```powershell
# 自動檢測並補齊缺少的 winget 工具，並注入至 PowerShell Profile
.\profile\install.ps1 -InstallTools

# 僅同步 Helper 函數至系統 PowerShell Profile ($PROFILE)
.\profile\install.ps1
```

---

## 3. 指令集與分工矩陣 (No Redundancy)

### A. 原生 CLI 工具 (零包裝、Token 最佳化)
* `fd [pattern]`：檔案路徑定位。
* `rg -n [pattern] [path]`：程式碼檢索（比轉成 JSON 節省 40% Token）。
* `ast-grep -p '<pattern>'`：跨行 AST 語意語法比對。
* `git status -s`：極簡版控狀態清單（比 JSON 化更精簡且零錯誤）。

### B. Agent-Native 專屬 Helper (防爆、複合、降級)
* `outline <file>`：提取結構大綱。**AST-first** (`ast-grep outline --view=expanded`)，非支援語言自動降級為縮排增強 **Regex** (`rg -n`)。
* `lsj [path]`：目錄壓縮清單，回傳標準 JSON Array (`[{"n":"file.ts","t":"f","b":1024}]`)。
* `topfiles [count] [path]`：大檔案雷達，事前規模感知防範讀檔 Token 爆炸。
* `pingj [url]`：本機 HTTP 伺服器探針，回傳 `{"ok":true,"code":200}`，防範 HTML 頁面噴發。
* `portj <port>`：複合查詢 Port 狀態、PID 與對應 Process 名稱。
* `psj [name] [top]`：依記憶體佔用排序之 Process 壓縮清單。
* `killp <target>`：靜默強制刪除程序（支援 PID 或名稱）。

---

## 4. 載入架構與發布演進 (Architecture & Roadmap)

目前採用 **方案 A (Dot-source)**，並保留 **方案 B (PowerShell Module)** 之擴充架構供日後發布參考：

### 方案 A：宣告式 Dot-source 引用（目前採用）
* **原理**：`install.ps1` 僅在 `$PROFILE` 寫入路徑檢查與引用：
  ```powershell
  # >>> AGENT-NATIVE HELPERS (START) >>>
  if (Test-Path "C:\path\to\profile\AgentHelpers.ps1") {
      . "C:\path\to\profile\AgentHelpers.ps1"
  }
  # <<< AGENT-NATIVE HELPERS (END) <<<
  ```
* **優點**：
  - **Single Source of Truth**：直接以 Git 倉儲為唯一真實來源，修改任何 helper 函式重開 terminal 即生效，不需重複安裝。
  - **保持 Profile 純淨**：`$PROFILE` 內無冗長邏輯，解耦性最高。

### 方案 B：正式 PowerShell 模組化（未來跨機器/封裝發布方案）
若未來需將本套件打包給團隊多台電腦或透過 winget / scoop / PSGallery 發布，標準做法為封裝為本機模組：
1. 本專案已提供對應清單與腳本：
   - 模組定義：[`AgentHelpers.psd1`](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/profile/AgentHelpers.psd1) + [`AgentHelpers.psm1`](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/profile/AgentHelpers.psm1)
   - 模組安裝腳本：[`install-module.ps1`](file:///C:/Users/Administrator/Desktop/skill開發/agent-infra/profile/install-module.ps1)
2. 執行方式：
   ```powershell
   # 建立 Junction/Symlink 連結（開發者推薦：零拷貝、同步即時生效、免改 $PROFILE）
   .\profile\install-module.ps1

   # 實體拷貝發布模式（分發至非開發環境）
   .\profile\install-module.ps1 -Copy

   # 移除模組
   .\profile\install-module.ps1 -Uninstall
   ```
3. **優點**：完全免動使用者的 `$PROFILE`，PowerShell 具有 Module Auto-loading 機制，只要輸入指令即可隨選自動載入。

