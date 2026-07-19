---
name: pi-agent-delegate
description: Allows Antigravity to delegate tasks (e.g. coding, refactoring, code review) to the local 'pi' CLI.
---

# Pi Agent Delegate Skill

This skill allows Antigravity (the parent agent) to orchestrate and delegate sub-tasks to the locally installed `pi` coding agent CLI for maximum speed, offline/local LLM execution, or specific model capabilities.

## ⚠️ Critical Rule: Stdin Pipe
Because the task runner keeps `stdin` open, `pi` will hang indefinitely waiting for standard input to end (to support pipeline parsing).
**You MUST always prefix all `pi` commands with `$null |`** when running via PowerShell to close the input stream immediately.

## Recommended Command Structure
- **Non-interactive Execution**: Always use `-p` (or `--print`) to run `pi` non-interactively so the result is immediately returned.
- **Offline Mode**: Use the `--offline` flag to skip startup update checks (prevents network timeouts).
- **Auto-Approve**: Use the `-a` flag to auto-trust project folders and local files.

### Base Command Template:
```powershell
$null | pi --offline -a --model <model-name> -p "<prompt>"
```

### Common Invocation Examples:
- **General Code Generation / Refactoring**:
  ```powershell
  $null | pi --offline -a --model nvidia/stepfun-ai/step-3.7-flash -p "Refactor the function in src/utils.js to use async/await"
  ```
- **Code Review (Read-only)**:
  ```powershell
  $null | pi --offline -a --tools read,grep -p "Review the security of src/auth.js and print suggestions"
  ```

## Output Handling
- The output of the command will contain the result processed by `pi`.
- Parse the result and incorporate it into the workspace or main task response.

## 💡 Tips & Troubleshooting (Lesson Learned)
- **Model Selection & Latency**: Large reasoning models (like `qwen3.5-122b-a10b`) may take several minutes to generate large files (e.g. 1000+ lines HTML). For faster feedback, prefer fast Flash models like `nvidia/stepfun-ai/step-3.7-flash`.
- **Log Buffering on Windows**: On Windows/PowerShell, output redirection is often buffered. The task log may remain empty or missing until `pi` writes a chunk of progress or exits. Do not assume the process is hung immediately; give it 1–2 minutes if generating large content.

