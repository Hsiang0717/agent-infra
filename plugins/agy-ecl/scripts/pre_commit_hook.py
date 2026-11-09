#!/usr/bin/env python3
"""
ECL Zero-Token Pre-Commit Hook
1. Calibrates physical line coordinates via ecl_sync with 0 AI tokens (<50ms).
2. Automatically stages updated .agent/semantics/*.yaml files into the active commit.
3. Performs structural diff audit to detect causal changes (export, EventBus) and issue hints.
"""

import re
import subprocess
import sys
from pathlib import Path

# Structural patterns that alter causality
STRUCTURAL_PATTERNS = [
    re.compile(r"^[+-]\s*export\s+(?:const|function|class|default)", re.MULTILINE),
    re.compile(r"^[+-]\s*(?:emit|dispatch)\(['\"][A-Z0-9_]+['\"]", re.MULTILINE),
    re.compile(r"^[+-]\s*(?:on|subscribe)\(['\"][A-Z0-9_]+['\"]", re.MULTILINE),
    re.compile(r"^[+-]\s*@(Get|Post|Put|Delete|Patch)\(", re.MULTILINE),
]


def run_coordinate_sync(root_dir: Path, sem_dir: Path):
    from ecl_sync import sync_semantics_file

    total_synced = 0
    modified_yamls = []

    single_ecl = root_dir / ".agent" / "ecl.yaml"
    if single_ecl.exists():
        count = sync_semantics_file(single_ecl, root_dir)
        if count > 0:
            total_synced += count
            modified_yamls.append(str(single_ecl))

    if sem_dir.exists():
        for yml in sem_dir.glob("*.yaml"):
            if yml.name == "INDEX.yaml":
                continue
            count = sync_semantics_file(yml, root_dir)
            if count > 0:
                total_synced += count
                modified_yamls.append(str(yml))

    if modified_yamls:
        subprocess.run(["git", "add"] + modified_yamls, cwd=root_dir, check=False)
        print(
            f"[ECL Hook] Auto-calibrated {total_synced} coordinates across {len(modified_yamls)} ECL file(s)."
        )

    return total_synced


def check_structural_diff(root_dir: Path):
    try:
        proc = subprocess.run(
            ["git", "diff", "--cached", "--unified=0"],
            cwd=root_dir,
            capture_output=True,
            text=True,
            check=False,
        )
        diff_text = proc.stdout
    except Exception:
        return

    structural_hits = []
    for pattern in STRUCTURAL_PATTERNS:
        matches = pattern.findall(diff_text)
        if matches:
            structural_hits.extend(matches)

    if structural_hits:
        print(
            "[ECL Notice] Staged diff contains structural causality changes (export/events)."
        )
        print(
            "             Remember to review or enrich .agent/semantics/ if causal flows shifted."
        )


def main():
    root_dir = Path.cwd()
    sem_dir = root_dir / ".agent" / "semantics"
    single_ecl = root_dir / ".agent" / "ecl.yaml"
    if not single_ecl.exists() and not sem_dir.exists():
        sys.exit(0)

    # Make sure scripts dir is in sys.path to import ecl_sync
    script_dir = Path(__file__).resolve().parent
    if str(script_dir) not in sys.path:
        sys.path.insert(0, str(script_dir))

    # Step 1: Run coordinate sync (<50ms, 0 tokens)
    run_coordinate_sync(root_dir, sem_dir)

    # Step 2: Check structural changes
    check_structural_diff(root_dir)

    sys.exit(0)


if __name__ == "__main__":
    main()
