#!/usr/bin/env python3
"""run-task.py - trusted task runner with post-commit external evidence (task H4).

FAKE WRITER ONLY. The ADR (docs/decisions/2026-07-21-agent-harness-hardening-v1.md) does not grant a
real coding-agent adapter, host fallback, credentials, network or auto-merge; this runner accepts
`writer.adapter == "fake"` (docs/harness/tests/fake_writer.py inside the H3 sandbox) and nothing else.

Pipeline (supervisor = this process; the writer only ever sees its workspace at /work):

  1. Refuse early: kill switch (ENGRAM_RUNNER_DISABLED=1), invalid request, unknown writer adapter,
     budgets above the pilot caps, a budget the task requires enforced that cannot be enforced (turn
     cap, provider cost cap), wrong repo (base not in it), base ref mismatch, dirty base (tracked
     changes in the repository checkout), runs root inside a worktree/git dir, a second writer (lock).
  2. Per attempt: export the base byte-exactly into a fresh plain directory a<N>/ws (no .git, so the
     writer cannot reach git state), run the fake writer there through sandbox-adapter.run_isolated.
  3. Hash the workspace into a TREE (plumbing, no filters / ignore rules) and run check-scope.py
     base..tree. Only if scope passes is the candidate COMMIT created (commit-tree, fixed runner
     identity, parent = base) and pinned by a create-only ref refs/engram-runner/candidates/<task>/<run>/a<N>r<M>.
     Scope refusal stops the run (exit 4): there is no retry around a security refusal.
  4. Gate: export the candidate commit into a SEPARATE clean checkout a<N>/c<M> and run the approved
     checks there (gate task-v2 with target_sha = candidate). Afterwards the checkout is re-hashed: any
     tracked or untracked change made by a check is `gate_mutated_checkout` (fail, never a pass).
  5. record-evidence.record() writes evidence-v2 + run record + receipt into a<N>/g<M>; then
     record-evidence.verify() must accept it for the exact candidate before the run is `passed`.
  6. Gate failure -> repair round (writer runs again in the same workspace) while the repair budget
     lasts; writer failure / repairs exhausted -> next attempt while the attempt budget lasts.

Budgets (pilot defaults are also the maximum a request may ask for):
  wall_seconds 2700 (45 min)   enforced by the supervisor deadline + adapter manifest timeouts
  attempts 2, repair 1         enforced by supervisor counters
  writer_concurrency 1         enforced by an exclusive lock on <git-common-dir>/engram-runner.lock: one
                               writer per repository, whatever the runs root (never writer-reachable)
  check timeout                enforced by the sandbox adapter (docker kill), per registry entry
  turn_cap 20                  NOT enforced: the fake writer adapter has no observable turns
  cost_cap_usd                 NOT enforceable: no provider adapter; a request that sets it is refused
Unenforced limits are recorded as such and never reported as enforced.

Exit codes / final line `RUN_STATUS: <STATUS> task=<id> run=<run> reason=<...>`:
  0 PASSED, 1 FAILED (gate/writer failure, budget exhausted), 2 REFUSED (input/policy),
  3 UNAVAILABLE (sandbox missing; never a pass, no host fallback), 4 SCOPE_REFUSED.
Logs, workspaces, scope reports, candidate refs and evidence are kept on every path; `cleanup`
removes only one run's scratch (ws/ and c<M>/), never refs, evidence or another task's directories.
"""

from __future__ import annotations

import argparse
import datetime
import fcntl
import hashlib
import importlib.util
import json
import math
import os
import platform
import re
import shutil
import sys
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence


def _load_sibling(name: str, module_name: Optional[str] = None) -> Any:
    path = Path(__file__).resolve().parent / f"{name}.py"
    module_name = module_name or name.replace("-", "_")
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
SCOPE = _load_sibling("check-scope")
REC = _load_sibling("record-evidence")
SBX = _load_sibling("sandbox-adapter")
VE = _load_sibling("validate-evidence", "validate_evidence")

BIN_DIR = Path(__file__).resolve().parent
HARNESS_DIR = BIN_DIR.parent
FAKE_WRITER = HARNESS_DIR / "tests" / "fake_writer.py"
DEFAULT_REGISTRY, DEFAULT_CATALOG = REC.DEFAULT_REGISTRY, REC.DEFAULT_CATALOG
TCB_FILES = ("run-task.py", "check-scope.py", "record-evidence.py", "harness_git.py", "sandbox-adapter.py",
             "sandbox_registry.py", "validate-evidence.py")

