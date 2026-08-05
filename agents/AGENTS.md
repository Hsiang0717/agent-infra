# OVERALL
This ruleset governs system behavior across all phases of task orientation, planning, implementation, editing, troubleshooting, and verification for AI Coding Agents.

<agent_rules>

  <rule id="environment_orientation" phase="pre-flight">
    ### Environment & Capability Orientation
    **TIMING:** Before Any Action / Initial Orientation
    **DIRECTIVE:** CRITICAL: Inspect environment, configurations, and active tools BEFORE planning.

    - **INVENTORY:** Check available tools, CLI environment, dependency manifests, and workspace bounds.
    - **NO ASSUMPTIONS:** Work strictly within confirmed environment context. Never assume unverified permissions or missing tools.
    - **BAD:** Invoking unverified commands or adding dependencies without checking manifests.
    - **GOOD:** Performing a quick inventory of available tools before formulating an execution plan.
  </rule>

  <rule id="semantic_navigation" phase="orientation">
    ### Semantic & LSP Navigation
    **TIMING:** Exploring Code Architecture & Finding Symbols
    **DIRECTIVE:** PREFER semantic LSP for symbols; FAST FALLBACK to text search if LSP is offline or over-matched.

    - **ROUTING:** Use semantic LSP tools (`definition`, `references`) for code symbols; use text search (`grep` / `rg`) ONLY for text/configs.
    - **FAST FALLBACK:** If text search returns > 10 symbol matches OR LSP is unindexed, switch tools immediately.
    - **BAD:** Using plain text search for common symbols like `execute()` and reading 50 irrelevant lines.
    - **GOOD:** Using LSP symbol references first, falling back to scoped text search if LSP is unavailable.
  </rule>

  <rule id="think_before_coding" phase="general">
    ### Think Before Coding
    **TIMING:** Planning Phase
    **DIRECTIVE:** CRITICAL: DO NOT assume. DO NOT hide confusion. ALWAYS surface tradeoffs.

    - **INTENT ALIGNMENT:** Describe user intent and present multiple interpretations/tradeoffs before implementing.
    - **PROACTIVE INTERVIEW:** Before major refactors or complex design choices, proactively interview the user to walk down decision branches and align on goals.
    - **BAD:** Silently choosing between complex virtual scrolling vs standard pagination without presenting options.
    - **GOOD:** Proactively initiating an architectural interview to present options and tradeoffs before code mutation.
  </rule>

  <rule id="simplicity_first" phase="coding">
    ### Simplicity First
    **TIMING:** Implementation Phase
    **DIRECTIVE:** CRITICAL: Minimum code that solves the problem. NOTHING speculative.

    - **NO OVER-ENGINEERING:** Do not add unused flexibility, single-use abstractions, or unrequested features.
    - **BAD:** Creating generic plugin interfaces or complex wrapper classes for a single helper function.
    - **GOOD:** Writing minimal, direct functions that solve the exact requirement.
  </rule>

  <rule id="tool_selection" phase="editing">
    ### Tool Selection & Scope
    **TIMING:** Editing Existing Files
    **DIRECTIVE:** CRITICAL: PRIORITIZE incremental editing (targeted diffs / patch edits) over full file overwriting.

    - **INCREMENTAL EDIT:** Use line-level or chunk-level targeted replacement for existing files.
    - **REWRITE EXCEPTION:** Use full file overwrite ONLY for creating new files or when performing an authorized systemic refactor (`know_when_to_pivot`).
    - **BAD:** Replacing a 500-line file just to modify 3 lines of logic.
    - **GOOD:** Targeting strictly the affected lines with precise incremental edits.
  </rule>

  <rule id="preserve_contracts" phase="editing">
    ### Preserve API Contracts & Update Invocation Sites
    **TIMING:** Modifying Functions, Components, or Schemas
    **DIRECTIVE:** CRITICAL: Never alter an existing contract without updating all call sites across the codebase.

    - **SEARCH INVOCATIONS:** Find all callers before modifying function signatures, component props, or schemas.
    - **ATOMIC UPDATE:** Update all invocation sites in the same task to prevent runtime signature mismatches.
    - **BAD:** Changing a function signature or component prop name without updating callers, causing downstream runtime errors.
    - **GOOD:** Finding all reference sites via LSP/grep and updating the signature and all callers together in an atomic edit.
  </rule>

  <rule id="log_evidence_first" phase="troubleshooting">
    ### Log & Evidence-Driven Debugging
    **TIMING:** Diagnosing Failures & Errors
    **DIRECTIVE:** CRITICAL: Inspect raw logs and stack traces BEFORE formulating hypotheses or writing code.

    - **INSPECT LOGS FIRST:** Fetch and read full failure tracebacks before modifying source code.
    - **NO GUESSWORK:** Base diagnosis strictly on empirical log evidence, not speculation.
    - **BAD:** Hypothesizing a cause and editing code immediately after a failure without reading actual error logs.
    - **GOOD:** Fetching un-truncated log output, isolating the exact failing line and error contract, and fixing based on concrete evidence.
  </rule>

  <rule id="know_when_to_pivot" phase="troubleshooting">
    ### Know When to Pivot & Refactor
    **TIMING:** Troubleshooting & Defect Fixing
    **DIRECTIVE:** CRITICAL: STOP patch-stacking. Recognize local minima and force a systemic reset.

    - **TRACK ATTEMPTS:** If a fix fails twice, STOP editing immediately. Step back, re-evaluate architecture, and pivot to a clean refactor or systemic reset rather than stacking fragile patches.
    - **AUDIT BOUNDARIES:** Inspect build output artifacts, bundler configs, and runtime sandbox isolation constraints — do not limit debugging to source components.
    - **BAD:** Repeatedly micro-patching component lifecycle hooks for a loading hang while ignoring bundler chunk-splitting failures or overwritten CSS files.
    - **GOOD:** Halting micro-patches after 2 failures, stepping back to inspect build output files and runtime sandbox boundaries, pivoting to a clean refactor or systemic redesign to solve root cause.
  </rule>

  <rule id="verify_before_done" phase="verification">
    ### Verify Before Completion
    **TIMING:** Before Declaring Completion
    **DIRECTIVE:** CRITICAL: NEVER claim a task is resolved without concrete empirical verification.

    - **RUN BUILD/TEST:** Always run compile/build or test commands after code modifications to confirm clean success.
    - **BAD:** Declaring a bug fixed or feature complete immediately after editing a file without running build commands.
    - **GOOD:** Running build/test verification commands (e.g. `npm run compile` / test suite) and verifying zero errors before declaring success.
  </rule>

</agent_rules>
