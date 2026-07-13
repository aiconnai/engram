#!/usr/bin/env python3
"""Enforce deterministic retrieval floors and portable Criterion budgets."""

from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Sequence, Tuple


METRICS = ("recall@10", "mrr", "ndcg@10")
UNITS_TO_SECONDS = {
    "ns": 1e-9,
    "us": 1e-6,
    "µs": 1e-6,
    "ms": 1e-3,
    "s": 1.0,
}
TIME_RE = re.compile(
    r"^\s*time:\s+\[[0-9.eE+-]+\s+(?:ns|us|µs|ms|s)\s+"
    r"([0-9.eE+-]+)\s+(ns|us|µs|ms|s)\s+"
    r"[0-9.eE+-]+\s+(?:ns|us|µs|ms|s)\]"
)


class BudgetError(ValueError):
    """A named quality-budget contract violation."""


def load_json(path: Path, label: str) -> Dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise BudgetError(f"{label}: cannot load {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise BudgetError(f"{label}: root must be an object")
    return value


def require_object(parent: Mapping[str, Any], key: str, label: str) -> Dict[str, Any]:
    value = parent.get(key)
    if not isinstance(value, dict):
        raise BudgetError(f"{label}.{key}: must be an object")
    return value


def require_metric_map(parent: Mapping[str, Any], key: str, label: str) -> Dict[str, float]:
    raw = require_object(parent, key, label)
    if set(raw) != set(METRICS):
        raise BudgetError(f"{label}.{key}: must contain exactly {', '.join(METRICS)}")
    metrics: Dict[str, float] = {}
    for metric in METRICS:
        value = raw[metric]
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise BudgetError(f"{label}.{key}.{metric}: must be numeric")
        number = float(value)
        if not math.isfinite(number) or not 0.0 <= number <= 1.0:
            raise BudgetError(f"{label}.{key}.{metric}: must be within [0, 1]")
        metrics[metric] = number
    return metrics


def validate_update(
    budgets: Mapping[str, Any],
    floors: Mapping[str, float],
    observed: Mapping[str, float],
    hot_paths: Mapping[str, Any],
    prior_criterion: Mapping[str, float],
) -> None:
    update = require_object(budgets, "baseline_update", "budgets")
    revision = update.get("source_revision")
    if not isinstance(revision, str) or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        raise BudgetError("budgets.baseline_update.source_revision: must be a 40-character Git SHA")
    for key in ("reviewer", "rationale"):
        value = update.get(key)
        if not isinstance(value, str) or not value.strip():
            raise BudgetError(f"budgets.baseline_update.{key}: must be non-empty")
    evidence = update.get("evidence")
    if not isinstance(evidence, list) or not evidence or not all(
        isinstance(item, str) and item.strip() for item in evidence
    ):
        raise BudgetError("budgets.baseline_update.evidence: must contain reviewed evidence paths")
    before = require_metric_map(update, "before_metrics", "budgets.baseline_update")
    after = require_metric_map(update, "after_metrics", "budgets.baseline_update")
    for metric in METRICS:
        if not math.isclose(after[metric], floors[metric], rel_tol=0.0, abs_tol=1e-12):
            raise BudgetError(
                f"budgets.baseline_update.after_metrics.{metric}: must equal the committed floor"
            )
        if not math.isclose(after[metric], observed[metric], rel_tol=0.0, abs_tol=1e-12):
            raise BudgetError(
                f"budgets.baseline_update.after_metrics.{metric}: must equal the measured retrieval value"
            )
        if after[metric] < before[metric]:
            raise BudgetError(
                f"budget floor reduction: {metric} {before[metric]} -> {after[metric]}"
            )

    before_criterion = require_object(
        update, "before_criterion", "budgets.baseline_update"
    )
    after_criterion = require_object(
        update, "after_criterion", "budgets.baseline_update"
    )
    expected_names = set(hot_paths)
    for key, values in (
        ("before_criterion", before_criterion),
        ("after_criterion", after_criterion),
    ):
        if set(values) != expected_names:
            raise BudgetError(
                f"budgets.baseline_update.{key}: must contain exactly the named hot paths"
            )
    for name in sorted(expected_names):
        committed_seconds = parse_budget_timing(hot_paths[name], f"criterion baseline: {name}")
        before_seconds = parse_budget_timing(
            before_criterion[name], f"baseline update before_criterion: {name}"
        )
        after_seconds = parse_budget_timing(
            after_criterion[name], f"baseline update after_criterion: {name}"
        )
        if not math.isclose(
            after_seconds, committed_seconds, rel_tol=0.0, abs_tol=1e-18
        ):
            raise BudgetError(
                f"baseline update after_criterion: {name} must equal the committed baseline"
            )
        prior_seconds = prior_criterion.get(name)
        if prior_seconds is None or not math.isclose(
            before_seconds, prior_seconds, rel_tol=0.0, abs_tol=1e-18
        ):
            raise BudgetError(
                f"baseline update before_criterion: {name} must equal immutable source-revision evidence"
            )
        if after_seconds > before_seconds:
            raise BudgetError(
                f"criterion budget relaxation: {name} {before_seconds} -> {after_seconds} seconds"
            )


def parse_budget_timing(raw: Any, label: str) -> float:
    if not isinstance(raw, dict):
        raise BudgetError(f"{label}: must be an object")
    value = raw.get("baseline_value")
    unit = raw.get("baseline_unit")
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise BudgetError(f"{label}: must have a numeric baseline_value")
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise BudgetError(f"{label}: baseline_value must be finite and positive")
    if unit not in UNITS_TO_SECONDS:
        raise BudgetError(f"{label}: unsupported unit {unit!r}")
    return number * UNITS_TO_SECONDS[unit]


def parse_criterion(text: str, names: Sequence[str]) -> Dict[str, Tuple[float, str]]:
    lines = text.splitlines()
    parsed: Dict[str, Tuple[float, str]] = {}
    wanted = set(names)
    for index, line in enumerate(lines):
        name = line.strip()
        if name not in wanted:
            continue
        for candidate in lines[index + 1 : index + 6]:
            match = TIME_RE.match(candidate)
            if match is not None:
                value = float(match.group(1))
                if not math.isfinite(value) or value <= 0:
                    raise BudgetError(
                        f"criterion: {name} median must be finite and strictly positive"
                    )
                parsed[name] = (value, match.group(2))
                break
    missing = sorted(wanted - set(parsed))
    if missing:
        raise BudgetError(f"criterion: missing named hot path(s): {', '.join(missing)}")
    return parsed


def load_prior_criterion(budgets: Mapping[str, Any]) -> Dict[str, float]:
    update = require_object(budgets, "baseline_update", "budgets")
    revision = update.get("source_revision")
    criterion = require_object(budgets, "criterion", "budgets")
    source_path = criterion.get("baseline_source_path")
    if not isinstance(revision, str) or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
        raise BudgetError("budgets.baseline_update.source_revision: must be a 40-character Git SHA")
    if not isinstance(source_path, str) or not source_path.strip():
        raise BudgetError("budgets.criterion.baseline_source_path: must be non-empty")
    try:
        completed = subprocess.run(
            ["git", "show", f"{revision}:{source_path}"],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as exc:
        raise BudgetError(
            "criterion immutable baseline: cannot read source_revision evidence; "
            "fetch full Git history before running the checker"
        ) from exc

    hot_paths = require_object(criterion, "hot_paths", "budgets.criterion")
    parsed: Dict[str, float] = {}
    for line in completed.stdout.splitlines():
        name, separator, measurement = line.rpartition(": ")
        if not separator or name not in hot_paths:
            continue
        parts = measurement.split()
        if len(parts) != 2 or parts[1] not in UNITS_TO_SECONDS:
            continue
        try:
            value = float(parts[0])
        except ValueError:
            continue
        if not math.isfinite(value) or value <= 0:
            raise BudgetError(
                f"criterion immutable baseline: {name} must be finite and positive"
            )
        parsed[name] = value * UNITS_TO_SECONDS[parts[1]]
    missing = sorted(set(hot_paths) - set(parsed))
    if missing:
        raise BudgetError(
            "criterion immutable baseline: missing named hot path(s): " + ", ".join(missing)
        )
    return parsed


def validate(
    budgets: Mapping[str, Any],
    retrieval: Mapping[str, Any],
    criterion_text: str,
    prior_criterion: Mapping[str, float],
) -> Dict[str, Any]:
    if budgets.get("schema_version") != "engram.quality-budgets.v1":
        raise BudgetError("budgets.schema_version: expected engram.quality-budgets.v1")
    if retrieval.get("schema_version") != "engram.quality-baseline.v1":
        raise BudgetError("retrieval.schema_version: expected engram.quality-baseline.v1")
    update = require_object(budgets, "baseline_update", "budgets")
    if update.get("source_revision") != retrieval.get("source_revision"):
        raise BudgetError(
            "budgets.baseline_update.source_revision: must match the frozen retrieval source revision"
        )

    retrieval_policy = require_object(budgets, "retrieval", "budgets")
    floors = require_metric_map(retrieval_policy, "floors", "budgets.retrieval")
    observed = require_metric_map(retrieval, "metrics", "retrieval")

    retrieval_report = []
    for metric in METRICS:
        if observed[metric] < floors[metric]:
            raise BudgetError(
                f"retrieval floor: {metric} observed {observed[metric]} below {floors[metric]}"
            )
        retrieval_report.append(
            {"metric": metric, "floor": floors[metric], "observed": observed[metric]}
        )
    criterion = require_object(budgets, "criterion", "budgets")
    maximum = criterion.get("maximum_regression_ratio")
    if isinstance(maximum, bool) or not isinstance(maximum, (int, float)) or maximum != 1.15:
        raise BudgetError("budgets.criterion.maximum_regression_ratio: expected 1.15")
    hot_paths = require_object(criterion, "hot_paths", "budgets.criterion")
    if not hot_paths:
        raise BudgetError("budgets.criterion.hot_paths: must not be empty")
    validate_update(budgets, floors, observed, hot_paths, prior_criterion)
    parsed = parse_criterion(criterion_text, list(hot_paths))

    performance_report = []
    for name, raw_baseline in hot_paths.items():
        observed_value, observed_unit = parsed[name]
        baseline_seconds = parse_budget_timing(raw_baseline, f"criterion baseline: {name}")
        observed_seconds = observed_value * UNITS_TO_SECONDS[observed_unit]
        ratio = observed_seconds / baseline_seconds
        if ratio > float(maximum) + 1e-12:
            raise BudgetError(
                f"criterion regression: {name} ratio {ratio:.4f} exceeds {float(maximum):.2f}"
            )
        performance_report.append(
            {
                "path": name,
                "baseline_seconds": baseline_seconds,
                "observed_seconds": observed_seconds,
                "ratio": ratio,
                "maximum_ratio": float(maximum),
            }
        )

    return {
        "schema_version": "engram.quality-budget-report.v1",
        "status": "pass",
        "retrieval": retrieval_report,
        "criterion": performance_report,
    }


def run_self_test(
    budgets: Mapping[str, Any],
    retrieval: Mapping[str, Any],
    criterion_text: str,
    prior_criterion: Mapping[str, float],
) -> Dict[str, Any]:
    validate(budgets, retrieval, criterion_text, prior_criterion)
    degraded_retrieval = json.loads(json.dumps(retrieval))
    degraded_retrieval["metrics"]["mrr"] = max(
        0.0, float(budgets["retrieval"]["floors"]["mrr"]) - 0.01
    )
    try:
        validate(budgets, degraded_retrieval, criterion_text, prior_criterion)
    except BudgetError as exc:
        retrieval_error = str(exc)
        if "retrieval floor: mrr" not in retrieval_error:
            raise BudgetError(f"self-test degraded retrieval returned wrong error: {exc}") from exc
    else:
        raise BudgetError("self-test degraded retrieval unexpectedly passed")

    hot_paths = budgets["criterion"]["hot_paths"]
    name = next(iter(hot_paths))
    baseline = hot_paths[name]
    seconds = (
        float(baseline["baseline_value"])
        * UNITS_TO_SECONDS[baseline["baseline_unit"]]
        * 1.16
    )
    synthetic = f"{name}\n                        time:   [{seconds} s {seconds} s {seconds} s]\n"
    for other_name, other in list(hot_paths.items())[1:]:
        other_seconds = float(other["baseline_value"]) * UNITS_TO_SECONDS[other["baseline_unit"]]
        synthetic += (
            f"{other_name}\n"
            f"                        time:   [{other_seconds} s {other_seconds} s {other_seconds} s]\n"
        )
    try:
        validate(budgets, retrieval, synthetic, prior_criterion)
    except BudgetError as exc:
        criterion_error = str(exc)
        if f"criterion regression: {name}" not in criterion_error:
            raise BudgetError(f"self-test 116% regression returned wrong error: {exc}") from exc
    else:
        raise BudgetError("self-test 116% regression unexpectedly passed")

    invalid = "".join(
        f"{path}\n                        time:   [-1 s -1 s -1 s]\n"
        for path in hot_paths
    )
    try:
        validate(budgets, retrieval, invalid, prior_criterion)
    except BudgetError as exc:
        invalid_timing_error = str(exc)
        if "strictly positive" not in invalid_timing_error:
            raise BudgetError(f"self-test invalid timing returned wrong error: {exc}") from exc
    else:
        raise BudgetError("self-test negative Criterion timing unexpectedly passed")

    relaxed_budgets = json.loads(json.dumps(budgets))
    relaxed_name = next(iter(hot_paths))
    relaxed_budgets["criterion"]["hot_paths"][relaxed_name]["baseline_value"] *= 1000
    relaxed_budgets["baseline_update"]["after_criterion"][relaxed_name][
        "baseline_value"
    ] *= 1000
    relaxed_budgets["baseline_update"]["before_criterion"][relaxed_name][
        "baseline_value"
    ] *= 1000
    try:
        validate(relaxed_budgets, retrieval, criterion_text, prior_criterion)
    except BudgetError as exc:
        relaxation_error = str(exc)
        if (
            f"baseline update before_criterion: {relaxed_name} must equal immutable"
            not in relaxation_error
        ):
            raise BudgetError(f"self-test budget relaxation returned wrong error: {exc}") from exc
    else:
        raise BudgetError("self-test Criterion budget relaxation unexpectedly passed")

    return {
        "schema_version": "engram.quality-budget-self-test.v1",
        "status": "pass",
        "cases": [
            {"case": "retrieval_metric_decrement", "blocked_by": retrieval_error},
            {"case": "criterion_116_percent", "blocked_by": criterion_error},
            {"case": "criterion_negative_timing", "blocked_by": invalid_timing_error},
            {"case": "criterion_budget_relaxation", "blocked_by": relaxation_error},
        ],
    }


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--budgets", type=Path, required=True)
    parser.add_argument("--retrieval", type=Path, required=True)
    parser.add_argument("--criterion", type=Path, required=True)
    parser.add_argument("--self-test-degraded", action="store_true")
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        budgets = load_json(args.budgets, "budgets")
        retrieval = load_json(args.retrieval, "retrieval")
        retrieval_policy = require_object(budgets, "retrieval", "budgets")
        configured_path = retrieval_policy.get("baseline_path")
        if configured_path != args.retrieval.as_posix():
            raise BudgetError(
                "budgets.retrieval.baseline_path: must match the checked retrieval artifact"
            )
        update = require_object(budgets, "baseline_update", "budgets")
        evidence = update.get("evidence")
        if isinstance(evidence, list):
            missing_evidence = [
                item
                for item in evidence
                if isinstance(item, str) and not Path(item).is_file()
            ]
            if missing_evidence:
                raise BudgetError(
                    "budgets.baseline_update.evidence: missing path(s): "
                    + ", ".join(missing_evidence)
                )
        criterion_text = args.criterion.read_text(encoding="utf-8")
        prior_criterion = load_prior_criterion(budgets)
        if args.self_test_degraded:
            report = run_self_test(budgets, retrieval, criterion_text, prior_criterion)
        else:
            report = validate(budgets, retrieval, criterion_text, prior_criterion)
    except (BudgetError, OSError) as exc:
        print(json.dumps({"status": "fail", "error": str(exc)}, sort_keys=True))
        return 1
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
