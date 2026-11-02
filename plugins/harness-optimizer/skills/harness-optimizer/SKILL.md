---
name: harness-optimizer
description: Recursive code, algorithm, prompt, and test-harness optimization framework inspired by Google Research Harness2. Use when the user wants to iteratively train, optimize, or evolve code, algorithms, LLM prompts, or evaluation harnesses using epochs, composite loss functions, semantic gradients, black-box dataset isolation, pre-scaffold foolproofing, and two-tier subagent context firewall execution.
---
# Harness Optimizer

An autonomous, language-agnostic recursive optimization framework inspired by Google Research Harness2. It treats code, algorithms, prompts, and the evaluation harness itself as trainable parameters, improving them through structured epochs, composite loss functions, semantic gradients, Git checkpointing, and strict black-box dataset isolation.

---

## Architecture: Two-Tier Orchestrator-Worker Pattern

To protect the main conversation from token exhaustion during multi-epoch runs, the framework enforces a **Two-Tier Architecture**:

```text
User <=======> Main Agent (Orchestrator)
                  ├── Phase 0: Pre-Flight Audit & Option Selection
                  ├── Phase 1: Scaffolding & Epoch 0 Baseline Checkpoint
                  ├── Phase 2 (Delegated): invoke_subagent(TypeName: "self", Role: "Harness Training Worker")
                  │     └── [Context Firewall]: Epochs 1..N Training Loop, Rollbacks & Commits
                  └── Phase 3: Final Verification, Playbook Distillation & Report to User
```

- **Main Agent (Orchestrator)**: Interacts with the user, audits the workspace, proposes architectures, confirms parameters, conducts final acceptance, and leads interactive perceptual/sensory evaluation loops.
- **Subagent (Training Worker)**: Executes automated multi-epoch training loops (`max_epochs >= 3`) inside an isolated session, absorbing compiler noise and test failures without bloating the main context. Returns a high-density summary report upon completion.
- **Adaptive Execution Modes**:
  - *Automated Mode*: Batch benchmarks and deterministic test suites fully delegate to Subagent.
  - *Sensory/Interactive Mode*: Real-time audio, video, or GUI tasks maintain Main Agent dialogue for human perceptual feedback across epochs.

---

## Phase 0: Pre-Flight Audit, Foolproofing, and Option Selection (Mandatory Gate)

Never generate scaffolding or mutate project code immediately. First inspect the workspace and present a structured diagnostic report with a recommended architecture and alternative options. Wait for explicit user confirmation before building the scaffold.

### Step 1: Read-Only Project Audit & Foolproofing Checks
Perform these safety checks before proposing any setup:
1. **Git Safety Check**: Verify `git status -s` and current branch. Ensure uncommitted changes are safely committed or stashed before creating optimization branches.
2. **Black-Box Dataset Leak Audit**: Identify dataset paths (`dataset/`, `eval/`, `test_data/`, `*holdout*`). Verify how the evaluation runner will encapsulate them to prevent direct agent inspection.
3. **Language & Toolchain Detection**: Check manifests (`pyproject.toml`, `package.json`, `Cargo.toml`, `go.mod`) and verify available compilers, runtimes (`uv`, `node`, `cargo`), and test runners.
4. **Side-Effect & Resource Guard**: Ensure test execution does not perform unbounded network calls or persistent database writes. Determine a strict execution timeout (e.g., 30s).
5. **Interface Contract Identification**: Identify public functions, CLI arguments, or API boundaries that must remain frozen during mutations.

### Step 2: Present Recommendations and Alternatives
Present a clear **Assessment & Selection Proposal** covering four dimensions:

1. **Execution & Isolation Strategy**
   - *Option A (Recommended)*: Local virtual environment / native toolchain with subprocess timeout and path-restricted runner wrapper.
   - *Option B*: Sandboxed process execution with read-only source mounts.
   - *Option C*: Custom CLI wrapper around existing test scripts.

