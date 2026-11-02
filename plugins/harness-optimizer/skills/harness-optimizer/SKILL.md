---
name: harness-optimizer
description: Agent-native autonomous recursive code, algorithm, prompt, and test-harness optimization framework inspired by Google Research Harness2. Use when optimizing or evolving code, algorithms, LLM prompts, or evaluation harnesses via autonomous pre-flight synthesis, mandatory probe/schema certification, relative-baseline tiered loss, holdout anti-overfitting gate, and two-tier subagent context firewall execution with minimal human intervention.
---
# Harness Optimizer (Agent-Native Autonomous Edition)

An autonomous, language-agnostic recursive optimization framework inspired by Google Research Harness2. Built for **Agent-Native execution with minimal human intervention**, it autonomously audits the target repository, synthesizes a project-specific evaluation harness, certifies scaffolding via a strict Probe & Schema Gate, executes bounded multi-epoch mutations inside a subagent context firewall, and verifies generalization against a blind Holdout set.

---

## Architecture: Two-Tier Orchestrator-Worker Pattern

To protect the main conversation from token exhaustion during multi-epoch runs, the framework enforces a **Two-Tier Architecture**:

```text
User (Minimal Intervention: Only provides missing oracle/assets if blocked)
  │
  ▼
Main Agent (Orchestrator)
  ├── Phase 0: Autonomous Pre-Flight Discovery & Strategy Synthesis (Zero-Friction)
  ├── Phase 1: 5-Step Scaffolding, Probe Certification & Dual Baseline Lock (Train + Holdout)
  ├── Phase 2 (Delegated): invoke_subagent(TypeName: "harness-worker", Role: "Harness Training Worker")
  │     └── [Context Firewall]: Single-Pass Epochs 1..N (Mutate -> Eval -> Commit / Clean-Rollback)
  └── Phase 3: Blind Holdout Verification, Artifact Gallery Audit & Playbook Distillation
```

- **Main Agent (Orchestrator)**: Autonomously audits the workspace, synthesizes `.harness/`, certifies the Probe Gate, delegates multi-epoch training, runs final Holdout & Gallery checks, and reports results. Pauses to ask the human **only** if critical information (ground-truth oracle, missing credentials, or ambiguous breaking changes) cannot be derived from the workspace.
- **Subagent (Training Worker)**: Executes automated multi-epoch training loops (`max_epochs >= 3`) inside an isolated session on branch `harness/<topic>`, absorbing compiler traces and failed mutations without polluting the main context.

---

## Phase 0: Autonomous Pre-Flight Audit & Strategy Synthesis (Zero-Friction Gate)

Do **not** interrupt the user with multi-choice questionnaires when the repository already contains sufficient context. Perform a fast, read-only empirical audit and autonomously select the optimal configuration:

### Step 1: Automated Read-Only Audit
1. **Git State Check**: Inspect `git status -s` and `git branch --show-current`.
2. **Dataset & Split Discovery**: Locate existing tests and datasets (`tests/`, `eval/`, `dataset/`, `*holdout*`). Partition or designate `probe` (1 fast sample), `train` (iterative optimization), and `holdout` (sealed generalization check).
3. **Toolchain & Runtime Detection**: Inspect manifests (`pyproject.toml`, `package.json`, `Cargo.toml`, `go.mod`) to identify test runners and compilers.
4. **Boundary Classification**: Identify `mutable_paths` (target modules to optimize) and `frozen_interfaces` (public APIs, CLI entrypoints, and test files that must never be modified).

### Step 2: Autonomous Strategy Selection (Domain Matrix)
Select the domain-appropriate loss weights and mutation strategy automatically based on task characteristics:

| Task Domain | Loss Weights (`fail` / `latency` / `diff`) | Mutation Strategy |
| :--- | :--- | :--- |
| **Correctness & Bug Fixing** | `0.75` / `0.15` / `0.10` | **Greedy Hill-Climbing + Reflexion**: Single targeted fix per epoch; shift approach after 2 stalls. |
| **Algorithm & Latency Tuning** | `0.40` / `0.50` / `0.10` | **Two-Stage Curriculum**: Lock 100% pass rate first, then optimize relative latency against Epoch 0 baseline. |
| **Visual / TikZ / UI / Generative** | `0.70` / `0.15` / `0.15` | **Multimodal Artifact Reflection**: Render output -> Side-by-side critique vs ground truth -> Distill geometric/layout invariants. |
| **Real-Time / Streaming / Systems** | `0.70` / `0.20` / `0.10` | **Invariant-Guided Mutation**: Optimize buffer sizing, batching, and lock contention under strict timeout guards. |

