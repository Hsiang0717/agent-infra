# OVERALL
This ruleset governs system behavior across all phases of task orientation, planning, implementation, editing, troubleshooting, and verification.

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
    **TIMING:** When exploring code architecture, tracing types, or finding call sites.
    **DIRECTIVE:** PREFER semantic LSP for symbols; FALLBACK to text search when LSP is absent or for plain text.

    - **ROUTING:** Use `execute_lsp` (`definition`, `references`, `hover`) for code symbols; use `grep_search` ONLY for plain text/configs.
    - **FAST FALLBACK:** If `grep_search` returns > 10 symbol matches OR if LSP is unindexed/unavailable, IMMEDIATELY switch tools without looping.
    - **BAD:** Using plain text `grep_search` for a common symbol name like `execute()` and reading 50 irrelevant lines.
    - **GOOD:** Using `execute_lsp` (`references`) first, falling back to scoped `grep_search` if LSP is offline.
  </rule>

  <rule id="think_before_coding" phase="general">
    ### Think Before Coding
    **TIMING:** Planning Phase
    **DIRECTIVE:** CRITICAL: DO NOT assume. DO NOT hide confusion. ALWAYS surface tradeoffs.

    - **INTENT ALIGNMENT:** Describe user intent and present multiple interpretations/tradeoffs before implementing.
    - **PROACTIVE INTERVIEW:** Before major refactors or complex design choices, proactively interview the user using `ask_question` to walk down decision branches and align on goals.
    - **BAD:** Silently choosing between complex virtual scrolling vs standard pagination without presenting options.
    - **GOOD:** Proactively initiating an architectural interview using `ask_question` to present options and tradeoffs before code mutation.
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
    **DIRECTIVE:** CRITICAL: PRIORITIZE incremental editing (`replace_file_content`, `multi_replace_file_content`) over overwriting (`write_to_file`).

    - **INCREMENTAL EDIT:** Use `replace_file_content` (contiguous) or `multi_replace_file_content` (non-contiguous) for existing files.
    - **REWRITE EXCEPTION:** Use `write_to_file` ONLY for creating new files or when performing an authorized systemic refactor (`know_when_to_pivot`).
    - **BAD:** Using `write_to_file` to replace a 500-line file just to modify 50 lines logic.
    - **GOOD:** Using `replace_file_content` targeting strictly the affected lines.
  </rule>

  <rule id="know_when_to_pivot" phase="troubleshooting">
    ### Know When to Pivot & Refactor
    **TIMING:** Troubleshooting & Defect Fixing
    **DIRECTIVE:** CRITICAL: STOP patch-stacking. Recognize local minima and force a systemic reset.

    - **TRACK ATTEMPTS:** If a fix fails twice, STOP editing immediately. Step back and re-evaluate end-to-end execution flow.
    - **AUDIT BOUNDARIES:** Inspect build output artifacts (`list_dir` / `view_file`), bundler configs, and sandbox isolation constraints — do not limit debugging to source components.
    - **BAD:** Repeatedly micro-patching component lifecycle hooks for a loading hang while ignoring bundler chunk-splitting failures or overwritten CSS files.
    - **GOOD:** Halting micro-patches after 2 failures, stepping back to inspect build output files and Webview sandbox boundaries to solve root cause.
  </rule>

  <rule id="verify_before_done" phase="verification">
    ### Verify Before Completion
    **TIMING:** Before Declaring Completion
    **DIRECTIVE:** CRITICAL: NEVER claim a task is resolved without concrete empirical verification.

    - **RUN BUILD/TEST:** Always run compile/build or test commands (`run_command`) after code modifications to confirm clean success.
    - **BAD:** Declaring a bug fixed or feature complete immediately after editing a file without running build commands.
    - **GOOD:** Running `run_command` (e.g. `npm run compile` / packaging) and verifying zero errors before declaring success.
  </rule>

</agent_rules>