2. **Loss Function Composition**
   - *Option A (Accuracy-First)*: Correctness / fail rate (70%), execution latency (20%), diff complexity (10%).
   - *Option B (Performance / Algorithm Tuning)*: Correctness (40%), latency/memory profiling (50%), diff complexity (10%).
   - *Option C (Prompt / Output Quality)*: Output semantic score / rubric distance (70%) + token cost / latency (30%).
   - *Option D (Sensory / Real-Time Streaming)*: Underrun/glitch immunity (70%), responsiveness/latency (20%), complexity penalty (10%) + human auditory/visual feedback gate.

3. **Optimizer & Mutation Strategy**
   - *Option A (Greedy Hill-Climbing + Reflexion)*: Single targeted mutation per epoch with immediate rollback on regression; reflection triggered after 2 consecutive stalls.
   - *Option B (Best-of-N Candidates)*: Propose 2-3 candidate hypotheses per epoch, evaluate each, and commit the lowest-loss candidate.
   - *Option C (Two-Stage Curriculum)*: Stage 1 reaches 100% test pass rate; Stage 2 automatically pivots loss weights to optimize latency and memory.

4. **Hyperparameters & Stopping Criteria**
   - Propose defaults for `max_epochs` (e.g. 6-10), `patience` (e.g. 3), `learning_rate` (max lines/files touched per epoch), and `target_loss`.

**Pause and wait for the user to approve the Recommended Plan or choose custom options before proceeding.**

---

## Phase 1: Bootstrap & Harness Scaffolding (Mandatory Certification Gate)

Once confirmed by the user, follow the strict 5-step scaffolding sequence. Agents are **strictly prohibited from mutating project source code** until Step 4 exits with code 0:

1. **Initialize Baseline Checkpoint**:
   - Create a dedicated optimization branch:
     ```powershell
     git checkout -b harness/<topic>
     ```
2. **Scaffold the `.harness/` Environment**:
   - Copy or adapt templates from `<skill_dir>/templates/`:
     - `.harness/config.json`: Filled with Tiered Loss weights (Tier 1 Crash/Syntax Gate, Tier 2 Quality/Latency), hyperparams, and frozen boundaries (`config.json.tmpl`).
     - `.harness/eval_runner.py`: Black-box test runner with Train/Holdout split and artifact diff support (`eval_runner.py.tmpl`).
     - `.harness/playbook.md`: Learned rules, positive invariants, and regression traps (`playbook.md.tmpl`).
     - `.harness/training_log.jsonl`: Append-only training history.
3. **Mandatory Single-Sample Probe Run (`--probe`)**:
   - Before running the entire dataset, execute the runner with `--probe` on 1 single sample:
     ```powershell
     uv run python .harness/eval_runner.py --probe
     ```
   - Verifies runtime safety, execution timeout, memory isolation, and JSON serialization without waiting for full dataset battery.
4. **Mandatory Schema Certification Gate (Strict Blocking Gate)**:
   - Validate `.harness/metrics.json` strictly using the validator script:
     ```powershell
     uv run python <skill_dir>/scripts/validate_metrics.py .harness/metrics.json
     ```
   - **Hard Rule**: If this script fails or exits non-zero, the scaffold is incomplete. You MUST fix `eval_runner.py` before modifying any codebase files. Never bypass or treat schema validation as optional.
5. **Epoch 0 Full Baseline Run & Checkpoint**:
   - Execute the runner on the full training dataset:
     ```powershell
     uv run python .harness/eval_runner.py --epoch 0
     ```
   - Record `Epoch 0` baseline loss as `best_loss` and commit baseline scaffold:
     ```powershell
     git add .harness/ && git commit -m "chore(harness): initialize verified evaluation harness and epoch 0 baseline"
     ```

---

## Phase 2: Task-Level Training Loop (Subagent Delegation Protocol)

### Delegating to Worker Subagent (`max_epochs >= 3`)
Invoke the `self` subagent to shield the main context:

```markdown
invoke_subagent(
  TypeName: "self",
  Role: "Harness Training Worker",
  Prompt: "Execute Harness training loop for project. Branch: harness/baseline. Config: .harness/config.json. Best loss: <initial_loss>. Execute Epochs 1 to <max_epochs> following Phase 2 protocol (Forward -> Backward -> Bounded Mutation -> Test -> Commit/Rollback). Do not print raw dataset content. When converged or patience reached, return a dense Markdown summary reporting: (1) Baseline vs Best metrics, (2) Commit log, (3) Distilled rules in playbook.md."
)
```

