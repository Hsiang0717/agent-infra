---
name: git-feat-flow
description: AI-adaptive Git workflow for Feature Branches. Supports automatic project analysis, custom rule generation (.git-feat-flow.md), granular commits, and safe rebase management.
---

# Git Feature Flow (AI-Powered)

This skill utilizes AI semantic analysis to adapt to any project structure, transforming the Agent into a Git consultant with deep project context.

## Core Workflow

### 1. AI Project Adaptation
1. **Pilot (AI) Command**: Invoke `scripts/project_initializer.js`.
2. **Executor (Script) Execution**: Scans directory structure, critical files, and **Remote Status**. Outputs project metadata in JSON format.
3. **Pilot (AI) Reasoning**: Reads JSON to determine project type, appropriate Scope categories, and **Workflow Mode (local/collaborative)**. Generates `.git-feat-flow.md`.

### 2. Branching Guard & Granular Commits
1. **Pilot (AI) Pre-check**: Invoke `git branch --show-current`. If on `main`/`master`, perform automatic branch diversion (create feature branch).
2. **Executor (Script) Classification**: Invoke `scripts/git_analyzer.js` to automatically group changed files based on project rules.
3. **Pilot (AI) Semantic Analysis**: Analyze `git diff` and grouping results to generate precise Conventional Commits for each group.
4. **Gatekeeper (AI) Protocol**: If `workflow_mode` is `collaborative`, **HALT all automated actions** after `git push`. Instruct the user to initiate a Merge Request (MR/PR). **DO NOT** merge into protected branches locally.

### 3. Safe Synchronization (Safe Rebase)
1. **Executor (Script) Check**: Invoke `scripts/rebase_helper.js` to ensure the workspace is clean and not on the main branch.
2. **Pilot (AI) Sync**: Execute `git rebase`. If conflicts occur, AI analyzes conflict blocks and provides resolution suggestions.

### 4. Post-Merge Cleanup
1. **Pilot (AI) Verification**: Before cleanup, verify that the remote branch has been successfully merged (via user confirmation or API check).
2. **Executor (Script) Execution**: Invoke `scripts/cleanup_helper.js` to switch back to the main branch, sync with remote, and safely delete the local feature branch.


## Command Reference

- **"Analyze project and setup git-feat-flow"**: Start the adaptation process, detect environment, and set `workflow_mode`.
- **"Help me commit these changes in batches"**: Perform granular commits. In collaborative mode, automatically enter a waiting state after Pushing.
- **"Sync changes and handle conflicts"**: Execute Rebase with AI conflict assistance.
- **"Task complete, help me cleanup the branch"**: Execute only AFTER confirmation of merge to ensure local/remote synchronization.

## Configuration: .git-feat-flow.md
Located in the project root, containing:
- `workflow_mode`: `local` (personal) or `collaborative` (team-based, requires MR).
- `mappings`: Path-to-Scope/Type mappings.
- `global_rules`: Categorization logic based on extensions or specific files.
- `ai_adaptation`: Whether to allow LLM dynamic adjustments during execution.
