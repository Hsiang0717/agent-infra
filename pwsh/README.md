# ⌨️ PowerShell Interactive KeyBindings & PSReadLine Tuning

專為開發者日常打造的流暢終端鍵盤操作與歷史預測設定。

---

## 🌟 核心特性

1. **上下文感知的右方向鍵 (RightArrow)**：
   - **游標在行中**：向前移動一格 (`ForwardChar`)。
   - **游標在行尾**：僅採納預測提示的「**下一個單字**」(`AcceptNextSuggestionWord`)，無需整行全吃。
   - **`Ctrl + RightArrow`**：游標在行尾時直接採納整行預測 (`AcceptSuggestion`)。

2. **智慧歷史搜尋 (上下鍵)**：
   - 鍵入前綴關鍵字後按 `UpArrow` / `DownArrow`，過濾歷史紀錄並自動將游標定位至行尾，隨時可按 Enter 執行或微調參數。

3. **行內輪替補全 (Tab / Shift+Tab)**：
   - 停用干擾的跳出選單，回歸如 Bash/Zsh 般的行內快速切換。

4. **環境安全防護**：
   - 僅在 `UserInteractive` 且未重導向輸出時啟用，避免在背景或 Agent 執行時報錯。

5. **Atuin 智慧歷史整合 (可選，目錄感知優先)**：
   - 自動偵測系統是否安裝 Atuin (`winget install atuinsh.atuin`)。若存在則啟用 SQLite 毫秒級索引，按 `UpArrow` 或 `Ctrl+R` 時優先展示當前目錄執行過的指令，並保留原有的 Tab 輪替與右鍵採納體驗。

---

## 🚀 安裝方式

### 方式 1：Dot-source 引用（推薦日常本機開發）
自動在 `$PROFILE` 寫入乾淨的外部引用判斷：
```powershell
.\pwsh\install.ps1
```

### 方式 2：PowerShell 模組發布 (Module Link / Copy)
將鍵盤設定封裝為標準 PowerShell 模組並部署至 `$HOME\Documents\PowerShell\Modules\KeyBindings`：
```powershell
# 建立 Junction 連結（開發推薦：修改本專案 KeyBindings.ps1/psm1 立即全域生效）
.\pwsh\install-module.ps1

# 實體複製模式（分發至其他電腦）
.\pwsh\install-module.ps1 -Copy

# 移除模組
.\pwsh\install-module.ps1 -Uninstall
```
> [!NOTE]
> 當安裝為模組後，亦可在 `$PROFILE` 僅寫入 `Import-Module KeyBindings` 進行載入。

