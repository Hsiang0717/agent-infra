# Harness Optimizer & Black-Box Optimization Protocol (AAEP Edition)

This protocol governs iterative code, algorithm, prompt, and test-harness optimization under the Autonomous Agent Engineering Protocol (AAEP). It enforces black-box dataset integrity, atomic rollback hygiene, and context firewall delegation.

---

## 1. Black-Box Dataset Isolation Guardrail (Zero-Leakage Boundary)

- **Absolute Read Freeze on Datasets**:
  - Agents are strictly forbidden from directly reading, inspecting, or searching the contents of sealed evaluation datasets, ground-truth label files, or holdout test cases (e.g. paths matching `dataset/`, `eval/`, `test_data/`, `*holdout*`, `*.jsonl`, `*.csv` containing validation pairs).
  - Never execute file dump commands (`cat`, `type`, `read_file`, `head`) or content search (`rg`, `grep`) targeting holdout evaluation data.
- **Sanitized Metric Interface**:
  - All feedback from the test environment must pass strictly through the evaluation runner's sanitized JSON output (`eval_runner`).
  - Runner outputs must report statistical metrics (`pass_rate`, `composite_loss`, `latency_ms`, failure category counts) and anonymized semantic gradients (e.g., error type, line coordinate, profiling bottleneck) without leaking expected answers or input payloads.
- **Zero Special-Casing (Anti-Overfitting)**:
  - Optimizations must represent authentic generalized logic or algorithmic improvements.
  - Hardcoding outputs, injecting `if input == 'x': return 'y'`, or pattern-matching specific test samples is strictly forbidden and constitutes optimization fraud.

---

## 2. Atomic Rollback & Checkpoint Protocol

- **Clean Baseline Tagging**:
  - Before modifying any codebase parameter, establish a baseline Git checkpoint (branch or tag `harness-baseline`).
- **Bounded Mutation (Learning Rate)**:
  - Each optimization epoch must respect the configured diff budget (e.g., max 1-2 functions or <= 80 diff lines per step). Never rewrite unrelated modules during an optimization step.
  - Frozen interfaces and public API contracts must remain strictly unmodified.
- **Zero-Tolerance Regression Rollback**:
  - If an epoch yields `new_loss >= best_loss` or causes functional regressions, immediately discard the uncommitted changes:
    ```powershell
    git checkout -- <modified_files>
    ```
  - Never allow regressed edits or syntax failures to compound into subsequent epochs.
- **Atomic Success Commit**:
  - When `new_loss < best_loss`, stage the winning diff and create an atomic Conventional Commit:
    ```powershell
    git commit -m "perf(harness): epoch <N> loss <best_loss> (<improvement_percent>%)"
    ```
  - Record the winning hypothesis in `.harness/training_log.jsonl`.

---

## 3. Two-Tier Orchestrator-Worker Protocol (Context Firewall)

To prevent catastrophic token debt and preserve context agility during multi-epoch runs:

- **Main Agent (Orchestrator)**:
  - **Owns Phase 0**: Conducts read-only pre-flight checks, formulates options, and explicitly aligns with the user on hyperparameters, loss weights, and runner architecture.
  - **Owns Verification & Closure (Phase 3)**: Reviews the final report, verifies the winning commit, and distills high-level playbook learnings to the user.
- **Subagent (Training Worker)**:
  - **Owns Multi-Epoch Training (Phase 2)**: When `max_epochs >= 3`, the Main Agent must delegate the iterative loop to a subagent (`self`) via `invoke_subagent`.
  - **Context Firewall**: All intermediate test failures, compiler traces, and rollback churn remain contained within the worker's sandboxed session.
  - **Subagent Contract**: The worker returns only a dense summary matrix upon completion (or early stop), reporting:
    1. Baseline vs. Best Checkpoint metrics (`composite_loss`, `pass_rate`, `latency`).
    2. Winning commits and modified files.
    3. Distilled rules added to `.harness/playbook.md`.