REQUEST_VERSION = "runner-request-v1"
SUMMARY_VERSION = "runner-summary-v1"
PILOT_BUDGETS = {"wall_seconds": 2700, "attempts": 2, "repair": 1, "writer_concurrency": 1, "turn_cap": 20}
ENFORCEMENT = {
    "wall_seconds": (True, "supervisor deadline; each sandbox phase gets the remaining time as its manifest timeout"),
    "attempts": (True, "supervisor counter"),
    "repair": (True, "supervisor counter"),
    "writer_concurrency": (True, "exclusive non-blocking lock on <git-common-dir>/engram-runner.lock (per repository)"),
    "check_timeout": (True, "sandbox adapter: per-check timeout from the registry, container killed and removed"),
    "turn_cap": (False, "fake writer adapter has no observable turns; recorded, not enforced"),
    "cost_cap_usd": (False, "no provider adapter; a provider cost cap is not enforceable"),
}
FAKE_BEHAVIORS = {"write_file", "delete_file", "claim_pass", "exit_code", "outlive_timeout", "benign"}
REQUEST_KEYS = {"request_version", "task_id", "base_sha", "policy_version", "writer", "required_checks", "allowed_paths"}
OPTIONAL_KEYS = {"protected_paths", "allow_lockfiles", "budgets", "require_enforced"}
WRITER_KEYS = {"adapter", "invocations"}
WRITER_OPTIONAL = {"model_requested", "effort_requested"}
RUNNER_IDENTITY = {"GIT_AUTHOR_NAME": "engram-runner", "GIT_AUTHOR_EMAIL": "runner@engram.invalid",
                   "GIT_COMMITTER_NAME": "engram-runner", "GIT_COMMITTER_EMAIL": "runner@engram.invalid"}
LOCK_NAME = "engram-runner.lock"
EXIT = {"passed": 0, "failed": 1, "refused": 2, "unavailable": 3, "scope_refused": 4}


class Refused(Exception):
    def __init__(self, code: str, detail: str = ""):
        super().__init__(f"{code}: {detail}")
        self.code, self.detail = code, detail


class Stop(Exception):
    """Ends the run with a final status (not an input refusal)."""

    def __init__(self, status: str, reason: str):
        super().__init__(reason)
        self.status, self.reason = status, reason


