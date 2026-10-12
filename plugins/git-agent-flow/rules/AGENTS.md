# Git Commit & Conventional Workflow Guidelines

## Core Principles
- **HEAD is human-accepted history**: `git log` must strictly contain clean, semantic, human-approved Conventional Commits.
- **Atomic Commits**: Each commit represents a single logical change or concern. Do not bundle unrelated changes across components into a single commit.

## Workflow Directives
1. **Change Inspection**:
   - Inspect modified and untracked files before committing using `git status -s` or `git status --porcelain`.
2. **Cluster by Intent & Architecture**:
   - Group changed files logically by type and architectural scope:
     - `feat(<scope>)`: New feature or user-facing addition
     - `fix(<scope>)`: Bug fix
     - `refactor(<scope>)`: Code reorganization without behavioral change
     - `docs(<scope>)`: Documentation updates
     - `test(<scope>)`: Adding or modifying tests
     - `perf(<scope>)`: Performance improvements
     - `chore(<scope>)`: Build tooling, dependency, or configuration updates
3. **Execution (`/smart-commit`)**:
   - For single-purpose updates, run `/smart-commit "<type>(<scope>): <message>"`.
   - For multi-module changes, formulate a plan JSON and invoke `git_smart_commit.ps1 -PlanJson '<json>'` to commit changes in clean atomic groups.
