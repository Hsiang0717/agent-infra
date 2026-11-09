#!/usr/bin/env python3
"""
ECL AST-Grep Coordinate Synchronizer
Zero-token deterministic line synchronizer using ast-grep.
Maintains physical code coordinates in .agent/semantics/*.yaml.
"""

import json
import re
import subprocess
from pathlib import Path


def resolve_symbol_in_file(file_path: Path, symbol: str) -> tuple[int, int, str] | None:
    if not file_path.exists():
        return None

    decl_keywords = [
        "struct ",
        "fn ",
        "function ",
        "class ",
        "def ",
        "export ",
        "const ",
        "let ",
        "var ",
        "type ",
    ]

    # Try ast-grep with symbol
    cmd = ["ast-grep", "run", "-p", symbol, "--json=compact", str(file_path)]
    matches = []
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
        if proc.stdout.strip():
            matches = json.loads(proc.stdout)
    except (OSError, ValueError):
        matches = []

    with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
        file_lines = f.readlines()
    total_lines = len(file_lines)

    start_0 = None
    anchor_line = ""

    if matches:
        best_match = matches[0]
        for m in matches:
            line_text = m.get("lines", "")
            if any(kw in line_text for kw in decl_keywords):
                best_match = m
                break
        start_0 = best_match["range"]["start"]["line"]
        anchor_line = best_match.get("lines", "").strip()
    else:
        # Fallback: regex search for declaration line in file
        pattern = re.compile(rf"\b{re.escape(symbol)}\b")
        decl_candidates = []
        other_candidates = []
        for idx, line in enumerate(file_lines):
            if pattern.search(line):
                if any(kw in line for kw in decl_keywords):
                    decl_candidates.append((idx, line.strip()))
                else:
                    other_candidates.append((idx, line.strip()))

        if decl_candidates:
            start_0, anchor_line = decl_candidates[0]
        elif other_candidates:
            start_0, anchor_line = other_candidates[0]
        else:
            return None

    # Step 2a: Enclosure Radar - Expand upwards to include associated constants, decorators, docstrings
    expanded_start_0 = start_0
    for i in range(start_0 - 1, max(-1, start_0 - 10), -1):
        prev_line = file_lines[i].strip()
        if not prev_line:
            continue
        if any(
            prev_line.startswith(p)
            for p in [
                "const ",
                "pub const ",
                "static ",
                "pub static ",
                "#[",
                "@",
                "///",
                "//",
                "#",
                "/*",
                "*",
                "type ",
                "pub type ",
            ]
        ):
            expanded_start_0 = i
        else:
            break
    start_line = expanded_start_0 + 1
    end_line = start_line

    ext = file_path.suffix.lower()
    if ext in [
        ".ts",
        ".tsx",
        ".js",
        ".jsx",
        ".rs",
        ".go",
        ".java",
        ".c",
        ".cpp",
        ".cs",
    ]:
        # Brace counting from start_0
        brace_count = 0
        found_open = False
        for i in range(start_0, total_lines):
            l = file_lines[i]
            for ch in l:
                if ch == "{":
                    brace_count += 1
                    found_open = True
                elif ch == "}":
                    brace_count -= 1
            if found_open and brace_count <= 0:
                end_line = i + 1
                break
        if not found_open or end_line == start_line:
            end_line = min(start_line + 15, total_lines)
    elif ext in [".py"]:
        # Indentation matching for Python
        base_indent = len(file_lines[start_0]) - len(file_lines[start_0].lstrip())
        for i in range(start_0 + 1, total_lines):
            l = file_lines[i]
            if not l.strip() or l.strip().startswith("#"):
                continue
            curr_indent = len(l) - len(l.lstrip())
            if curr_indent <= base_indent:
                end_line = i
                break
        else:
            end_line = total_lines
    else:
        end_line = min(start_line + 20, total_lines)

    return (start_line, end_line, anchor_line)


def sync_semantics_file(yaml_path: Path, root_dir: Path) -> int:
    with open(yaml_path, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.splitlines(keepends=True)
    changed_count = 0
    new_lines = []

    node_pattern = re.compile(
        r"^(\s*(?:-\s*)?(?:id|node|path|file|target|entrypoint|location):\s*)([^\s#]+)#L(\d+)-(\d+)(.*)$"
    )
    symbol_pattern = re.compile(r"^\s*(?:symbol|fn|func|struct|class|name):\s*(\w+)")

    i = 0
    while i < len(lines):
        line = lines[i]
        m_node = node_pattern.match(line)
        if m_node:
            prefix = m_node.group(1)
            rel_file = m_node.group(2)
            cur_start = int(m_node.group(3))
            cur_end = int(m_node.group(4))
            suffix = m_node.group(5)

            symbol = None
            for j in range(i + 1, min(i + 4, len(lines))):
                m_sym = symbol_pattern.match(lines[j])
                if m_sym:
                    symbol = m_sym.group(1)
                    break

            if symbol:
                target_file = root_dir / rel_file
                res = resolve_symbol_in_file(target_file, symbol)
                if res:
                    new_start, new_end, _new_anchor = res
                    if new_start != cur_start or new_end != cur_end:
                        line = f"{prefix}{rel_file}#L{new_start}-{new_end}{suffix}\n"
                        changed_count += 1
                        print(
                            f"[{yaml_path.name}] Updated {symbol} ({rel_file}): #L{cur_start}-{cur_end} -> #L{new_start}-{new_end}"
                        )

        new_lines.append(line)
        i += 1

    if changed_count > 0:
        with open(yaml_path, "w", encoding="utf-8") as f:
            f.writelines(new_lines)

    return changed_count


def main():
    import argparse

    parser = argparse.ArgumentParser(description="ECL Coordinate Synchronizer")
    parser.add_argument("--root", default=".", help="Project root directory")
    parser.add_argument(
        "--semantics-dir", default=None, help="Semantics YAML directory"
    )
    args = parser.parse_args()

    root_dir = Path(args.root).resolve()
    total_synced = 0

    single_ecl = root_dir / ".agent" / "ecl.yaml"
    if single_ecl.exists():
        total_synced += sync_semantics_file(single_ecl, root_dir)

    sem_dir = (
        Path(args.semantics_dir).resolve()
        if args.semantics_dir
        else root_dir / ".agent" / "semantics"
    )
    if sem_dir.exists():
        for yml in sem_dir.glob("*.yaml"):
            if yml.name == "INDEX.yaml":
                continue
            total_synced += sync_semantics_file(yml, root_dir)

    print(f"ECL Sync complete. Total coordinates updated: {total_synced}")


if __name__ == "__main__":
    main()
