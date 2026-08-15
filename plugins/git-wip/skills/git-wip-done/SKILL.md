---
name: git-wip-done
description: "Squashes and cleans up top consecutive WIP commits into a single clean commit."
usage: "/git-wip-done [commit_message]"
---

# Done Command Handler

When the `/git-wip-done` command is received or when finalizing WIP snapshots:

1. **Parameter Handling**:
   - If the user provides a commit message: pass it via `-Message "<message>"`.
   - If no message is provided: run the script with the default message, or summarize the current session's changes into an appropriate semantic commit message.

2. **Execute Command**:
   ```powershell
   pwsh -NoProfile -File ./scripts/git_squash_wip.ps1 -Message "<Commit Message>"
   ```
   *Note: If the script cannot be directly located due to different installation paths, the Agent may directly execute Git commands (`git reset --soft HEAD~<WIP_COUNT>` and `git commit -m "<Commit Message>"`) to squash consecutive WIP commits.*

3. **Report Result**: Output the number of squashed WIP commits along with the resulting commit hash and message.