#!/usr/bin/env python3
"""
ECL Bootstrap Engine
Scans project source code topology using ast-grep and generates .agent/semantics/
Produces L0 INDEX.yaml and L1 <domain>.yaml skeletons ready for LLM semantic enrichment.
"""

import re
import sys
from pathlib import Path

# Entrypoint patterns for common frameworks & languages
ENTRYPOINT_PATTERNS = [
    (r"pub\s+(?:async\s+)?fn\s+(\w+)", "RUST_PUB_FN"),
    (r"pub\s+struct\s+(\w+)", "RUST_PUB_STRUCT"),
    (r"export\s+const\s+(\w+)\s*:\s*React\.FC", "REACT_COMPONENT"),
    (r"export\s+default\s+function\s+(\w+)", "REACT_COMPONENT"),
    (r"export\s+function\s+(\w+Controller|\w+Handler|\w+)", "TS_EXPORT_FN"),
    (r"@(?:Get|Post|Put|Delete|Patch)\(['\"]([^'\"]+)['\"]\)", "NEST_ROUTER"),
    (r"def\s+(api_\w+|handle_\w+|\w+_hook|\w+_handler)\(", "PYTHON_HANDLER"),
]

EVENT_EMIT_PATTERN = r"(?:emit|dispatch|send| invoke_on_event_loop)\(['\"]?([A-Za-z0-9_]+)"
EVENT_LISTEN_PATTERN = r"(?:on|subscribe|case|on_\w+)\(['\"]?([A-Za-z0-9_]+)"


