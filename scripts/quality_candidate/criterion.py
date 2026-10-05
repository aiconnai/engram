"""Criterion evidence bound to one candidate.

``capture-criterion-candidate.py`` writes a marker header in front of the raw
Criterion output; the runner refuses any file without a matching marker. The
marker binds: candidate SHA and tree, normalized features, toolchain, a
supervisor id, a UTC capture time, the bench argv and a SHA256 of the body.
It is tamper-evident against stale files, other candidates and hand-edited
numbers, not an authentication mechanism: trust comes from capture and runner
executing inside the same supervised job.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from datetime import timedelta
from pathlib import Path
from typing import Any, Dict, List, Mapping, Tuple

from .common import (
    CRITERION_CEILING,
    FUTURE_SKEW_SECONDS,
    MAX_CRITERION_AGE_SECONDS,
    SHA256_RE,
    SHA_RE,
    CandidateError,
    format_utc,
    parse_utc,
    sha256_bytes,
    utc_now,
)

MAGIC = "# engram-criterion-candidate v1"
SEPARATOR = "# ---"
REQUIRED_KEYS = (
    "candidate_sha",
    "candidate_tree",
    "features",
    "toolchain",
    "supervisor",
    "captured_at",
    "bench_argv",
    "bench_params",
    "hardware",
    "body_sha256",
)
JSON_KEYS = ("bench_argv", "bench_params", "hardware")


def render(meta: Mapping[str, Any], body: str) -> str:
    """Build a marked Criterion file (used by capture and by tests)."""
    lines = [MAGIC]
    for key in REQUIRED_KEYS:
        if key == "body_sha256":
            value = sha256_bytes(body.encode("utf-8"))
        else:
            value = meta[key]
        if key in JSON_KEYS:
            value = json.dumps(value, sort_keys=True, separators=(",", ":"))
        lines.append(f"# {key}={value}")
    lines.append(SEPARATOR)
    return "\n".join(lines) + "\n" + body


def parse(text: str) -> Tuple[Dict[str, Any], str]:
    lines = text.split("\n")
    if not lines or lines[0] != MAGIC:
        raise CandidateError(
            "criterion: no candidate marker (expected first line "
            f"{MAGIC!r}); a bare or historical Criterion file is not candidate evidence"
        )
    meta: Dict[str, Any] = {}
    index = 1
    while index < len(lines) and lines[index] != SEPARATOR:
        line = lines[index]
        if not line.startswith("# ") or "=" not in line:
            raise CandidateError(f"criterion: malformed marker line {line!r}")
        key, _, value = line[2:].partition("=")
        if key in meta:
            raise CandidateError(f"criterion: duplicate marker key {key}")
        meta[key] = value
        index += 1
    if index >= len(lines):
        raise CandidateError("criterion: marker is not terminated by the separator line")
    missing = [key for key in REQUIRED_KEYS if key not in meta]
    if missing:
        raise CandidateError(f"criterion: marker missing key(s) {', '.join(missing)}")
    for key in JSON_KEYS:
        try:
            meta[key] = json.loads(meta[key])
        except json.JSONDecodeError as exc:
            raise CandidateError(f"criterion: marker {key} is not JSON") from exc
    return meta, "\n".join(lines[index + 1 :])


def verify(text: str, expected: Mapping[str, str]) -> Tuple[Dict[str, Any], str]:
    """Check the marker against the runner's own view of the candidate."""
    meta, body = parse(text)
    for key in ("candidate_sha", "candidate_tree", "features", "toolchain", "supervisor"):
        if meta[key] != expected[key]:
            raise CandidateError(
                f"criterion bound to a different {key}: marker {meta[key]!r}, "
                f"candidate {expected[key]!r}"
            )
    if not SHA_RE.fullmatch(meta["candidate_sha"]) or not SHA256_RE.fullmatch(meta["body_sha256"]):
        raise CandidateError("criterion: marker contains malformed hashes")
    if sha256_bytes(body.encode("utf-8")) != meta["body_sha256"]:
        raise CandidateError("criterion: body does not match the marker hash (edited after capture)")
    captured = parse_utc(meta["captured_at"], "criterion marker captured_at")
    now = utc_now()
    if captured - now > timedelta(seconds=FUTURE_SKEW_SECONDS):
        raise CandidateError(f"criterion: captured_at {meta['captured_at']} is in the future")
    if now - captured > timedelta(seconds=MAX_CRITERION_AGE_SECONDS):
        raise CandidateError(
            f"criterion is stale: captured {meta['captured_at']}, "
            f"older than {MAX_CRITERION_AGE_SECONDS // 3600}h"
        )
    return meta, body


def _load_budgets_module(repo: Path, script: str):
    path = repo / script
    spec = importlib.util.spec_from_file_location("check_quality_budgets", path)
    if spec is None or spec.loader is None:
        raise CandidateError(f"cannot load {script}")
    module = importlib.util.module_from_spec(spec)
    # Importing must not leave bytecode that would make the checkout look dirty.
    previous, sys.dont_write_bytecode = sys.dont_write_bytecode, True
    try:
        spec.loader.exec_module(module)  # type: ignore[union-attr]
    finally:
        sys.dont_write_bytecode = previous
    return module


def compare_hot_paths(repo: Path, script: str, body: str, budgets: Mapping[str, Any]) -> List[Dict[str, Any]]:
    """Compare candidate medians with the budgets.json baseline at the 1.15 ceiling."""
    module = _load_budgets_module(repo, script)
    criterion = budgets.get("criterion")
    if not isinstance(criterion, dict):
        raise CandidateError("budgets.criterion: must be an object")
    ceiling = criterion.get("maximum_regression_ratio")
    if isinstance(ceiling, bool) or not isinstance(ceiling, (int, float)) or ceiling != CRITERION_CEILING:
        raise CandidateError(f"budgets.criterion.maximum_regression_ratio: expected {CRITERION_CEILING}")
    hot_paths = criterion.get("hot_paths")
    if not isinstance(hot_paths, dict) or not hot_paths:
        raise CandidateError("budgets.criterion.hot_paths: must not be empty")
    try:
        parsed = module.parse_criterion(body, list(hot_paths))
    except module.BudgetError as exc:
        raise CandidateError(str(exc)) from exc
    report = []
    for name, raw in hot_paths.items():
        try:
            baseline = module.parse_budget_timing(raw, f"criterion baseline: {name}")
        except module.BudgetError as exc:
            raise CandidateError(str(exc)) from exc
        value, unit = parsed[name]
        observed = value * module.UNITS_TO_SECONDS[unit]
        ratio = observed / baseline
        if ratio > CRITERION_CEILING + 1e-12:
            raise CandidateError(
                f"criterion regression: {name} ratio {ratio:.4f} exceeds {CRITERION_CEILING:.2f}"
            )
        report.append(
            {
                "path": name,
                "baseline_seconds": baseline,
                "observed_seconds": observed,
                "ratio": ratio,
                "maximum_ratio": CRITERION_CEILING,
            }
        )
    return report


def now_stamp() -> str:
    return format_utc(utc_now())
