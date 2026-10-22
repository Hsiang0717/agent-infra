---
trigger: always_on
description: "Enforce uv as the primary Python environment and toolchain on this system"
---

# Python Toolchain Protocol

- The system Python environment and package manager is `uv`.
- Always prefer `uv` commands when executing or managing Python:
  - Run scripts: `uv run <script.py>`
  - Run modules: `uv run python -m <module>`
  - Add packages: `uv add <package>`
  - Remove packages: `uv remove <package>`
  - Manage environments: `uv venv`
  - Python versions: `uv python list`, `uv python install <version>`
- Avoid direct invocation of unmanaged global `python` or `pip` unless explicitly instructed.

## Static Verification & Linting Protocol (Golden Guard)
When verifying Python edits, enforce the two-tier fast-feedback loop:
1. **Tier 1 (Syntax Gate - AST/Compile)**:
   - `uv run python -m py_compile <file>`: Instant check for parse/indentation/syntax validity without external dependencies.
2. **Tier 2 (Semantic Hygiene & Auto-Healing - Ruff)**:
   - Lint check: `uvx ruff check <file>` (catches undefined variables `F821`, dead imports `F401`, syntax/logic flaws).
   - Safe autofix: `uvx ruff check --fix <file>` (auto-cleans dead imports, applies safe corrections).
   - Format: `uvx ruff format <file>` (standardizes code style).
