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
