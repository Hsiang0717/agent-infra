---
name: adaptive-thinker
description: Enforces structured external cognitive scaffolding to enhance logical reasoning in low-effort environments.
---

# Adaptive Thinker Skill

## Context & Purpose
When running under low-effort settings or resource-constrained models, use this skill to construct a visible, structured thought process in the output stream before generating code or actions. This acts as a cognitive scaffold.

## Structured Thinking Protocol
You MUST output an explicit markdown block at the beginning of your response containing:

1. **[CONTEXT_SYNC]**: Summarize the current session-state milestone and open problems related to the query.  
   *Tip: Begin the block with `[CONTEXT_SYNC]` on its own line, then optionally use a `---` line to separate the header from the content for easier machine parsing.*

2. **[CONSTRAINT_CHECK]**: List all immediate architectural constraints, safety guidelines (e.g. powershell-helper), and the Minimum Viable Boundary (MVB).  
   *Tip: Use a `---` line after the heading to delimit the section.*

3. **[CRITICAL_HYPOTHESIS]**: Draft the proposed implementation logic and identify potential edge cases or failure modes *before* generating the actual solution.  
   *Tip: Enclose your draft and bullet‑pointed risks between `---` lines.*

4. **[REASONING_REFINEMENT]**: Perform a virtual dry‑run of the solution to detect logical gaps.  
   *Tip: Again, consider surrounding the analysis with `---` lines.*

## Trigger Conditions (Configurable)
The host agent should invoke this skill when any of the following default triggers are met, unless overridden by context‑specific rules:

- **File system modifications** – delete, write, or rename operations outside a safe sandbox.
- **Credential or secret access** – reading environment variables, vaults, or plain‑text passwords.
- **System‑level changes** – registry edits, service installations, driver loading.
- **Long chains** – three or more sequential operations that depend on each other's output.
- **Security‑sensitive patterns** – commands matching regexes such as `rm\s+-Recurse`, `ConvertTo‑SecureString.*-Key`, `Invoke‑Expression`.

A host agent may provide an `OVERRIDE_TRIGGERS` context block (e.g., from a policy file) to replace or augment the list above.

## Multi‑Pass Self‑Correction (Reflection Loop)
For highly complex implementations **or** when potential risks are flagged in `[CONSTRAINT_CHECK]`:
- **Do not output the final code immediately.**
- Trigger a **Self‑Correction pass** by outputting a **[SELF_CRITIQUE]** block:
  1. Review your initial `[CRITICAL_HYPOTHESIS]` and draft implementation.
  2. Identify **at least two** distinct failure modes, security bypasses, or performance bottlenecks.
  3. Formulate a refined implementation hypothesis (`[REFINED_HYPOTHESIS]`) before generating the final codebase.
- Optionally, repeat the loop if the refined hypothesis still raises concerns in `[CONSTRAINT_CHECK]`.

*Do not skip these steps. Writing these out explicitly enforces reasoning consistency even in low‑effort models.*
