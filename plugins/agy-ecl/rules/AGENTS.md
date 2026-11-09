# External Cognitive Layer (ECL) — Subtractive Invariant Protocol

ECL (`.agent/ecl.yaml`) is a **Cross-Module Causal & Invariant Shield**, NOT a replacement for `rg` code search.

## 1. When NOT to Use ECL (Zero Discovery Tax)
- **Read-Only Q&A, Value/Config Lookups, & Symbol Search**:
  Use native `rg -n "<pattern>" src/` and `read_file` directly.
  **NEVER read `.agent/ecl.yaml` when simply answering questions about constants, configs, or locating definitions.**

## 2. Automatic Hook Injection & Pre-Edit Gate
- **Auto-Injected `[ECL Shield]`**: When `read_file` accesses a file governed by `.agent/ecl.yaml`, the runtime hook automatically injects `[ECL Shield (<cluster>): ... | Rules: ...]`. Strictly obey all cited rules and downstream `chain` side-effects when editing.
- **Cross-Module / Async Debugging**: If debugging race conditions or modifying uninspected modules where `.agent/ecl.yaml` exists, run `rg -n -C 8 "<module_or_symbol>" .agent/ecl.yaml` before `edit_file` to verify `chain:` and `rules:`.

## 3. Subtractive Distillation Rules (`/ecl-learn` & `/ecl-bootstrap`)
- **Single-File Density**: Store all causal clusters in a single compact `.agent/ecl.yaml` (no multi-file `INDEX.yaml` directory trees).
- **Zero Obvious Code Duplication**: Never create clusters for self-explanatory pure functions or simple constants. Record ONLY invisible cross-module couplings, temporal race conditions, and platform pitfalls.
