# agy-ecl: External Cognitive Layer Plugin

`agy-ecl` is an agent-native cognitive infrastructure plugin that shifts discovery and invariant resolution from read-time brute-force search to write-time causal clusters.

---

## Features

1. **Causal Closure Modeling**: Groups code by operational side-effects rather than directory mirroring.
2. **Deterministic AST Synchronization**: Uses `ast-grep` to keep `#L{start}-{end}` coordinates synchronized with 0 Token cost.
3. **Three-Tier Progressive Disclosure**: Enforces L0 (Index) ➔ L1 (Domain) ➔ L2 (Surgical Slice) reading protocol to eliminate context pollution.
4. **Institutional Memory Loop**: Anchors defensive invariants and lessons learned directly inside the relevant code clusters.

---

## Directory Structure

```
.agent/semantics/
├── INDEX.yaml            # L0 Global Intent Routing (<200 tokens)
├── player.yaml           # L1 Domain Causal Graph
└── billing.yaml          # L1 Domain Causal Graph
```

---

## Commands & Skills

- `/ecl-bootstrap [src_dir]`: Scans codebase and initializes `.agent/semantics/` with LLM-enriched causal clusters.
- `/ecl-sync`: Recalibrates physical line numbers across all YAML clusters using `ast-grep`.
- `/ecl-learn [incident_summary]`: Distills bug fix lessons into non-negotiable architectural invariants in relevant cluster rules.
