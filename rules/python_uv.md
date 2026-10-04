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
