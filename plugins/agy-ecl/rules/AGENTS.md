# External Cognitive Layer (ECL) — Subtractive Invariant Protocol

ECL (`.agent/ecl.yaml`) is a **Cross-Module Causal & Invariant Shield**, NOT a replacement for `rg` code search.

## 1. When NOT to Use ECL (Zero Discovery Tax)
- **Read-Only Q&A, Value/Config Lookups, & Symbol Search**:
  Use native `rg -n "<pattern>" src/` and `read_file` directly.
  **NEVER read `.agent/ecl.yaml` when simply answering questions about constants, configs, or locating definitions.**

## 2. Automatic Hook Injection, Empirical Precedence & Closure Maintenance
- **Empirical Truth & User Intent First**: Explicit user refactor/architecture decisions and live source code **always override** stale ECL entries. Never block an intentional refactor or revert new code to match an obsolete rule; treat a once-injected cluster during overhaul as a risk checklist (preserve underlying business invariants in the new design).
- **Tiered Hook Signals (Inject-Once per Working Window)**:
  - **`[ECL Shield (...)]`** (Fresh): Strictly obey cited rules and downstream `chain` side-effects unless contradicted by live code or explicit user refactor intent.
  - **`[ECL Advisory (...)]`** (Drifted since last sync): Verify cited rules against live code before applying; do not follow blindly if implementation has shifted.
  - **`[ECL Post-Edit (...)]` / `[ECL Cleanup (...)]`**: After modifying, renaming, or removing tracked symbols/files, autonomously update or prune `.agent/ecl.yaml` (or re-run `/ecl-bootstrap` after a full architectural reshuffle) and run `/ecl-sync` before task closure.
- **Cross-Module / Async Debugging**: If debugging race conditions or modifying uninspected modules where `.agent/ecl.yaml` exists, run `rg -n -C 8 "<module_or_symbol>" .agent/ecl.yaml` before `edit_file` to verify `chain:` and `rules:`.

## 3. Subtractive Causal Ontology & Subagent Boundary (`/ecl-learn` & `/ecl-bootstrap`)
- **Single-File Causal Ledger**: Store all clusters in one compact `.agent/ecl.yaml`. Capture ONLY **business logic intent**, **causal chains (`Producer ➔ Store/IPC/Event ➔ Consumer`)**, **decoupled cross-module dependencies**, and **event/temporal ordering** so subsequent actions make systemic sense.
- **Not Traditional Code Comments**: NEVER explain literal code syntax, obvious pure functions, or simple constants—record only invisible couplings and invariants that cannot be seen by reading a single file in isolation.
- **Subagent Delegation Boundary**: `/ecl-bootstrap` MUST be delegated to a `self` subagent (Context Firewall against multi-file scan pollution); `/ecl-learn` and `/ecl-sync` MUST run directly in the active agent where debugging context already resides.