def extract_project_topology(src_dir: Path, root_dir: Path):
    topology = {"entrypoints": [], "events": {}, "nodes": []}

    # Supported file extensions
    exts = ["*.rs", "*.ts", "*.tsx", "*.js", "*.jsx", "*.py", "*.go"]
    source_files = []
    for ext in exts:
        source_files.extend(src_dir.rglob(ext))

    for p in source_files:
        # Ignore common non-source directories
        rel_str = str(p.relative_to(root_dir)).replace("\\", "/")
        if any(
            ign in rel_str
            for ign in ["node_modules", "dist", ".git", ".agent", "build"]
        ):
            continue

        try:
            with open(p, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()
        except OSError:
            continue

        for idx, line in enumerate(lines, start=1):
            # 1. Entrypoint identification
            for pattern, ep_type in ENTRYPOINT_PATTERNS:
                m = re.search(pattern, line)
                if m:
                    sym_name = m.group(1)
                    topology["entrypoints"].append(
                        {
                            "file": rel_str,
                            "symbol": sym_name,
                            "type": ep_type,
                            "line": idx,
                            "anchor": line.strip()[:65],
                        }
                    )
            # 2. Event Emitter identification
            for em in re.finditer(EVENT_EMIT_PATTERN, line):
                evt = em.group(1)
                topology["events"].setdefault(evt, {"emitters": [], "listeners": []})
                topology["events"][evt]["emitters"].append(
                    {"file": rel_str, "line": idx, "anchor": line.strip()[:65]}
                )
            # 3. Event Listener identification
            for ls in re.finditer(EVENT_LISTEN_PATTERN, line):
                evt = ls.group(1)
                topology["events"].setdefault(evt, {"emitters": [], "listeners": []})
                topology["events"][evt]["listeners"].append(
                    {"file": rel_str, "line": idx, "anchor": line.strip()[:65]}
                )

    return topology


def build_cluster_skeletons(topology: dict, root_dir: Path):
    from ecl_sync import resolve_symbol_in_file

    clusters_by_domain = {}

    for ep in topology["entrypoints"]:
        file_path = root_dir / ep["file"]
        res = resolve_symbol_in_file(file_path, ep["symbol"])
        if res:
            start_l, end_l, anchor_line = res
        else:
            start_l = ep["line"]
            end_l = min(start_l + 20, 100)
            anchor_line = ep["anchor"]

        # Derive domain from path or naming convention
        parts = ep["file"].split("/")
        domain = "core"
        if len(parts) > 2:
            domain = parts[1].lower()
        elif len(parts) == 2:
            domain = parts[0].lower()

        cluster_id = f"flow_{ep['symbol'].lower()}"
        cluster = {
            "cluster_id": cluster_id,
            "summary": f"[TODO_LLM_SUMMARY: Domain business logic & causal purpose of {ep['symbol']} (NOT literal code description)]",
            "entrypoint": {
                "id": f"{ep['file']}#L{start_l}-{end_l}",
                "symbol": ep["symbol"],
                "anchor": anchor_line,
            },
            "chain": [],
            "rules": [
                f"[TODO_LLM_RULE: Event ordering, decoupled dependencies & causal invariants for {ep['symbol']} (NO code comments)]"
            ],
        }

        # Check for correlated event listeners or chained functions
        for evt, data in topology.get("events", {}).items():
            if any(e["file"] == ep["file"] for e in data["emitters"]):
                for l in data["listeners"]:
                    cluster["chain"].append(
                        {
                            "node": f"{l['file']}#L{l['line']}-{l['line'] + 15}",
                            "symbol": evt,
                            "effect": f"[TODO_LLM_EFFECT: Downstream causal side-effect & state mutation on {evt}]",
                        }
                    )

        clusters_by_domain.setdefault(domain, []).append(cluster)

    return clusters_by_domain


def generate_ecl_file(clusters_by_domain: dict, output_path: Path):
    output_path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# External Cognitive Layer (ECL) — High-Signal Causal & Business Invariant Ledger\n",
        "# Captures ONLY business logic, causal chains, decoupled dependencies, and event ordering (NEVER literal code comments).\n\n",
        "clusters:\n",
    ]
    for clusters in clusters_by_domain.values():
        for c in clusters:
            lines.append(f"  - cluster_id: {c['cluster_id']}\n")
            lines.append(f"    summary: {c['summary']}\n")
            lines.append("    entrypoint:\n")
            lines.append(f"      id: {c['entrypoint']['id']}\n")
            lines.append(f"      symbol: {c['entrypoint']['symbol']}\n")

            if c["chain"]:
                lines.append("    chain:\n")
                for ch in c["chain"]:
                    lines.append(f"      - node: {ch['node']}\n")
                    lines.append(f"        symbol: {ch['symbol']}\n")
                    lines.append(f"        effect: {ch['effect']}\n")
            else:
                lines.append("    chain: []\n")

            lines.append("    rules:\n")
            for r in c["rules"]:
                lines.append(f"      - {r}\n")
            lines.append("\n")

    with open(output_path, "w", encoding="utf-8") as f:
        f.writelines(lines)
    print(f"Generated single-file ECL ledger: {output_path}")


def main():
    import argparse

    parser = argparse.ArgumentParser(description="ECL Bootstrap Engine")
    parser.add_argument("--src", default="src", help="Source code directory to scan")
    parser.add_argument("--root", default=".", help="Project root directory")
    parser.add_argument(
        "--out", default=".agent/ecl.yaml", help="Output ECL YAML file path"
    )
    args = parser.parse_args()

    root_dir = Path(args.root).resolve()
    src_dir = (
        (root_dir / args.src).resolve()
        if not Path(args.src).is_absolute()
        else Path(args.src).resolve()
    )
    out_path = (
        (root_dir / args.out).resolve()
        if not Path(args.out).is_absolute()
        else Path(args.out).resolve()
    )

    if not src_dir.exists():
        print(f"Error: Source directory {src_dir} does not exist.", file=sys.stderr)
        sys.exit(1)

    print(f"Scanning project topology in: {src_dir}")
    topology = extract_project_topology(src_dir, root_dir)
    print(
        f"Found {len(topology['entrypoints'])} entrypoints and {len(topology['events'])} event channels."
    )

    clusters_by_domain = build_cluster_skeletons(topology, root_dir)
    generate_ecl_file(clusters_by_domain, out_path)
    print("ECL Bootstrap skeleton generation completed successfully.")


if __name__ == "__main__":
    main()
