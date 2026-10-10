# Antigravity CLI Statusline (Rust 原生高效能版)

這是專為 Google Antigravity CLI 設計的極致輕量原生狀態列 (`statusline.exe`)，以純 Rust 打造，擺脫 PowerShell 啟動延遲與記憶體開銷。

## 核心架構特性

* **極致效能**：冷啟動執行耗時 < 20ms，記憶體佔用極低。
* **固定 3 行自適應儀表板（Zero-Jitter Dashboard）**：依據終端寬度（60 / 80 / 100+ 欄位）階梯式緊湊化進度條與數值，徹底防止換行跳動。
* **按需延遲載入（Lazy Loading）**：CLI Payload 已帶有 VCS 分支時，完全跳過 Git 查詢；非 Git 目錄透過純目錄探針短路返回。
* **Morandi 莫蘭迪配色**：低飽和度 24-bit TrueColor，並精準支援原生 Nerd Font 圖標與 `--classic` 文字相容雙模式。

## 建置與安裝

請在本專案目錄執行 PowerShell：

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\status\install.ps1 -Build
```

安裝腳本會：
1. 使用 `cargo build --release` 編譯 `statusline-rs` 專案。
2. 將 `statusline.exe` 部署至 `$env:USERPROFILE\.antigravity\statusline.exe`。
3. 自動清理 `$env:USERPROFILE\.antigravity\` 目錄中過去舊版 PowerShell 的殘留檔案。

預覽安裝變更（不實際更動）：

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\status\install.ps1 -WhatIf
```

## 設定 Antigravity CLI

開啟 Antigravity CLI 設定檔：

```text
C:\Users\<Username>\.gemini\antigravity-cli\settings.json
```

設定 `statusLine` 指向原生執行檔：

```json
{
  "statusLine": {
    "type": "custom",
    "command": "C:/Users/<Username>/.antigravity/statusline.exe",
    "enabled": true
  }
}
```

## 測試與驗證

測試空輸入：

```powershell
'{}' | & "$env:USERPROFILE\.antigravity\statusline.exe"
```

顯示狀態圖例：

```powershell
& "$env:USERPROFILE\.antigravity\statusline.exe" --legend
```

## 命令列參數

### Classic icon 模式（無 Nerd Font 終端相容）

```json
"command": "C:/Users/<Username>/.antigravity/statusline.exe --classic"
```

支援的別名：`--classic`, `--no-nerdfont`, `--compatibility`

### Legend 圖例模式

```powershell
& "$env:USERPROFILE\.antigravity\statusline.exe" --legend
```
