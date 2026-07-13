#!/usr/bin/env python3
"""Validate candidate and publication receipts without performing writes."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path

SHA_RE = re.compile(r"^[0-9a-f]{40}$")
TASK_RE = re.compile(r"^[0-9]+$")


class ReceiptError(ValueError):
    pass


def parse_receipt(path: Path) -> dict[str, str]:
    if not path.is_file():
        raise ReceiptError(f"receipt not found: {path}")
    values: dict[str, str] = {}
    for raw in path.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip().lower().replace("-", "_")
        value = value.strip().strip("`")
        if key in values:
            raise ReceiptError(f"duplicate field {key!r} in {path}")
        values[key] = value
    return values


def require_fields(values: dict[str, str], fields: set[str], path: Path) -> None:
    missing = sorted(field for field in fields if not values.get(field))
    if missing:
        raise ReceiptError(f"{path} missing fields: {', '.join(missing)}")


def validate_timestamp(value: str) -> None:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise ReceiptError("timestamp must be RFC3339") from exc
    if parsed.tzinfo is None:
        raise ReceiptError("timestamp must include a timezone")


def validate_publication_receipt(
    path: Path, *, task: str, decision: str, manifest: dict[str, object] | None
) -> dict[str, str]:
    values = parse_receipt(path)
    common = {"schema_version", "task", "channel", "decision", "timestamp"}
    require_fields(values, common, path)
    if values["schema_version"] != "engram-publication-receipt-v1":
        raise ReceiptError(f"{path} has unsupported schema")
    if values["task"] != task or values["decision"] != decision:
        raise ReceiptError(f"{path} task/decision does not match its filename")
    validate_timestamp(values["timestamp"])
    if decision == "approval":
        require_fields(values, {"version", "sha", "exact_user_text"}, path)
        if not SHA_RE.fullmatch(values["sha"]):
            raise ReceiptError(f"{path} has invalid sha")
        if manifest is not None:
            manifest_sha = str(manifest.get("sha", ""))
            if values["sha"] != manifest_sha:
                raise ReceiptError(f"{path} approval SHA differs from manifest")
    else:
        require_fields(values, {"reason", "external_write"}, path)
        if values["external_write"].lower() != "none":
            raise ReceiptError(f"{path} deferral must record external_write: none")
    return values


def publication_state(
    evidence: Path, task: str, required: str | None, manifest_path: Path | None
) -> dict[str, str]:
    if not TASK_RE.fullmatch(task):
        raise ReceiptError("publication task must be numeric")
    manifest = json.loads(manifest_path.read_text()) if manifest_path else None
    approval = evidence / f"task-{task}-publication-approval.md"
    deferral = evidence / f"task-{task}-publication-deferral.md"
    present = [("approval", approval), ("deferral", deferral)]
    present = [(kind, path) for kind, path in present if path.exists()]
    if len(present) != 1:
        raise ReceiptError(f"task {task} must have exactly one approval XOR deferral receipt")
    decision, path = present[0]
    if required and decision != required:
        raise ReceiptError(f"task {task} requires {required}, found {decision}")
    return validate_publication_receipt(
        path, task=task, decision=decision, manifest=manifest
    )


def expand_tasks(spec: str) -> list[int]:
    result: list[int] = []
    for item in filter(None, (part.strip() for part in spec.split(","))):
        if "-" in item:
            start_text, end_text = item.split("-", 1)
            start, end = int(start_text), int(end_text)
            if start > end:
                raise ReceiptError(f"invalid descending range: {item}")
            result.extend(range(start, end + 1))
        else:
            result.append(int(item))
    if len(set(result)) != len(result):
        raise ReceiptError("receipt task list contains duplicates")
    return result


def git_output(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def build_manifest(args: argparse.Namespace) -> None:
    evidence = args.evidence_dir
    approval_tasks = set(expand_tasks(args.publication_approvals or ""))
    deferral_tasks = set(expand_tasks(args.publication_deferrals or ""))
    overlap = approval_tasks & deferral_tasks
    if overlap:
        raise ReceiptError(
            "publication tasks cannot be declared as both approval and deferral: "
            + ", ".join(str(task) for task in sorted(overlap))
        )
    publication_tasks = approval_tasks | deferral_tasks
    receipts: list[dict[str, object]] = []
    for task in expand_tasks(args.receipts):
        if task in publication_tasks:
            required_decision = "approval" if task in approval_tasks else "deferral"
            state = publication_state(
                evidence, str(task), required_decision, None
            )
            if state["decision"] == "deferral" and not args.allow_deferred:
                raise ReceiptError(f"task {task} is deferred but --allow-deferred is absent")
            candidates = [
                evidence / f"task-{task}-publication-{state['decision']}.md"
            ]
        else:
            candidates = [evidence / f"task-{task}-engram-10-of-10.md"]
        for path in candidates:
            if not path.is_file():
                raise ReceiptError(f"receipt not found: {path}")
            receipts.append(
                {
                    "task": task,
                    "path": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                }
            )
    cargo_version = "unknown"
    cargo = Path("Cargo.toml")
    if cargo.is_file():
        match = re.search(r'^version\s*=\s*"([^"]+)"', cargo.read_text(), re.M)
        if match:
            cargo_version = match.group(1)
    output = {
        "schema_version": "engram-candidate-manifest-v1",
        "sha": git_output("rev-parse", "HEAD"),
        "base_sha": git_output("merge-base", "HEAD", "origin/main"),
        "candidate_ref": git_output("branch", "--show-current"),
        "version": cargo_version,
        "receipts": receipts,
        "statement": "THIS RECEIPT DOES NOT AUTHORIZE PUBLICATION",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n")


def write_receipt(path: Path, task: str, decision: str, sha: str = "0" * 40) -> None:
    fields = [
        "schema_version: engram-publication-receipt-v1",
        f"task: {task}",
        "channel: GitHub",
        f"decision: {decision}",
        "timestamp: 2026-07-13T00:00:00Z",
    ]
    if decision == "approval":
        fields += ["version: v0.22.0", f"sha: {sha}", "exact_user_text: approved"]
    else:
        fields += ["reason: approval not granted", "external_write: none"]
    path.write_text("\n".join(fields) + "\n")


def self_test(mode: str) -> None:
    with tempfile.TemporaryDirectory() as raw:
        evidence = Path(raw)
        manifest = evidence / "manifest.json"
        manifest.write_text(json.dumps({"sha": "0" * 40}) + "\n")
        if mode == "approval":
            write_receipt(evidence / "task-36-publication-approval.md", "36", "approval")
            state = publication_state(evidence, "36", "approval", manifest)
            assert state["decision"] == "approval"
        elif mode == "deferral":
            write_receipt(evidence / "task-36-publication-deferral.md", "36", "deferral")
            state = publication_state(evidence, "36", "deferral", manifest)
            assert state["external_write"] == "none"
        else:
            write_receipt(evidence / "task-36-publication-approval.md", "36", "approval")
            write_receipt(evidence / "task-36-publication-deferral.md", "36", "deferral")
            try:
                publication_state(evidence, "36", None, manifest)
            except ReceiptError:
                pass
            else:
                raise AssertionError("conflicting publication state was accepted")
            (evidence / "task-36-publication-approval.md").unlink()
            try:
                publication_state(evidence, "36", "approval", manifest)
            except ReceiptError:
                pass
            else:
                raise AssertionError("deferral was accepted as an approval")
    print(f"check-candidate-receipts {mode} self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-dir", type=Path, default=Path(".omo/evidence"))
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--self-test-publication-deferral", action="store_true")
    parser.add_argument("--self-test-conflicting-publication-state", action="store_true")
    parser.add_argument("--publication-state")
    parser.add_argument("--require", choices=("approval", "deferral"))
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--final", action="store_true")
    parser.add_argument("--receipts")
    parser.add_argument("--publication-approvals")
    parser.add_argument("--publication-deferrals")
    parser.add_argument("--allow-deferred", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        modes = sum(
            bool(value)
            for value in (
                args.self_test,
                args.self_test_publication_deferral,
                args.self_test_conflicting_publication_state,
                args.publication_state,
                args.final,
            )
        )
        if modes != 1:
            raise ReceiptError("select exactly one validation mode")
        if args.self_test:
            self_test("approval")
        elif args.self_test_publication_deferral:
            self_test("deferral")
        elif args.self_test_conflicting_publication_state:
            self_test("conflict")
        elif args.publication_state:
            publication_state(
                args.evidence_dir, args.publication_state, args.require, args.manifest
            )
            print("publication receipt: PASS")
        else:
            if not args.receipts or not args.output:
                raise ReceiptError("--final requires --receipts and --output")
            build_manifest(args)
            print(f"candidate manifest written: {args.output}")
    except (ReceiptError, OSError, json.JSONDecodeError, subprocess.CalledProcessError) as exc:
        print(f"check-candidate-receipts: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
