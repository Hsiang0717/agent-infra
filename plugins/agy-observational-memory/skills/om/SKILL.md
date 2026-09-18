---
name: observational-memory
description: >-
  Autonomously record verified milestones, pin durable architectural invariants,
  prune superseded observations via om drop, or resolve 12-character hex memory tags [abcdef123456].
---

# Observational Memory (`om`)

Observational Memory provides deterministic dual-layer memory (Observations + Reflections) for Antigravity.
The latest folded memory projection is **automatically injected before every model call** via the `PreInvocation` hook, giving you instant situational awareness without manual lookup.

Your responsibility is to **autonomously maintain memory accuracy** across the task lifecycle.

---

## Proactive Autonomous Protocol

### 1. Milestone Settlement (`om record`)
When a feature is implemented, a bug fix is empirically verified with tests, or an architectural choice is finalized:
```bash
node bin/om.cjs record "<atomic_summary>" -r high
```
- `-r critical`: System-critical state shifts, workspace wipes, or breaking environmental changes.
- `-r high`: Verified milestones, architectural forks, or non-obvious bug fixes.
- `-r medium`: Routine intermediate progress.

### 2. Invariant Establishment (`om pin`)
When establishing durable cross-session constraints (repo conventions, framework rules, toolchain boundaries, or persistent preferences):
```bash
node bin/om.cjs pin "<durable_rule>"
```
*Pinned reflections are saved to `.agents/memory/reflections.json` and automatically persist across future sessions.*

### 3. Hex Tag Resolution (`om recall <id>`)
When encountering a 12-character hex tag (e.g. `[d4e5f6a1b2c3]`) in injected memory or dialogue and you need the exact historical timestamp, relevance rating, or full context:
```bash
node bin/om.cjs recall <id>
```

### 4. Autonomous Pruning & Compaction (`om drop`, `om compact`)
Memory hygiene is an ongoing autonomous duty:
- **Drop Noise**: When a bug is solved, a refuted hypothesis is discarded, or temporary steps are completed, immediately prune stale observations:
  ```bash
  node bin/om.cjs drop <id1> <id2> ...
  ```
- **Synthesize & Promote**: When active observations exceed ~15-20 items or reveal a durable pattern:
  1. Pin the unified rule: `node bin/om.cjs pin "<distilled_rule>"`.
  2. Drop the redundant raw observation IDs: `node bin/om.cjs drop <id1> <id2> ...`.
