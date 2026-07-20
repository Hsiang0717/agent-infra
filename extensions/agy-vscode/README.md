---
name: agy-extension
version: 0.0.5
publisher: Hsiang0717
license: MIT
engines:
  vscode: "^1.70.0"
repository: https://github.com/Hsiang0717/skill-development
---

# AGY CLI — VS Code Extension

A VS Code extension for launching AGY CLI and composing **Intent Conflict** prompts — a merge-conflict-style format that lets you describe code intent inline, then hand it to an AI without writing a separate instruction.

---

## Features

### 🚀 Open AGY CLI

Launches AGY CLI in a native VS Code terminal editor panel.

- Automatically detects `.venv` and activates it before starting `agy`
- Prefers `pwsh` over `powershell.exe` on Windows
- Prevents double-activation conflicts with `VIRTUAL_ENV` / `CONDA_PREFIX`

### ✏️ Wrap as Intent Conflict (`Alt + -`)

Select any block of code and press `Alt + -`. The selection is replaced **inline** with a merge-conflict skeleton:

```
<<<<<<< Current (Line 5, gateway.ts)
const gatewayAddress = "192.168.1.100";
const port = 8080;
connect(gatewayAddress, port);
=======
█  ← cursor lands here, type your intent in natural language
>>>>>>> Incoming
```

- Works on **any text file** (no extension restrictions)
- Cursor automatically jumps to the intent line after `=======`
- The `Current` block preserves your original code for context
- The `Incoming` block is where you describe what you want

### 📋 Copy as Intent Prompt (`Alt + Shift + C`)

Scans the current file for all conflict blocks and copies only the **essential metadata** to the clipboard — no full file content, letting the AI read the source itself.

**Copied output example:**

```
File: src/config/gateway.ts

# Lines 5–11
<<<<<<< Current (Line 5, gateway.ts)
const gatewayAddress = "192.168.1.100";
const port = 8080;
connect(gatewayAddress, port);
=======
從環境變數讀取，失敗時 fallback 到 127.0.0.1:8080，加 3 次 retry
>>>>>>> Incoming
```

> If no conflict blocks are found, a warning is shown — use `Alt + -` first.

---

## Keybindings

| Keybinding       | Command                     |
|------------------|-----------------------------|
| `Alt + -`        | Wrap as Intent Conflict      |
| `Alt + Shift + C`| Copy as Intent Prompt        |

Both commands are also available via **right-click context menu** under the `AGY` group.

---

## Configuration

| Setting                          | Type   | Default | Description |
|----------------------------------|--------|---------|-------------|
| `agy.intentDiff.prefixTemplate`  | string | `""`    | System instruction prefix prepended to every copied prompt. Useful for setting AI persona or task context. |

**Example prefix:**
```
You are a code conflict resolver. Read the file at the given path and apply the intent described in the Incoming section.
```

---

## Typical Workflow

```
1. Open any source file
2. Select the code you want to change
3. Press Alt + -  →  conflict skeleton inserted, cursor at intent line
4. Type your intent in natural language
5. Press Alt + Shift + C  →  conflict metadata copied to clipboard
6. Paste into AGY, pi, or any AI chat
```

---

## Development

```bash
# Compile
pnpm run compile

# Watch mode
pnpm run watch

# Package (run compile first, then skip prepublish in sandbox)
npx @vscode/vsce package --no-dependencies
```

Press `F5` to open a VS Code Extension Development Host with the extension loaded.
Source: [`src/extension.ts`](src/extension.ts)
