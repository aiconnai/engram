#!/usr/bin/env python3
"""record-evidence.py - external, post-commit evidence of the trusted runner (task H4).

record(): called IN-PROCESS by the supervisor (run-task.py) after a gate run. It reads only
supervisor-owned inputs - the sandbox adapter's outcome.json, the raw docker-attach log files under
run/logs, the staged gate task / registry / catalog - recomputes every hash from bytes, and writes an
evidence-v2 document, a run record and a receipt into the gate root. Nothing the writer produced (its
stdout, files in its workspace, a "PASS" it prints) is ever an input. There is no CLI for recording.

verify(): the only way evidence is accepted. Fail-closed; every check below must hold:
  * the runs root is trusted: absolute, a real directory owned by us, not group/world writable, marked
    by the runner (.engram-runs-root), not nested inside another runs root (a writer workspace is), and
    outside every worktree and git dir of the repository;
  * the receipt sits exactly at <runs_root>/<task_id>/<run_id>/a<N>/g<M>/receipt.json with no symlink
    and no group/world-writable directory on the way (writer workspaces are .../a<N>/ws/ and gate
    checkouts .../a<N>/c<M>/, so nothing a writer or a check could write ever has that shape);
  * the receipt names the candidate the operator expects; evidence.json matches the receipt hash;
  * git (neutralized): the candidate commit exists, its tree is the recorded tree, its only parent is
    the base, and the runner's candidate ref still points at it;
  * validate-evidence.py (H2) accepts the evidence with every expectation supplied (candidate, base,
    tree, policy version and catalog hash from the TRUSTED registry / catalog, logs dir, gate task),
    i.e. the equivalent of --require-expectations; log hashes are recomputed from bytes;
  * every sha256_manifest entry is recomputed; staged registry/catalog equal the trusted copies;
  * recorder identity, verdict pass, and the post-gate checkout tree equal to the candidate tree.

Integrity scope: this protects against the sandboxed writer and against drift / tampering after the
gate. It is not a signature: a host process running as the same user can rewrite a whole bundle.
CLI: record-evidence.py verify ... ; exit 0 VERIFIED, 1 REFUSED, 2 usage. Final line
`EVIDENCE: VERIFIED candidate=<sha>` or `EVIDENCE: REFUSED reason=<code> <detail>`.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
import os
import re
import stat
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple


def _load_sibling(name: str, module_name: Optional[str] = None) -> Any:
    path = Path(__file__).resolve().parent / f"{name}.py"
    module_name = module_name or name
    cached = sys.modules.get(module_name)
    if cached is not None and getattr(cached, "__file__", None) == str(path):
        return cached
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


HG = _load_sibling("harness_git")
VE = _load_sibling("validate-evidence", "validate_evidence")

HARNESS_DIR = Path(__file__).resolve().parent.parent
SCHEMAS_DIR = HARNESS_DIR / "schemas"
DEFAULT_REGISTRY = HARNESS_DIR / "checks" / "registry.json"
DEFAULT_CATALOG = SCHEMAS_DIR / "check-catalog-v1.json"

RECORDER_ID = "engram-runner/record-evidence-v1"
RECEIPT_VERSION = "runner-receipt-v1"
RUN_RECORD_VERSION = "runner-run-record-v1"
CANDIDATE_REF_PREFIX = "refs/engram-runner/candidates/"
RUNS_ROOT_MARKER = ".engram-runs-root"
TASK_ID_RE = re.compile(r"[a-zA-Z0-9_-]{1,128}")
RUN_ID_RE = re.compile(r"[0-9]{8}T[0-9]{6}Z-[0-9a-f]{8}")
ATTEMPT_RE, GATE_RE = re.compile(r"a[1-9][0-9]{0,2}"), re.compile(r"g[0-9]{1,3}")
HEX40 = re.compile(r"[0-9a-f]{40}")
RECEIPT_KEYS = {
    "receipt_version", "recorder", "task_id", "run_id", "base_sha", "candidate_sha", "candidate_ref", "tree_sha",
    "post_gate_tree_sha", "verdict", "evidence", "evidence_sha256", "recorded_at",
}
STATUS_MAP = {"passed": "pass", "failed": "fail", "timeout": "timeout", "error": "fail"}
LOG_CAPTURE = "raw docker-attach stdout/stderr bytes captured by the supervisor; unfiltered (no RTK)"


class RecordError(Exception):
    """Evidence could not be recorded faithfully. The run is not a pass."""


class VerifyRefused(Exception):
    def __init__(self, code: str, detail: str = ""):
        super().__init__(f"{code}: {detail}")
        self.code, self.detail = code, detail


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def now_utc() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def _write_exclusive(path: Path, payload: Mapping[str, Any]) -> bytes:
    data = (json.dumps(payload, indent=2, sort_keys=True) + "\n").encode("utf-8")
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as handle:
        handle.write(data)
    os.chmod(path, 0o400)
    return data


def _duration_ms(started: Optional[str], finished: Optional[str]) -> int:
    if not started or not finished:
        return 0
    fmt = "%Y-%m-%dT%H:%M:%S.%fZ"
    try:
        delta = datetime.datetime.strptime(finished, fmt) - datetime.datetime.strptime(started, fmt)
    except ValueError:
        return 0
    return max(0, min(int(delta.total_seconds() * 1000), 86_400_000))


def _command_label(argv: Sequence[str]) -> str:
    label = json.dumps(list(argv), ensure_ascii=True)
    if len(label) > 512:
        raise RecordError("check argv is longer than the evidence command field (512)")
    return label


# --- record ----------------------------------------------------------------------------------------

def _load_json(path: Path) -> Any:
    data, errors = VE.read_json_file(path)
    if errors:
        raise RecordError(f"{path.name}: {errors[0]}")
    return data


def _log_entry(gate_root: Path, logs_dir: Path, record: Mapping[str, Any], stream: str) -> Optional[Tuple[str, str]]:
    """(bundle-relative path, sha256) of one adapter log, cross-checked against the outcome; None if absent."""
    name = Path(str(record.get(f"{stream}_log", ""))).name
    path = logs_dir / name
    if not name or not os.path.lexists(path):
        return None
    if path.is_symlink() or not path.is_file() or path.resolve().parent != logs_dir.resolve():
        raise RecordError(f"log {name} is not a regular file inside run/logs")
    digest = sha256_file(path)
    claimed = (record.get("log_sha256") or {}).get(stream)
    if claimed != digest:
        raise RecordError(f"log {name}: bytes on disk do not match the hash the adapter streamed")
    return str(path.relative_to(gate_root)), digest


def _not_run_log(gate_root: Path, index: int, check_id: str, reason: str) -> Tuple[str, str]:
    rel = Path("recorder") / f"{index:02d}-{check_id}.not-run.log"
    (gate_root / "recorder").mkdir(mode=0o700, exist_ok=True)
    data = f"check '{check_id}' did not run: {reason}\n".encode("utf-8")
    fd = os.open(gate_root / rel, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as handle:
        handle.write(data)
    return str(rel), sha256_bytes(data)


def _evidence_checks(gate_root: Path, task: Mapping[str, Any], outcome: Mapping[str, Any],
                     manifest: Dict[str, str]) -> List[Dict[str, Any]]:
    logs_dir = gate_root / "run" / "logs"
    ran = {rec.get("id"): rec for rec in outcome.get("checks", []) if isinstance(rec, dict)}
    checks: List[Dict[str, Any]] = []
    for index, check_id in enumerate(task["required_checks"]):
        rec = ran.get(check_id)
        stdout = _log_entry(gate_root, logs_dir, rec, "stdout") if rec else None
        stderr = _log_entry(gate_root, logs_dir, rec, "stderr") if rec else None
        if rec is None or stdout is None:
            reason = (rec or {}).get("status") or outcome.get("reason") or "an earlier check failed"
            log_path, log_hash = _not_run_log(gate_root, index, check_id, str(reason))
            manifest[log_path] = log_hash
            status, exit_code, argv, duration = ("skipped" if rec is None else "fail"), -1, [check_id], 0
        else:
            log_path, log_hash = stdout
            manifest[log_path] = log_hash
            if stderr is not None:
                manifest[stderr[0]] = stderr[1]
            status = STATUS_MAP.get(str(rec.get("status")), "fail")
            if status == "pass" and any((rec.get("truncated") or {}).values()):
                status = "warn"  # the full log was not captured: never a pass
            code = rec.get("exit_code")
            exit_code = code if isinstance(code, int) and not isinstance(code, bool) and -255 <= code <= 255 else -1
            if status in ("fail", "timeout") and exit_code == 0:
                exit_code = -1
            argv, duration = rec.get("argv") or [check_id], _duration_ms(rec.get("started_at"), rec.get("finished_at"))
        checks.append({"id": check_id, "status": status, "exit_code": exit_code, "duration_ms": duration,
                       "command": _command_label(argv), "log_hash": log_hash, "log_path": log_path,
                       "message": f"sandbox status {(rec or {}).get('status', 'not-run')}"})
    return checks


def _verdict(checks: Sequence[Mapping[str, Any]]) -> str:
    statuses = [c["status"] for c in checks]
    if any(s in ("fail", "timeout") for s in statuses):
        return "fail"
    return "pass" if all(s == "pass" for s in statuses) else "warn"


def _environment(ctx: Mapping[str, Any], outcome: Mapping[str, Any], hashes: Mapping[str, str]) -> Dict[str, str]:
    writer = ctx.get("writer") or {}
    limits = outcome.get("limits_enforced") or {}
    env = {
        "run_id": ctx["run_id"], "attempt": str(ctx["attempt"]), "repair_round": str(ctx["round"]),
        "candidate_ref": ctx["candidate_ref"], "post_gate_tree_sha": ctx["post_gate_tree_sha"],
        "image": str(outcome.get("image", "")), "image_id": str(limits.get("image_id") or "unavailable"),
        "docker_endpoint": str(outcome.get("docker_endpoint", "")), "limits_verified": str(bool(limits.get("verified"))).lower(),
        "executed_on_host": "false", "log_capture": LOG_CAPTURE,
        "writer_adapter": str(writer.get("adapter", "unavailable")),
        "writer_harness_version": str(writer.get("harness_version", "unavailable")),
        "writer_cli_version": str(writer.get("cli_version", "unavailable")),
        "model_requested": str(writer.get("model_requested", "unspecified")),
        "effort_requested": str(writer.get("effort_requested", "unspecified")),
        "writer_identity_reported": str(writer.get("identity_reported", "unavailable")),
        "changed_paths_count": str(len(ctx.get("changed_paths", []))),
        "human_merge_required": "true",
    }
    env.update({f"{k.replace('/', '_').replace('.', '_')}_sha256": v for k, v in hashes.items()})
    return {k: v[:512] for k, v in env.items()}


def record(gate_root: Path, ctx: Mapping[str, Any]) -> Dict[str, Any]:
    """Write evidence.json, run-record.json and receipt.json into gate_root. Returns the receipt.

    ctx (supervisor-built): task_id, run_id, attempt, round, base_sha, candidate_sha, candidate_ref,
    tree_sha, post_gate_tree_sha, changed_paths, writer{...}, budgets{...}, tools{...}, tcb{...}.
    """
    gate_root = Path(gate_root)
    task = _load_json(gate_root / "inputs" / "gate-task.json")
    outcome = _load_json(gate_root / "run" / "outcome.json")
    registry = _load_json(gate_root / "inputs" / "registry.json")
    if task.get("target_sha") != ctx["candidate_sha"] or task.get("base_sha") != ctx["base_sha"]:
        raise RecordError("gate task does not bind the candidate / base given by the supervisor")
    if outcome.get("manifest_sha256") != sha256_file(gate_root / "inputs" / "gate-task.json"):
        raise RecordError("the adapter ran a different gate task than the staged one")
    if not outcome.get("checks"):
        raise RecordError(f"no check ran (sandbox status {outcome.get('status')}: {outcome.get('reason', '')})")
    manifest: Dict[str, str] = {}
    for rel in ("inputs/gate-task.json", "inputs/registry.json", "inputs/catalog.json", "run/outcome.json", "scope.json"):
        manifest[rel] = sha256_file(gate_root / rel)
    checks = _evidence_checks(gate_root, task, outcome, manifest)
    verdict = _verdict(checks)
    if verdict == "pass" and outcome.get("status") != "passed":
        raise RecordError(f"sandbox status {outcome.get('status')} disagrees with passing checks")
    run_record = {
        "run_record_version": RUN_RECORD_VERSION, "recorder": RECORDER_ID, **{k: ctx[k] for k in sorted(ctx)},
        "sandbox_outcome": {k: outcome.get(k) for k in ("status", "reason", "argv", "started_at", "finished_at",
                                                          "limits_enforced", "supervisor_limits", "image", "docker_endpoint",
                                                          "run_id", "executed_on_host", "target_sha_bound")},
        "target_sha_bound_by": "runner: gate task target_sha == candidate commit checked out at /work",
        "log_capture": LOG_CAPTURE,
    }
    manifest["run-record.json"] = sha256_bytes(_write_exclusive(gate_root / "run-record.json", run_record))
    tools = ctx.get("tools") or {}
    evidence = {
        "schema_version": "evidence-v2", "policy_version": registry["policy_version"], "timestamp": now_utc(),
        "task_id": ctx["task_id"], "commit_sha": ctx["candidate_sha"], "base_sha": ctx["base_sha"],
        "tree_sha": ctx["tree_sha"], "verdict": verdict, "checks": checks, "sha256_manifest": dict(sorted(manifest.items())),
        "recorder": RECORDER_ID,
        "toolchain": f"python {tools.get('python', '?')}; git {tools.get('git', '?')}; image {outcome.get('image', '?')}"[:256],
        "environment": _environment(ctx, outcome, {"registry": manifest["inputs/registry.json"],
                                                   "catalog": manifest["inputs/catalog.json"],
                                                   "gate_task": manifest["inputs/gate-task.json"],
                                                   "scope_report": manifest["scope.json"],
                                                   "run_record": manifest["run-record.json"]}),
    }
    evidence_bytes = _write_exclusive(gate_root / "evidence.json", evidence)
    receipt = {
        "receipt_version": RECEIPT_VERSION, "recorder": RECORDER_ID, "task_id": ctx["task_id"], "run_id": ctx["run_id"],
        "base_sha": ctx["base_sha"], "candidate_sha": ctx["candidate_sha"], "candidate_ref": ctx["candidate_ref"],
        "tree_sha": ctx["tree_sha"], "post_gate_tree_sha": ctx["post_gate_tree_sha"], "verdict": verdict,
        "evidence": "evidence.json", "evidence_sha256": sha256_bytes(evidence_bytes), "recorded_at": now_utc(),
    }
    _write_exclusive(gate_root / "receipt.json", receipt)
    return receipt


# --- verify ----------------------------------------------------------------------------------------

def _private_dir(path: Path, what: str) -> None:
    info = os.lstat(path)
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
        raise VerifyRefused("untrusted_location", f"{what} is not a real directory: {path}")
    if info.st_uid != os.geteuid() or info.st_mode & 0o022:
        raise VerifyRefused("untrusted_location", f"{what} is not owned by us or is group/world writable: {path}")


def trusted_runs_root(repo: Path, runs_root: Path) -> Path:
    if not Path(runs_root).is_absolute():
        raise VerifyRefused("untrusted_location", "runs root must be an absolute path")
    if not os.path.isdir(runs_root):
        raise VerifyRefused("untrusted_location", f"runs root does not exist: {runs_root}")
    _private_dir(Path(runs_root), "runs root")
    root = Path(os.path.realpath(runs_root))
    marker = root / RUNS_ROOT_MARKER
    if marker.is_symlink() or not marker.is_file():
        raise VerifyRefused("untrusted_location", f"{root} is not a runner runs root (no {RUNS_ROOT_MARKER})")
    for ancestor in root.parents:
        if (ancestor / RUNS_ROOT_MARKER).exists():
            raise VerifyRefused("nested_runs_root", f"{root} lies inside another runs root ({ancestor}); "
                                                    "writer workspaces and gate checkouts live there")
    for untrusted in HG.untrusted_roots(repo):
        if HG.is_within(root, untrusted) or HG.is_within(untrusted, root):
            raise VerifyRefused("untrusted_location", f"runs root overlaps a worktree or git dir: {untrusted}")
    return root


def _receipt_location(runs_root: Path, given_root: Path, receipt_path: Path) -> Tuple[Path, List[str]]:
    """Gate root of a receipt at <root>/<task>/<run>/a<N>/g<M>/receipt.json; no component may be a symlink."""
    if not Path(receipt_path).is_absolute():
        raise VerifyRefused("untrusted_location", "receipt path must be absolute")
    parts = Path(os.path.abspath(receipt_path)).parts
    for prefix in (Path(os.path.abspath(given_root)), runs_root):
        if parts[: len(prefix.parts)] == prefix.parts:
            rel = list(parts[len(prefix.parts):])
            break
    else:
        raise VerifyRefused("untrusted_location", "receipt is not under the runs root")
    if len(rel) != 5 or rel[4] != "receipt.json" or not (TASK_ID_RE.fullmatch(rel[0]) and RUN_ID_RE.fullmatch(rel[1])
                                                          and ATTEMPT_RE.fullmatch(rel[2]) and GATE_RE.fullmatch(rel[3])):
        raise VerifyRefused("untrusted_location", "receipt must be <runs_root>/<task>/<run>/a<N>/g<M>/receipt.json "
                                                  f"(got {'/'.join(rel)})")
    current = runs_root
    for part in rel[:4]:
        current = current / part
        _private_dir(current, "evidence directory")  # lstat: a symlinked component is refused here
    info = os.lstat(current / "receipt.json")
    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.geteuid() or info.st_mode & 0o022:
        raise VerifyRefused("untrusted_location", "receipt is not a private regular file (symlinks are refused)")
    return current, rel


def _load_receipt(path: Path) -> Dict[str, Any]:
    data, errors = VE.read_json_file(path)
    if errors or not isinstance(data, dict) or set(data) != RECEIPT_KEYS:
        raise VerifyRefused("receipt_invalid", str(errors[0]) if errors else "unexpected receipt keys")
    if data["receipt_version"] != RECEIPT_VERSION or data["recorder"] != RECORDER_ID or data["evidence"] != "evidence.json":
        raise VerifyRefused("receipt_invalid", "unknown receipt version, recorder or evidence name")
    for key in ("base_sha", "candidate_sha", "tree_sha", "post_gate_tree_sha"):
        if not (isinstance(data[key], str) and HEX40.fullmatch(data[key])):
            raise VerifyRefused("receipt_invalid", f"{key} is not a 40-hex SHA")
    return data


def _verify_git(repo: Path, receipt: Mapping[str, Any], expect_candidate: str) -> Tuple[str, str]:
    commit = HG.resolve_commit(repo, expect_candidate)
    if commit != expect_candidate:
        raise VerifyRefused("candidate_missing", f"{expect_candidate} is not a commit in {repo}")
    tree = HG.tree_of(repo, commit)
    if tree != receipt["tree_sha"]:
        raise VerifyRefused("tree_mismatch", "candidate tree differs from the recorded tree")
    parents = HG.parents_of(repo, commit)
    if parents != [receipt["base_sha"]]:
        raise VerifyRefused("base_mismatch", f"candidate parents {parents} are not exactly the recorded base")
    ref = receipt["candidate_ref"]
    if not (isinstance(ref, str) and ref.startswith(CANDIDATE_REF_PREFIX)) or HG.ref_target(repo, ref) != commit:
        raise VerifyRefused("candidate_ref_mismatch", f"runner ref {ref!r} does not point at the candidate")
    if receipt["post_gate_tree_sha"] != tree:
        raise VerifyRefused("gate_mutated_checkout", "the gate checkout changed while the checks ran")
    return tree, receipt["base_sha"]


def _verify_manifest(gate_root: Path, evidence: Mapping[str, Any]) -> None:
    for rel, digest in evidence["sha256_manifest"].items():
        parts = rel.split("/") if isinstance(rel, str) else [""]
        if not isinstance(rel, str) or rel.startswith("/") or "\\" in rel or any(p in ("", ".", "..") for p in parts):
            raise VerifyRefused("manifest_mismatch", f"manifest key {rel!r} is not a plain path inside the gate root")
        current = gate_root
        for part in rel.split("/"):
            current = current / part
            if current.is_symlink():
                raise VerifyRefused("manifest_mismatch", f"{rel} passes through a symlink")
        if not current.is_file():
            raise VerifyRefused("manifest_mismatch", f"{rel} is missing")
        if sha256_file(current) != digest:
            raise VerifyRefused("manifest_mismatch", f"{rel} changed after recording")


def verify(*, repo: Path, runs_root: Path, receipt_path: Path, expect_candidate: str,
           registry_path: Path = DEFAULT_REGISTRY, catalog_path: Path = DEFAULT_CATALOG,
           now: Optional[datetime.datetime] = None) -> Dict[str, Any]:
    """Accept evidence only for the exact, immutable, verified candidate. Raises VerifyRefused."""
    if not (isinstance(expect_candidate, str) and HEX40.fullmatch(expect_candidate)):
        raise VerifyRefused("usage", "expected candidate must be a 40-hex SHA")
    root = trusted_runs_root(repo, runs_root)
    gate_root, rel = _receipt_location(root, Path(runs_root), Path(receipt_path))
    receipt = _load_receipt(gate_root / "receipt.json")
    if (receipt["task_id"], receipt["run_id"]) != (rel[0], rel[1]):
        raise VerifyRefused("receipt_invalid", "receipt task/run differ from its location")
    if receipt["candidate_sha"] != expect_candidate:
        raise VerifyRefused("candidate_mismatch", "receipt is for a different candidate (any byte change makes a new SHA)")
    evidence_path = gate_root / "evidence.json"
    if evidence_path.is_symlink() or not evidence_path.is_file() or sha256_file(evidence_path) != receipt["evidence_sha256"]:
        raise VerifyRefused("evidence_hash_mismatch", "evidence.json differs from the receipt")
    tree, base = _verify_git(Path(repo), receipt, expect_candidate)
    trusted_registry, trusted_catalog = Path(registry_path).read_bytes(), Path(catalog_path).read_bytes()
    for staged, trusted in (("inputs/registry.json", trusted_registry), ("inputs/catalog.json", trusted_catalog)):
        if sha256_file(gate_root / staged) != sha256_bytes(trusted):
            raise VerifyRefused("policy_drift", f"{staged} differs from the trusted copy")
    policy = json.loads(trusted_registry.decode("utf-8"))["policy_version"]
    catalog_sha = sha256_bytes(trusted_catalog)
    task_path = gate_root / "inputs" / "gate-task.json"
    task_result = VE.validate_file_detailed(task_path, "task-v2", SCHEMAS_DIR, True, VE.Expectations(
        candidate_sha=expect_candidate, base_sha=base, policy_version=policy, catalog_sha256=catalog_sha,
        catalog_path=Path(catalog_path), now=now))
    if not task_result.ok:
        raise VerifyRefused("task_invalid", "; ".join(str(e) for e in task_result.errors[:2]))
    task = json.loads(task_path.read_text(encoding="utf-8"))
    expectations = VE.Expectations(candidate_sha=expect_candidate, base_sha=base, tree_sha=tree, policy_version=policy,
                                   catalog_sha256=catalog_sha, catalog_path=Path(catalog_path),
                                   logs_dir=gate_root, task=task, now=now)
    result = VE.validate_file_detailed(evidence_path, "evidence-v2", SCHEMAS_DIR, True, expectations)
    if not result.ok or not result.expectations_complete:
        detail = "; ".join(str(e) for e in result.errors[:3]) or "expectations not fully applied"
        raise VerifyRefused("evidence_invalid", detail)
    evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
    _verify_manifest(gate_root, evidence)
    if evidence["recorder"] != RECORDER_ID:
        raise VerifyRefused("recorder_untrusted", f"recorder {evidence['recorder']!r}")
    env = evidence.get("environment") or {}
    if env.get("candidate_ref") != receipt["candidate_ref"] or env.get("post_gate_tree_sha") != tree:
        raise VerifyRefused("evidence_invalid", "evidence environment does not bind the candidate ref / post-gate tree")
    if evidence["verdict"] != "pass" or receipt["verdict"] != "pass":
        raise VerifyRefused("not_passed", f"evidence verdict is {evidence['verdict']}")
    return {"status": "verified", "candidate_sha": expect_candidate, "tree_sha": tree, "base_sha": base,
            "candidate_ref": receipt["candidate_ref"], "evidence": str(evidence_path)}


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Verify trusted-runner evidence (task H4)")
    sub = parser.add_subparsers(dest="command", required=True)
    ver = sub.add_parser("verify", help="accept evidence only for the exact verified candidate")
    ver.add_argument("--repo", required=True, type=Path)
    ver.add_argument("--runs-root", required=True, type=Path)
    ver.add_argument("--receipt", required=True, type=Path)
    ver.add_argument("--expect-candidate", required=True)
    ver.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    ver.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    try:
        args = _parser().parse_args(argv)
    except SystemExit as exc:
        return 2 if exc.code else 0
    try:
        summary = verify(repo=args.repo, runs_root=args.runs_root, receipt_path=args.receipt,
                         expect_candidate=args.expect_candidate, registry_path=args.registry, catalog_path=args.catalog)
    except VerifyRefused as exc:
        print(f"EVIDENCE: REFUSED reason={exc.code} {exc.detail}")
        return 2 if exc.code == "usage" else 1
    except (OSError, ValueError, KeyError, HG.GitError) as exc:
        print(f"EVIDENCE: REFUSED reason=error {type(exc).__name__}: {exc}")
        return 1
    print(json.dumps(summary, indent=2, sort_keys=True))
    print(f"EVIDENCE: VERIFIED candidate={summary['candidate_sha']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
