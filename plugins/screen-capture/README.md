# screen-capture Plugin

High-performance Windows DirectX / `Windows.Graphics.Capture` (WGC) screen and window capture plugin optimized for autonomous LLM agents and multimodal verification.

---

## Features

- **DirectX GPU Extraction (WGC)**: Captures hardware-accelerated Chrome, Edge, VS Code, and Electron windows directly from GPU VRAM without black-screen artifacts.
- **Zero Terminal Interference**: Automatically hides the calling console window during capture using dynamic `ShowWindow` with RAII restoration.
- **LLM Agent-Centric (ACI)**: Pure key-value machine-parseable output (`[CAPTURED_IMAGE]`), zero decorative emojis/ASCII tables.
- **Autonomous Error Diagnostic**: Automatically reports available window titles and PIDs when target window resolution fails.
- **Ultra Low Latency**: Compiled native Rust binary with < 30ms execution time and zero runtime dependencies.

---

## Directory Structure

```text
plugins/screen-capture/
├── plugin.json                    # Antigravity plugin manifest
├── README.md                      # Plugin documentation
├── bin/
│   └── capture-screen.exe         # Native WGC release binary (414 KB)
├── skills/
│   └── screen-capture/
│       ├── SKILL.md               # Agent skill definition
│       └── bin/
│           └── capture-screen.exe # Local skill binary copy for direct resolution
└── crate/                         # Complete Rust source project
    ├── Cargo.toml
    └── src/
        └── main.rs
```

---

## CLI Usage

```powershell
# Fullscreen capture
capture-screen.exe --fullscreen --out "scratch/fullscreen.png"

# Target by process name
capture-screen.exe --name "msedge.exe" --out "scratch/browser.png"

# Target by window title (wildcard)
capture-screen.exe --title "*DevTools*" --out "scratch/devtools.png"

# Target by PID
capture-screen.exe --pid 1234 --out "scratch/app.png"
```
