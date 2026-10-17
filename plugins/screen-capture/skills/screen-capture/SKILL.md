---
name: screen-capture
description: Unified AI-native screen capture utility for autonomous UI verification (Mode A) and collaborative interactive snip (Mode B) via capture-screen.exe.
---

# Screen Capture & Visual Grounding Skill

AI-native screen capture tool powered by a high-performance standalone Rust binary (`capture-screen.exe`). Provides two operational modes:

1. **Mode A: Autonomous Verification (Agent-Driven)**: Direct DirectX / Windows.Graphics.Capture of running applications and browser windows for closed-loop visual checks via `view_file`.
2. **Mode B: Collaborative Interactive Snip (Human-in-the-Loop)**: Seamlessly triggers Windows Snipping Tool via `--snip`, captures user-selected areas (including pen/color markups), and streams observations directly into the multimodal conversation.

---

## 1. Locate Binary
- Local: `<skill_path>/bin/capture-screen.exe` or `.\plugins\screen-capture\bin\capture-screen.exe`
- Global: `~/.gemini/config/plugins/screen-capture/bin/capture-screen.exe`

---

## Mode A: Autonomous Window Capture & Closed-Loop Inspection

Used autonomously by the agent to visually inspect UI changes, layout fidelity, and styling without user intervention.

### Command Syntax:
```powershell
# A. Inspect available windows & virtual desktops (<10ms)
<bin>/capture-screen.exe --list

# B. Target by process name (Recommended for dev GUIs & web apps)
<bin>/capture-screen.exe --name "msedge" --out "scratch/ui.png"

# C. Target by window title wildcard (Works across virtual desktops)
<bin>/capture-screen.exe --title "*localhost*" --out "scratch/ui.png"

# D. Target by PID
<bin>/capture-screen.exe --pid 1234 --out "scratch/ui.png"

# E. Fullscreen (Only when full taskbar/desktop context is strictly required)
<bin>/capture-screen.exe --fullscreen --out "scratch/fullscreen.png"
```

### Visual Grounding with `view_file`:
```
view_file(AbsolutePath="<workspace>/scratch/ui.png")
```
- Audit visual layout, padding, font rendering, colors, and responsive wrapping.

### Window Lifecycle & Handling Unstarted Applications:
1. Verify target window availability via `<bin>/capture-screen.exe --list`.
2. If the application is not running or previously exited/crashed:
   - Launch it into the interactive desktop session via `start-gui <path> [args] [workdir]`.
   - Wait 1-2 seconds (`Start-Sleep -Seconds 2`) for GPU swapchain and window initialization.
   - Capture the newly opened window using `<bin>/capture-screen.exe --name "<proc>" --out "<out.png>"`.
3. If an application exits or crashes immediately upon launch, inspect its logs before retrying.

---

## Mode B: Collaborative Interactive Snip Workflow

Used when:
- The user requests interactive screen capture (e.g., via `/screen-capture` or asking the agent to inspect a specific screen area).
- The user wants to crop a region or draw annotations (arrows, highlights, notes) to point out visual defects, layout bugs, or design requirements.
- Autonomous window capture is occluded, minimized, or constrained by background isolation.

### 1. Trigger Snip via Rust Binary
```powershell
<bin>/capture-screen.exe --snip --out "scratch/user_snip.png"
```
- Automatically activates native Windows Snipping Tool in the user's interactive session.
- The user selects the target region, marks details with pen/highlighter tools, clicks Copy, and closes the Snipping Tool window (`X`).
- The binary monitors window lifecycle: closing the window serves as the definitive completion signal, instantly extracting and saving the annotated image.

### 2. Inspect Screenshot via `view_file`
```
view_file(AbsolutePath="<workspace>/scratch/user_snip.png")
```
- Identify marked-up colors, callouts, arrows, and target UI components.

### 3. Ground Analysis & Execute Action
- Correlate the visual evidence directly with the relevant code (e.g., CSS, templates, components, state logic).
- Confirm user expectations and proceed with surgical implementation or fixes.
- Re-verify using Mode A or appropriate test commands once completed.
