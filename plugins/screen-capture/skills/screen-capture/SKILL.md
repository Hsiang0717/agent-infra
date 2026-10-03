---
name: screen-capture
description: Autonomous screen and window capture utility for UI verification, visual bug diagnosis, and multimodal inspection using view_file.
---

# Screen Capture & Visual Inspection Skill

Provides autonomous screenshot capture of Windows GUI applications or the desktop screen, enabling closed-loop visual inspection and verification with `view_file`.

## When to Use This Skill
- **GUI / Web UI Verification**: After modifying frontend, web UI, Electron, or desktop applications, to verify visual rendering and styling.
- **Visual Bug Diagnosis**: When diagnosing UI layout glitches, rendering errors, or unexpected GUI popups.
- **Collaborative Review**: When discussing UI appearance or design with the user.

> [!IMPORTANT]
> Never assume visual states or ask the user to provide screenshots when a live GUI process is running. Use this skill to autonomously capture and inspect the window.

## How to Execute

### 1. Capture Target Window or Screen (Native Rust CLI)
Locate `capture-screen.exe` from this skill's directory or the plugin `bin/`:
- Relative to this skill: `<skill_path>/bin/capture-screen.exe`
- Or from workspace root: `.\plugins\screen-capture\bin\capture-screen.exe`
- Or from global plugins: `~/.gemini/config/plugins/screen-capture/bin/capture-screen.exe`

It leverages the modern `Windows.Graphics.Capture` (WGC) API for Direct3D11 GPU frame extraction (capturing hardware-accelerated Chrome/Electron/DWM windows with zero black-screen issues) and automatically hides the console during capture:

```powershell
# Option A: Capture by process name (Recommended for quick targeting)
<path_to>/capture-screen.exe --name "chrome.exe" --out "scratch/ui_check.png"

# Option B: Capture by window title wildcard
<path_to>/capture-screen.exe --title "*DevTools*" --out "scratch/ui_check.png"

# Option C: Capture by process ID (PID)
<path_to>/capture-screen.exe --pid 1234 --out "scratch/ui_check.png"

# Option D: Capture full screen
<path_to>/capture-screen.exe --fullscreen --out "scratch/fullscreen.png"
```

### 2. Autonomous Visual Inspection
Immediately call the `view_file` tool on the resulting output path (e.g. `scratch/ui_check.png`).
The image will be loaded directly into your multimodal vision context.

### 3. Verify & Validate
- Inspect layout alignment, margins, text wrapping, and colors.
- Check for error dialogs, missing assets, or broken styling.
- Report observations or confirm verification against requirements.
