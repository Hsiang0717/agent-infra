---
name: handoff
description: Synthesizes and packages the current session trajectory into a high-density handoff prompt and writes it to the system clipboard for seamless session continuation.
---

# Session Handoff Skill (`/handoff`)

Execute this skill when the user invokes `/handoff` or requests handing off the current conversation session to a new session.

## Core Workflow

1. **Git & Workspace Snapshot (Ground-Truth Anchoring)**:
   Before writing the handoff prompt, run a fast Git probe to capture actual disk state rather than relying on memory:
   ```powershell
   git branch --show-current; git status -s; git log -n 3 --oneline
   ```
   - **Verified Milestone (Safe Checkpoint)**: If completed work passes syntax/verification gates on a topic branch (never commit directly on `main`/`master`), create an atomic Conventional Commit first so the incoming session has a clean rollback anchor.
   - **In-Flight / Unverified WIP (Preserve Working Tree)**: If edits are mid-flight or failing compilation, **do NOT force a broken commit**. Leave them in the working tree and explicitly record all modified/untracked files (`git status -s`) in the handoff prompt so the incoming agent does not accidentally discard or `git checkout --` them.

2. **High-Density Context Synthesis (Zero Chronological Fluff)**:
   Synthesize a tailored, high-signal handoff prompt covering five mandatory sections (omit narrative chat history; retain only causal conclusions and exact coordinates):
   - **Goal & Core Objective**: The primary problem and target acceptance criteria.
   - **Causal Context & Ruled-Out Traps (Why & Pitfalls)**: Why the current design/approach was chosen, plus **approaches already tried and ruled out** (with root causes) so the incoming agent never repeats failed loops.
   - **Current State & Git Snapshot**: Active branch, latest commit SHA, empirical verification status (what compiles/passes vs. current failing errors), and exact uncommitted WIP files.
   - **Files & Line Ranges to Read First**: Concrete file paths with exact line coordinates (e.g. `src/session.rs (L45-L90)`) and what to inspect in each, enabling zero-search startup.
   - **Immediate Next Action**: The exact first command to run or specific function/line range to modify upon starting the new session.

3. **Write to System Clipboard (UTF-8 & Single-Quoted Here-String)**:
   Always enforce UTF-8 pipe encoding and wrap the payload in a **single-quoted Here-String (`@' ... '@`)** piped into `handoff --stdin` (or `agy-handoff --stdin`).
   - **CRITICAL**: Never use double-quoted `@" ... "@`, which silently expands and corrupts `$variables` and `$(...)` subexpressions inside code snippets.
   - *Tip*: If the user only asks to copy the latest raw assistant reply verbatim without synthesizing a handoff, run `handoff` with no arguments (extracts the last `PLANNER_RESPONSE` directly from `transcript.jsonl`).

   ```powershell
   [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
   $OutputEncoding = [System.Text.Encoding]::UTF8
   @'
   # [Session Handoff] <Concise Task Title>

   ## 1. Goal & Core Objective
   - <Target outcome and acceptance criteria>

   ## 2. Causal Context & Ruled-Out Traps (Do Not Repeat)
   - **Why current approach**: <Key architectural constraint or root cause discovered>
   - **Ruled-out / Failed attempts**: <What failed and why — prevent regression loops>

   ## 3. Current State & Git Snapshot
   - **Branch & HEAD**: `<branch>` @ `<commit-sha>` (`<commit-subject>`)
   - **Working Tree (`git status -s`)**: <Clean OR list of uncommitted WIP files — DO NOT discard>
   - **Verification Status**: <Commands run, passing baseline, and any active error/failing test>

   ## 4. Priority Files to Read First (Zero-Search Coordinates)
   - `<path/to/file>` (Lines `<start>-<end>`): <Why to read / key symbol or state>

   ## 5. Immediate Next Steps
   1. <Exact first command or surgical edit to execute immediately>
   2. <Subsequent verification step>
   '@ | handoff --stdin
   ```

4. **Response Protocol (Strict Context Protection)**:
   - **NEVER output the full handoff prompt in the chat response.** The content is already in the clipboard; printing it in chat bloats and pollutes the context window.
   - Output only a concise confirmation (2 to 3 sentences):
     - Confirmation that the handoff prompt has been written to the system clipboard (including character count from CLI output).
     - A brief 1-sentence summary of the Git branch/WIP state and immediate next step.
     - A reminder advising the user to paste directly (`Ctrl+V`) into the new session.
