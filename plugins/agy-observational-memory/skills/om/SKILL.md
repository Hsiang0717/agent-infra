---
name: observational-memory
description: >-
  Autonomously record verified milestones, manage current focus/next steps, settle atomic checkpoints,
  pin durable architectural invariants (with auto-deduplication), and resolve 12-character hex memory tags [abcdef123456].
---

# Observational Memory (`om`) 2.0

Observational Memory provides deterministic situational awareness with **Current Focus state machines**, **Reflections (durable invariants with auto-deduplication)**, and **Observations (verifiable event milestones)** for Antigravity.
The latest folded memory projection is **automatically injected before every model call** via the `PreInvocation` hook, giving you instant context without manual lookup.

Your responsibility is to **autonomously maintain memory accuracy and hygiene** across the task lifecycle.

---

## ⚡ Autonomous Trigger Rules (When to Call `om`)

| Event / Scenario | Trigger Condition | Recommended Command |
| :--- | :--- | :--- |
| **Milestone / Verification Pass** | Tests pass, feature complete, or bug verified | `node bin/om.cjs checkpoint "<milestone>" --next "<next_action>"` |
| **Task / Hypothesis Superseded** | A previous task is resolved, replaced, or refuted | Pass `--resolves <id1,id2>` or run `node bin/om.cjs drop <id1> <id2>` |
| **Cross-Session Invariant Finalized** | Repo convention, tool constraint, or durable preference established | `node bin/om.cjs pin "<durable_rule>"` (auto-deduplicated) |
| **Rule Deprecated or Modified** | A convention is changed or no longer applies | `node bin/om.cjs pin "<rule>" --replace <id>` or `node bin/om.cjs unpin <id>` |
| **Session Handoff / Plan Switch** | Starting a new goal, pivoting, or pausing work | `node bin/om.cjs focus "<goal>" --next "<action>"` |

---

## 🛠️ CLI Reference & Examples

### 1. Atomic Checkpoint (`om checkpoint`) - Preferred for Milestones
Settles a verified milestone, updates the active Focus & Next Action, and automatically resolves prior observation IDs in a single atomic operation:
```bash
node bin/om.cjs checkpoint "Database schema migration applied and verified" --next "Implement CRUD endpoints" --resolves d4e5f6a1b2c3
```

### 2. Task Focus Management (`om focus`)
Directly update or clear the active target and next action to maintain 100% cold-start certainty across session resets:
```bash
node bin/om.cjs focus "Implementing AST validator" --next "Run property-based test suite"
node bin/om.cjs focus --clear
```

### 3. Progress Recording (`om record`)
Record an atomic observation, optionally resolving superseded IDs:
```bash
node bin/om.cjs record "Optimized SQLite queries with indexed WAL mode" -r high --resolves 23109baf1fbe,023ff2c2ab32
```
- `-r critical`: System-critical state shifts, workspace wipes, or breaking environmental changes.
- `-r high`: Verified milestones, architectural forks, or non-obvious bug fixes.
- `-r medium`: Routine intermediate progress (default).

### 4. Invariant Establishment & Lifecycle (`om pin`, `om unpin`)
Establish durable cross-session constraints (saved to `.agents/memory/reflections.json`):
```bash
# Auto-deduplicates: merges/updates existing reflection if similarity >= 80%
node bin/om.cjs pin "Repository mandates Bun runtime and strict TypeScript"

# Explicit replacement of a specific rule ID
node bin/om.cjs pin "Repository mandates Node 22+ with pnpm" --replace a1b2c3d4e5f6

# Remove deprecated reflections
node bin/om.cjs unpin a1b2c3d4e5f6
```

### 5. Hex Tag Resolution (`om recall <id>`)
When encountering a 12-character hex tag (e.g. `[d4e5f6a1b2c3]`) in injected memory and you need the full timestamp, relevance rating, or exact source context:
```bash
node bin/om.cjs recall d4e5f6a1b2c3
```

### 6. Autonomous Pruning (`om drop`)
Prune obsolete raw observations from active memory:
```bash
node bin/om.cjs drop <id1> <id2> ...
```