def _now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def _sha256(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def _write_json(path: Path, payload: Any) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


# --- request validation --------------------------------------------------------------------------

def _str_list(value: Any, where: str, allow_empty: bool = False) -> List[str]:
    if not isinstance(value, list) or (not value and not allow_empty) or not all(isinstance(v, str) and v for v in value):
        raise Refused("request_invalid", f"{where} must be a {'possibly empty ' if allow_empty else 'non-empty '}list of strings")
    return list(value)


def _budgets(raw: Any) -> Dict[str, Any]:
    raw = {} if raw is None else raw
    if not isinstance(raw, dict) or not set(raw) <= set(PILOT_BUDGETS) | {"cost_cap_usd"}:
        raise Refused("request_invalid", f"budgets may only contain {sorted(set(PILOT_BUDGETS) | {'cost_cap_usd'})}")
    if raw.get("cost_cap_usd") is not None:
        raise Refused("budget_not_enforceable", "a provider cost cap cannot be enforced by the fake writer adapter")
    budgets: Dict[str, Any] = {}
    for key, cap in PILOT_BUDGETS.items():
        value, low = raw.get(key, cap), (0 if key == "repair" else 1)
        if not isinstance(value, int) or isinstance(value, bool) or not low <= value <= cap:
            raise Refused("budget_above_pilot_cap", f"budgets.{key} must be an integer in [{low}, {cap}] (pilot cap)")
        budgets[key] = value
    if budgets["writer_concurrency"] != 1:
        raise Refused("budget_above_pilot_cap", "writer_concurrency is fixed at 1")
    return budgets


def _writer(raw: Any) -> Dict[str, Any]:
    if not isinstance(raw, dict) or not WRITER_KEYS <= set(raw) <= WRITER_KEYS | WRITER_OPTIONAL:
        raise Refused("request_invalid", f"writer must contain {sorted(WRITER_KEYS)} (optional {sorted(WRITER_OPTIONAL)})")
    if raw["adapter"] != "fake":
        raise Refused("writer_not_granted", f"writer adapter {raw['adapter']!r}: only the fake writer is granted (ADR)")
    invocations = raw["invocations"]
    if not isinstance(invocations, list) or not invocations:
        raise Refused("request_invalid", "writer.invocations must be a non-empty list")
    for i, inv in enumerate(invocations):
        args = _str_list(inv, f"writer.invocations[{i}]")
        if args[0] not in FAKE_BEHAVIORS:
            raise Refused("request_invalid", f"fake writer behavior {args[0]!r} is not exposed by the runner")
    for key in WRITER_OPTIONAL:
        if key in raw and not (isinstance(raw[key], str) and 0 < len(raw[key]) <= 128):
            raise Refused("request_invalid", f"writer.{key} must be a short string")
    return dict(raw)


def load_request(path: Path) -> Dict[str, Any]:
    data, errors = VE.read_json_file(Path(path))
    if errors:
        raise Refused("request_invalid", str(errors[0]))
    if not isinstance(data, dict) or not REQUEST_KEYS <= set(data) <= REQUEST_KEYS | OPTIONAL_KEYS:
        raise Refused("request_invalid", f"request keys must be {sorted(REQUEST_KEYS)} (+ optional {sorted(OPTIONAL_KEYS)})")
    if data["request_version"] != REQUEST_VERSION:
        raise Refused("request_invalid", f"request_version must be {REQUEST_VERSION}")
    if not (isinstance(data["task_id"], str) and REC.TASK_ID_RE.fullmatch(data["task_id"])):
        raise Refused("request_invalid", "task_id must match [a-zA-Z0-9_-]{1,128}")
    if not (isinstance(data["base_sha"], str) and REC.HEX40.fullmatch(data["base_sha"])):
        raise Refused("request_invalid", "base_sha must be a 40-hex SHA")
    request = dict(data)
    request["writer"] = _writer(data["writer"])
    request["required_checks"] = _str_list(data["required_checks"], "required_checks")
    request["allowed_paths"] = _str_list(data["allowed_paths"], "allowed_paths")
    request["protected_paths"] = _str_list(data.get("protected_paths", []), "protected_paths", allow_empty=True)
    if not isinstance(data.get("allow_lockfiles", False), bool):
        raise Refused("request_invalid", "allow_lockfiles must be a boolean")
    request["allow_lockfiles"] = data.get("allow_lockfiles", False)
    request["budgets"] = _budgets(data.get("budgets"))
    required = _str_list(data.get("require_enforced", []), "require_enforced", allow_empty=True)
    unknown = [k for k in required if k not in ENFORCEMENT]
    if unknown:
        raise Refused("request_invalid", f"require_enforced has unknown limits {unknown}")
    not_enforced = [k for k in required if not ENFORCEMENT[k][0]]
    if not_enforced:
        raise Refused("budget_not_enforceable", f"the task requires {not_enforced} enforced, which this adapter cannot do")
    request["require_enforced"] = required
    return request


def budget_report(budgets: Mapping[str, int]) -> Dict[str, Any]:
    report = {}
    for key, (enforced, how) in ENFORCEMENT.items():
        report[key] = {"value": budgets.get(key), "enforced": enforced, "by" if enforced else "limitation": how}
    return report


# --- repository and runs-root checks -------------------------------------------------------------

def check_repo(repo: Path, request: Mapping[str, Any], base_ref: str) -> None:
    top = HG.toplevel(repo)
    if top is None:
        raise Refused("wrong_repo", f"{repo} is not a git repository")
    if HG.resolve_commit(repo, request["base_sha"]) != request["base_sha"]:
        raise Refused("wrong_repo", f"base {request['base_sha']} is not a commit of {repo}")
    resolved = HG.resolve_commit(repo, base_ref)
    if resolved != request["base_sha"]:
        raise Refused("base_ref_mismatch", f"{base_ref} resolves to {resolved}, the request pins {request['base_sha']}")
    dirty = HG.git(repo, ["status", "--porcelain=v1", "-z", "--untracked-files=no", "--ignore-submodules=none"])
    if dirty.strip(b"\0"):
        raise Refused("dirty_base", "the repository checkout has uncommitted tracked changes")


def prepare_runs_root(repo: Path, runs_root: Path) -> Path:
    if not Path(runs_root).is_absolute():
        raise Refused("untrusted_runs_root", "runs root must be an absolute path")
    for untrusted in HG.untrusted_roots(repo):  # before anything is created there
        if HG.is_within(Path(runs_root), untrusted) or HG.is_within(untrusted, Path(runs_root)):
            raise Refused("untrusted_runs_root", f"runs root overlaps a worktree or git dir: {untrusted}")
    created = False
    if not Path(runs_root).exists():
        Path(runs_root).mkdir(parents=True, mode=0o700)
        created = True
    marker = Path(runs_root) / REC.RUNS_ROOT_MARKER
    if not os.path.lexists(marker):
        if not created and (not Path(runs_root).is_dir() or any(Path(runs_root).iterdir())):
            raise Refused("untrusted_runs_root", f"{runs_root} exists, is not empty and is not a runner runs root")
        marker.write_text("engram trusted-runner runs root (task H4); do not place inside a worktree\n", encoding="utf-8")
        marker.chmod(0o600)
    try:
        return REC.trusted_runs_root(repo, runs_root)
    except REC.VerifyRefused as exc:
        raise Refused("untrusted_runs_root", exc.detail) from exc


# --- run state ------------------------------------------------------------------------------------

@dataclass
class Run:
    repo: Path
    request: Dict[str, Any]
    run_dir: Path
    run_id: str
    registry_path: Path
    catalog_path: Path
    docker: str
    deadline: float
    registry: Any
    registry_bytes: bytes
    catalog_bytes: bytes
    writer_calls: int = 0
    events: List[Dict[str, Any]] = field(default_factory=list)
    result: Dict[str, Any] = field(default_factory=dict)

    def remaining(self) -> int:
        left = self.deadline - time.monotonic()
        if left < 1:
            raise Stop("failed", "wall_budget_exhausted")
        return max(1, int(math.floor(left)))

    def protected_for_manifest(self) -> List[str]:
        paths = [*SCOPE.BUILTIN_PROTECTED_PREFIXES, *SCOPE.BUILTIN_PROTECTED_FILES, *self.request["protected_paths"]]
        return list(dict.fromkeys(paths))[:256]


def _task_manifest(run: Run, target: str, checks: Sequence[str], caps: Sequence[str], phase: str) -> Dict[str, Any]:
    return {
        "schema_version": "task-v2", "policy_version": run.registry.policy_version, "task_id": run.request["task_id"],
        "target_sha": target, "base_sha": run.request["base_sha"], "allowed_capabilities": list(caps),
        "required_checks": list(checks), "timeout_seconds": run.remaining(),
        "allowed_paths": run.request["allowed_paths"], "protected_paths": run.protected_for_manifest(),
        "metadata": {"runner_run_id": run.run_id, "phase": phase},
    }


def _stage(dir_path: Path, files: Mapping[str, Any]) -> None:
    dir_path.mkdir(mode=0o700)
    for name, payload in files.items():
        target = dir_path / name
        if isinstance(payload, bytes):
            fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, "wb") as handle:
                handle.write(payload)
        else:
            _write_json(target, payload)


