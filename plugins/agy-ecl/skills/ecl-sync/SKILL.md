---
name: ecl-sync
description: "Synchronizes all physical line coordinates (#L{start}-{end}) in .agent/ecl.yaml using ast-grep with zero AI token consumption."
usage: "/ecl-sync [target_dir]"
---

# /ecl-sync Command Handler

This command executes deterministic AST-based line recalibration across `.agent/ecl.yaml`.

---

## Execution Protocol

1. Run the synchronizer engine:
   ```powershell
   uv run python "$env:USERPROFILE/.gemini/config/plugins/agy-ecl/scripts/ecl_sync.py" --root .
   ```
2. Inspect the output count of updated coordinates.
3. If executed before a Git commit, stage the updated YAML file:
   ```powershell
   git add .agent/ecl.yaml
   ```
