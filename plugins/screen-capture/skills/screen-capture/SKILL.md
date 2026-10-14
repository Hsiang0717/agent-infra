---
name: screen-capture
description: Autonomous screen and window capture utility for UI verification, visual bug diagnosis, and multimodal inspection using view_file.
---

# Screen Capture & Visual Inspection Skill

Autonomous screenshot capture of Windows GUI applications or the desktop screen for closed-loop visual verification via `view_file`.

> [!IMPORTANT]
> **Prerequisites & Autonomous Closed-Loop**:
> `capture-screen` captures **already running** windows; it does NOT launch applications for you.
> 1. **Launch the target app yourself**: Use `Start-Process`, web dev server (`npm run dev`), or `schtasks /it` (for native Win32/Slint/Qt requiring desktop focus).
> 2. **Allow rendering time**: Wait 1~2 seconds (`Start-Sleep -Seconds 2`) for GPU/DWM window initialization before capturing.
> 3. **Zero user requests**: Never ask the user to launch apps or take screenshots. Own verification end-to-end.

## Execution Workflow

### 1. Locate Binary
- Local: `<skill_path>/bin/capture-screen.exe` or `.\plugins\screen-capture\bin\capture-screen.exe`
- Global: `~/.gemini/config/plugins/screen-capture/bin/capture-screen.exe`

### 2. Target Discernment & Capture
Never lazily default to `--fullscreen` (leaks background, dilutes resolution, cannot capture other Virtual Desktops). Prioritize specific targets:

```powershell
# A. Inspect available windows & virtual desktops (<10ms)
<bin>/capture-screen.exe --list

# B. Target by process name (Recommended for dev GUIs & browsers)
<bin>/capture-screen.exe --name "msedge" --out "scratch/ui.png"

# C. Target by window title wildcard (Works across virtual desktops)
<bin>/capture-screen.exe --title "*localhost*" --out "scratch/ui.png"

# D. Target by PID
<bin>/capture-screen.exe --pid 1234 --out "scratch/ui.png"

# E. Fullscreen (Only when full desktop/taskbar context is strictly required)
<bin>/capture-screen.exe --fullscreen --out "scratch/fullscreen.png"
```

### 3. Visual Grounding & Inspection
Immediately inspect the captured image via `view_file`:
```
view_file(AbsolutePath="<workspace>/scratch/ui.png")
```
- Audit visual layout, padding, font rendering, colors, and responsive wrapping.
- Verify resolution, inspect UI defects, and confirm against user requirements.

---

## Troubleshooting: Desktop Isolation for Native Apps (Win32/Slint/Qt)

Agent shell commands (`run_command`) run inside an isolated background desktop (`exbox-...`), not the active user desktop (`WinSta0\Default`).
While packaged/UWP apps (`calc.exe`) are auto-routed to `Default`, some native binaries (e.g. Slint/Winit) fail to acquire initial window focus or terminate immediately when launched from a background shell.

**Interactive Launch Workaround (`schtasks /it`)**:
Launch the binary directly into the user's interactive session with full focus without modifying application code:
```powershell
# 1. Dispatch into interactive session WinSta0\Default
schtasks /create /tn "RunGuiTest" /tr "<path_to_binary.exe>" /sc once /st 23:59 /f /it /rl highest
schtasks /run /tn "RunGuiTest"
Start-Sleep -Seconds 2

# 2. Capture and clean up
<bin>/capture-screen.exe --name "<binary.exe>" --out "scratch/gui.png"
schtasks /delete /tn "RunGuiTest" /f
```