# --- phases ---------------------------------------------------------------------------------------

def run_writer(run: Run, ws: Path, attempt_dir: Path, rnd: int) -> Any:
    invocations = run.request["writer"]["invocations"]
    args = invocations[min(run.writer_calls, len(invocations) - 1)]
    run.writer_calls += 1
    timeout = min(3600, run.remaining())
    registry = {"registry_version": "check-registry-v1", "policy_version": run.registry.policy_version,
                "sandbox": json.loads(run.registry_bytes)["sandbox"],
                "checks": {"fake_writer": {"argv": ["python3", "/tcb/tools/fake_writer.py", *args], "timeout_seconds": timeout,
                                           "note": "runner-generated writer entry (fake writer only)"}}}
    catalog = {"catalog_version": "check-catalog-v1", "checks": {"fake_writer": {"description": "fake writer (H4 runner)"}}}
    inputs = attempt_dir / f"w{rnd}-inputs"
    manifest = _task_manifest(run, run.request["base_sha"], ["fake_writer"],
                              ["workspace_read", "workspace_write", "sandbox_container", "network_none"], "writer")
    _stage(inputs, {"manifest.json": manifest, "registry.json": registry, "catalog.json": catalog})
    outcome = SBX.run_isolated(inputs / "manifest.json", ws, attempt_dir / f"w{rnd}", registry_path=inputs / "registry.json",
                               catalog_path=inputs / "catalog.json", docker_bin=run.docker,
                               tcb_files={"fake_writer.py": FAKE_WRITER})
    if outcome.status == SBX.STATUS_UNAVAILABLE:
        raise Stop("unavailable", f"sandbox_unavailable: {outcome.reason}")
    if outcome.status == SBX.STATUS_REFUSED:
        raise Stop("refused", f"sandbox_refused: {outcome.reason}")
    return outcome