### Inner Epoch Execution Cycle (Worker or Direct)
For each epoch `E = 1 .. max_epochs`:

1. **Forward Pass**: Execute `.harness/eval_runner.py --epoch E` to obtain `composite_loss`.
2. **Backward Pass (Semantic Gradient Extraction)**:
   - Analyze sanitized stack traces, error line coordinates, and profiler hotspots.
   - Consult `.harness/playbook.md` to avoid previously failed patterns.
3. **Optimizer Step (Bounded Mutation)**:
   - Formulate a clear hypothesis (e.g., "Replace linear search with binary search in helper function").
   - Apply edits surgically via `edit_file`, staying strictly within `learning_rate` limits.
   - **Never read holdout dataset files directly.**
4. **Validation, Checkpoint, or Rollback**:
   - Re-run `.harness/eval_runner.py --epoch E`.
   - **If `new_loss < best_loss`**:
     - Update `best_loss = new_loss`. Reset stall counter to 0.
     - Commit changes: `git commit -m "perf(harness): epoch E loss <loss> (<pct>% improvement)"`.
     - Log entry in `.harness/training_log.jsonl`.
   - **If `new_loss >= best_loss` or regression occurs**:
     - Immediately rollback: `git checkout -- <modified_files>`.
     - Record failed hypothesis in `.harness/playbook.md` under Negative Heuristics.
     - Increment stall counter.
5. **Early Stopping & Reflexion**:
   - If `best_loss <= target_loss`: Stop and declare convergence.
   - If `stall_counter >= patience`: Perform Reflexion step (shift algorithmic approach) or stop training.

---

## Phase 3: Meta-Loop Evolution & Distillation

When training concludes:

1. **Blind Holdout Generalization Gate**:
   - Execute the runner once on the sealed Holdout dataset:
     ```powershell
     uv run python .harness/eval_runner.py --holdout
     ```
   - Ensure the optimized code generalizes to unseen cases without overfitting. If holdout regression is detected, alert the user and offer to step back to the best generalized checkpoint.
2. **Interactive Artifact Gallery Audit (For Visual / Multimodal Tasks)**:
   - Generate or refresh the unified HTML comparison gallery:
     ```powershell
     uv run python .harness/eval_runner.py --gallery
     ```
   - Empirically inspect side-by-side pairs (Ground Truth vs. Rendered/Actual output) using `read_file` or browser to ensure zero perceptual or structural defects before closure.
3. **Playbook Distillation**:
   - Consolidate winning refactoring patterns, invariant requirements, and regression traps into `.harness/playbook.md`.
4. **Adaptive Curriculum Shift (Optional)**:
   - If functional pass rate reached 100%, offer to shift loss schedule toward execution latency and memory footprint for Stage 2.
5. **Executive Training Report**:
   - Present a clean comparison table to the user:
     | Metric | Baseline (Epoch 0) | Best Checkpoint | Improvement |
     | :--- | :--- | :--- | :--- |
     | Composite Loss | `0.420` | `0.085` | **-79.8%** |
     | Test Pass Rate | `60.0%` | `100.0%` | **+40.0%** |
     | Holdout Generalization | `58.0%` | `98.0%` | **+40.0%** |
     | Latency | `142ms` | `48ms` | **-66.2%** |
     | Total Commits | - | 4 commits | - |

---

## Gotchas & Anti-Patterns

- **Never Mutate Code Before Scaffolding Certification**: Modifying project code before `validate_metrics.py` exits 0 on the probe run is strictly prohibited.
- **Never Peek at Black-Box Data**: Inspecting holdout tests directly invalidates the benchmark and constitutes optimization fraud.
- **Never Skip Phase 0 User Approval**: Building scaffolding or creating Git branches before presenting options violates user alignment.
- **Enforce Immediate Rollbacks**: Never leave regressed or non-compiling code in the working tree across epochs.
- **Enforce Context Firewall**: Never run long multi-epoch training loops directly in the main conversation if `max_epochs >= 3`.
