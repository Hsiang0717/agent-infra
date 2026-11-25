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


def get_staged_name_status(root_dir: Path) -> list[tuple[str, str]]:
    try:
        proc = subprocess.run(
            ["git", "diff", "--cached", "--name-status"],
            cwd=root_dir,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return []

    entries: list[tuple[str, str]] = []
    for raw in proc.stdout.splitlines():
        parts = raw.split("\t")
        if len(parts) < 2:
            continue
        status = parts[0].strip()
        # For rename (R100\told\tnew), track the old path that ECL might still reference.
        path = parts[1].strip().replace("\\", "/")
        entries.append((status, path))
    return entries


def collect_all_tracked_files(root_dir: Path, sem_dir: Path) -> set[str]:
    from ecl_sync import collect_tracked_files

    tracked: set[str] = set()
    single_ecl = root_dir / ".agent" / "ecl.yaml"
    if single_ecl.exists():
        tracked |= collect_tracked_files(single_ecl)
    if sem_dir.exists():
        for yml in sem_dir.glob("*.yaml"):
            if yml.name != "INDEX.yaml":
                tracked |= collect_tracked_files(yml)
    return {p.lower() for p in tracked}


def run_coordinate_sync(root_dir: Path, sem_dir: Path) -> tuple[int, list[str]]:
    from ecl_sync import sync_semantics_file

    total_synced = 0
    modified_yamls = []
    stale_entries: list[str] = []

    single_ecl = root_dir / ".agent" / "ecl.yaml"
    if single_ecl.exists():
        count = sync_semantics_file(single_ecl, root_dir, stale_entries)
        if count > 0:
            total_synced += count
            modified_yamls.append(str(single_ecl))

    if sem_dir.exists():
        for yml in sem_dir.glob("*.yaml"):
            if yml.name == "INDEX.yaml":
                continue
            count = sync_semantics_file(yml, root_dir, stale_entries)
            if count > 0:
                total_synced += count
                modified_yamls.append(str(yml))

    if modified_yamls:
        subprocess.run(["git", "add"] + modified_yamls, cwd=root_dir, check=False)
        print(
            f"[ECL Hook] Auto-calibrated {total_synced} coordinates across {len(modified_yamls)} ECL file(s)."
        )

    if stale_entries:
        print(
            f"[ECL STALE] {len(stale_entries)} broken ECL reference(s) detected (see above). "
            "Update or prune .agent/ecl.yaml."
        )

    return total_synced, stale_entries


def check_tracked_and_structural_diff(
    root_dir: Path,
    staged_before_sync: list[tuple[str, str]],
    tracked_files: set[str],
):
    ecl_touched_by_user = any(
        p.lower() == ".agent/ecl.yaml" or p.lower().startswith(".agent/semantics/")
        for _, p in staged_before_sync
    )

    removed_or_renamed: list[str] = []
    modified_tracked: list[str] = []
    for status, path in staged_before_sync:
        norm = path.lower()
        if norm in tracked_files:
            if status.startswith(("D", "R")):
                removed_or_renamed.append(path)
            elif status.startswith("M"):
                modified_tracked.append(path)

    if removed_or_renamed and not ecl_touched_by_user:
        print(
            f"[ECL STALE] Tracked file(s) renamed/deleted without updating ECL: {', '.join(removed_or_renamed)}"
        )
    elif modified_tracked and not ecl_touched_by_user:
        print(
            f"[ECL Review] Tracked file(s) modified ({', '.join(modified_tracked)}); "
            "verify rules/chain in .agent/ecl.yaml still hold."
        )

    try:
        proc = subprocess.run(
            ["git", "diff", "--cached", "--unified=0"],
            cwd=root_dir,
            capture_output=True,
            text=True,
            check=False,
        )
        diff_text = proc.stdout
    except OSError:
        return

    structural_hits = []
    for pattern in STRUCTURAL_PATTERNS:
        matches = pattern.findall(diff_text)
        if matches:
            structural_hits.extend(matches)

    if structural_hits and not ecl_touched_by_user:
        print(
            "[ECL Notice] Staged diff contains structural causality changes (export/events)."
        )
        print(
            "             Remember to review or enrich .agent/ecl.yaml if causal flows shifted."
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

    # Capture user-staged files BEFORE auto-calibration adds .agent/ecl.yaml
    staged_before_sync = get_staged_name_status(root_dir)
    tracked_files = collect_all_tracked_files(root_dir, sem_dir)

    # Step 1: Run coordinate sync + stale reference audit (<50ms, 0 tokens)
    run_coordinate_sync(root_dir, sem_dir)

    # Step 2: Check tracked-file semantic drift and structural causality changes
    check_tracked_and_structural_diff(root_dir, staged_before_sync, tracked_files)

    sys.exit(0)


if __name__ == "__main__":
    main()
