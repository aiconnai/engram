#!/usr/bin/env python3
"""Measure the harness mandatory read set (bytes, lines, tokens) or read one whole section (task H6).

Usage:
  python3 docs/harness/bin/measure-context.py [--json] [--tokenizer auto|tiktoken|approx]
                                              [--root DIR] [--files F ...]
  python3 docs/harness/bin/measure-context.py --section FILE.md#anchor [--root DIR]

Mandatory read set = the files AGENTS.md / CLAUDE.md / bootstrap.sh tell every agent to read, in
that order, with the *active plan* resolved from the "Active plan" row of docs/harness/progress.md.
`--files` replaces the set (paths relative to --root); a missing file is reported under
"missing" and makes the run fail (exit 1) -- a measurement never silently skips a mandatory file.

Tokenizers (the label is always printed; an approximation is never presented as exact):
  tiktoken:cl100k_base   exact BPE count with the `tiktoken` package, only if it is importable AND
                         the vocabulary is already cached locally. The tool never touches the
                         network: sockets are blocked while the encoding loads.
  approx:utf8-bytes/4    ceil(utf8_bytes / 4). A documented approximation, NOT a tokenizer. Measured
                         against cl100k_base on this repository's mandatory read set (mixed
                         Portuguese/English Markdown) it understates by about 6% (66,436 vs 70,842).
`--tokenizer auto` (default) uses tiktoken when available, otherwise the labelled approximation and
records why in "fallback_reason". `--tokenizer tiktoken` fails (exit 1) instead of falling back.
Note: cl100k_base is the encoding Engram itself uses for budgets (tiktoken-rs); it is a proxy for
any particular model's context accounting, not the tokenizer of a specific model.

`--section FILE.md#anchor` prints the whole section (heading through the line before the next
heading of the same or higher level, GitHub anchor rules) so a historical entry is read complete,
never cut in the middle of a paragraph. Exit 1 when the anchor does not exist.

Exit codes: 0 ok, 1 measurement/section failure, 2 usage error. Offline; standard library only.
"""

from __future__ import annotations

import argparse
import json
import math
import re
import socket
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from doc_links import section_text  # noqa: E402

APPROX_NAME = "approx:utf8-bytes/4"
TIKTOKEN_NAME = "tiktoken:cl100k_base"

MANDATORY_BEFORE_PLAN = [
    "docs/harness/SPEC.md",
    "docs/harness/INVARIANTS.md",
    "docs/harness/WHAT_WE_DONT_DO.md",
    "docs/harness/GATES.md",
    "docs/harness/CODE_REVIEW_POLICY.md",
    "docs/harness/security/anthropic-reference-harness.md",
    "docs/harness/README.md",
    "docs/harness/progress.md",
]
MANDATORY_AFTER_PLAN = ["AGENTS.md", "CLAUDE.md", "INVARIANTS.md", "STANDARDS.md", "ERRORS_AND_LESSONS.md"]


class TokenizerUnavailable(Exception):
    pass


def _blocked_socket(*_a, **_k):
    raise OSError("network access is blocked while loading the tokenizer")


def load_tiktoken():
    """Return a count function for cl100k_base, or raise TokenizerUnavailable (never uses the network)."""
    try:
        import tiktoken  # noqa: PLC0415
    except Exception as exc:  # ImportError or a broken install
        raise TokenizerUnavailable(f"tiktoken is not importable ({exc.__class__.__name__}: {exc})") from exc
    real_socket = socket.socket
    socket.socket = _blocked_socket  # type: ignore[assignment,misc]
    try:
        encoding = tiktoken.get_encoding("cl100k_base")
    except Exception as exc:
        raise TokenizerUnavailable(
            f"tiktoken cl100k_base vocabulary is not cached locally ({exc.__class__.__name__}); the tool never downloads it"
        ) from exc
    finally:
        socket.socket = real_socket  # type: ignore[misc]
    return lambda text: len(encoding.encode(text, disallowed_special=()))


def approx_count(text: str) -> int:
    return math.ceil(len(text.encode("utf-8")) / 4)


def choose_tokenizer(choice: str):
    """-> (count_fn, info dict). Raises TokenizerUnavailable only for choice == 'tiktoken'."""
    if choice == "approx":
        return approx_count, {"name": APPROX_NAME, "exact": False, "fallback_reason": "requested explicitly"}
    try:
        return load_tiktoken(), {"name": TIKTOKEN_NAME, "exact": True, "fallback_reason": None}
    except TokenizerUnavailable as exc:
        if choice == "tiktoken":
            raise
        return approx_count, {"name": APPROX_NAME, "exact": False, "fallback_reason": str(exc)}


def mandatory_files(root: Path) -> list[str]:
    plan = ""
    progress = root / "docs/harness/progress.md"
    if progress.is_file():
        m = re.search(r"(?m)^\|\s*Active plan\s*\|\s*`?([^`|]+?)`?\s*\|\s*$", progress.read_text(encoding="utf-8"))
        if m:
            plan = m.group(1).strip()
    return MANDATORY_BEFORE_PLAN + ([plan] if plan else []) + MANDATORY_AFTER_PLAN


def measure(root: Path, files: list[str], count) -> dict:
    rows, missing = [], []
    for rel in files:
        path = root / rel
        if not path.is_file():
            missing.append(rel)
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        rows.append({"path": rel, "bytes": len(text.encode("utf-8")), "lines": len(text.splitlines()), "tokens": count(text)})
    totals = {k: sum(r[k] for r in rows) for k in ("bytes", "lines", "tokens")}
    return {"files": rows, "missing": missing, "totals": totals}


def render_table(result: dict, tokenizer: dict) -> str:
    out = [f"TOKENIZER: {tokenizer['name']} (exact={str(tokenizer['exact']).lower()})"]
    if tokenizer.get("fallback_reason"):
        out.append(f"NOTE: {tokenizer['fallback_reason']}")
    out.append(f"{'bytes':>9} {'lines':>6} {'tokens':>8}  path")
    for r in result["files"]:
        out.append(f"{r['bytes']:>9} {r['lines']:>6} {r['tokens']:>8}  {r['path']}")
    t = result["totals"]
    out.append(f"{t['bytes']:>9} {t['lines']:>6} {t['tokens']:>8}  TOTAL ({len(result['files'])} files)")
    for rel in result["missing"]:
        out.append(f"MISSING: {rel}")
    return "\n".join(out)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Measure the harness mandatory read set or read one section")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--tokenizer", default="auto", choices=["auto", "tiktoken", "approx"])
    parser.add_argument("--root", default=str(Path(__file__).resolve().parent.parent.parent.parent))
    parser.add_argument("--files", nargs="+")
    parser.add_argument("--section", metavar="FILE.md#anchor")
    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:  # argparse exits 2 on bad usage (incl. unknown --tokenizer)
        return int(exc.code or 0)
    root = Path(args.root).resolve()

    if args.section:
        rel, _, anchor = args.section.partition("#")
        path = root / rel
        if not anchor or not path.is_file():
            print(f"section not found: {args.section} (file missing or no #anchor)", file=sys.stderr)
            return 1
        text = section_text(path, anchor)
        if text is None:
            print(f"section not found: #{anchor} in {rel}", file=sys.stderr)
            return 1
        sys.stdout.write(text)
        return 0

    try:
        count, info = choose_tokenizer(args.tokenizer)
    except TokenizerUnavailable as exc:
        print(f"tiktoken requested but unavailable: {exc}", file=sys.stderr)
        return 1
    result = measure(root, args.files or mandatory_files(root), count)
    result["tokenizer"] = info
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(render_table(result, info))
    return 1 if result["missing"] else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
