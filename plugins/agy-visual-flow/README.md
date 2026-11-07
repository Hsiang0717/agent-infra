# agy-visual-flow Plugin

AI-Native Visual Perception, ViT Tile Optimization, and Deterministic RPA Suite for Antigravity agents.

## Overview
Replaces legacy single-shot screen capture with a comprehensive, multimodal interaction engine:
- **Vision Transformer (ViT) Tile Optimization**: Proportionally scales screenshots to boundary multiples of 64px (default max 1024px), eliminating fragmented token waste and enhancing OCR clarity.
- **Hierarchical Dual-Grid Targeting**: Renders `A1..D4` macro cells and `1..9` keypad micro cells for zero-CV black-box element localization.
- **Motion UX Storyboard**: Stitches consecutive frames across interaction sequences into a unified **768x768 2x2 grid**, allowing models to audit animation and layout transitions in a single visual step.
- **Deterministic RPA Click**: Sub-millisecond cursor jump, window activation, and complete `DOWN -> UP` click execution.
- **Interactive Snip**: Windows Snipping Tool integration for collaborative visual feedback.

## Directory Structure
```
plugins/agy-visual-flow/
├── plugin.json
├── README.md
├── bin/
│   └── agy-visual-flow.exe
├── crate/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── image_opt.rs
│       └── rpa.rs
└── skills/
    └── visual-flow/
        ├── SKILL.md
        └── bin/
            └── agy-visual-flow.exe
```
