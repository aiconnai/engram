#!/usr/bin/env python3
"""Aggregate per-target results of an optional nightly lane into one report.

Used by scripts/run-fuzz-smoke.sh and scripts/run-miri-smoke.sh. Statuses are
explicit and never collapse into each other:

  pass         the target executed and succeeded
  fail         the target (or the lane contract) failed
  not-run      the lane could not run (tool/infrastructure unavailable)
  unsupported  the platform cannot run the lane

Overall precedence: fail > not-run > unsupported > pass. `pass` requires at
least one row, every row `pass`, and no lane-level error. The process exit code
mirrors the overall status (pass 0, fail 1, not-run 2, unsupported 3), so a
non-pass can never be turned into a green job.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

STATUSES = ("pass", "fail", "not-run", "unsupported")
EXIT_CODES = {"pass": 0, "fail": 1, "not-run": 2, "unsupported": 3}
ROW_FIELDS = (
    "target",
    "status",
    "stage",
    "exit_code",
    "elapsed_secs",
    "executed",
    "corpus_files",
    "seed_files",
    "reproducer",
    "note",
)
INT_FIELDS = ("elapsed_secs", "executed", "corpus_files", "seed_files")


def parse_rows(text: str) -> list[dict]:
    rows = []
    for number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        cells = line.split("\t")
        if len(cells) != len(ROW_FIELDS):
            raise ValueError(
                f"row {number}: expected {len(ROW_FIELDS)} tab-separated fields, got {len(cells)}"
            )
        row = dict(zip(ROW_FIELDS, cells))
        if row["status"] not in STATUSES:
            raise ValueError(f"row {number}: unknown status {row['status']!r}")
        for field in INT_FIELDS:
            row[field] = int(row[field]) if row[field].lstrip("-").isdigit() else None
        rows.append(row)
    return rows


def overall_status(rows: list[dict], lane_errors: list[str], lane_status: str | None) -> str:
    """`lane_status` is an explicit lane-wide not-run/unsupported verdict."""
    if lane_errors or any(row["status"] == "fail" for row in rows):
        return "fail"
    if lane_status in ("not-run", "unsupported"):
        return lane_status
    if not rows:
        return "fail"  # an empty lane must never read as success
    for status in ("not-run", "unsupported"):
        if any(row["status"] == status for row in rows):
            return status
    return "pass"


def build_report(lane: str, rows: list[dict], lane_errors: list[str], lane_status: str | None, settings: dict) -> dict:
    errors = list(lane_errors)
    if not rows and not errors and lane_status is None:
        errors.append("no targets were reported (empty lane is not a pass)")
    counts = {status: sum(1 for row in rows if row["status"] == status) for status in STATUSES}
    return {
        "lane": lane,
        "status": overall_status(rows, errors, lane_status),
        "lane_status": lane_status,
        "lane_errors": errors,
        "counts": counts,
        "targets": rows,
        "settings": settings,
    }


def render_markdown(report: dict) -> str:
    lines = [
        f"# {report['lane']} report: {report['status'].upper()}",
        "",
        "counts: " + ", ".join(f"{k}={v}" for k, v in report["counts"].items()),
    ]
    for error in report["lane_errors"]:
        lines.append(f"- lane error: {error}")
    if report["lane_status"]:
        lines.append(f"- lane status: {report['lane_status']}")
    lines += ["", "| target | status | stage | exit | elapsed_s | executed | corpus | seeds | reproducer | note |", "|---|---|---|---|---|---|---|---|---|---|"]
    for row in report["targets"]:
        cells = [row["target"], row["status"], row["stage"], row["exit_code"], row["elapsed_secs"], row["executed"],
                 row["corpus_files"], row["seed_files"], row["reproducer"], row["note"]]
        lines.append("| " + " | ".join("" if c is None else str(c) for c in cells) + " |")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--lane", required=True)
    parser.add_argument("--rows", required=True, help="TSV file: " + ", ".join(ROW_FIELDS))
    parser.add_argument("--out", required=True, help="output directory")
    parser.add_argument("--lane-error", action="append", default=[], help="lane-level contract violation (forces fail)")
    parser.add_argument("--lane-status", choices=("not-run", "unsupported"), help="lane-wide verdict when nothing could run")
    parser.add_argument("--setting", action="append", default=[], metavar="KEY=VALUE")
    args = parser.parse_args(argv)

    try:
        rows = parse_rows(Path(args.rows).read_text(encoding="utf-8")) if Path(args.rows).exists() else []
    except ValueError as exc:
        print(f"optional_lane_report: invalid rows: {exc}", file=sys.stderr)
        return EXIT_CODES["fail"]
    settings = dict(item.split("=", 1) for item in args.setting if "=" in item)
    report = build_report(args.lane, rows, args.lane_error, args.lane_status, settings)

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / f"{args.lane}-report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    markdown = render_markdown(report)
    (out / f"{args.lane}-report.md").write_text(markdown, encoding="utf-8")
    sys.stdout.write(markdown)
    return EXIT_CODES[report["status"]]


if __name__ == "__main__":
    sys.exit(main())
