# Antigravity CLI Statusline 安裝指南

這個 statusline 是一個輕量高效的 PowerShell 狀態列套件。`statusline.ps1` 是唯一入口，
Git 與電源查詢則封裝於同資料夾的 `.psm1` 模組中，具備以下核心架構特性：

* **固定 3 行自適應儀表板（Zero-Jitter Dashboard）**：依據終端寬度（60 / 80 / 100+ 欄位）階梯式緊湊化進度條與數值，徹底防止換行跳動。
* **按需延遲載入（Lazy Loading）**：CLI Payload 已帶有 VCS 分支時，完全跳過 Git 模組與外部 `git.exe` 呼叫；非 Git 目錄透過純 .NET 快速探針短路返回。
* **極速反射查詢**：電源模組採用 .NET 型別快取與反射載入，擺脫 `Add-Type` 的編譯延遲（縮短至 ~9ms）。
* **Morandi 莫蘭迪配色**：低飽和度 24-bit TrueColor，並精準支援原生 Nerd Font 圖標與 `--classic` 文字相容雙模式。

## 安裝

請在本專案目錄執行 PowerShell：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\status\install.ps1
```

*(若在專案上一層目錄執行，路徑為 `.\agent-infra\status\install.ps1`)*

請不要只複製 `statusline.ps1`；主腳本會從 `$PSScriptRoot` 自動載入 `Status.Git.psm1` 與 `Status.Power.psm1`。

## 設定 Antigravity CLI

開啟：

```text
C:\Users\<Username>\.gemini\antigravity-cli\settings.json
```

加入或更新：

```json
{
  "statusLine": {
    "type": "custom",
    "command": "pwsh -NoProfile -ExecutionPolicy Bypass -File C:/Users/<Username>/.antigravity/statusline.ps1",
    "enabled": true
  }
}
```

請將 `<Username>` 替換成實際 Windows 使用者名稱。若使用 Windows PowerShell，
可以把 `pwsh` 改成 `powershell`。

## 更新

更新程式碼後，重新執行安裝指令即可覆蓋主腳本與模組：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\agent-infra\status\install.ps1
```

預覽不實際覆蓋檔案：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\agent-infra\status\install.ps1 -WhatIf
```

## 測試

測試空輸入：

```powershell
'{}' | pwsh -NoProfile -ExecutionPolicy Bypass -File "$env:USERPROFILE\.antigravity\statusline.ps1"
```

顯示狀態圖例：

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File "$env:USERPROFILE\.antigravity\statusline.ps1" --legend
```

## 命令列參數

### Classic icon 模式

適用於沒有 Nerd Font 的終端：

```json
"command": "pwsh -NoProfile -ExecutionPolicy Bypass -File C:/Users/<Username>/.antigravity/statusline.ps1 --classic"
```

支援的別名：

```text
--classic
--no-nerdfont
--compatibility
```

### Legend 模式

支援：

```text
--legend
-l
legend
```

## 可選環境變數

```powershell
$env:ANTIGRAVITY_STATUS_NO_POWER = '1'
```

設定後會停用電池 / AC 查詢，適合遠端或高頻刷新環境。
