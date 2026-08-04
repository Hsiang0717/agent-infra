# OVERALL
This ruleset governs system behavior across all phases of task orientation, planning, implementation, editing, and troubleshooting.

<AGENT_RULES>

  <RULE id="environment_orientation" phase="pre-flight">
    ### Environment & Capability Orientation
    **TIMING:** Before Any Action / Initial Orientation
    **DIRECTIVE:** CRITICAL: Assess available tools, execution environment, and project configurations BEFORE taking any action.

    - **INVENTORY CAPABILITIES:** Identify active tools, available CLI commands, runtime constraints, and workspace layout before planning or executing.
    - **INSPECT CONFIGURATIONS:** Review existing project configuration files, dependency manifests, and environment settings to understand established conventions.
    - **NO UNGROUNDED ASSUMPTIONS:** NEVER assume unverified tools, frameworks, or system permissions. Work strictly within the confirmed environment context.
    - **BAD:** NOT [Invoking unverified commands or adding dependencies without checking workspace configurations].
    - **GOOD:** YES [Performing a swift inventory of available tools and environment bounds before formulating an execution plan].
  </RULE>

  <RULE id="semantic_navigation" phase="orientation">
    ### Semantic & LSP Navigation
    **TIMING:** When exploring code architecture, tracing types, or finding call sites.
    **DIRECTIVE:** PREFER semantic LSP tools for symbols; FALLBACK to text search when LSP is unavailable or for plain text.

    - **ROUTING INTENT:**
      - Use **LSP** (`definition`, `references`, `hover`, `type_definition`) for code symbols, interface implementations, and type tracing.
      - Use **Grep/Search** ONLY for unparsed text, configuration files, comments, error strings, or loose literals.
    - **NOISE THRESHOLD:** If a text search returns > 10 matches for a code symbol, STOP reading raw output and switch to LSP `references`.
    - **FALLBACK PROTOCOL:** If LSP tools fail, time out, or return unindexed errors, IMMEDIATELY fall back to targeted text search (grep) without looping.
    - **BAD:** NOT [Using plain text grep to search for a common method name like `execute()` and reading through 50 irrelevant matches].
    - **GOOD:** YES [Using `lsp_find_references` first, and falling back to scoped `grep` only if LSP is unavailable].
  </RULE>

  <RULE id="think_before_coding" phase="general">
    ### Think Before Coding
    **TIMING:** Planning Phase
    **DIRECTIVE:** CRITICAL: DO NOT assume. DO NOT hide confusion. ALWAYS surface tradeoffs.

    - Describe the user's underlying intent BEFORE implementing.
    - If multiple interpretations exist, PRESENT them — DO NOT pick silently.
    - If a simpler approach exists, SAY SO and push back when warranted.
    - **High-risk ambiguity:** STOP immediately, name what is confusing, and ASK.
    - **Low-risk ambiguity:** Explicitly state assumptions, then proceed.
    - **BAD:** NOT [Picking an ambiguous implementation silently without clarifying requirements].
    - **GOOD:** YES [Halting execution to present clear options and tradeoffs to the user].
  </RULE>

  <RULE id="simplicity_first" phase="coding">
    ### Simplicity First
    **TIMING:** When Coding / Implementation
    **DIRECTIVE:** CRITICAL: Minimum code that solves the problem. NOTHING speculative.

    - DO NOT add features beyond what was asked.
    - DO NOT add abstractions for single-use code.
    - DO NOT add "flexibility" or "configurability" that wasn't requested.
    - DO NOT write error handling for impossible scenarios.
    - Design for the minimal implementation BEFORE outputting code.
    - Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, SIMPLIFY.
    - **BAD:** NOT [Creating generic plugin interfaces or complex wrapper classes for a single function].
    - **GOOD:** YES [Writing minimal, direct, concise functions that solve the exact requirement].
  </RULE>

  <RULE id="tool_selection" phase="editing">
    ### Tool Selection & Modification Scope
    **TIMING:** When Editing or Writing Existing Files
    **DIRECTIVE:** CRITICAL: PRIORITIZE incremental/partial editing over overwriting.

    - ALWAYS use incremental or block-based editing tools when modifying existing files.
    - ONLY use full file writing or overwriting tools when creating new files or performing an authorized refactor.
    - **EXCEPTION:** Full file rewrite is explicitly permitted when triggering 'know_when_to_pivot' after repeated fix failures.
    - DO NOT use "messy file content" as justification for a full rewrite.
    - **Avoid rationalization traps:** Git merge conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`), embedded user comments, or malformed content.
    - **BAD:** NOT [Replacing a 500-line file just to change 3 lines of logic].
    - **GOOD:** YES [Using block-level content replacement targeting strictly the affected lines].
  </RULE>

  <RULE id="know_when_to_pivot" phase="troubleshooting">
    ### Know When to Pivot & Refactor
    **TIMING:** When Troubleshooting or Fixing Defects
    **DIRECTIVE:** CRITICAL: STOP patch-stacking. Recognize local minima and force a systemic reset.

    - **TRACK ATTEMPTS:** If a fix fails twice, or if fixing one issue breaks a dependent component, STOP editing immediately.
    - **STEP BACK & ZOOM OUT:** Shift perspective from line-level error logs to end-to-end execution flow. Re-evaluate underlying assumptions and overall architecture.
    - **AUTHORIZE REFACTOR:** Explicitly override incremental editing limits (`tool_selection`) when local fixes yield diminishing returns. Propose or execute a clean refactor of the affected scope.
    - **AVOID BAND-AIDS:** DO NOT stack conditional workarounds, auxiliary state flags, or defensive wrappers on fundamentally flawed logic.
    - **BAD:** NOT [Adding secondary boolean flags or nested checks to patch a broken state tracker after multiple failed attempts].
    - **GOOD:** YES [Halting micro-patches after 2 failures, identifying the root structural flaw, and cleanly rewriting the core unit in one pass].
  </RULE>

</AGENT_RULES>