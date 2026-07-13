#!/usr/bin/env python3
"""Validate the aggregate security gate and its required-context chain."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


class CheckError(RuntimeError):
    """A security-gate contract violation."""


def load_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as exc:
        raise CheckError(f"could not read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise CheckError(f"{path}: root must be an object")
    return value


def parse_jobs(path: Path) -> dict[str, dict[str, object]]:
    """Parse the small job/name/needs subset used by the CI contract."""

    try:
        lines = path.read_text().splitlines()
    except OSError as exc:
        raise CheckError(f"could not read workflow {path}: {exc}") from exc
    jobs: dict[str, dict[str, object]] = {}
    in_jobs = False
    current: str | None = None
    collecting_needs = False
    for line in lines:
        if line == "jobs:":
            in_jobs = True
            continue
        if not in_jobs:
            continue
        match = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if match:
            current = match.group(1)
            jobs[current] = {"name": current, "needs": []}
            collecting_needs = False
            continue
        if current is None:
            continue
        name = re.match(r"^    name:\s*(.+?)\s*$", line)
        if name:
            jobs[current]["name"] = name.group(1).strip("'\"")
            continue
        inline = re.match(r"^    needs:\s*\[(.*?)\]\s*$", line)
        if inline:
            jobs[current]["needs"] = [
                item.strip().strip("'\"")
                for item in inline.group(1).split(",")
                if item.strip()
            ]
            collecting_needs = False
            continue
        if re.match(r"^    needs:\s*$", line):
            jobs[current]["needs"] = []
            collecting_needs = True
            continue
        dependency = re.match(r"^      -\s+([A-Za-z0-9_-]+)\s*$", line)
        if collecting_needs and dependency:
            needs = jobs[current]["needs"]
            assert isinstance(needs, list)
            needs.append(dependency.group(1))
            continue
        if collecting_needs and line.strip() and not line.startswith("      "):
            collecting_needs = False
    return jobs


def evaluate(results: dict[str, str], constituents: list[str], allowed: set[str]) -> bool:
    for job in constituents:
        result = results.get(job, "missing")
        if result == "success":
            continue
        if result == "skipped" and job in allowed:
            continue
        return False
    return True


def validate_matrix(matrix: dict) -> None:
    constituents = matrix.get("constituents")
    scenarios = matrix.get("scenarios")
    if not isinstance(constituents, list) or not constituents:
        raise CheckError("matrix needs a non-empty constituents list")
    if not all(isinstance(item, str) and item for item in constituents):
        raise CheckError("matrix constituent IDs must be non-empty strings")
    if not isinstance(scenarios, list) or not scenarios:
        raise CheckError("matrix needs scenarios")
    for scenario in scenarios:
        results = scenario.get("results", {})
        allowed = set(scenario.get("allowed_skips", []))
        expected = scenario.get("expected") == "success"
        if evaluate(results, constituents, allowed) != expected:
            raise CheckError(f"matrix scenario disagrees with gate policy: {scenario.get('name')}")
    baseline = {job: "success" for job in constituents}
    if not evaluate(baseline, constituents, set()):
        raise CheckError("all-success matrix must pass")
    for job in constituents:
        failed = dict(baseline)
        failed[job] = "failure"
        if evaluate(failed, constituents, set()):
            raise CheckError(f"constituent failure did not fail closed: {job}")


def required_context_names(payload: dict) -> set[str]:
    names = {item for item in payload.get("contexts", []) if isinstance(item, str)}
    for item in payload.get("checks", []):
        if isinstance(item, dict) and isinstance(item.get("context"), str):
            names.add(item["context"])
    return names


def has_path(jobs: dict[str, dict[str, object]], start: str, target: str) -> bool:
    pending = [start]
    seen: set[str] = set()
    while pending:
        job = pending.pop()
        if job == target:
            return True
        if job in seen:
            continue
        seen.add(job)
        details = jobs.get(job, {})
        dependencies = details.get("needs", [])
        if isinstance(dependencies, list):
            pending.extend(item for item in dependencies if isinstance(item, str))
    return False


def validate_workflow(
    matrix: dict, jobs: dict[str, dict[str, object]], workflow_text: str
) -> None:
    aggregate = matrix.get("aggregate_job")
    if aggregate not in jobs:
        raise CheckError(f"workflow missing aggregate job {aggregate}")
    needs = jobs[aggregate].get("needs", [])
    missing = set(matrix["constituents"]) - set(needs if isinstance(needs, list) else [])
    if missing:
        raise CheckError(f"aggregate does not depend on: {', '.join(sorted(missing))}")
    required_tokens = (
        "if: always()",
        "scripts/check-security-gate.py",
        "--results-json",
        "toJSON(needs)",
    )
    absent = [token for token in required_tokens if token not in workflow_text]
    if absent:
        raise CheckError(f"aggregate runtime enforcement missing: {', '.join(absent)}")


def validate_required_chain(matrix: dict, jobs: dict[str, dict[str, object]], payload: dict) -> None:
    required = required_context_names(payload)
    aggregate = str(matrix["aggregate_job"])
    candidates = [job for job, details in jobs.items() if details.get("name") in required]
    if not any(has_path(jobs, job, aggregate) for job in candidates):
        raise CheckError(
            "no live required context transitively depends on security-gate; "
            f"required={sorted(required)}"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--matrix", type=Path, required=True)
    parser.add_argument("--workflow", type=Path, default=Path(".github/workflows/ci.yml"))
    parser.add_argument("--required-contexts", type=Path)
    parser.add_argument("--results-json")
    parser.add_argument("--event", default="pull_request")
    parser.add_argument("--self-test-failure", action="store_true")
    parser.add_argument("--self-test-unrequired", action="store_true")
    args = parser.parse_args()
    try:
        matrix = load_json(args.matrix)
        validate_matrix(matrix)
        jobs = parse_jobs(args.workflow)
        workflow_text = args.workflow.read_text()
        validate_workflow(matrix, jobs, workflow_text)
        if args.results_json is not None:
            payload = json.loads(args.results_json)
            if not isinstance(payload, dict):
                raise CheckError("--results-json must be an object")
            results = {
                job: details.get("result", "missing")
                if isinstance(details, dict)
                else "missing"
                for job, details in payload.items()
            }
            allowed_by_event = matrix.get("allowed_skips_by_event", {})
            allowed = set(allowed_by_event.get(args.event, []))
            if not evaluate(results, matrix["constituents"], allowed):
                raise CheckError(f"security constituents failed for event {args.event}")
            print("security-gate runtime results: PASS")
            return 0
        if args.self_test_failure:
            for state in ("failure", "cancelled", "timed_out"):
                baseline = {job: "success" for job in matrix["constituents"]}
                baseline[matrix["constituents"][0]] = state
                if evaluate(baseline, matrix["constituents"], set()):
                    raise CheckError(f"{state} self-test unexpectedly passed")
            missing = {job: "success" for job in matrix["constituents"][1:]}
            if evaluate(missing, matrix["constituents"], set()):
                raise CheckError("missing-result self-test unexpectedly passed")
            print("security-gate failure self-test: PASS")
            return 0
        if args.self_test_unrequired:
            try:
                validate_required_chain(matrix, jobs, {"contexts": ["Format"]})
            except CheckError:
                print("security-gate unrequired-context self-test: PASS")
                return 0
            raise CheckError("unrequired-context self-test unexpectedly passed")
        if args.required_contexts is None:
            raise CheckError("--required-contexts is required outside self-test modes")
        validate_required_chain(matrix, jobs, load_json(args.required_contexts))
    except CheckError as exc:
        print(f"security-gate check failed: {exc}", file=sys.stderr)
        return 1
    print("security-gate contract: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