> [!IMPORTANT]
> **Minimal Human Intervention Rule**:
> Proceed **immediately and autonomously to Phase 1** unless you hit a hard blocker:
> 1. **Missing Oracle**: No tests, metrics, or ground-truth fixtures exist and none can be synthesized from specifications.
> 2. **Missing Secrets/Assets**: External API keys, databases, or user files are absent.
> 3. **Ambiguous Breaking Change**: The goal cannot be achieved without altering a public interface contract.
> *Only in those 3 cases*, ask a concise, targeted question to obtain the missing information.

> [!CAUTION]
> **Dynamic Algorithm Synthesis Principle**:
> Files in `templates/` are structural archetypes, **NOT static boilerplate**. In Phase 1, you must synthesize `execute_suite(split, timeout_sec)` inside `.harness/eval_runner.py` to execute the project's actual runtime, parse real sample counts, and set `TEMPLATE_ADAPTED = True`.

---

## Phase 1: Bootstrap & 5-Step Scaffolding Certification Gate

Follow this exact 5-step sequence. **Modifying project source code is strictly forbidden until Step 4 exits with code 0 and Step 5 locks the baseline:**

1. **Create Dedicated Topic Branch**:
   ```powershell
   git checkout -b harness/<topic>
   ```
2. **Synthesize the `.harness/` Trifecta**:
   - Adapt templates from `<skill_dir>/templates/`:
     - `.harness/config.json`: Configure `mutable_paths`, `frozen_interfaces`, `loss_weights`, `min_improvement_delta` (`0.005`), and `max_holdout_degradation` (`0.05`).
     - `.harness/eval_runner.py`: Implement `execute_suite(split, timeout_sec)` for `'probe'`, `'train'`, and `'holdout'`, then set `TEMPLATE_ADAPTED = True`.
     - `.harness/playbook.md`: Record frozen contracts and initial architectural invariants.
     - `.harness/training_log.jsonl`: Create empty append-only log file.
3. **Mandatory 1-Sample Probe Run (`--probe`)**:
   ```powershell
   uv run python .harness/eval_runner.py --probe
   ```
   Verifies end-to-end execution, timeout handling, and metric serialization in seconds.
4. **Mandatory Schema & Anti-Boilerplate Certification Gate (Blocking Gate)**:
   ```powershell
   uv run python <skill_dir>/scripts/validate_metrics.py .harness/metrics.json
   ```
   - **Hard Gate**: Validates that `template_adapted is True`, `total_count > 0`, `passed_count + failed_count == total_count`, `frozen_violation is False`, and `composite_loss` is normalized in `[0.0, 1.0]`. If this command exits non-zero, you **must** fix `.harness/eval_runner.py` before touching any project code.
5. **Dual Baseline Lock (Epoch 0 Train + Holdout) & Commit**:
   - Run full Train baseline and Holdout baseline so `.harness/baseline.json` records `baseline_latency_ms`, `epoch_0_train_loss`, and `epoch_0_holdout_loss`:
     ```powershell
     uv run python .harness/eval_runner.py --epoch 0
     uv run python .harness/eval_runner.py --epoch 0 --holdout
     ```
   - Commit the certified harness:
     ```powershell
     git add .harness/ && git commit -m "chore(harness): certify evaluation harness and lock epoch 0 train/holdout baselines"
     ```

---

## Phase 2: Single-Pass Task-Level Training Loop (Context Firewall)

### Delegating to Worker Subagent (`max_epochs >= 3`)
Invoke the bundled `harness-worker` custom subagent (defined in `agents/harness-worker.md`, or fallback to `self`) to shield the primary context window:

```markdown
invoke_subagent(
  TypeName: "harness-worker",
  Role: "Harness Training Worker",
  Prompt: "Execute autonomous Harness training loop on branch harness/<topic>. Config: .harness/config.json. Baseline metrics: .harness/baseline.json (initial best_loss: <epoch_0_train_loss>). Execute Epochs 1 to <max_epochs> using the Single-Pass Epoch Cycle (Backward Gradient Analysis -> Bounded Mutation in mutable_paths -> Forward Eval -> Atomic Commit or Clean Rollback). Enforce min_improvement_delta and never touch frozen_interfaces or read holdout datasets. Return a dense Markdown summary reporting: (1) Epoch 0 vs Best Train metrics, (2) Commit log, (3) Distilled heuristics in .harness/playbook.md."
)
```

