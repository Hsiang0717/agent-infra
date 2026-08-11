---
name: agy-vscode
version: 0.0.5
publisher: Hsiang0717
license: MIT
engines:
  vscode: "^1.70.0"
repository: https://github.com/Hsiang0717/skill-development
---

# AGY CLI — VS Code Extension

A VS Code extension for quickly launching AGY CLI in a native terminal editor panel with automatic environment detection.

---

## Features

### 🚀 Open AGY CLI

Launches AGY CLI in a native VS Code terminal editor panel.

- Automatically detects `.venv` in the workspace root and activates it before starting `agy`
- Prefers `pwsh` over `powershell.exe` on Windows with `-ExecutionPolicy Bypass`
- Prevents double-activation conflicts with `VIRTUAL_ENV` / `CONDA_PREFIX`

---

## Development

```bash
# Compile
pnpm run compile

# Watch mode
pnpm run watch

# Package
pnpm run package
```

Press `F5` to open a VS Code Extension Development Host with the extension loaded.
Source: [`src/extension.ts`](src/extension.ts)
