# Antigravity CLI Custom Statusline Installation Guide

This guide describes how to install and configure the custom PowerShell-based statusline telemetry dashboard for the Antigravity CLI.

---

## Installation Steps

### 1. Copy the Script File
Copy the `statusline.ps1` script to your user directory's `.antigravity` folder:
- **Source:** `agent-infra/status/statusline.ps1`
- **Destination:** `C:\Users\<Username>\.antigravity\statusline.ps1`

*Note: Replace `<Username>` with your Windows system username.*

### 2. Configure `settings.json`
Open the Antigravity configuration file located at:
`C:\Users\<Username>\.gemini\antigravity-cli\settings.json`

Add or update the `"statusLine"` block inside the JSON object (make sure to replace `<Username>` with your actual Windows username):

```json
  "statusLine": {
    "type": "custom",
    "command": "pwsh -NoProfile -ExecutionPolicy Bypass -File C:/Users/<Username>/.antigravity/statusline.ps1",
    "enabled": true
  }
```

---

## Script Parameter Guide

The statusline script supports the following command-line parameters to customize behavior:

### 1. Classic Icon Compatibility Mode (`--classic` / `--no-nerdfont` / `--compatibility`)
By default, the statusline renders high-fidelity icons designed for terminals configured with a **Nerd Font**. If your terminal font does not support Nerd Font characters (showing them as broken boxes or garbled text), append this parameter to force plain Unicode/ASCII symbols:

- **Configuration syntax:**
  ```json
  "command": "pwsh -NoProfile -ExecutionPolicy Bypass -File C:/Users/<Username>/.antigravity/statusline.ps1 --classic"
  ```
- **Icon Changes:**
  - `READY`: Uses `●` instead of ``
  - `THINKING`: Uses `◆` instead of `󰟷`
  - `WORKING`: Uses `⚙` instead of ``
  - `VCS Branch`: Uses `╱` instead of ``
  - `Sandbox Net/No-Net`: Uses text labels `ON (net)` / `ON (no-net)` instead of symbols

### 2. Statusline Legend (`--legend` / `-l` / `legend`)
Run this parameter manually in your terminal to output a dynamic telemetry layout legend explaining what each icon and state represents:

- **Command:**
  ```powershell
  pwsh -NoProfile -ExecutionPolicy Bypass -File C:\Users\<Username>\.antigravity\statusline.ps1 --legend
  ```