def _workspace_stop(exc: Any, other_status: str, prefix: str = "") -> Stop:
    if exc.code == "deadline_exceeded":
        return Stop("failed", "wall_budget_exhausted")
    return Stop(other_status, f"{prefix}{exc.code}: {exc.path!r}")


def build_candidate_tree(run: Run, ws: Path, scratch: Path) -> str:
    try:
        return HG.build_tree(run.repo, ws, scratch, deadline=run.deadline)
    except HG.WorkspaceError as exc:
        raise _workspace_stop(exc, "scope_refused") from exc


def scope_check(run: Run, tree: str, report_path: Path) -> Any:
    report = SCOPE.check_scope(run.repo, run.request["base_sha"], tree, run.request["allowed_paths"],
                               run.request["protected_paths"], run.request["allow_lockfiles"])
    _write_json(report_path, report.to_dict())
    if report.verdict != "pass":
        codes = ",".join(sorted({f.code for f in report.findings}))
        raise Stop("scope_refused", f"scope_refused: {codes}")
    return report


def commit_candidate(run: Run, tree: str, attempt: int, rnd: int) -> Dict[str, str]:
    message = (f"engram-runner candidate task={run.request['task_id']} run={run.run_id} attempt={attempt} round={rnd}\n\n"
               f"base={run.request['base_sha']}\n")
    sha = HG.git(run.repo, ["commit-tree", tree, "-p", run.request["base_sha"], "-m", message],
                 extra_env=RUNNER_IDENTITY).decode().strip()
    ref = f"{REC.CANDIDATE_REF_PREFIX}{run.request['task_id']}/{run.run_id}/a{attempt}r{rnd}"
    HG.git(run.repo, ["update-ref", "-m", "engram-runner candidate", "--create-reflog", ref, sha, HG.ZERO_OID])
    return {"sha": sha, "ref": ref}


def run_gate(run: Run, attempt_dir: Path, rnd: int, candidate: Mapping[str, str], tree: str) -> Dict[str, Any]:
    checkout, gate_root = attempt_dir / f"c{rnd}", attempt_dir / f"g{rnd}"
    checkout.mkdir(mode=0o700)
    try:
        HG.export_tree(run.repo, candidate["sha"], checkout, deadline=run.deadline)
    except HG.WorkspaceError as exc:
        raise _workspace_stop(exc, "failed", "candidate_export_failed: ") from exc
    gate_root.mkdir(mode=0o700)
    manifest = _task_manifest(run, candidate["sha"], run.request["required_checks"],
                              ["workspace_read", "test_exec", "sandbox_container", "network_none"], "gate")
    _stage(gate_root / "inputs", {"gate-task.json": manifest, "registry.json": run.registry_bytes,
                                  "catalog.json": run.catalog_bytes})
    outcome = SBX.run_isolated(gate_root / "inputs" / "gate-task.json", checkout, gate_root / "run",
                               registry_path=gate_root / "inputs" / "registry.json",
                               catalog_path=gate_root / "inputs" / "catalog.json", docker_bin=run.docker)
    if outcome.status == SBX.STATUS_UNAVAILABLE:
        raise Stop("unavailable", f"sandbox_unavailable: {outcome.reason}")
    if outcome.status == SBX.STATUS_REFUSED:
        raise Stop("refused", f"sandbox_refused: {outcome.reason}")
    try:
        post_tree = HG.build_tree(run.repo, checkout, gate_root, deadline=run.deadline)
    except HG.WorkspaceError as exc:
        raise _workspace_stop(exc, "failed", "gate_mutated_checkout: ") from exc
    return {"outcome": outcome, "post_tree": post_tree, "gate_root": gate_root, "tree": tree}


def tool_versions(repo: Path) -> Dict[str, str]:
    return {"python": platform.python_version(), "git": HG.git(repo, ["version"]).decode().strip().replace("git version ", "")}


