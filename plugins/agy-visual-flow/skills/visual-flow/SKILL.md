---
name: visual-flow
description: AI-native visual perception and deterministic RPA suite for autonomous UI/UX verification, hierarchical dual-grid targeting, 768x768 storyboard motion inspection, and deterministic click execution via agy-visual-flow.exe.
---

# Visual Flow & Autonomous RPA Skill

AI-native visual perception and deterministic RPA tool powered by a high-performance standalone Rust binary (`agy-visual-flow.exe`). Provides end-to-end visual grounding, motion UX inspection, and deterministic interaction:

1. **Autonomous Window & Screen Perception**: DirectX / Windows.Graphics.Capture (WGC) with fallback to GDI for robust, multi-desktop UI verification.
2. **Vision Transformer (ViT) Tile Optimization (Default)**: Automatically scales images to proportional max-dimensions (default 1024px) snapped to 64px boundaries, eliminating fragmented tiles and maximizing OCR/layout clarity with minimal token overhead.
3. **Hierarchical Dual-Grid Targeting (`--grid`)**: Injects high-contrast macro boundaries (`A1..D4`) and subtle micro subcells (`1..9` keypad order) onto screenshots for zero-CV black-box element localization.
4. **Motion UX Storyboard (`--storyboard [sec]`)**: Captures sequential interaction frames across 1-10 seconds and stitches them into a compact **768x768 2x2 grid** (`STEP 1..4`), enabling rapid diagnostic of layout shift, animations, and transitions.
5. **Deterministic RPA Click (`--click <grid|x,y>`)**: Executes zero-latency focus assurance, direct cursor placement, and full `DOWN -> UP` event-pair firing without human-like delays.
6. **Collaborative Interactive Snip (`--snip`)**: Triggers Windows Snipping Tool for user-assisted cropping and annotation.

---

## 1. Locate Binary
- Local: `<skill_path>/bin/agy-visual-flow.exe` or `.\plugins\agy-visual-flow\bin\agy-visual-flow.exe`
- Global: `~/.gemini/config/plugins/agy-visual-flow/bin/agy-visual-flow.exe`

---

## 2. Autonomous UI Verification (Mode A)

### Standard Inspection with ViT Tile Optimization:
```powershell
# Capture window by process name with ViT tile boundary alignment (e.g. 1024x576)
<bin>/agy-visual-flow.exe --name "msedge" --out "scratch/ui.png"

# Inspect list of active windows
<bin>/agy-visual-flow.exe --list
```

### Black-Box Hierarchical Grid Targeting:
When interacting with a window lacking accessibility IDs:
```powershell
# Capture window with Macro (A1..D4) + Micro (1..9) dual grid
<bin>/agy-visual-flow.exe --name "rs-yt-player" --grid --out "scratch/grid.png"
```
Read `scratch/grid.png` via `read_file`:
1. Identify macro zone (e.g. `B3`).
2. Identify keypad micro position (e.g. `5` for center, `1` for top-left).
3. Execute deterministic click:
```powershell
<bin>/agy-visual-flow.exe --click "B3-5"
```

### Motion & UX Storyboard Inspection:
When assessing interactive state transitions, loading skeletons, or animations across A ➔ B ➔ C:
```powershell
# Record a 2-second interaction sequence composed into a single 768x768 2x2 image
<bin>/agy-visual-flow.exe --name "rs-yt-player" --storyboard 2 --out "scratch/storyboard.png"
```
Read `scratch/storyboard.png` via `read_file` to evaluate `STEP 1` through `STEP 4` in a single tool call.

---

## 3. Collaborative Interactive Snip (Mode B)

Used when user marks specific visual defects with highlighter/pen tools:
```powershell
<bin>/agy-visual-flow.exe --snip --out "scratch/user_snip.png"
```
Read the saved snip via `read_file` and ground code edits on the marked coordinates.