### Single-Pass Inner Epoch Cycle (Epochs `E = 1 .. max_epochs`)
Because Epoch `E - 1` (or Epoch `0`) already produced `best_loss` and `sanitized_gradients`, each epoch runs the evaluator **only once**:

1. **Backward Pass (Semantic Gradient Analysis)**:
   - Inspect `sanitized_gradients` from the latest `.harness/metrics.json` and review `.harness/playbook.md` to avoid known regression traps.
2. **Optimizer Step (Bounded Mutation)**:
   - Formulate a concrete hypothesis and apply surgical edits via `edit_file` strictly within `constraints.mutable_paths` and `learning_rate` limits (`max_lines_changed_per_epoch`, `max_files_touched_per_epoch`).
   - **Never modify `frozen_interfaces` or inspect holdout dataset files.**
3. **Forward Evaluation & Validation**:
   - Execute the runner once on the training split and validate schema:
     ```powershell
     uv run python .harness/eval_runner.py --epoch <E>
     uv run python <skill_dir>/scripts/validate_metrics.py .harness/metrics.json
     ```
4. **Atomic Checkpoint or Clean Rollback**:
   - **If `new_loss <= best_loss - min_improvement_delta` and `not frozen_violation`**:
     - Update `best_loss = new_loss` and reset `stall_counter = 0`.
     - Append winning pattern to `.harness/playbook.md` and `.harness/training_log.jsonl`.
     - Commit atomically:
       ```powershell
       git add <mutable_paths> .harness/playbook.md .harness/training_log.jsonl
       git commit -m "perf(harness): epoch <E> loss <new_loss> (<pct>% improvement)"
       ```
   - **Otherwise (`new_loss > best_loss - min_improvement_delta` or regression/violation)**:
     - Immediately execute **Zero-Residue Clean Rollback** (restores modified files AND removes untracked files in mutable paths):
       ```powershell
       git checkout -- <mutable_paths>
       git clean -fd <mutable_paths>
       ```
     - Record the failed hypothesis under Regression Traps in `.harness/playbook.md` and increment `stall_counter`.
5. **Early Stopping & Reflexion**:
   - If `best_loss <= target_loss`: Stop and declare convergence.
   - If `stall_counter >= patience`: Trigger algorithmic Reflexion (pivot strategy) or conclude training.

---

## Phase 3: Blind Holdout Verification, Gallery Audit & Distillation

When the training loop finishes:

1. **Blind Holdout Generalization Gate**:
   ```powershell
   uv run python .harness/eval_runner.py --epoch <final_epoch> --holdout
   ```
   - Compares Holdout loss against `epoch_0_holdout_loss` in `.harness/baseline.json`.
   - If `overfitting_detected` is `true` (degradation > `max_holdout_degradation`), rollback to the last generalized commit and report the overfitting boundary.
2. **Empirical Artifact Gallery Audit (For Visual / Multimodal / Structural Tasks)**:
   ```powershell
   uv run python .harness/eval_runner.py --gallery
   ```
   - Generates `.harness/gallery.html` from `.harness/artifacts_manifest.json`. Inspect outputs empirically via `read_file` or screen capture to confirm perceptual quality.
3. **Playbook Distillation & Executive Report**:
   - Finalize `.harness/playbook.md` and present a concise summary table:
     | Metric | Baseline (Epoch 0) | Best Checkpoint | Delta |
     | :--- | :--- | :--- | :--- |
     | Composite Loss (Train) | `0.4200` | `0.0850` | **-79.8%** |
     | Pass Rate (Train) | `60.0%` | `100.0%` | **+40.0%** |
     | Holdout Loss (Blind) | `0.4500` | `0.0920` | **-79.6% (No Overfit)** |
     | Latency | `142.0ms` | `48.0ms` | **-66.2%** |

---

## Gotchas & Anti-Patterns

- **Never Halt for Trivial Confirmations**: Do not ask the user to pick loss weights or epoch numbers when they can be synthesized autonomously.
- **Never Bypass `TEMPLATE_ADAPTED`**: Running `eval_runner.py` without implementing project-specific `execute_suite()` and passing `validate_metrics.py` is strictly blocked.
- **Never Leave Untracked Debris on Rollback**: Always pair `git checkout -- <mutable_paths>` with `git clean -fd <mutable_paths>`.
- **Never Peek at Holdout Data**: Reading sealed holdout files directly invalidates the optimization.