def tcb_hashes() -> Dict[str, str]:
    files = {name: BIN_DIR / name for name in TCB_FILES}
    files["fake_writer.py"] = FAKE_WRITER
    return {name: _sha256(path) for name, path in files.items()}


def record_gate(run: Run, gate: Mapping[str, Any], candidate: Mapping[str, str], scope: Any, attempt: int, rnd: int) -> Dict[str, Any]:
    gate_root = gate["gate_root"]
    _write_json(gate_root / "scope.json", scope.to_dict())
    writer = run.request["writer"]
    ctx = {
        "task_id": run.request["task_id"], "run_id": run.run_id, "attempt": attempt, "round": rnd,
        "base_sha": run.request["base_sha"], "candidate_sha": candidate["sha"], "candidate_ref": candidate["ref"],
        "tree_sha": gate["tree"], "post_gate_tree_sha": gate["post_tree"],
        "changed_paths": [c.path for c in scope.changes],
        "writer": {"adapter": "fake_writer", "harness_version": f"fake_writer.py sha256={_sha256(FAKE_WRITER)}",
                   "cli_version": "unavailable (fake writer has no CLI)",
                   "model_requested": writer.get("model_requested", "unspecified"),
                   "effort_requested": writer.get("effort_requested", "unspecified"),
                   "identity_reported": "unavailable (fake writer reports no model identity)",
                   "invocations_used": run.writer_calls},
        "budgets": budget_report(run.request["budgets"]),
        "approval": {"human_merge_required": True, "auto_merge": False, "approval_recorded": "none"},
        "tools": tool_versions(run.repo), "tcb_sha256": tcb_hashes(),
    }
    try:
        receipt = REC.record(gate_root, ctx)
    except (REC.RecordError, OSError, KeyError, ValueError) as exc:
        return {"recorded": False, "reason": f"evidence_not_recorded: {exc}"}
    return {"recorded": True, "receipt": receipt, "receipt_path": str(gate_root / "receipt.json")}


def attempt_once(run: Run, attempt: int) -> bool:
    attempt_dir = run.run_dir / f"a{attempt}"
    attempt_dir.mkdir(mode=0o700)
    ws = attempt_dir / "ws"
    ws.mkdir(mode=0o700)
    try:
        HG.export_tree(run.repo, run.request["base_sha"], ws, deadline=run.deadline)
    except HG.WorkspaceError as exc:
        raise _workspace_stop(exc, "refused", "unsupported_base: ") from exc
    previous_tree = HG.tree_of(run.repo, run.request["base_sha"])
    for rnd in range(run.request["budgets"]["repair"] + 1):
        event: Dict[str, Any] = {"attempt": attempt, "round": rnd}
        run.events.append(event)
        writer = run_writer(run, ws, attempt_dir, rnd)
        event["writer_status"] = writer.status
        if writer.status != SBX.STATUS_PASSED:
            event["result"] = f"writer_{writer.status}: {writer.reason}"
            return False
        tree = build_candidate_tree(run, ws, attempt_dir)
        if tree == previous_tree:  # nothing new since the base (or since the failed candidate)
            event["result"] = "writer_no_changes"
            return False
        previous_tree = tree
        scope = scope_check(run, tree, attempt_dir / f"scope-{rnd}.json")
        candidate = commit_candidate(run, tree, attempt, rnd)
        event.update(candidate=candidate["sha"], candidate_ref=candidate["ref"])
        gate = run_gate(run, attempt_dir, rnd, candidate, tree)
        event["gate_status"] = gate["outcome"].status
        recorded = record_gate(run, gate, candidate, scope, attempt, rnd)
        event["evidence"] = recorded.get("receipt_path") or recorded.get("reason")
        if gate["post_tree"] != tree:
            raise Stop("failed", "gate_mutated_checkout: a check changed tracked or untracked files of the candidate checkout")
        if gate["outcome"].status != SBX.STATUS_PASSED or not recorded["recorded"]:
            event["result"] = f"gate_{gate['outcome'].status}: {gate['outcome'].reason or recorded.get('reason', '')}"
            continue
        try:
            verified = REC.verify(repo=run.repo, runs_root=run.run_dir.parent.parent, receipt_path=Path(recorded["receipt_path"]),
                                  expect_candidate=candidate["sha"], registry_path=run.registry_path, catalog_path=run.catalog_path)
        except REC.VerifyRefused as exc:
            raise Stop("failed", f"evidence_unverified: {exc.code}: {exc.detail}") from exc
        event["result"] = "passed"
        run.result = {"candidate": candidate, "receipt": recorded["receipt_path"], "verified": verified}
        return True
    return False


