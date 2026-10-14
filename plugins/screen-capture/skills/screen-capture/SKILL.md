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

### 1. Target Discernment Protocol
**Never lazily default to `--fullscreen`**: Fullscreen dilutes image resolution, leaks irrelevant background desktop windows, and **cannot capture windows located on other Virtual Desktops**!

Follow this priority hierarchy to discern the target:
1. **Agent-Launched GUI / Desktop App**:
   - If you launched a Python GUI, Qt, Electron, or compiled binary: target by process name (`--name "electron.exe"`) or PID (`--pid <pid>`).
2. **User-Opened Web Page or App**:
   - If verifying a web app (e.g. `localhost:3000` / Vite / Next.js) or user-opened tool: target by window title wildcard (`--title "*localhost*"`, `--title "*Edge*"`) or browser process (`--name "msedge"`, `--name "chrome"`).
3. **Target Discovery & Virtual Desktops (`--list`)**:
   - If unsure of exact window title or process name, run `--list` (< 10ms) first to inspect all open windows across all virtual desktops:
     ```powershell
     <path_to>/capture-screen.exe --list
     ```
   - Target windows by name/title will capture successfully via Direct3D even if located on **another Virtual Desktop** or behind other windows.
4. **Fullscreen (`--fullscreen`)**:
   - Use ONLY when verifying whole-desktop layout, OS taskbar/system tray notifications, or multi-window desktop interactions.

### 2. Capture Execution
Locate `capture-screen.exe`:
- Relative to this skill: `<skill_path>/bin/capture-screen.exe`
- Or from workspace root: `.\plugins\screen-capture\bin\capture-screen.exe`
- Or from global plugins: `~/.gemini/config/plugins/screen-capture/bin/capture-screen.exe`

```powershell
# 1. Inspect candidate windows (when unsure or exploring virtual desktops)
<path_to>/capture-screen.exe --list

# 2. Targeted capture by process name (Recommended for dev GUIs/browsers)
<path_to>/capture-screen.exe --name "msedge" --out "scratch/ui_check.png"

# 3. Targeted capture by window title wildcard (Works across virtual desktops!)
<path_to>/capture-screen.exe --title "*localhost*" --out "scratch/ui_check.png"

# 4. Targeted capture by PID
<path_to>/capture-screen.exe --pid 1234 --out "scratch/ui_check.png"

# 5. Fullscreen (Only when whole-desktop context is genuinely needed)
<path_to>/capture-screen.exe --fullscreen --out "scratch/fullscreen.png"
```

### 2. Autonomous Visual Inspection
Immediately call the `view_file` tool on the resulting output path (e.g. `scratch/ui_check.png`).
The image will be loaded directly into your multimodal vision context.

### 3. Verify & Validate
- Inspect layout alignment, margins, text wrapping, and colors.
- Check for error dialogs, missing assets, or broken styling.
- Report observations or confirm verification against requirements.
