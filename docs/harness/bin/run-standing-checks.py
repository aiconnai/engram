#!/usr/bin/env python3
"""run-standing-checks.py - read-only standing checks with ownership (task O4).

ALERT-ONLY. A goal (docs/harness/goals/registry.json, schema goal-v1) SELECTS an approved check by ID from the
H3 check registry and names an accountable owner; it never carries a command. This runner:

  * validates the goals registry strictly (duplicate keys, extra/shell/command fields, unknown check ID,
    timeout above the registry or sandbox policy, missing owner/runbook are all refused);
  * takes a per-goal exclusive lock under <git-common-dir>/engram-standing-checks/ (supervisor-only place,
    never mounted into a container): a concurrent run of the same goal is refused, never queued;
  * exports the run SHA into a plain directory and runs the approved check through the H3 sandbox adapter
    (sandbox-adapter.run_isolated: docker, pinned unix-socket endpoint, no network, no credentials, no
    host fallback). It never starts a writer, never creates a commit, ref, branch, PR or message, and
    re-hashes the checkout afterwards: a check that changed it is a failure, never a pass;
  * records a receipt (run SHA, policy, toolchain, registry/goals hashes, outcome and log hashes) and, on any
    non-pass status, a LOCAL alert report that names the owner and the runbook. Nothing is delivered: the
    hand-off to the owner needs a channel and a human authorization of its own (see the alert `delivery`).

Statuses: pass | fail | timeout | unavailable | refused | error. Only `pass` is a pass. Timeout, failure, a
missing outcome/log artifact, a log hash mismatch, a mutated checkout or a missing Docker are never a pass.

Modes (how the run was triggered; recorded in every receipt):
  manual      explicit --goal only; runs the goal whatever its schedule (including `manual`)
  dispatched  a human pressed the workflow button: explicit --goal, or --schedule for every goal with it
  scheduled   --schedule daily|weekly only (no --goal); goals with schedule `manual` are never selected

Exit codes: 0 every selected goal passed; 1 fail/timeout/error; 2 refused (policy, registry, lock, usage);
3 unavailable (sandbox missing); 4 not-run (nothing selected). Final line:
  STANDING_CHECKS: <STATUS> mode=<mode> selected=<n> passed=<k> summary=<path>
Kill switch: ENGRAM_STANDING_CHECKS_DISABLED=1 refuses every run.
"""

from __future__ import annotations

import argparse
import datetime
import fcntl
import hashlib
import importlib.util
import json
import os
import platform
import re
import stat
import sys
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, List, Mapping, Optional, Sequence, Tuple


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
SBX = _load_sibling("sandbox-adapter")
VE = _load_sibling("validate-evidence", "validate_evidence")

BIN_DIR = Path(__file__).resolve().parent
HARNESS_DIR = BIN_DIR.parent
DEFAULT_GOALS = HARNESS_DIR / "goals" / "registry.json"
DEFAULT_SCHEMA = HARNESS_DIR / "schemas" / "goal-v1.schema.json"
DEFAULT_REGISTRY, DEFAULT_CATALOG = SBX.DEFAULT_REGISTRY, SBX.DEFAULT_CATALOG
TCB_FILES = ("run-standing-checks.py", "sandbox-adapter.py", "sandbox_registry.py", "harness_git.py", "validate-evidence.py")

GOALS_VERSION = "goal-v1"
RECEIPT_VERSION = "standing-receipt-v1"
ALERT_VERSION = "standing-alert-v1"
SUMMARY_VERSION = "standing-summary-v1"
MODES = ("manual", "dispatched", "scheduled")
SCHEDULES = ("daily", "weekly", "manual")
SCHEDULER_SCHEDULES = ("daily", "weekly")
GOAL_ID_RE = re.compile(r"[a-z0-9][a-z0-9_-]{0,63}")
LOCK_DIR = "engram-standing-checks"
MAX_INPUT_BYTES = 1024 * 1024
SNAPSHOT_MAX_ENTRIES, SNAPSHOT_MAX_BYTES = 50000, 512 * 1024 * 1024
# The only capabilities a standing check is ever granted: read the export, run an approved check, no network.
READ_ONLY_CAPABILITIES = ("workspace_read", "test_exec", "sandbox_container", "network_none")