def execute(run: Run) -> Dict[str, Any]:
    status, reason = "failed", ""
    try:
        for attempt in range(1, run.request["budgets"]["attempts"] + 1):
            run.remaining()
            if attempt_once(run, attempt):
                status, reason = "passed", "evidence_verified"
                break
        else:
            last = run.events[-1].get("result", "") if run.events else ""
            if time.monotonic() > run.deadline - 1:
                raise Stop("failed", "wall_budget_exhausted")
            reason = f"attempts_exhausted ({run.request['budgets']['attempts']} attempt(s), repair {run.request['budgets']['repair']}): {last}"
    except Stop as stop:
        status, reason = stop.status, stop.reason
    except (HG.GitError, SCOPE.ScopeInputError, OSError) as exc:
        status, reason = "failed", f"internal_error: {type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001 - an unexpected supervisor failure is a visible failure, never a pass
        status, reason = "failed", f"internal_error: {type(exc).__name__}: {exc}"
    if status != "passed" and run.events and "result" not in run.events[-1]:
        run.events[-1]["result"] = reason
    return {"status": status, "reason": reason}


# --- entry points ---------------------------------------------------------------------------------

def _new_run_id() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:8]


def _summary(run_dir: Optional[Path], payload: Dict[str, Any]) -> Dict[str, Any]:
    if run_dir is not None and run_dir.is_dir():
        _write_json(run_dir / "summary.json", payload)
    return payload


