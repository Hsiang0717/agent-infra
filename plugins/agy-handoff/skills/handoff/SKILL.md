---
name: handoff
description: Synthesizes and packages the current session trajectory into a high-density handoff prompt and writes it to the system clipboard for seamless session continuation.
---

# Session Handoff Skill (`/handoff`)

Execute this skill when the user invokes `/handoff` or requests handing off the current conversation session to a new session.

## Core Workflow

1. **Dynamic Context Synthesis**:
   Review the current conversation history and synthesize a tailored, high-density handoff prompt. Do not use generic boilerplate. Include concrete file paths, executed verification results, and pending actions:
   - **Goal & Core Objective**: The primary problem this session was addressing.
   - **Completed Work & Empirical Verification**: Exact modifications made, commands executed, and verification results.
   - **Key Files & Architecture**: Concrete file paths and critical schema/contracts.
   - **Current Checkpoint & Next Steps**: The exact next action the incoming agent should execute immediately.

2. **Write to System Clipboard**:
   Feed the synthesized handoff text into `handoff --stdin` (or `agy-handoff --stdin`) using standard input to write directly to the Windows system clipboard without encoding issues.
   Example:
   ```powershell
   @"
   [Session Handoff Prompt]
   This task is handed off from a previous session. Review the status below and continue the work:
   ...
   "@ | handoff --stdin
   ```

3. **Response Protocol (Strict Context Protection)**:
   - **NEVER output the full handoff prompt in the chat response.** The content is already placed in the clipboard; printing the full text in chat pollutes and bloats the context window.
   - Output only a concise confirmation (2 to 3 sentences):
     - Confirmation that the handoff prompt has been written to the system clipboard.
     - A brief 1-2 sentence summary of key handoff topics.
     - A reminder advising the user to paste directly (`Ctrl+V`) into the new session.
