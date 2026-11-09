---
name: ecl-learn
description: "Distills lessons learned from recent bug fixes or refactors and atomically records them as non-negotiable architectural invariants in the relevant Causal Cluster rules."
usage: "/ecl-learn [incident_description_or_issue_ref]"
---

# /ecl-learn Command Handler (Institutional Memory Loop)

The `/ecl-learn` command permanently anchors defensive invariants, platform quirks, and bug fix lessons into the single-file External Cognitive Layer (`.agent/ecl.yaml`).

---

## Execution Protocol

> **Execution Boundary (Stay in Active Agent — Do NOT Spawn Subagent)**:
> Execute `/ecl-learn` directly inside the agent session that just diagnosed and fixed the issue. Because the causal chain and root cause are already in active working memory, delegating to a fresh subagent wastes tokens re-reading diffs and loses diagnostic nuance.

### Step 1: Diff & Blast Radius Audit
Inspect the recent commit or active working tree modifications:
```powershell
git diff HEAD~1 --stat
```
or for uncommitted work:
```powershell
git diff --stat
```
Identify the modified files, functions/components, and the root cause of the fix.

---

### Step 2: Cluster Resolution
Check `.agent/ecl.yaml` to locate the `cluster_id` that governs the modified file (or determine if a new cluster entry should be added):
```powershell
rg -n -C 6 "<modified_file>" .agent/ecl.yaml
```

---

### Step 3: High-Density Causal & Invariant Distillation
Formulate an ultra-dense, imperative architectural rule that makes future downstream modifications make sense:
- **What to Capture**: Causal relationships (`A 觸發 B 導致 C`), business logic constraints, decoupled dependency chains, and strict event/temporal ordering (`必須先 X 才能 Y，否則 Z 被覆寫`).
- **Format**: `<Event Order / Causal Constraint>（<Business Logic / Decoupled Dependency / Pitfall>）`
- **Example**: `用戶拖曳進度條（user_scrub）期間，必須嚴格阻斷 100ms Polling Timer 覆寫 UI 滑桿位置以防跳針`
- **Anti-Pattern (Not Code Comments)**: Never write traditional code comments or literal explanations of what a function does locally. Record ONLY cross-module causal links, state-ordering invariants, and hidden couplings.

---

### Step 4: Atomic Rule Injection
Surgically append the distilled rule into the `rules:` section of the target Causal Cluster in `.agent/ecl.yaml` using `edit_file`.

---

### Step 5: Coordinate Verification & Closure
Run the synchronizer to guarantee `#L{start}-{end}` coordinate integrity:
```powershell
uv run python "$env:USERPROFILE/.gemini/config/plugins/agy-ecl/scripts/ecl_sync.py" --root .
```
Stage the updated file:
```powershell
git add .agent/ecl.yaml
```
Output a confirmation summary citing the updated Cluster ID and the newly active defensive rule.
