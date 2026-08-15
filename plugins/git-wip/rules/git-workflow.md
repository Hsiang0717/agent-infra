# Git WIP Workflow Guidelines

1. **No Manual WIP Commits**: The Agent does not need to manually run `git add` or `git commit` via `run_command` during development. The Stop Hook automatically creates a `WIP` snapshot when the turn finishes.
2. **Finalization Guidance**: When milestones or task goals are achieved, proactively inform the user that they can run `/git-wip-done <commit_message>` to squash WIP commits into a clean commit.