def run_task(*, repo: Path, request_path: Path, runs_root: Path, base_ref: str = "HEAD",
             registry_path: Path = DEFAULT_REGISTRY, catalog_path: Path = DEFAULT_CATALOG, docker: str = "docker") -> Dict[str, Any]:
    """Run one task end to end. Never raises for policy problems; returns the summary dict."""
    started = _now()
    base: Dict[str, Any] = {"summary_version": SUMMARY_VERSION, "started_at": started, "task_id": None, "run_id": None}
    if os.environ.get("ENGRAM_RUNNER_DISABLED") == "1":
        return {**base, "status": "refused", "reason": "runner_disabled (ENGRAM_RUNNER_DISABLED=1)"}
    lock_fd: Optional[int] = None
    run_dir: Optional[Path] = None
    try:
        request = load_request(request_path)
        base["task_id"] = request["task_id"]
        repo = Path(repo).resolve()
        registry_bytes, catalog_bytes = Path(registry_path).read_bytes(), Path(catalog_path).read_bytes()
        registry = _load_registry_bytes(registry_bytes, catalog_bytes)
        if request["policy_version"] != registry.policy_version:
            raise Refused("policy_mismatch", f"request policy {request['policy_version']} != registry {registry.policy_version}")
        check_repo(repo, request, base_ref)
        try:
            lock_fd = os.open(HG.git_common_dir(repo) / LOCK_NAME, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        except (OSError, HG.GitError) as exc:
            raise Refused("runner_lock_unavailable", f"cannot open the repository writer lock: {exc}") from exc
        try:
            fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise Refused("writer_concurrency_cap", "another writer run holds this repository's lock "
                                                    "(writer concurrency is 1 per repository)") from exc
        except OSError as exc:
            raise Refused("runner_lock_unavailable", f"cannot lock the repository writer lock: {exc}") from exc
        root = prepare_runs_root(repo, runs_root)
        run_id = _new_run_id()
        base["run_id"] = run_id
        (root / request["task_id"]).mkdir(mode=0o700, exist_ok=True)
        run_dir = root / request["task_id"] / run_id
        run_dir.mkdir(mode=0o700)
        _stage(run_dir / "policy", {"request.json": request, "registry.json": registry_bytes, "catalog.json": catalog_bytes})
        run = Run(repo, request, run_dir, run_id, Path(registry_path), Path(catalog_path), docker,
                  time.monotonic() + request["budgets"]["wall_seconds"], registry, registry_bytes, catalog_bytes)
        outcome = execute(run)
        return _summary(run_dir, {**base, **outcome, "finished_at": _now(), "run_dir": str(run_dir),
                                  "budgets": budget_report(request["budgets"]), "events": run.events, "result": run.result})
    except Refused as exc:
        return _summary(run_dir, {**base, "status": "refused", "reason": f"{exc.code}: {exc.detail}", "finished_at": _now()})
    except SBX.Refusal as exc:
        return _summary(run_dir, {**base, "status": "refused", "reason": f"registry_invalid: {exc}", "finished_at": _now()})
    finally:
        if lock_fd is not None:
            os.close(lock_fd)


def _load_registry_bytes(registry_bytes: bytes, catalog_bytes: bytes) -> Any:
    """Parse the registry from the exact bytes that will be staged for every phase (no re-read)."""
    scratch = Path(os.path.realpath(os.environ.get("TMPDIR", "/tmp"))) / f"engram-runner-{uuid.uuid4().hex}"
    scratch.mkdir(mode=0o700)
    try:
        (scratch / "registry.json").write_bytes(registry_bytes)
        (scratch / "catalog.json").write_bytes(catalog_bytes)
        return SBX.load_registry(scratch / "registry.json", scratch / "catalog.json")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


def cleanup(*, repo: Path, runs_root: Path, task_id: str, run_id: str) -> List[str]:
    """Remove one run's scratch (writer workspaces and gate checkouts). Never refs, evidence or other runs."""
    if not (REC.TASK_ID_RE.fullmatch(task_id) and REC.RUN_ID_RE.fullmatch(run_id)):
        raise Refused("request_invalid", "invalid task or run id")
    try:  # never creates anything: an absent or unmarked runs root is refused
        root = REC.trusted_runs_root(Path(repo), runs_root)
    except REC.VerifyRefused as exc:
        raise Refused("untrusted_runs_root", exc.detail) from exc
    run_dir = root / task_id / run_id
    if run_dir.is_symlink() or not run_dir.is_dir():
        raise Refused("request_invalid", f"no such run directory {run_dir}")
    removed = []
    for attempt_dir in sorted(p for p in run_dir.iterdir() if REC.ATTEMPT_RE.fullmatch(p.name)):
        if attempt_dir.is_symlink():
            raise Refused("untrusted_runs_root", f"{attempt_dir} is a symlink")
        for scratch in sorted(attempt_dir.iterdir()):
            if (scratch.name == "ws" or re.fullmatch(r"c[0-9]{1,3}", scratch.name)) and not scratch.is_symlink():
                shutil.rmtree(scratch)
                removed.append(str(scratch.relative_to(root)))
    return removed


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Trusted task runner, fake writer only (task H4)")
    sub = parser.add_subparsers(dest="command", required=True)
    run_cmd = sub.add_parser("run", help="run a task request end to end")
    run_cmd.add_argument("--repo", required=True, type=Path)
    run_cmd.add_argument("--request", required=True, type=Path)
    run_cmd.add_argument("--runs-root", required=True, type=Path)
    run_cmd.add_argument("--base-ref", default="HEAD")
    run_cmd.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    run_cmd.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    run_cmd.add_argument("--docker", default="docker")
    clean = sub.add_parser("cleanup", help="remove one run's workspaces and checkouts (keeps refs, logs, evidence)")
    clean.add_argument("--repo", required=True, type=Path)
    clean.add_argument("--runs-root", required=True, type=Path)
    clean.add_argument("--task-id", required=True)
    clean.add_argument("--run-id", required=True)
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    try:
        args = _parser().parse_args(argv)
    except SystemExit as exc:
        return 2 if exc.code else 0
    if args.command == "cleanup":
        try:
            removed = cleanup(repo=args.repo, runs_root=args.runs_root, task_id=args.task_id, run_id=args.run_id)
        except Refused as exc:
            print(f"CLEANUP: REFUSED reason={exc.code} {exc.detail}")
            return 2
        print(f"CLEANUP: OK removed={len(removed)}")
        return 0
    summary = run_task(repo=args.repo, request_path=args.request, runs_root=args.runs_root, base_ref=args.base_ref,
                       registry_path=args.registry, catalog_path=args.catalog, docker=args.docker)
    print(json.dumps(summary, indent=2, sort_keys=True, default=str))
    candidate = (summary.get("result") or {}).get("candidate", {}).get("sha", "")
    print(f"RUN_STATUS: {summary['status'].upper()} task={summary.get('task_id')} run={summary.get('run_id')}"
          f"{' candidate=' + candidate if candidate else ''} reason={summary['reason']}")
    return EXIT.get(summary["status"], 1)


if __name__ == "__main__":
    sys.exit(main())
