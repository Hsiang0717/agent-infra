# Harness Optimizer & Black-Box Optimization Protocol (AAEP Edition)

This protocol governs iterative code, algorithm, prompt, and test-harness optimization under the Autonomous Agent Engineering Protocol (AAEP). It enforces black-box dataset integrity, scaffolding foolproofing, holdout anti-overfitting, atomic rollback hygiene, and context firewall delegation.

---

## 1. Scaffolding Integrity & Pre-Flight Probe Gate (Zero-Mutation Boundary)

- **Absolute Freeze Before Scaffold Certification**:
  - Agents are strictly forbidden from modifying, refactoring, or optimizing any project code before the `.harness/` infrastructure is 100% complete, verified, and certified via the 4-step Pre-Flight Gate.
- **Mandatory 4-Step Scaffolding Gate**:
  1. **Topic Branch**: Dedicated branch created off clean state (`harness/<topic>`).
  2. **Trifecta Scaffold**: Complete `.harness/` configuration files present (`config.json`, `eval_runner.py`, `playbook.md`).
  3. **Mandatory 1-Sample Probe Run**: Execute runner with `--probe` (1 single test case) to prove end-to-end execution, timeout safety, and clean JSON serialization without running the full battery.
  4. **Strict Schema Gate**: Validate output strictly via `validate_metrics.py .harness/metrics.json`. The gate passes ONLY if validation exits with code 0 and zero missing fields (`composite_loss`, `pass_rate`, `latency_ms`, `diff_lines`).
- **Baseline Epoch 0 Lock**:
  - Full dataset Epoch 0 baseline must be recorded and committed before any mutation turn begins.

---

## 2. Black-Box Dataset Isolation & Holdout Generalization Gate (Anti-Overfitting)

- **Absolute Read Freeze on Datasets**:
  - Agents are strictly forbidden from directly reading, inspecting, or searching the contents of sealed evaluation datasets, ground-truth label files, or holdout test cases (e.g. paths matching `dataset/`, `eval/`, `test_data/`, `*holdout*`, `*.jsonl`, `*.csv` containing validation pairs).
  - Never execute file dump commands (`cat`, `type`, `read_file`, `head`) or content search (`rg`, `grep`) targeting holdout evaluation data.
- **Holdout Set Blind Verification (Anti-Overfitting Gate)**:
  - Dataset must maintain a distinct Train / Holdout split.
  - The Holdout set remains strictly blind during inner training epochs and is probed only at Epoch 0 and candidate winning checkpoints.
  - If Train Loss improves while Holdout Loss regresses by > 5%, the optimizer must flag `OVERFITTING_DETECTED` and rollback to the best generalized checkpoint.
- **Stateless Purity & Cache Isolation Invariant**:
  - Every evaluation case must run with isolated memory/state or reset module-level caches between runs. Cross-sample state accumulation or leakage across cases is strictly forbidden.
- **Sanitized Metric Interface**:
  - All feedback from the test environment must pass strictly through the evaluation runner's sanitized JSON output (`eval_runner`).
  - Runner outputs must report statistical metrics (`pass_rate`, `composite_loss`, `latency_ms`, failure category counts) and anonymized semantic gradients without leaking expected answers or input payloads.
- **Zero Special-Casing (Anti-Cheating)**:
  - Optimizations must represent authentic generalized logic or algorithmic improvements.
  - Hardcoding outputs, injecting `if input == 'x': return 'y'`, or pattern-matching specific test samples is strictly forbidden and constitutes optimization fraud.

---

## 3. Atomic Rollback & Checkpoint Protocol

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

## 4. Two-Tier Orchestrator-Worker Protocol (Context Firewall)

To prevent catastrophic token debt and preserve context agility during multi-epoch runs:

- **Main Agent (Orchestrator)**:
  - **Owns Phase 0 & Phase 1**: Conducts read-only pre-flight checks, aligns parameters, builds the scaffold, certifies the Probe Gate, and commits Epoch 0 baseline.
  - **Owns Verification & Closure (Phase 3)**: Reviews the final report, verifies the winning commit, audits visual/artifact gallery, and distills high-level playbook learnings to the user.
- **Subagent (Training Worker)**:
  - **Owns Multi-Epoch Training (Phase 2)**: When `max_epochs >= 3`, the Main Agent must delegate the iterative loop to a subagent (`self`) via `invoke_subagent`.
  - **Context Firewall**: All intermediate test failures, compiler traces, and rollback churn remain contained within the worker's sandboxed session.
  - **Subagent Contract**: The worker returns only a dense summary matrix upon completion (or early stop), reporting:
    1. Baseline vs. Best Checkpoint metrics (`composite_loss`, `pass_rate`, `latency`).
    2. Winning commits and modified files.
    3. Distilled rules added to `.harness/playbook.md`.

---

## 5. Universal Artifact Diff & Multimodal Verification Protocol

- **Artifact Paired Output**:
  - For projects generating non-scalar outputs (SVG, UI layouts, images, audio, formatted text, ASTs), the evaluation runner must pair each sample with reference artifacts (`ground_truth` vs `actual_output`).
- **Interactive Gallery Verification Gate**:
  - Unit tests or compiler passes (Exit code 0) do not constitute completion for perceptual/artifact tasks.
  - Evaluation harnesses must output a lightweight, self-contained HTML gallery enabling side-by-side verification across all cases.
  - Final acceptance requires empirical visual or multimodal verification via `read_file` or screenshot inspection before declaring the optimization done.