PASS, FAIL, TIMEOUT, UNAVAILABLE, REFUSED, ERROR = "pass", "fail", "timeout", "unavailable", "refused", "error"
ADAPTER_TO_STATUS = {SBX.STATUS_FAILED: FAIL, SBX.STATUS_TIMEOUT: TIMEOUT, SBX.STATUS_UNAVAILABLE: UNAVAILABLE,
                     SBX.STATUS_REFUSED: REFUSED, SBX.STATUS_ERROR: ERROR}
DELIVERY_NOTE = ("local report only: nothing was sent. Hand-off to the owner needs a channel and a human "
                 "authorization of its own; this runner never messages, commits, opens PRs or remediates.")


class Refusal(Exception):
    """Input or policy problem: nothing runs. Carries a stable code."""

    def __init__(self, code: str, detail: str = ""):
        super().__init__(f"{code}: {detail}" if detail else code)
        self.code, self.detail = code, detail


def _now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _write_json(path: Path, payload: Any) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def _read_regular(path: Path, what: str) -> bytes:
    try:
        info = os.lstat(path)
    except OSError as exc:
        raise Refusal("input_unreadable", f"{what} {path}: {exc.strerror or type(exc).__name__}") from exc
    if not stat.S_ISREG(info.st_mode) or info.st_size > MAX_INPUT_BYTES:
        raise Refusal("input_unsafe", f"{what} {path} must be a regular file (no symlink) up to {MAX_INPUT_BYTES} bytes")
    return Path(path).read_bytes()


def _parse_strict(raw: bytes, what: str) -> Any:
    try:
        return VE.parse_json_strict(raw.decode("utf-8"))
    except (UnicodeDecodeError, VE.StrictJsonError) as exc:
        raise Refusal("input_invalid", f"{what}: {exc}") from exc


# --- goals registry ---------------------------------------------------------------------------------

@dataclass(frozen=True)
class Goal:
    id: str
    check_id: str
    owner: str
    schedule: str
    timeout_seconds: int
    on_failure: str
    runbook: str

    def to_dict(self) -> Dict[str, Any]:
        return {"id": self.id, "check_id": self.check_id, "owner": self.owner, "schedule": self.schedule,
                "timeout_seconds": self.timeout_seconds, "on_failure": self.on_failure, "runbook": self.runbook}


def parse_goals(goals_raw: bytes, schema_raw: bytes, registry: Any, runbook_exists: Callable[[str], bool]) -> Tuple[Goal, ...]:
    """Strictly validate the goals registry against goal-v1 and the H3 registry policy. Raises Refusal."""
    data = _parse_strict(goals_raw, "goals registry")
    schema = _parse_strict(schema_raw, "goal schema")
    if not isinstance(schema, dict):
        raise Refusal("input_invalid", "goal schema is not an object")
    errors = VE.PureSchemaValidator(schema).validate(data)
    if not errors and VE.HAS_JSONSCHEMA:
        try:
            divergent = VE._jsonschema_errors(VE.jsonschema, schema, data)
        except Exception as exc:  # noqa: BLE001 - a crashing cross-check fails closed
            raise Refusal("goals_invalid", f"jsonschema cross-check crashed: {type(exc).__name__}") from exc
        if divergent:
            raise Refusal("goals_invalid", f"jsonschema rejects what the built-in validator accepted: {divergent[0].message}")
    if errors:
        raise Refusal("goals_invalid", "; ".join(str(e) for e in errors[:3]))
    if data["policy_version"] != registry.policy_version:
        raise Refusal("policy_mismatch", f"goals policy {data['policy_version']} != check registry {registry.policy_version}")
    goals: List[Goal] = []
    seen = set()
    for raw in data["goals"]:
        goal = Goal(**raw)
        if goal.id in seen:
            raise Refusal("goals_invalid", f"duplicate goal id {goal.id!r}")
        seen.add(goal.id)
        spec = registry.checks.get(goal.check_id)
        if spec is None:
            raise Refusal("unknown_check", f"goal {goal.id}: check '{goal.check_id}' is not in the approved check registry")
        limit = min(spec.timeout_seconds, registry.sandbox.max_total_timeout_seconds)
        if goal.timeout_seconds > limit:
            raise Refusal("timeout_above_policy", f"goal {goal.id}: timeout {goal.timeout_seconds}s exceeds the registry limit {limit}s")
        if not runbook_exists(goal.runbook):
            raise Refusal("runbook_missing", f"goal {goal.id}: runbook {goal.runbook} does not exist")
        goals.append(goal)
    return tuple(goals)


