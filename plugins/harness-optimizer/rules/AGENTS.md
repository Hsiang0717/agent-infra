# Harness Optimizer & Black-Box Optimization Protocol (AAEP Agent-Native Edition)

This protocol governs autonomous iterative code, algorithm, prompt, and test-harness optimization under the Autonomous Agent Engineering Protocol (AAEP). It enforces **Agent-Native end-to-end autonomy (minimal human intervention)**, black-box dataset integrity, mandatory probe/schema certification, relative-baseline tiered loss, holdout anti-overfitting, and clean atomic rollbacks.

---

## 0. Agent-Native Autonomy & Minimal Human Intervention Boundary

- **Autonomous End-to-End Execution by Default**:
  - When invoked, the Agent must autonomously audit the repository, detect toolchains/datasets/interfaces, synthesize the `.harness/` configuration and evaluation runner, certify the Probe Gate, run the training loop, and deliver the verified result **without halting for routine confirmation**.
  - Do NOT present multi-choice questionnaires for parameters (loss weights, epoch counts, mutation strategies) that can be empirically inferred from the codebase.
- **Strict Human Intervention Boundary (Ask ONLY for Missing Ground Truth)**:
  - Interrupt autonomous execution to ask the user **ONLY** when:
    1. **Missing Evaluation Oracle**: No test suite, benchmark, dataset, or verifiable objective criteria exist and none can be deterministically derived from project specifications.
    2. **External Blockers**: Required credentials, external endpoints, hardware devices, or user-supplied input assets are missing.
    3. **Irreconcilable Contract Conflict**: Optimizing the target requires breaking a public API contract and user intent is ambiguous.

---

## 1. Scaffolding Integrity & 5-Step Pre-Flight Probe Gate (Zero-Mutation Boundary)

- **Absolute Freeze Before Scaffold Certification**:
  - Agents are strictly forbidden from modifying, refactoring, or optimizing any project source code before the `.harness/` infrastructure is 100% synthesized and certified via the 5-Step Pre-Flight Gate.
- **Mandatory 5-Step Scaffolding Gate**:
  1. **Topic Branch**: Dedicated branch created off a clean working tree (`harness/<topic>`).
  2. **Synthesized Trifecta Scaffold**: Complete `.harness/` files (`config.json`, project-adapted `eval_runner.py` with `TEMPLATE_ADAPTED = True`, and `playbook.md`). Blindly copying unadapted boilerplate is blocked.
  3. **Mandatory 1-Sample Probe Run**: Execute `uv run python .harness/eval_runner.py --probe` to verify runtime execution, timeout safety, and JSON output on a single sample/smoke slice.
  4. **Strict Schema & Anti-Boilerplate Gate**: Validate output via `validate_metrics.py .harness/metrics.json`. Certification passes ONLY if exit code is `0` (`template_adapted == True`, `total_count > 0`, `frozen_violation == False`, and normalized `composite_loss` in `[0.0, 1.0]`).
  5. **Dual Baseline Epoch 0 Lock**: Execute both `uv run python .harness/eval_runner.py --epoch 0` (Train baseline) and `uv run python .harness/eval_runner.py --epoch 0 --holdout` (Holdout baseline) to lock `.harness/baseline.json` (`baseline_latency_ms`, `epoch_0_train_loss`, `epoch_0_holdout_loss`), and commit `.harness/` before any code mutation begins.

---

## 2. Black-Box Dataset Isolation & Holdout Generalization Gate (Anti-Overfitting)

- **Absolute Read Freeze on Holdout Datasets**:
  - Agents are strictly forbidden from reading, dumping, or searching the contents of sealed holdout datasets or ground-truth answer files (`*holdout*`, `eval/`, `test_data/`).
