#!/usr/bin/env python3
"""Validation utility for Harness Optimizer metric JSON schema.

Usage:
    python validate_metrics.py <path_to_json_or_eval_output>
    cat output.json | python validate_metrics.py -
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REQUIRED_SCHEMA = {
    "epoch": int,
    "split": str,
    "template_adapted": bool,
    "composite_loss": (float, int),
    "pass_rate": (float, int),
    "passed_count": int,
    "failed_count": int,
    "total_count": int,
    "latency_ms": (float, int),
    "diff_lines": int,
    "frozen_violation": bool,
    "sanitized_gradients": list,
}

VALID_SPLITS = {"probe", "train", "holdout"}


def validate_metrics(data: dict) -> list[str]:
    """Validate data against harness metric schema and anti-boilerplate gates."""
    errors: list[str] = []

    for field, expected_type in REQUIRED_SCHEMA.items():
        if field not in data:
            errors.append(f"Missing required field: '{field}'")
            continue

        value = data[field]
        if isinstance(expected_type, tuple):
            if isinstance(value, bool) or not isinstance(value, expected_type):
                errors.append(
                    f"Field '{field}' has invalid type: expected {expected_type}, got {type(value).__name__}"
                )
        else:
            if (expected_type is int and isinstance(value, bool)) or not isinstance(value, expected_type):
                errors.append(
                    f"Field '{field}' has invalid type: expected {expected_type.__name__}, got {type(value).__name__}"
                )

    if data.get("template_adapted") is not True:
        errors.append(
            "Scaffold Certification Blocked: 'template_adapted' is False. "
            "You must dynamically synthesize project-specific evaluation logic in eval_runner.py "
            "and set TEMPLATE_ADAPTED = True instead of running the raw boilerplate template."
        )

    split = data.get("split")
    if isinstance(split, str) and split not in VALID_SPLITS:
        errors.append(f"'split' must be one of {sorted(VALID_SPLITS)} (got '{split}')")

    if data.get("frozen_violation") is True:
        errors.append("Contract Violation: 'frozen_violation' is True. Frozen interface files were modified.")

    total = data.get("total_count")
    passed = data.get("passed_count")
    failed = data.get("failed_count")
    if isinstance(total, int) and not isinstance(total, bool):
        if total <= 0:
            errors.append(f"'total_count' must be > 0 (got {total}). Empty test runs are invalid.")
        if isinstance(passed, int) and isinstance(failed, int):
            if passed < 0 or failed < 0:
                errors.append("'passed_count' and 'failed_count' must be non-negative.")
            elif passed + failed != total:
                errors.append(
                    f"Sample count mismatch: passed_count ({passed}) + failed_count ({failed}) != total_count ({total})"
                )

    if "composite_loss" in data and not isinstance(data["composite_loss"], bool):
        loss = data["composite_loss"]
        if isinstance(loss, (int, float)) and not (0.0 <= loss <= 1.0):
            errors.append(f"'composite_loss' must be normalized between 0.0 and 1.0 (got {loss})")

    if "pass_rate" in data and not isinstance(data["pass_rate"], bool):
        pr = data["pass_rate"]
        if isinstance(pr, (int, float)) and not (0.0 <= pr <= 1.0):
            errors.append(f"'pass_rate' must be between 0.0 and 1.0 (got {pr})")

    return errors


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: python validate_metrics.py <file_or_dash>", file=sys.stderr)
        return 2

    target = sys.argv[1]
    try:
        if target == "-":
            raw_text = sys.stdin.read().lstrip("\ufeff")
        else:
            raw_text = Path(target).read_text(encoding="utf-8-sig")

        data = json.loads(raw_text)
    except (json.JSONDecodeError, OSError) as e:
        print(f"[ERROR] Failed to read or parse JSON: {e}", file=sys.stderr)
        return 1

    errors = validate_metrics(data)
    if errors:
        print("[FAIL] Harness metric schema validation errors:", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(
        f"[OK] Valid harness metric: epoch={data.get('epoch')}, "
        f"loss={data.get('composite_loss')}, pass_rate={data.get('pass_rate')}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
