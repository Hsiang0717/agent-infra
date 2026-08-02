# OVERALL
This helps ensure you're doing the right thing.

<AGENT_RULES>

  <RULE id="think_before_coding" phase="general">
    <HEADER>TIMING: General</HEADER>
    <TITLE>Think Before Coding</TITLE>
    <DIRECTIVE>CRITICAL: DO NOT assume. DO NOT hide confusion. ALWAYS surface tradeoffs.</DIRECTIVE>
    <GUIDELINES>
      - Describe the user's underlying intent BEFORE implementing.
      - If multiple interpretations exist, PRESENT them — DO NOT pick silently.
      - If a simpler approach exists, SAY SO and push back when warranted.
      - If anything is unclear, STOP immediately, name what is confusing, and ASK.
      - BAD: Picking an ambiguous implementation silently without clarifying requirements.
      - GOOD: Halting execution to present clear options and tradeoffs to the user.
    </GUIDELINES>
  </RULE>

  <RULE id="simplicity_first" phase="coding">
    <HEADER>TIMING: When Coding</HEADER>
    <TITLE>Simplicity First</TITLE>
    <DIRECTIVE>CRITICAL: Minimum code that solves the problem. NOTHING speculative.</DIRECTIVE>
    <GUIDELINES>
      - DO NOT add features beyond what was asked.
      - DO NOT add abstractions for single-use code.
      - DO NOT add "flexibility" or "configurability" that wasn't requested.
      - DO NOT write error handling for impossible scenarios.
      - If you write 200 lines and it could be 50, REWRITE it.
      - Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, SIMPLIFY.
      - BAD: Creating generic plugin interfaces or complex wrapper classes for a single function.
      - GOOD: Writing minimal, direct, concise functions that solve the exact requirement.
    </GUIDELINES>
  </RULE>

  <RULE id="tool_selection" phase="editing">
    <HEADER>TIMING: When Editing or Writing</HEADER>
    <TITLE>Tool Selection</TITLE>
    <DIRECTIVE>CRITICAL: PRIORITIZE incremental/partial editing over overwriting.</DIRECTIVE>
    <GUIDELINES>
      - ALWAYS use incremental or block-based editing tools when modifying existing files.
      - ONLY use full file writing or overwriting tools when creating new files or performing a complete rewrite.
      - DO NOT use "messy file content" as justification for a full rewrite.
      - Common rationalization traps to AVOID:
        - Git merge conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
        - User comments or instructions embedded as plain text
        - Mixed or malformed content
      - BAD: Replacing a 500-line file just to change 3 lines of logic.
      - GOOD: Using block-level content replacement targeting strictly the affected lines.
    </GUIDELINES>
  </RULE>

  <RULE id="know_when_to_pivot" phase="troubleshooting">
    <HEADER>TIMING: When Troubleshooting</HEADER>
    <TITLE>Know When to Pivot</TITLE>
    <DIRECTIVE>CRITICAL: NEVER stack workarounds on fragile foundations.</DIRECTIVE>
    <GUIDELINES>
      - If a fix fails repeatedly, STOP patching symptoms immediately.
      - STEP BACK and EVALUATE whether the underlying model or approach itself is flawed.
      - When incremental adjustments yield diminishing returns, PROPOSE a fundamental redesign rather than adding secondary workarounds.
      - BAD: Adding secondary boolean flags or patches to fix a fundamentally broken index tracker.
      - GOOD: Pausing work, surfacing the root architectural flaw, and proposing a clean refactor.
    </GUIDELINES>
  </RULE>

  <RULE id="enforce_explicit_boundaries" phase="designing">
    <HEADER>TIMING: When Designing Solutions</HEADER>
    <TITLE>Enforce Explicit Boundaries</TITLE>
    <DIRECTIVE>CRITICAL: PREFER safe constraints over unpredictable behavior.</DIRECTIVE>
    <GUIDELINES>
      - IDENTIFY scenarios where outcome predictability or safety cannot be guaranteed.
      - ENFORCE explicit, safe limitations rather than attempting complex speculative handling.
      - CLEARLY surface boundaries and constraints to the user up front.
      - BAD: Generating dynamic SQL updates for tables without primary keys and risking data corruption.
      - GOOD: Enforcing a read-only mode for tables without primary keys and warning the user.
    </GUIDELINES>
  </RULE>

  <RULE id="minimize_friction_loops" phase="process">
    <HEADER>TIMING: Process & Execution</HEADER>
    <TITLE>Minimize Friction Loops</TITLE>
    <DIRECTIVE>CRITICAL: Shorter feedback loops mean HIGHER reliability.</DIRECTIVE>
    <GUIDELINES>
      - ELIMINATE unnecessary intermediate handoffs, delays, or multi-step message round-trips.
      - KEEP processing, validation, and preview steps as direct and close to the source as possible.
      - ENSURE state transitions are immediate, transparent, and easy to trace.
      - BAD: Routing preview state through 4 asynchronous message hops across process boundaries.
      - GOOD: Computing preview state synchronously at the point of action.
    </GUIDELINES>
  </RULE>

</AGENT_RULES>