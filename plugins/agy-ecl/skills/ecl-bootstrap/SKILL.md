---
name: ecl-bootstrap
description: "Initializes the External Cognitive Layer (ECL) by scanning project topology with ast-grep, constructing causal cluster skeletons, and enriching them with LLM domain summaries and rules."
usage: "/ecl-bootstrap [src_dir]"
---

# /ecl-bootstrap Command Handler

This command scans a target codebase, builds causal cluster skeletons, and enriches them into a single high-density `.agent/ecl.yaml` ledger.

---

## Execution Protocol

### Step 0: Mandatory Context Firewall (`self` Subagent Delegation)
- **If running in the Main Agent**: Do NOT scan and read 10+ source files in the primary conversation context. Immediately delegate this entire workflow via `invoke_subagent(TypeName="self", Role="ECL Topology & Causal Architect", Prompt="...")` and return the subagent's final cluster summary to the user.
- **If already running inside the delegated Subagent**: Proceed directly to Step 1.

### Step 1: Pre-Flight Topology Scan
Run the bootstrap engine on the target source directory:
```powershell
uv run python "$env:USERPROFILE/.gemini/config/plugins/agy-ecl/scripts/ecl_bootstrap.py" --src <src_dir> --root .
```

### Step 2: Subtractive Causal & Business Logic Enrichment Gate
1. Read the generated `.agent/ecl.yaml`.
2. **Aggressive Pruning (No Code Explanations)**: Delete trivial, local, or self-explanatory functions. ECL is **NOT traditional code comments or literal code documentation**—never restate what a single function visibly does on the surface.
3. Keep ONLY clusters where **logical decoupling hides behavioral coupling** (e.g., `Producer ➔ Event Bus / IPC / Shared Store / Timer ➔ Consumer`), and enrich every `[TODO_LLM_SUMMARY]` / `[TODO_LLM_RULE]` with:
   - **Business Logic & Intent (`summary`)**: Why this workflow exists at the domain level.
   - **Causal Chain & Hidden Dependencies (`chain` / `effect`)**: Which decoupled downstream modules, caches, or UI states react when this node fires.
   - **Event Ordering & Invariants (`rules`)**: Non-negotiable temporal sequence, state-lock priority, and race-condition guards so any subsequent agent action makes systemic sense.
   - Surgically update `.agent/ecl.yaml` using `edit_file`.

### Step 3: Verification & Zero-Drift Closure
Run `ecl_sync.py` to ensure all `#L{start}-{end}` coordinates match disk state:
```powershell
uv run python "$env:USERPROFILE/.gemini/config/plugins/agy-ecl/scripts/ecl_sync.py" --root .
```
Report the established Causal Clusters to the user.
