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
    "composite_loss": (float, int),
    "pass_rate": (float, int),
    "failed_count": int,
    "latency_ms": (float, int),
    "diff_lines": int,
    "sanitized_gradients": list,
}


def validate_metrics(data: dict) -> list[str]:
    """Validate data against harness metric schema."""
    errors: list[str] = []

    for field, expected_type in REQUIRED_SCHEMA.items():
        if field not in data:
            errors.append(f"Missing required field: '{field}'")
            continue

        value = data[field]
        if not isinstance(value, expected_type):
            errors.append(
                f"Field '{field}' has invalid type: expected {expected_type}, got {type(value).__name__}"
            )

    if "composite_loss" in data:
        loss = data["composite_loss"]
        if not (0.0 <= loss <= 1.0):
            errors.append(f"'composite_loss' must be normalized between 0.0 and 1.0 (got {loss})")

    if "pass_rate" in data:
        pr = data["pass_rate"]
        if not (0.0 <= pr <= 1.0):
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