def select_goals(goals: Sequence[Goal], mode: str, schedule: Optional[str], goal_ids: Sequence[str]) -> List[Goal]:
    """Mode rules decide WHICH goals run; a scheduler never picks a goal by name nor runs a `manual` one."""
    if mode not in MODES:
        raise Refusal("usage", f"mode must be one of {MODES}")
    by_id = {g.id: g for g in goals}
    unknown = [g for g in goal_ids if g not in by_id]
    if unknown:
        raise Refusal("unknown_goal", f"goal(s) not in the registry: {unknown}")
    if schedule is not None and schedule not in SCHEDULER_SCHEDULES:
        raise Refusal("usage", f"--schedule must be one of {SCHEDULER_SCHEDULES}")
    if mode == "manual":
        if not goal_ids or schedule is not None:
            raise Refusal("usage", "manual mode needs explicit --goal and no --schedule")
    elif mode == "dispatched":
        if bool(goal_ids) == (schedule is not None):
            raise Refusal("usage", "dispatched mode needs exactly one of --goal or --schedule")
    elif schedule is None or goal_ids:
        raise Refusal("usage", "scheduled mode needs --schedule and never --goal")
    if goal_ids:
        return [by_id[g] for g in dict.fromkeys(goal_ids)]
    return [g for g in goals if g.schedule == schedule]


# --- filesystem guards ------------------------------------------------------------------------------

def _own_private_dir(path: Path, what: str) -> Path:
    if not path.exists():
        path.mkdir(parents=True, mode=0o700)
    info = os.lstat(path)
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.geteuid() or info.st_mode & 0o022:
        raise Refusal("untrusted_dir", f"{what} {path} must be a directory owned by the current user, not group/world writable")
    return Path(os.path.realpath(path))


def prepare_output_dir(repo: Path, path: Path, what: str) -> Path:
    if not Path(path).is_absolute():
        raise Refusal("untrusted_dir", f"{what} must be an absolute path")
    for untrusted in HG.untrusted_roots(repo):  # checked before anything is created there
        if HG.is_within(Path(path), untrusted) or HG.is_within(untrusted, Path(path)):
            raise Refusal("untrusted_dir", f"{what} overlaps a worktree or git dir: {untrusted}")
    return _own_private_dir(Path(path), what)