- **Holdout Set Blind Verification (Anti-Overfitting Gate)**:
  - The Holdout set remains blind during inner training epochs (`1..N`) and is evaluated only at Epoch 0 (baseline anchor) and at candidate winning checkpoints (Phase 3).
  - If Train Loss improves while Holdout Loss degrades by more than `max_holdout_degradation` (default `0.05` / `5%`) relative to `epoch_0_holdout_loss`, the runner flags `overfitting_detected: true` and the agent must revert to the best generalized checkpoint.
- **Stateless Purity & Cache Isolation Invariant**:
  - Every evaluation sample must run with isolated state or reset module-level caches between cases. Cross-sample state leakage is strictly forbidden.
- **Sanitized Metric Interface & Zero Special-Casing**:
  - All test feedback must flow through `.harness/metrics.json` without leaking raw holdout payloads.
  - Hardcoding outputs, Lookup-Table (LUT) memorization, or `if input == 'x': return 'y'` special-casing constitutes optimization fraud and is strictly forbidden.

---

## 3. Atomic Rollback, Noise Floor & Contract Protection Protocol

- **Bounded Mutation & Frozen Contract Guard**:
  - Each epoch mutation must stay strictly within `constraints.mutable_paths` and respect `learning_rate` (`max_lines_changed_per_epoch`, `max_files_touched_per_epoch`).
  - Any modification to `constraints.frozen_interfaces` sets `frozen_violation = True` and triggers an immediate Tier-1 hard gate (`composite_loss = 1.0`).
- **Statistical Noise Threshold (`min_improvement_delta`)**:
  - A candidate mutation is accepted ONLY if `new_loss <= best_loss - min_improvement_delta` (default `0.005`) and `frozen_violation == False`. Micro-fluctuations from OS scheduling noise must not be committed.
- **Zero-Residue Clean Rollback**:
  - Whenever an epoch fails to beat `best_loss - min_improvement_delta` or triggers a Tier-1 gate, immediately purge both modified and untracked files in mutable paths:
    ```powershell
    git checkout -- <mutable_paths>
    git clean -fd <mutable_paths>
    ```
  - Record the failed hypothesis in `.harness/playbook.md` under Regression Traps.
- **Atomic Success Commit**:
  - When a mutation succeeds, stage only `mutable_paths` and `.harness/` logs and create an atomic Conventional Commit:
    ```powershell
    git add <mutable_paths> .harness/playbook.md .harness/training_log.jsonl
    git commit -m "perf(harness): epoch <E> loss <new_loss> (<improvement_pct>%)"
    ```

---

## 4. Two-Tier Orchestrator-Worker Protocol (Context Firewall)

- **Main Agent (Orchestrator)**:
  - **Owns Phase 0 & Phase 1**: Conducts autonomous pre-flight discovery, synthesizes `.harness/`, certifies the 5-Step Probe Gate, and locks Epoch 0 Train + Holdout baselines.
  - **Owns Phase 3 Closure**: Runs Holdout Generalization verification, inspects the Artifact Gallery (`--gallery`), distills `.harness/playbook.md`, and presents the final Executive Report.
- **Subagent (Training Worker)**:
  - **Owns Phase 2 (`max_epochs >= 3`)**: Main Agent delegates the multi-epoch loop via `invoke_subagent(TypeName: "harness-worker", Role: "Harness Training Worker")` (bundled in `agents/harness-worker.md`, or `self`) on branch `harness/<topic>`.
  - **Context Firewall**: Intermediate test traces and rollback churn stay inside the subagent session; only the structured summary matrix is returned.

---

## 5. Universal Artifact Diff & Multimodal Verification Protocol

- **Artifact Paired Output**:
  - For non-scalar outputs (TikZ/SVG, UI layouts, images, audio, formatted documents), `eval_runner.py` must write paired artifact records to `.harness/artifacts_manifest.json` (`id`, `status`, `ground_truth`, `actual_output`).
- **Empirical Gallery Gate**:
  - Before final closure on visual/perceptual tasks, execute `uv run python .harness/eval_runner.py --gallery` to generate `.harness/gallery.html` and empirically inspect rendered outputs via `read_file` or screen capture.
