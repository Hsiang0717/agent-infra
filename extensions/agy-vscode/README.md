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

Launches AGY CLI in a native VS Code terminal (editor panel or bottom panel).

- **Zero Latency Startup**: Removed fixed sleep delays for responsive terminal initialization.
- **Smart Virtual Environment Detection**: Automatically checks `.venv`, `venv`, VS Code Python interpreter settings, or a custom user path.
- **Shell Optimization**: Prefers `pwsh` over `powershell.exe` on Windows with `-ExecutionPolicy Bypass`.
- **Instance Management**: Supports reusing existing terminal or fresh restarts.
- **Prevents Conflicts**: Isolates `VIRTUAL_ENV` / `CONDA_PREFIX` during terminal boot.

---

## Extension Settings

This extension contributes the following settings:

| Setting | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `agy.terminalLocation` | `string` | `"editor"` | Where to open the AGY terminal (`"editor"` or `"panel"`). |
| `agy.reuseTerminal` | `boolean` | `false` | Reuse the existing AGY CLI terminal instead of restarting. |
| `agy.customPath` | `string` | `""` | Custom path or executable name for AGY CLI (defaults to `agy`). |
| `agy.customArguments` | `string` | `""` | Extra arguments passed to AGY CLI on startup. |
| `agy.venvPath` | `string` | `""` | Custom virtual environment path (relative or absolute). |

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
