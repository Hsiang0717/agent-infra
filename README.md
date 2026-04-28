# agent-infra
Best agent settings


## Default pwsh 7.x on gemini bash

use where to find gemini path
``` bash
where.exe gemini
```
insert script follow diff in gemini.ps1
``` ps1
#!/usr/bin/env pwsh
...


# INSERT START
# --- 防呆與自動切換邏輯 ---
# 嘗試尋找 pwsh.exe
$pwshExe = Get-Command pwsh.exe -ErrorAction SilentlyContinue

if ($pwshExe) {
    # 如果找到了 PS7
    $ps7Dir = Split-Path $pwshExe.Source -Parent
    # 1. 注入路徑
    $env:PATH = "$ps7Dir;$env:PATH"
    # 2. 設定別名 (讓內部呼叫 powershell 時優先指向 pwsh)
    Set-Alias -Name powershell -Value $pwshExe.Source -Force
}

# INSERT END


$basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent
...
```