---
name: ecl-bootstrap
description: "Initializes the External Cognitive Layer (ECL) by scanning project topology with ast-grep, constructing causal cluster skeletons, and enriching them with LLM domain summaries and rules."
usage: "/ecl-bootstrap [src_dir]"
---

# /ecl-bootstrap Command Handler

This command scans a target codebase, builds causal cluster skeletons, and enriches them into a single high-density `.agent/ecl.yaml` ledger.

---

## Execution Protocol

### Step 1: Pre-Flight Topology Scan
Run the bootstrap engine on the target source directory:
```powershell
uv run python "$env:USERPROFILE/.gemini/config/plugins/agy-ecl/scripts/ecl_bootstrap.py" --src <src_dir> --root .
```

### Step 2: Subtractive Semantic Enrichment Gate
1. Read the generated `.agent/ecl.yaml`.
2. Prune trivial/pure functions—keep ONLY clusters involving cross-module state couplings, timers, async channels, caches, or platform pitfalls.
3. For each remaining cluster containing `[TODO_LLM_SUMMARY]` or `[TODO_LLM_RULE]`:
   - Read the corresponding entrypoint and chain code slices (`read_file` with the exact line range).
   - Formulate the business intent (`summary`), downstream side-effects (`effect`), and non-negotiable defensive invariants (`rules`).
   - Surgically update `.agent/ecl.yaml` using `edit_file`.

### Step 3: Verification & Zero-Drift Closure
Run `ecl_sync.py` to ensure all `#L{start}-{end}` coordinates match disk state:
```powershell
uv run python "$env:USERPROFILE/.gemini/config/plugins/agy-ecl/scripts/ecl_sync.py" --root .
```
Report the established Causal Clusters to the user.
