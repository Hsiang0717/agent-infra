# ⌨️ Smart-Hub: Rust SQLite 3-Tier Command Predictor & PSReadLine Engine

專為 PowerShell 7 打造的原生 **三層智慧預測引擎 (`ICommandPredictor`)** 與鍵盤操控設定。
**無需按 `Ctrl+R`**，平常打字時下方自動浮出的原生 `ListView` 清單即由 Rust + SQLite 引擎即時驅動，分層顯示 **`[★ Smart]`**、**`[● Local]`**、**`[○ Global]`**！

---

## 🌟 核心特性

1. **原生浮動清單三層智慧預測 (`[★ Smart]` / `[● Local]` / `[○ Global]`)**：
   - 註冊三組獨立 `ICommandPredictor` 子系統，取代預設單調的 `<History>`：
     - **`[★ Smart]`**：專案環境自適應指令（自動識別 `Cargo.toml`、`package.json` scripts、`uv`/`pyproject.toml`、`docker-compose.yml`、Git）+ 虛別名 (`Virtual Aliases`) + 常用目錄跳轉 (`cd '...'`)。
     - **`[● Local]`**：當前資料夾 (`$PWD`) 與當前 Git 儲存庫下執行過的專屬歷史指令。
     - **`[○ Global]`**：跨目錄全域 SQLite 歷史指令（首次啟動自動無縫匯入既有 PSReadLine 歷史）。
   - **極速常駐管道 (`smart-hub serve` + `SmartHub.Predictor.dll`)**：
     - 透過預編譯 C# 轉接器與常駐 Rust 背景處理程序透過無 BOM UTF-8 管道通訊，單次按鍵三層預測延遲 **< 0.5ms**，完全符合 PowerShell `CommandPrediction` 20ms 硬時限。

2. **即刻剔除打錯指令 (`Alt + Backspace`)**：
   - 打錯指令後隨時按 `Alt + Backspace`，立即從 SQLite 歷史資料庫中刪除上一筆指令並刷新預測快取。

3. **流暢鍵盤互動調校 (`smart-hub init`)**：
   - **上下文感知右方向鍵 (`RightArrow`)**：游標在行中向前移動一格 (`ForwardChar`)；游標在行尾時僅採納預測提示的「**下一個單字**」(`AcceptNextSuggestionWord`)。
   - **`Ctrl + RightArrow`**：游標在行尾時直接採納整行預測 (`AcceptSuggestion`)。
   - **上下鍵歷史與清單導航 (`UpArrow` / `DownArrow`)**：浮動清單展開時直接上下選取 `[★ Smart]` / `[● Local]` / `[○ Global]` 候選項目；一般狀態下依前綴過濾歷史並將游標置於行尾。
   - **行內無阻輪替補全 (`Tab` / `Shift+Tab`)**：停用干擾選單，行內快速切換補全。

---

## 🚀 安裝與更新

比照 `status/` 架構，所有設定與轉接器皆內建於 Rust 引擎中，透過 `install.ps1` 一鍵編譯並部署至 `~/.antigravity/` 與 `$PROFILE`：

```powershell
# 編譯 Rust 引擎、預編譯 SmartHub.Predictor.dll 並寫入 $PROFILE
.\pwsh\install.ps1 -Build
```

---

## 🛠️ CLI 常用管理指令

```powershell
# 測試當前目錄下的三層即時預測結果
~/.antigravity/smart-hub.exe predict --query "git"

# 探測當前專案環境與自動識別的腳本動作
~/.antigravity/smart-hub.exe probe

# 管理虛別名 (Virtual Aliases，不污染全域 PowerShell Alias，直接顯示於 [★ Smart])
~/.antigravity/smart-hub.exe alias add dcup "docker compose up -d" --desc "啟動背景容器"
~/.antigravity/smart-hub.exe alias list
~/.antigravity/smart-hub.exe alias remove dcup

# 立即刪除上一筆歷史記錄
~/.antigravity/smart-hub.exe delete-last
```
