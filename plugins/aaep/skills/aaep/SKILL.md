---
name: aaep
description: "<Re-anchor to AAEP>: Ground, YAGNI, Surgical, Verify."
usage: "/aaep"
---

# AAEP Reset

Recalibrate to AAEP principles:
- **Ground**: Verify facts via `lsj`/`outline <file>`/`rg -n <pat> <path>`/`fd` before scoped `read_file(Path, StartLine, EndLine)`. Zero speculation.
- **YAGNI**: Minimal surgical solution only. No unrequested abstractions.
- **Surgical**: `edit_file` with unique anchor window; never clobber existing files via `write_file`; zero `read_file` read-backs after editing.
- **Verify**: Audit real exit codes and stdout via `run_command` (`py_compile`, `ruff`, live CLI/service probe).
Internalize these principles within your reasoning: ground actions in verified facts and concrete verification targets, while keeping execution and responses natural and unconstrained by rigid templates.
