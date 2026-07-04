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
2. **[CONSTRAINT_CHECK]**: List all immediate architectural constraints, safety guidelines (e.g. powershell-helper), and the Minimum Viable Boundary (MVB).
3. **[CRITICAL_HYPOTHESIS]**: Draft the proposed implementation logic and identify potential edge cases or failure modes *before* generating the actual solution.
4. **[REASONING_REFINEMENT]**: Perform a virtual dry-run of the solution to detect logical gaps.

## Multi-Pass Self-Correction (Reflection Loop)
For highly complex implementations or when potential risks are flagged in `[CONSTRAINT_CHECK]`:
- **Do not output the final code immediately.**
- Trigger a **Self-Correction pass** by outputting a **[SELF_CRITIQUE]** block:
  1. Review your initial `[CRITICAL_HYPOTHESIS]` and draft implementation.
  2. Identify at least two potential failure modes, security bypasses, or performance bottlenecks.
  3. Formulate a refined implementation hypothesis (`[REFINED_HYPOTHESIS]`) before generating the final codebase.

*Do not skip these steps. Writing these out explicitly enforces reasoning consistency even in low-effort models.*