def acquire_goal_lock(repo: Path, goal_id: str) -> int:
    """Exclusive non-blocking lock under the git common dir (supervisor-only; never mounted anywhere)."""
    try:
        lock_dir = HG.git_common_dir(repo) / LOCK_DIR
        lock_dir.mkdir(mode=0o700, exist_ok=True)
        if lock_dir.is_symlink():
            raise Refusal("goal_lock_unavailable", "lock directory is a symlink")
        fd = os.open(lock_dir / f"{goal_id}.lock", os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    except (OSError, HG.GitError) as exc:
        raise Refusal("goal_lock_unavailable", f"cannot open the goal lock: {exc}") from exc
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as exc:
        os.close(fd)
        raise Refusal("goal_locked", f"another run of goal '{goal_id}' holds its lock") from exc
    except OSError as exc:
        os.close(fd)
        raise Refusal("goal_lock_unavailable", f"cannot lock the goal: {exc}") from exc
    return fd


def snapshot_dir(root: Path) -> Dict[str, Tuple[Any, ...]]:
    """Bounded, git-free fingerprint of a directory (nothing is written anywhere)."""
    snap: Dict[str, Tuple[Any, ...]] = {}
    total = 0
    for dirpath, dirnames, filenames in os.walk(root, followlinks=False):
        dirnames.sort()
        for name in [*dirnames, *sorted(filenames)]:
            path = os.path.join(dirpath, name)
            rel = os.path.relpath(path, root)
            info = os.lstat(path)
            if len(snap) >= SNAPSHOT_MAX_ENTRIES:
                raise Refusal("snapshot_limit", f"checkout has more than {SNAPSHOT_MAX_ENTRIES} entries")
            if stat.S_ISLNK(info.st_mode):
                snap[rel] = ("link", os.readlink(path))
            elif stat.S_ISDIR(info.st_mode):
                snap[rel] = ("dir", stat.S_IMODE(info.st_mode))
            elif stat.S_ISREG(info.st_mode):
                total += info.st_size
                if total > SNAPSHOT_MAX_BYTES:
                    raise Refusal("snapshot_limit", f"checkout is larger than {SNAPSHOT_MAX_BYTES} bytes")
                digest = hashlib.sha256()
                with open(path, "rb") as handle:
                    for chunk in iter(lambda: handle.read(1 << 20), b""):
                        digest.update(chunk)
                snap[rel] = ("file", stat.S_IMODE(info.st_mode), digest.hexdigest())
            else:
                snap[rel] = ("special", stat.S_IFMT(info.st_mode))
    return snap


# --- outcome classification (never a pass without every artifact) -----------------------------------

def _file_sha(path: Path) -> Optional[str]:
    try:
        if path.is_symlink() or not path.is_file():
            return None
        return _sha256_bytes(path.read_bytes())
    except OSError:
        return None


def classify(outcome: Any, spec: Any, sandbox_dir: Path) -> Tuple[str, str, List[Dict[str, Any]]]:
    """Map a sandbox outcome to (status, reason, log records). `pass` needs every artifact to be present and consistent."""
    status = ADAPTER_TO_STATUS.get(outcome.status)
    if status is not None:
        return status, outcome.reason or f"sandbox status {outcome.status}", _log_records(outcome, sandbox_dir)[0]
    if outcome.status != SBX.STATUS_PASSED:
        return ERROR, f"unknown sandbox status {outcome.status!r}", []
    logs, log_problems = _log_records(outcome, sandbox_dir)
    problems = list(log_problems)
    record = outcome.checks[0] if len(outcome.checks) == 1 else None
    if outcome.exit_code != 0:
        problems.append(f"exit code {outcome.exit_code!r} is not 0")
    if record is None or record.get("id") != spec.id or record.get("status") != SBX.STATUS_PASSED or record.get("exit_code") != 0:
        problems.append("the outcome does not hold exactly one passed record for the approved check")
    elif not record.get("container_removed"):
        problems.append("the check container was not verified removed")
    if tuple(outcome.argv) != (tuple(spec.argv),):
        problems.append("the argv that ran differs from the approved registry argv")
    if outcome.limits_enforced.get("verified") is not True:
        problems.append("container limits were not verified by docker inspect")
    persisted = sandbox_dir / "outcome.json"
    try:
        data = json.loads(persisted.read_text(encoding="utf-8")) if persisted.is_file() and not persisted.is_symlink() else None
    except (OSError, ValueError):
        data = None
    if not isinstance(data, dict) or data.get("status") != SBX.STATUS_PASSED or data.get("run_id") != outcome.run_id:
        problems.append("outcome.json is missing or does not match the run")
    if problems:
        return ERROR, "artifact verification failed: " + "; ".join(problems), logs
    return PASS, "", logs


def _log_records(outcome: Any, sandbox_dir: Path) -> Tuple[List[Dict[str, Any]], List[str]]:
    records: List[Dict[str, Any]] = []
    problems: List[str] = []
    for check in outcome.checks:
        for label in ("stdout", "stderr"):
            raw_path = check.get(f"{label}_log")
            expected = (check.get("log_sha256") or {}).get(label)
            if not raw_path:
                continue
            path = Path(raw_path)
            actual = _file_sha(path) if HG.is_within(path, sandbox_dir) else None
            records.append({"check": check.get("id"), "stream": label, "path": str(path), "sha256": actual})
            if actual is None:
                problems.append(f"{label} log is missing or outside the run directory")
            elif not (check.get("truncated") or {}).get(label) and actual != expected:
                problems.append(f"{label} log hash differs from the supervisor record")
    if outcome.status == SBX.STATUS_PASSED and not records:
        problems.append("no log was recorded for a passing check")
    return records, problems


# --- one goal ---------------------------------------------------------------------------------------

@dataclass
class Batch:
    repo: Path
    mode: str
    schedule: Optional[str]
    batch_id: str
    sha: str
    tree: str
    runs_root: Path
    alerts_dir: Path
    docker: str
    registry: Any
    registry_raw: bytes
    catalog_raw: bytes
    goals_raw: bytes
    schema_raw: bytes
    policy_version: str
    tcb: Dict[str, str]
    toolchain: Dict[str, str]


def _manifest(batch: Batch, goal: Goal, run_id: str) -> Dict[str, Any]:
    return {
        "schema_version": "task-v2", "policy_version": batch.policy_version, "task_id": f"standing-{goal.id}-{run_id}"[:128],
        "target_sha": batch.sha, "base_sha": batch.sha, "allowed_capabilities": list(READ_ONLY_CAPABILITIES),
        "required_checks": [goal.check_id], "timeout_seconds": goal.timeout_seconds,
        "allowed_paths": ["docs/harness/"], "protected_paths": [".git/", ".github/"],
        "metadata": {"standing_goal": goal.id, "standing_mode": batch.mode, "standing_batch": batch.batch_id},
    }


def _stage(directory: Path, files: Mapping[str, Any]) -> None:
    directory.mkdir(mode=0o700)
    for name, payload in files.items():
        if isinstance(payload, bytes):
            fd = os.open(directory / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, "wb") as handle:
                handle.write(payload)
        else:
            _write_json(directory / name, payload)


def _execute(batch: Batch, goal: Goal, run_dir: Path, run_id: str) -> Tuple[str, str, Dict[str, Any]]:
    """Export, run in the sandbox, classify, re-check the checkout. Returns (status, reason, evidence)."""
    spec = batch.registry.checks[goal.check_id]
    inputs, checkout, sandbox_dir = run_dir / "inputs", run_dir / "checkout", run_dir / "sandbox"
    _stage(inputs, {"manifest.json": _manifest(batch, goal, run_id), "registry.json": batch.registry_raw,
                    "catalog.json": batch.catalog_raw})
    checkout.mkdir(mode=0o700)
    HG.export_tree(batch.repo, batch.sha, checkout)
    before = snapshot_dir(checkout)
    outcome = SBX.run_isolated(inputs / "manifest.json", checkout, sandbox_dir, registry_path=inputs / "registry.json",
                               catalog_path=inputs / "catalog.json", docker_bin=batch.docker)
    status, reason, logs = classify(outcome, spec, sandbox_dir)
    mutated = snapshot_dir(checkout) != before
    if mutated and status == PASS:
        status, reason = FAIL, "gate_mutated_checkout: the check changed the exported checkout"
    evidence = {"outcome_status": outcome.status, "outcome_exit_code": outcome.exit_code,
                "outcome_sha256": _file_sha(sandbox_dir / "outcome.json"), "outcome_path": str(sandbox_dir / "outcome.json"),
                "image": outcome.image, "docker_endpoint": outcome.docker_endpoint, "logs": logs,
                "checkout_unchanged": not mutated}
    return status, reason, evidence


def run_goal(batch: Batch, goal: Goal) -> Dict[str, Any]:
    started, run_id = _now(), uuid.uuid4().hex[:16]
    result: Dict[str, Any] = {"goal_id": goal.id, "run_id": run_id, "status": ERROR, "reason": "", "receipt": None, "alert": None}
    lock_fd: Optional[int] = None
    run_dir: Optional[Path] = None
    evidence: Dict[str, Any] = {}
    try:
        lock_fd = acquire_goal_lock(batch.repo, goal.id)
        goal_root = batch.runs_root / "goals" / goal.id
        goal_root.mkdir(parents=True, mode=0o700, exist_ok=True)
        run_dir = goal_root / run_id
        run_dir.mkdir(mode=0o700)
        result["status"], result["reason"], evidence = _execute(batch, goal, run_dir, run_id)
    except Refusal as exc:
        result["status"] = REFUSED
        result["reason"] = str(exc)
    except (HG.WorkspaceError, HG.GitError, OSError) as exc:
        result["status"], result["reason"] = ERROR, f"{type(exc).__name__}: {exc}"
    except Exception as exc:  # noqa: BLE001 - an unexpected supervisor failure must be a visible non-pass
        result["status"], result["reason"] = ERROR, f"internal error: {type(exc).__name__}: {exc}"
    finally:
        if lock_fd is not None:
            os.close(lock_fd)
    receipt = _receipt(batch, goal, result, started, evidence)
    if run_dir is not None:
        try:
            _write_json(run_dir / "receipt.json", receipt)
            result["receipt"] = str(run_dir / "receipt.json")
        except OSError as exc:
            result["status"] = ERROR if result["status"] == PASS else result["status"]
            result["reason"] = f"{result['reason']} (receipt not written: {type(exc).__name__})".strip()
    if result["status"] != PASS:
        result["alert"] = write_alert(batch, goal, result, receipt)
    return result


def _receipt(batch: Batch, goal: Goal, result: Mapping[str, Any], started: str, evidence: Mapping[str, Any]) -> Dict[str, Any]:
    return {
        "receipt_version": RECEIPT_VERSION, "goal": goal.to_dict(), "mode": batch.mode, "schedule": batch.schedule,
        "batch_id": batch.batch_id, "run_id": result["run_id"], "status": result["status"], "reason": result["reason"],
        "started_at": started, "finished_at": _now(), "run_sha": batch.sha, "tree_sha": batch.tree,
        "policy_version": batch.policy_version, "toolchain": {**batch.toolchain, "image": evidence.get("image", ""),
                                                              "docker_endpoint": evidence.get("docker_endpoint", "")},
        "inputs_sha256": {"goals": _sha256_bytes(batch.goals_raw), "goal_schema": _sha256_bytes(batch.schema_raw),
                          "check_registry": _sha256_bytes(batch.registry_raw), "check_catalog": _sha256_bytes(batch.catalog_raw)},
        "tcb_sha256": batch.tcb, "outcome": {k: evidence.get(k) for k in ("outcome_status", "outcome_exit_code", "outcome_path", "outcome_sha256")},
        "logs": evidence.get("logs", []), "checkout_unchanged": evidence.get("checkout_unchanged"),
        "authority": {"writer_started": False, "repo_mutated": False, "network": "none", "credentials": "none",
                      "on_failure": goal.on_failure, "delivery": "local-only"},
    }


def write_alert(batch: Batch, goal: Goal, result: Mapping[str, Any], receipt: Mapping[str, Any]) -> Optional[str]:
    """LOCAL alert report naming owner and runbook. Never sends anything."""
    receipt_path = result.get("receipt")
    alert = {
        "alert_version": ALERT_VERSION, "goal_id": goal.id, "check_id": goal.check_id, "owner": goal.owner,
        "runbook": goal.runbook, "status": result["status"], "reason": result["reason"], "run_id": result["run_id"],
        "run_sha": batch.sha, "mode": batch.mode, "batch_id": batch.batch_id, "receipt_path": receipt_path,
        "receipt_sha256": _file_sha(Path(receipt_path)) if receipt_path else None,
        "log_sha256": {f"{r['check']}.{r['stream']}": r["sha256"] for r in receipt.get("logs", [])},
        "remediation": "none (alert_only)", "delivery": {"sent": False, "channel": None, "note": DELIVERY_NOTE},
        "created_at": _now(),
    }
    stamp = alert["created_at"].replace(":", "").replace(".", "")
    path = batch.alerts_dir / f"{stamp}-{goal.id}-{result['run_id']}.json"
    try:
        _write_json(path, alert)
    except OSError:
        return None
    return str(path)


# --- batch ------------------------------------------------------------------------------------------

def _aggregate(results: Sequence[Mapping[str, Any]]) -> Tuple[str, int]:
    statuses = {r["status"] for r in results}
    if not results:
        return "not-run", 4
    if statuses & {FAIL, TIMEOUT, ERROR}:
        return "fail", 1
    if REFUSED in statuses:
        return "refused", 2
    if UNAVAILABLE in statuses:
        return "unavailable", 3
    return "pass", 0


def _tcb_hashes() -> Dict[str, str]:
    return {name: _sha256_bytes((BIN_DIR / name).read_bytes()) for name in TCB_FILES}


def _tool_versions(repo: Path) -> Dict[str, str]:
    return {"python": platform.python_version(), "git": HG.git(repo, ["version"]).decode().strip().replace("git version ", "")}


def run_standing_checks(*, repo: Path, runs_root: Path, mode: str, schedule: Optional[str] = None,
                        goal_ids: Sequence[str] = (), goals_path: Path = DEFAULT_GOALS, schema_path: Path = DEFAULT_SCHEMA,
                        registry_path: Path = DEFAULT_REGISTRY, catalog_path: Path = DEFAULT_CATALOG, docker: str = "docker",
                        ref: str = "HEAD", alerts_dir: Optional[Path] = None) -> Dict[str, Any]:
    """Run the selected goals once. Never raises for policy problems; returns the summary dict (see `exit_code`)."""
    summary: Dict[str, Any] = {"summary_version": SUMMARY_VERSION, "mode": mode, "schedule": schedule, "started_at": _now(),
                               "results": [], "selected": 0, "passed": 0}
    try:
        if os.environ.get("ENGRAM_STANDING_CHECKS_DISABLED") == "1":
            raise Refusal("standing_checks_disabled", "ENGRAM_STANDING_CHECKS_DISABLED=1")
        repo = Path(repo).resolve()
        if HG.toplevel(repo) is None:
            raise Refusal("wrong_repo", f"{repo} is not a git repository")
        sha = HG.resolve_commit(repo, ref)
        if sha is None:
            raise Refusal("wrong_repo", f"{ref} does not resolve to a commit of {repo}")
        registry_raw, catalog_raw = _read_regular(registry_path, "check registry"), _read_regular(catalog_path, "check catalog")
        goals_raw, schema_raw = _read_regular(goals_path, "goals registry"), _read_regular(schema_path, "goal schema")
        try:
            registry = SBX.load_registry(registry_path, catalog_path)
        except SBX.Refusal as exc:
            raise Refusal("registry_invalid", str(exc)) from exc
        goals = parse_goals(goals_raw, schema_raw, registry, lambda rel: _blob_exists(repo, sha, rel))
        selected = select_goals(goals, mode, schedule, goal_ids)
        runs = prepare_output_dir(repo, runs_root, "runs root")
        alerts = prepare_output_dir(repo, alerts_dir if alerts_dir is not None else runs / "alerts", "alerts dir")
        batch = Batch(repo, mode, schedule, uuid.uuid4().hex[:16], sha, HG.tree_of(repo, sha), runs, alerts, docker, registry,
                      registry_raw, catalog_raw, goals_raw, schema_raw, registry.policy_version, _tcb_hashes(), _tool_versions(repo))
        summary.update(run_sha=sha, batch_id=batch.batch_id, policy_version=batch.policy_version, selected=len(selected))
        summary["results"] = [run_goal(batch, goal) for goal in selected]
        summary["passed"] = sum(1 for r in summary["results"] if r["status"] == PASS)
        summary["status"], summary["exit_code"] = _aggregate(summary["results"])
        summary["path"] = _write_summary(runs, summary)
    except Refusal as exc:
        summary.update(status="refused", exit_code=2, reason=str(exc), path=None)
    except (HG.GitError, OSError) as exc:
        summary.update(status="refused", exit_code=2, reason=f"{type(exc).__name__}: {exc}", path=None)
    summary["finished_at"] = _now()
    return summary


def _blob_exists(repo: Path, sha: str, rel: str) -> bool:
    try:
        HG.git(repo, ["cat-file", "-e", f"{sha}:{rel}"])
    except HG.GitError:
        return False
    return True


def _write_summary(runs: Path, summary: Mapping[str, Any]) -> Optional[str]:
    directory = runs / "summaries"
    try:
        directory.mkdir(mode=0o700, exist_ok=True)
        path = directory / f"{summary['batch_id']}.json"
        _write_json(path, {**summary, "finished_at": _now()})
        return str(path)
    except OSError:
        return None


# --- CLI --------------------------------------------------------------------------------------------

def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Read-only standing checks with ownership (alert-only; see the module docstring)")
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("validate", "run"):
        cmd = sub.add_parser(name, help="validate the goals registry" if name == "validate" else "run selected goals once")
        cmd.add_argument("--goals", type=Path, default=DEFAULT_GOALS)
        cmd.add_argument("--schema", type=Path, default=DEFAULT_SCHEMA)
        cmd.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
        cmd.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    run_cmd = sub.choices["run"]
    run_cmd.add_argument("--mode", required=True, choices=MODES)
    run_cmd.add_argument("--schedule", choices=SCHEDULER_SCHEDULES)
    run_cmd.add_argument("--goal", action="append", default=[], help="goal id (repeatable); not allowed in scheduled mode")
    run_cmd.add_argument("--repo", type=Path, required=True)
    run_cmd.add_argument("--runs-root", type=Path, required=True)
    run_cmd.add_argument("--alerts-dir", type=Path)
    run_cmd.add_argument("--ref", default="HEAD")
    run_cmd.add_argument("--docker", default="docker")
    return parser


def _validate_cli(args: argparse.Namespace) -> int:
    try:
        registry = SBX.load_registry(args.registry, args.catalog)
        root = HARNESS_DIR.parent.parent
        goals = parse_goals(_read_regular(args.goals, "goals registry"), _read_regular(args.schema, "goal schema"), registry,
                            lambda rel: (root / rel).is_file())
    except (Refusal, SBX.Refusal) as exc:
        print(f"GOALS: FAIL {exc}", file=sys.stderr)
        return 2
    print(f"GOALS: OK goals={len(goals)} owners={len({g.owner for g in goals})}")
    return 0


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.command == "validate":
        return _validate_cli(args)
    summary = run_standing_checks(repo=args.repo, runs_root=args.runs_root, mode=args.mode, schedule=args.schedule,
                                  goal_ids=args.goal, goals_path=args.goals, schema_path=args.schema,
                                  registry_path=args.registry, catalog_path=args.catalog, docker=args.docker, ref=args.ref,
                                  alerts_dir=args.alerts_dir)
    for item in summary["results"]:
        if item["alert"]:
            print(f"ALERT: goal={item['goal_id']} status={item['status']} report={item['alert']}", file=sys.stderr)
    print(json.dumps(summary, indent=2, sort_keys=True))
    print(f"STANDING_CHECKS: {summary['status'].upper()} mode={summary['mode']} selected={summary['selected']} "
          f"passed={summary['passed']} summary={summary.get('path')}")
    return summary["exit_code"]


if __name__ == "__main__":
    sys.exit(main())
