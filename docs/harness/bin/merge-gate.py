#!/usr/bin/env python3
"""merge-gate.py - read-only, deterministic merge-policy evaluator (task H5).

It answers one question - is this exact PR head ELIGIBLE for a human merge decision under the
merge policy? - and never acts on the answer: it does not approve, merge, deploy, publish, push or
write anything to the repository. The human owner still decides; `eligible` only means that every
piece of trusted evidence is present, consistent and bound to the same candidate.

Trust model (ADR 2026-07-21 "Agent harness hardening v1", Wave 4):
  * The evaluator, merge_gate_ci.py, the check registry / catalog, the review-v2 schema and the
    security-gate policy are read from the checkout THIS script lives in. Run it from a trusted
    revision (the base branch, or a copy taken from it) against the candidate repository given
    with --repo; a script changed by the PR under evaluation must never judge that PR.
  * Head and base are re-resolved from git (--head-ref / --base-ref), never read from a payload,
    and re-checked (with --integrated-ref) immediately before the decision is emitted. When a ref
    is a raw SHA (the CI workflow) that re-check is vacuous: the human merge must still compare the
    live PR head with `bound.head`.
  * Operator-owned inputs (review receipt, identity allowlist, CI check runs, workflow runs,
    review-gate copy) are opened once without following symlinks and validated on the open file
    (regular, one link, ours, not group/world writable, no write ACL), with a parent directory that
    is ours and not group/world writable, ancestors that are ours or root and not writable by
    others unless sticky, outside every
    worktree / git dir of the judged repository. Free-text fields written by a writer or reviewer
    (`reviewer`, a lineage name, a "PASS") never authenticate anything.
  * Trusted-runner evidence is accepted only through record-evidence.py verify() (task H4).

Decision sections (each must be "pass" for `eligible`; anything missing, unknown or inconsistent
is refused): refs, evidence, gate_history, review_receipt, review_scope, review, reviewer, lineage,
ci, integrated, head_recheck. Output: one JSON document on stdout; summary line on stderr.
Exit codes: 0 eligible, 1 refused, 2 usage error.
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
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, List, Mapping, Optional, Sequence, Tuple

BIN_DIR = Path(__file__).resolve().parent
REPO_ROOT = BIN_DIR.parent.parent.parent
SECURITY_GATE_SCRIPT = REPO_ROOT / "scripts" / "check-security-gate.py"


def _load(path: Path, module_name: str) -> Any:
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


RE = _load(BIN_DIR / "record-evidence.py", "record_evidence")
CI = _load(BIN_DIR / "merge_gate_ci.py", "merge_gate_ci")
HG, VE = RE.HG, RE.VE

EVALUATOR_ID = "engram-merge-gate-v1"
# The merge policy each accepted harness policy version maps to, pinned to the fingerprint of the CI
# constants in merge_gate_ci.py: changing those constants without re-pinning here is
# `policy_mismatch`. Any other policy version is refused.
MERGE_POLICIES = {"harness-hardening-v1": {
    "id": "merge-policy-v1",
    "ci_policy_sha256": "763198f22702c0f19d83ea332c9920a80676d640370fba0ec369af239827edb6"}}
REQUIRED_CI_CONTEXTS = CI.REQUIRED_CI_CONTEXTS
IDENTITIES_VERSION = "merge-gate-identities-v1"
REVIEW_CONTEXT_ATTESTED = "fresh-readonly"
SECTIONS = ("refs", "evidence", "gate_history", "review_receipt", "review_scope", "review",
            "reviewer", "lineage", "ci", "integrated", "head_recheck")
RECEIPT_V1_KEYS = ("RECEIPT_VERSION", "TASK_ID", "BASE_SHA", "HEAD_SHA", "TREE_SHA", "DIFF_SHA256",
                   "REVIEW_ARTIFACT", "REVIEW_SHA256", "OPERATOR", "GATE_SCRIPT_SHA256")
RECEIPT_V2_KEYS = RECEIPT_V1_KEYS + ("REVIEWER", "POLICY_VERSION", "REVIEW_CONTEXT")
SCOPE_HEADER = ("SCOPE_MODE", "TASK_ID", "BASE_SHA", "HEAD_SHA", "TREE_SHA", "DIFF_SHA256")
TASK_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]{0,63}")
OID_RE = re.compile(r"[0-9a-f]{40}(?:[0-9a-f]{24})?")
SHA256_RE = re.compile(r"[0-9a-f]{64}")
IDENT_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._@-]{0,127}")
PRINTABLE_RE = re.compile(r"[ -~]{1,128}")
PLACEHOLDER_RE = re.compile(r"<.*>")
LEGACY_MARKER_RE = re.compile(rb"^\s*REVIEW_VERDICT:", re.MULTILINE | re.IGNORECASE)
MAX_RECEIPT_BYTES = 64 * 1024
MAX_GATE_BYTES = 4 * 1024 * 1024
MAX_JSON_BYTES = 16 * 1024 * 1024


class InputError(Exception):
    """An operator-owned input is unusable. `code` is the refusal reason."""

    def __init__(self, code: str, detail: str):
        super().__init__(f"{code}: {detail}")
        self.code, self.detail = code, detail


@dataclass(frozen=True)
class Inputs:
    repo: Path
    task: str
    head_ref: str
    base_ref: str
    policy_version: str
    expect_head: Optional[str] = None
    expect_base: Optional[str] = None
    runs_root: Optional[Path] = None
    evidence_receipt: Optional[Path] = None
    review_receipt: Optional[Path] = None
    review_gate: Optional[Path] = None
    expect_gate_sha256: Optional[str] = None
    identities: Optional[Path] = None
    ci_results: Optional[Path] = None
    workflow_runs: Optional[Path] = None
    integrated_ref: Optional[str] = None
    registry: Path = RE.DEFAULT_REGISTRY
    catalog: Path = RE.DEFAULT_CATALOG
    schemas_dir: Path = RE.SCHEMAS_DIR
    now: Optional[datetime.datetime] = None


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class Evaluation:
    """Collects refusal reasons per section; eligible only when every section passed."""

    def __init__(self) -> None:
        self.reasons: List[Dict[str, str]] = []
        self.sections: Dict[str, str] = {name: "not_evaluated" for name in SECTIONS}
        self.bound: Dict[str, Any] = {"task": None, "base": None, "head": None, "tree": None,
                                      "policy": None, "merge_policy": None,
                                      "ci_policy_sha256": CI.policy_fingerprint(),
                                      "evaluator_sha256": sha256_bytes(Path(__file__).read_bytes())}

    def refuse(self, section: str, code: str, detail: str) -> None:
        self.reasons.append({"section": section, "code": code, "detail": detail[:600]})
        self.sections[section] = "fail"

    def passed(self, section: str) -> None:
        if self.sections[section] == "not_evaluated":
            self.sections[section] = "pass"

    @property
    def eligible(self) -> bool:
        return not self.reasons and all(v == "pass" for v in self.sections.values())

    def document(self) -> Dict[str, Any]:
        if not self.eligible and not self.reasons:
            pending = [k for k, v in self.sections.items() if v != "pass"]
            self.reasons.append({"section": "evaluator", "code": "incomplete_evaluation",
                                 "detail": "sections not evaluated: " + ", ".join(pending)})
        head = self.bound.get("head")
        recheck = (f"the PR head must still be {head} at the moment of the human merge; any other "
                   "head needs a new evaluation") if head else "no head was resolved"
        return {
            "evaluator": EVALUATOR_ID, "decision": "eligible" if self.eligible else "refused",
            "reasons": self.reasons, "sections": self.sections, "bound": self.bound,
            "authority": ("decision only: nothing was approved, merged, deployed or published; "
                          "a human decides the merge"),
            "recheck_before_merge": recheck,
        }


# --- trusted operator-owned files ---------------------------------------------------------------

def _acl_grants_write(path: Path) -> bool:
    """True when an ACL grants write-like access or cannot be inspected (fail closed)."""
    flag = "-lde" if sys.platform == "darwin" else "-ld"
    try:
        listing = subprocess.run(["ls", flag, "--", str(path)], capture_output=True, text=True,
                                 timeout=10, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return True
    lines = listing.stdout.splitlines()
    if listing.returncode != 0 or not lines:
        return True
    perms = lines[0].split(" ", 1)[0]
    if len(perms) < 11 or perms[10] not in "+@":
        return False
    if sys.platform != "darwin":
        return True  # an ACL we do not inspect: refuse rather than guess
    write = r"allow .*(write|append|delete|add_file|add_subdirectory|chown)"
    return any(re.search(write, line) for line in lines[1:])


def _private(info: os.stat_result) -> bool:
    return info.st_uid == os.geteuid() and not info.st_mode & 0o022


def _ancestor_ok(info: os.stat_result) -> bool:
    """Ours or root; writable by others only with the sticky bit (e.g. /tmp)."""
    owner_ok = info.st_uid in (os.geteuid(), 0)
    return owner_ok and (not info.st_mode & 0o022 or bool(info.st_mode & stat.S_ISVTX))


def _read_fd(fd: int, limit: int, what: str) -> bytes:
    chunks, total = [], 0
    while True:
        chunk = os.read(fd, 1024 * 1024)
        if not chunk:
            return b"".join(chunks)
        total += len(chunk)
        if total > limit:
            raise InputError(f"{what}_malformed", f"{what} exceeds {limit} bytes")
        chunks.append(chunk)


def trusted_input(path: Path, untrusted: Sequence[Path], what: str,
                  limit: int) -> Tuple[Path, bytes]:
    """(physical path, bytes) of an operator-owned file, or InputError(<what>_untrusted)."""
    code = f"{what}_untrusted"
    if not Path(path).is_absolute():
        raise InputError(code, f"{what} path must be absolute: {path}")
    try:
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | getattr(os, "O_NONBLOCK", 0))
    except FileNotFoundError:
        raise InputError(f"{what}_unavailable", f"{what} not found: {path}") from None
    except OSError as exc:
        raise InputError(code, f"{what} cannot be opened without following links: {exc}") from None
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or not _private(info):
            raise InputError(code, f"{what} must be a regular, singly linked file of ours that is "
                                   f"not group/world writable: {path}")
        data = _read_fd(fd, limit, what)
    finally:
        os.close(fd)
    phys = Path(os.path.realpath(path))
    if not _private(os.lstat(phys.parent)):
        raise InputError(code, f"{what} directory must be ours and not group/world writable: "
                               f"{phys.parent}")
    bad = [str(a) for a in phys.parent.parents if not _ancestor_ok(os.lstat(a))]
    if bad:
        raise InputError(code, f"an ancestor of {what} is writable by others: {bad[0]}")
    if _acl_grants_write(phys) or _acl_grants_write(phys.parent):
        raise InputError(code, f"an ACL grants write access to {what} or its directory: {phys}")
    roots = [os.stat(r) for r in untrusted if os.path.exists(r)]
    for ancestor in (phys.parent, *phys.parent.parents):
        here = os.stat(ancestor)
        if any(os.path.samestat(here, r) for r in roots):  # filesystem identity, not spelling
            raise InputError(code, f"{what} lives inside a worktree or git dir ({ancestor})")
    return phys, data


def strict_json(data: bytes, what: str) -> Any:
    try:
        return VE.parse_json_strict(data.decode("utf-8"))
    except (UnicodeDecodeError, VE.StrictJsonError) as exc:
        raise InputError(f"{what}_malformed", f"not strict JSON: {exc}") from None


# --- review receipt (H1 format; v2 adds the trusted reviewer identity) ----------------------------

def parse_review_receipt(data: bytes) -> Dict[str, str]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise InputError("review_receipt_malformed", f"not UTF-8: {exc}") from exc
    values: Dict[str, str] = {}
    for line in text.split("\n"):
        if line == "" or line.startswith("#"):
            continue
        if "=" not in line or "\r" in line:
            raise InputError("review_receipt_malformed", "every line must be KEY=VALUE")
        key, value = line.split("=", 1)
        if key not in RECEIPT_V2_KEYS:
            raise InputError("review_receipt_malformed", f"unknown key {key!r}")
        if key in values:
            raise InputError("review_receipt_malformed", f"duplicate key {key}")
        if PLACEHOLDER_RE.fullmatch(value):
            raise InputError("review_receipt_malformed", f"unfilled <placeholder> in {key}")
        values[key] = value
    version = values.get("RECEIPT_VERSION")
    expected = {"1": RECEIPT_V1_KEYS, "2": RECEIPT_V2_KEYS}.get(version or "")
    if expected is None or set(values) != set(expected):
        raise InputError("review_receipt_malformed",
                         f"RECEIPT_VERSION {version!r} with keys {sorted(values)}")
    checks = [("BASE_SHA", OID_RE), ("HEAD_SHA", OID_RE), ("TREE_SHA", OID_RE),
              ("DIFF_SHA256", SHA256_RE), ("REVIEW_SHA256", SHA256_RE),
              ("GATE_SCRIPT_SHA256", SHA256_RE), ("OPERATOR", PRINTABLE_RE),
              ("TASK_ID", re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}"))]
    if version == "2":
        checks += [("REVIEWER", IDENT_RE), ("POLICY_VERSION", VE.POLICY_RE),
                   ("REVIEW_CONTEXT", IDENT_RE)]
    for key, pattern in checks:
        if not pattern.fullmatch(values[key]):
            raise InputError("review_receipt_malformed", f"{key} is malformed")
    if not values["REVIEW_ARTIFACT"]:
        raise InputError("review_receipt_malformed", "REVIEW_ARTIFACT is empty")
    return values


# --- evaluation sections ------------------------------------------------------------------------

def _resolve_refs(inp: Inputs, ev: Evaluation) -> Optional[Tuple[str, str, str]]:
    head = HG.resolve_commit(inp.repo, inp.head_ref)
    base = HG.resolve_commit(inp.repo, inp.base_ref)
    if head is None:
        ev.refuse("refs", "head_unresolved", f"--head-ref {inp.head_ref!r} does not name a commit")
    if base is None:
        ev.refuse("refs", "base_unresolved", f"--base-ref {inp.base_ref!r} does not name a commit")
    if head is None or base is None:
        return None
    tree = HG.tree_of(inp.repo, head)
    ev.bound.update({"head": head, "base": base, "tree": tree})
    if inp.expect_head and inp.expect_head != head:
        ev.refuse("refs", "head_mismatch",
                  f"operator observed head {inp.expect_head}, git has {head}")
    if inp.expect_base and inp.expect_base != base:
        ev.refuse("refs", "base_mismatch",
                  f"operator observed base {inp.expect_base}, git has {base}")
    if head == base:
        ev.refuse("refs", "empty_candidate", "head equals base: nothing to merge")
    pinned = MERGE_POLICIES.get(inp.policy_version)
    if pinned is None:
        ev.refuse("refs", "policy_unsupported", f"no merge policy for {inp.policy_version!r}")
    elif pinned["ci_policy_sha256"] != CI.policy_fingerprint():
        ev.refuse("refs", "policy_mismatch",
                  "merge_gate_ci.py constants do not match the CI policy "
                  f"pinned for {inp.policy_version!r}")
    try:
        registry = json.loads(Path(inp.registry).read_text(encoding="utf-8"))
        trusted_policy = registry["policy_version"]
    except (OSError, ValueError, KeyError, TypeError) as exc:
        ev.refuse("refs", "policy_unavailable", f"trusted registry unreadable: {exc}")
    else:
        if trusted_policy != inp.policy_version:
            ev.refuse("refs", "policy_mismatch", f"expected policy {inp.policy_version!r}, "
                                                 f"trusted registry is {trusted_policy!r}")
    ev.passed("refs")
    return head, base, tree


def _check_evidence(inp: Inputs, ev: Evaluation, head: str, base: str,
                    tree: str) -> Optional[Dict[str, Any]]:
    if inp.runs_root is None or inp.evidence_receipt is None:
        ev.refuse("evidence", "evidence_unavailable",
                  "no trusted-runner evidence (--runs-root/--evidence-receipt)")
        return None
    try:
        summary = RE.verify(repo=inp.repo, runs_root=inp.runs_root,
                            receipt_path=inp.evidence_receipt, expect_candidate=head,
                            registry_path=inp.registry, catalog_path=inp.catalog, now=inp.now)
        receipt = json.loads(Path(inp.evidence_receipt).read_text(encoding="utf-8"))
        evidence = json.loads(Path(summary["evidence"]).read_text(encoding="utf-8"))
    except RE.VerifyRefused as exc:
        ev.refuse("evidence", "evidence_refused", f"{exc.code}: {exc.detail}")
        return None
    except (OSError, ValueError, KeyError, HG.GitError) as exc:
        ev.refuse("evidence", "evidence_refused", f"{type(exc).__name__}: {exc}")
        return None
    if receipt.get("task_id") != inp.task:
        ev.refuse("evidence", "evidence_task_mismatch", f"task {receipt.get('task_id')!r}")
    if summary["base_sha"] != base:
        ev.refuse("evidence", "stale_base",
                  f"evidence base {summary['base_sha']} is not the current base {base}")
    if summary["tree_sha"] != tree:
        ev.refuse("evidence", "evidence_tree_mismatch", "evidence tree differs from the head tree")
    if evidence.get("policy_version") != inp.policy_version:
        ev.refuse("evidence", "evidence_policy_mismatch",
                  f"evidence policy {evidence.get('policy_version')!r}")
    ev.passed("evidence")
    return {"receipt": receipt, "evidence": evidence}


def _check_gate_history(inp: Inputs, ev: Evaluation, head: str,
                        evidence: Optional[Mapping[str, Any]]) -> None:
    """Any other gate run of the same task over the same candidate that did not pass refuses."""
    if evidence is None or inp.runs_root is None:
        ev.refuse("gate_history", "gate_history_unavailable",
                  "no verified evidence to locate the task's gate runs")
        return
    task_dir = Path(os.path.realpath(inp.runs_root)) / evidence["receipt"]["task_id"]
    for receipt_path in sorted(task_dir.glob("*/a*/g*/receipt.json")):
        try:
            if receipt_path.is_symlink():
                raise ValueError("symlinked receipt")
            data, errors = VE.read_json_file(receipt_path)
            if errors or not isinstance(data, dict):
                raise ValueError(str(errors[0]) if errors else "not an object")
        except (OSError, ValueError) as exc:
            ev.refuse("gate_history", "gate_history_unverifiable", f"{receipt_path}: {exc}")
            continue
        if data.get("candidate_sha") == head and data.get("verdict") != "pass":
            ev.refuse("gate_history", "conflicting_gate_result",
                      f"{receipt_path.relative_to(task_dir)} recorded {data.get('verdict')!r}")
    ev.passed("gate_history")


def _load_review_receipt(inp: Inputs, ev: Evaluation, untrusted: Sequence[Path], head: str,
                         base: str, tree: str) -> Optional[Dict[str, str]]:
    if inp.review_receipt is None:
        ev.refuse("review_receipt", "review_receipt_unavailable",
                  "no operator review receipt (--review-receipt)")
        return None
    try:
        _, data = trusted_input(inp.review_receipt, untrusted, "review_receipt", MAX_RECEIPT_BYTES)
        receipt = parse_review_receipt(data)
    except InputError as exc:
        ev.refuse("review_receipt", exc.code, exc.detail)
        return None
    if receipt["RECEIPT_VERSION"] == "1":
        ev.refuse("review_receipt", "reviewer_unavailable", "RECEIPT_VERSION=1 (H1 marker flow) "
                  "names no trusted reviewer: history only for the merge gate")
        return None
    expected = {"TASK_ID": (inp.task, "review_task_mismatch"),
                "BASE_SHA": (base, "review_base_mismatch"),
                "HEAD_SHA": (head, "review_head_mismatch"),
                "TREE_SHA": (tree, "review_tree_mismatch"),
                "POLICY_VERSION": (inp.policy_version, "review_policy_mismatch"),
                "REVIEW_CONTEXT": (REVIEW_CONTEXT_ATTESTED, "review_context_unattested")}
    for key, (want, code) in expected.items():
        if receipt[key] != want:
            ev.refuse("review_receipt", code, f"receipt {key}={receipt[key]!r}, expected {want!r}")
    ev.passed("review_receipt")
    return receipt


def _gate_env() -> Dict[str, str]:
    env = {k: os.environ[k] for k in ("PATH", "HOME", "TMPDIR") if k in os.environ}
    env["LC_ALL"] = "C"
    return env


def _run_scope(inp: Inputs, gate_bytes: bytes, base: str, head: str) -> Tuple[int, List[str]]:
    """Run the exact verified gate bytes from a private copy (cannot change after hashing)."""
    with tempfile.TemporaryDirectory(prefix="merge-gate-rg-") as tmp:
        copy = Path(tmp) / "review-gate.sh"
        copy.write_bytes(gate_bytes)
        argv = ["bash", str(copy), "scope", inp.task, "--repo", str(inp.repo), "--base", base,
                "--head", head]
        proc = subprocess.run(argv, capture_output=True, timeout=300, check=False, env=_gate_env(),
                              stdin=subprocess.DEVNULL)
    return proc.returncode, proc.stdout.decode("utf-8", "replace").splitlines()


def _check_review_scope(inp: Inputs, ev: Evaluation, untrusted: Sequence[Path],
                        receipt: Mapping[str, str], head: str, base: str, tree: str) -> None:
    """Recompute the H1 scope (diff sha256) with the operator's trusted review-gate copy."""
    if inp.review_gate is None or not inp.expect_gate_sha256:
        ev.refuse("review_scope", "review_scope_unavailable",
                  "no trusted review-gate copy / --expect-gate-sha256")
        return
    try:
        _, gate_bytes = trusted_input(inp.review_gate, untrusted, "review_gate", MAX_GATE_BYTES)
    except InputError as exc:
        ev.refuse("review_scope", exc.code, exc.detail)
        return
    gate_sha = sha256_bytes(gate_bytes)
    if gate_sha != inp.expect_gate_sha256 or gate_sha != receipt["GATE_SCRIPT_SHA256"]:
        ev.refuse("review_scope", "review_gate_mismatch",
                  f"gate copy {gate_sha}, operator expects {inp.expect_gate_sha256}, "
                  f"receipt binds {receipt['GATE_SCRIPT_SHA256']}")
        return
    try:
        rc, lines = _run_scope(inp, gate_bytes, base, head)
    except (OSError, subprocess.TimeoutExpired) as exc:
        ev.refuse("review_scope", "review_scope_failed", f"{type(exc).__name__}: {exc}")
        return
    if rc != 0 or not lines or not lines[-1].startswith("GATE_STATUS: ADVISORY"):
        ev.refuse("review_scope", "review_scope_failed", f"review-gate scope exited {rc}")
        return
    # Only the fixed header is read: it precedes every (writer-controlled) path line, so a file
    # name cannot inject a key. The gate's own sha256 was computed above, never parsed.
    header = [line.split("=", 1) for line in lines[: len(SCOPE_HEADER)]]
    if [kv[0] for kv in header] != list(SCOPE_HEADER):
        ev.refuse("review_scope", "review_scope_failed", "unexpected review-gate scope header")
        return
    scope = {k: v for k, v in header}
    observed = {"SCOPE_MODE": "final", "TASK_ID": inp.task, "BASE_SHA": base, "HEAD_SHA": head,
                "TREE_SHA": tree, "DIFF_SHA256": receipt["DIFF_SHA256"]}
    for key, want in observed.items():
        if scope.get(key) != want:
            code = "review_diff_mismatch" if key == "DIFF_SHA256" else "review_scope_failed"
            ev.refuse("review_scope", code,
                      f"recomputed {key}={scope.get(key)!r}, expected {want!r}")
    ev.passed("review_scope")


def _review_bytes(inp: Inputs, receipt: Mapping[str, str]) -> bytes:
    raw = Path(receipt["REVIEW_ARTIFACT"])
    path = raw if raw.is_absolute() else Path(inp.repo) / raw
    try:
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | getattr(os, "O_NONBLOCK", 0))
    except FileNotFoundError:
        raise InputError("review_missing", f"review artifact not found: {path}") from None
    except OSError as exc:
        raise InputError("review_untrusted",
                         f"review artifact unreadable or a symlink: {exc}") from None
    try:
        if not stat.S_ISREG(os.fstat(fd).st_mode):
            raise InputError("review_untrusted", f"review artifact must be a regular file: {path}")
        return _read_fd(fd, VE.MAX_FILE_BYTES, "review")
    finally:
        os.close(fd)


def _check_review(inp: Inputs, ev: Evaluation, receipt: Mapping[str, str], head: str,
                  base: str) -> Optional[Dict[str, Any]]:
    try:
        data = _review_bytes(inp, receipt)
    except InputError as exc:
        ev.refuse("review", exc.code, exc.detail)
        return None
    if sha256_bytes(data) != receipt["REVIEW_SHA256"]:
        ev.refuse("review", "review_hash_mismatch", "review artifact bytes differ from the receipt")
        return None
    if LEGACY_MARKER_RE.search(data):
        ev.refuse("review", "review_legacy_marker",
                  "REVIEW_VERDICT marker reviews are history only")
        return None
    try:
        parsed = strict_json(data, "review")
    except InputError as exc:
        ev.refuse("review", exc.code, exc.detail)
        return None
    if not isinstance(parsed, dict) or parsed.get("schema_version") != "review-v2":
        version = parsed.get("schema_version") if isinstance(parsed, dict) else None
        ev.refuse("review", "review_not_v2", f"schema_version {version!r}: only review-v2 counts")
        return None
    with tempfile.TemporaryDirectory(prefix="merge-gate-") as tmp:
        copy = Path(tmp) / "review.json"  # validate the exact bytes that were hashed (no TOCTOU)
        copy.write_bytes(data)
        expectations = VE.Expectations(candidate_sha=head, base_sha=base,
                                       policy_version=inp.policy_version, now=inp.now)
        result = VE.validate_file_detailed(copy, "review-v2", inp.schemas_dir, True, expectations)
    if not result.ok or not result.expectations_complete:
        detail = "; ".join(str(e) for e in result.errors[:3]) or "expectations not fully applied"
        ev.refuse("review", "review_invalid", detail)
        return None
    if parsed["verdict"] != "pass":
        ev.refuse("review", "review_not_pass", f"review verdict is {parsed['verdict']!r}")
    ev.passed("review")
    return parsed


def _identities_ok(data: Any) -> bool:
    keys = {"identities_version", "operators", "reviewers", "writers"}
    if not (isinstance(data, dict) and set(data) == keys
            and data["identities_version"] == IDENTITIES_VERSION
            and isinstance(data["operators"], list)
            and all(isinstance(o, str) and PRINTABLE_RE.fullmatch(o) for o in data["operators"])):
        return False
    for group in ("reviewers", "writers"):
        entries = data[group]
        if not isinstance(entries, dict) or not all(
                isinstance(v, dict) and set(v) == {"lineage"} and isinstance(v["lineage"], str)
                and IDENT_RE.fullmatch(v["lineage"]) for v in entries.values()):
            return False
    return True


def _load_identities(inp: Inputs, untrusted: Sequence[Path]) -> Dict[str, Any]:
    if inp.identities is None:
        raise InputError("reviewer_unavailable", "no trusted identity allowlist (--identities)")
    _, raw = trusted_input(inp.identities, untrusted, "identities", MAX_RECEIPT_BYTES)
    data = strict_json(raw, "identities")
    if not _identities_ok(data):
        raise InputError("identities_malformed", f"expected the {IDENTITIES_VERSION} layout")
    return data


def _check_reviewer(inp: Inputs, ev: Evaluation, untrusted: Sequence[Path],
                    receipt: Optional[Mapping[str, str]],
                    review: Optional[Mapping[str, Any]]) -> Optional[Dict[str, Any]]:
    try:
        identities = _load_identities(inp, untrusted)
    except InputError as exc:
        code = "reviewer_unavailable" if exc.code.endswith("_unavailable") else exc.code
        ev.refuse("reviewer", code, exc.detail)
        return None
    if receipt is None or review is None:
        ev.refuse("reviewer", "reviewer_unavailable", "no verified receipt + review to attribute")
        return identities
    if receipt["OPERATOR"] not in identities["operators"]:
        ev.refuse("reviewer", "operator_unauthorized", f"operator {receipt['OPERATOR']!r}")
    if receipt["REVIEWER"] not in identities["reviewers"]:
        ev.refuse("reviewer", "reviewer_unauthorized", f"reviewer {receipt['REVIEWER']!r}")
    if review.get("reviewer") != receipt["REVIEWER"]:
        ev.refuse("reviewer", "reviewer_claim_mismatch",
                  f"review claims {review.get('reviewer')!r}; "
                  f"the trusted receipt names {receipt['REVIEWER']!r}")
    ev.passed("reviewer")
    return identities


def _check_lineage(ev: Evaluation, identities: Optional[Mapping[str, Any]],
                   receipt: Optional[Mapping[str, str]], review: Optional[Mapping[str, Any]],
                   evidence: Optional[Mapping[str, Any]]) -> None:
    """Reviewer and writer lineages come from the trusted allowlist only, never from a name."""
    if identities is None or receipt is None or review is None or evidence is None:
        ev.refuse("lineage", "lineage_unavailable",
                  "reviewer or writer identity is not established")
        return
    reviewer = identities["reviewers"].get(receipt["REVIEWER"])
    environment = evidence["evidence"].get("environment") or {}
    adapter = environment.get("writer_adapter")  # recorded by the runner, not by the writer
    writer = identities["writers"].get(adapter) if isinstance(adapter, str) else None
    if reviewer is None or writer is None:
        ev.refuse("lineage", "lineage_unavailable",
                  f"no trusted lineage for reviewer / writer adapter {adapter!r}")
        return
    if reviewer["lineage"] == writer["lineage"]:
        ev.refuse("lineage", "lineage_not_independent", f"shared lineage {reviewer['lineage']!r}")
    claimed = (review.get("metadata") or {}).get("lineage")
    if claimed is not None and claimed != reviewer["lineage"]:
        ev.refuse("lineage", "lineage_claim_mismatch",
                  f"review claims lineage {claimed!r}, trusted is {reviewer['lineage']!r}")
    ev.passed("lineage")


def _ci_inputs(inp: Inputs, untrusted: Sequence[Path]) -> Tuple[List[Any], List[Any]]:
    if inp.ci_results is None:
        raise CI.CiRefusal("ci_results_unavailable", "no saved check runs (--ci-results)")
    if inp.workflow_runs is None:
        raise CI.CiRefusal("workflow_runs_unavailable", "no saved workflow runs (--workflow-runs)")
    loaded = []
    for path, what, key in ((inp.ci_results, "ci_results", "check_runs"),
                            (inp.workflow_runs, "workflow_runs", "workflow_runs")):
        try:
            _, raw = trusted_input(path, untrusted, what, MAX_JSON_BYTES)
            loaded.append(CI.pages(strict_json(raw, what), key, what))
        except InputError as exc:
            raise CI.CiRefusal(exc.code, exc.detail) from None
    return loaded[0], loaded[1]


def _check_ci(inp: Inputs, ev: Evaluation, untrusted: Sequence[Path], head: str,
              base: str) -> None:
    protected = CI.protected_changes(HG, inp.repo, base, head)
    if protected:
        ev.refuse("ci", "ci_policy_paths_changed", "the PR changes CI definitions or gate scripts, "
                  f"so its own CI results are not policy input: {', '.join(protected)[:300]}")
    if CI.workflow_blob(HG, inp.repo, head) != CI.workflow_blob(HG, inp.repo, base) \
            or not CI.workflow_blob(HG, inp.repo, base):
        ev.refuse("ci", "ci_workflow_changed",
                  f"{CI.CI_WORKFLOW_PATH} differs from the base or is absent")
    try:
        check_runs, workflow_runs = _ci_inputs(inp, untrusted)
        results, refusals = CI.results_by_context(check_runs, workflow_runs, head)
        gate = _load(SECURITY_GATE_SCRIPT, "check_security_gate")
    except CI.CiRefusal as exc:
        ev.refuse("ci", exc.code, exc.detail)
        return
    except (OSError, ImportError, SyntaxError) as exc:
        ev.refuse("ci", "ci_policy_unavailable", f"security-gate policy unavailable: {exc}")
        return
    for code, detail in refusals:
        ev.refuse("ci", code, detail)
    outcome, reasons = gate.verdict(results, list(REQUIRED_CI_CONTEXTS), set())
    if outcome != gate.PASS:
        ev.refuse("ci", "ci_blocked", f"{outcome}: {'; '.join(reasons)}")
    ev.passed("ci")


def _check_integrated(inp: Inputs, ev: Evaluation, tree: str) -> Optional[str]:
    """Merge queue / synthetic merge: the integrated tree needs its own evidence unless it IS the
    head tree."""
    if inp.integrated_ref is None:
        ev.passed("integrated")  # base is current (evidence): a merge onto it reproduces the tree
        return None
    integrated = HG.resolve_commit(inp.repo, inp.integrated_ref)
    if integrated is None:
        ev.refuse("integrated", "integrated_unresolved", f"{inp.integrated_ref!r} is not a commit")
        return None
    ev.bound["integrated"] = integrated
    if HG.tree_of(inp.repo, integrated) != tree:
        ev.refuse("integrated", "integrated_tree_unverified",
                  "the integrated (merge-queue/synthetic) "
                  "tree differs from the verified head tree; it needs its own evidence")
    ev.passed("integrated")
    return integrated


def _recheck(inp: Inputs, ev: Evaluation, head: str, base: str, integrated: Optional[str]) -> None:
    pairs = [(inp.head_ref, head, "head_changed_during_evaluation"),
             (inp.base_ref, base, "base_changed_during_evaluation")]
    if inp.integrated_ref is not None and integrated is not None:
        pairs.append((inp.integrated_ref, integrated, "integrated_changed_during_evaluation"))
    for ref, sha, code in pairs:
        if HG.resolve_commit(inp.repo, ref) != sha:
            ev.refuse("head_recheck", code, f"{ref} no longer points at {sha}")
    ev.passed("head_recheck")


def evaluate(inp: Inputs, before_recheck: Optional[Callable[[], None]] = None) -> Dict[str, Any]:
    """Read-only evaluation. `before_recheck` is a test hook run just before the final re-check."""
    ev = Evaluation()
    ev.bound.update({"task": inp.task, "policy": inp.policy_version,
                     "merge_policy": (MERGE_POLICIES.get(inp.policy_version) or {}).get("id")})
    refs = _resolve_refs(inp, ev)
    if refs is None:
        return ev.document()
    head, base, tree = refs
    untrusted = HG.untrusted_roots(inp.repo)
    evidence = _check_evidence(inp, ev, head, base, tree)
    _check_gate_history(inp, ev, head, evidence)
    receipt = _load_review_receipt(inp, ev, untrusted, head, base, tree)
    review = None
    if receipt is not None:
        _check_review_scope(inp, ev, untrusted, receipt, head, base, tree)
        review = _check_review(inp, ev, receipt, head, base)
    else:
        ev.refuse("review_scope", "review_scope_unavailable", "no verified review receipt")
        ev.refuse("review", "review_unavailable", "no verified review receipt")
    identities = _check_reviewer(inp, ev, untrusted, receipt, review)
    _check_lineage(ev, identities, receipt, review, evidence)
    _check_ci(inp, ev, untrusted, head, base)
    integrated = _check_integrated(inp, ev, tree)
    if before_recheck is not None:
        before_recheck()
    _recheck(inp, ev, head, base, integrated)
    return ev.document()


# --- CLI ----------------------------------------------------------------------------------------

def _parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Read-only merge-policy evaluator (task H5); "
                                            "never merges or approves.")
    p.add_argument("--repo", required=True, type=Path, help="candidate repository (untrusted)")
    p.add_argument("--task", required=True, help="expected task id (operator)")
    p.add_argument("--head-ref", required=True, help="ref naming the PR head NOW")
    p.add_argument("--base-ref", required=True, help="ref naming the merge target NOW")
    p.add_argument("--expect-policy-version", required=True)
    p.add_argument("--expect-head", help="head SHA the operator observed (cross-check)")
    p.add_argument("--expect-base", help="base SHA the operator observed (cross-check)")
    p.add_argument("--runs-root", type=Path, help="trusted runner runs root (H4)")
    p.add_argument("--evidence-receipt", type=Path, help="H4 .../a<N>/g<M>/receipt.json")
    p.add_argument("--review-receipt", type=Path, help="operator receipt, RECEIPT_VERSION=2")
    p.add_argument("--review-gate", type=Path, help="trusted review-gate.sh copy")
    p.add_argument("--expect-gate-sha256", help="sha256 of that review-gate copy (operator)")
    p.add_argument("--identities", type=Path, help="operator identity / lineage allowlist JSON")
    p.add_argument("--ci-results", type=Path, help="saved gh api .../commits/<head>/check-runs")
    p.add_argument("--workflow-runs", type=Path, help="saved gh api .../actions/runs?head_sha=")
    p.add_argument("--integrated-ref", help="merge-queue / synthetic merge commit")
    p.add_argument("--registry", type=Path, default=RE.DEFAULT_REGISTRY)
    p.add_argument("--catalog", type=Path, default=RE.DEFAULT_CATALOG)
    return p


def _usage_problem(args: argparse.Namespace) -> Optional[str]:
    if not TASK_RE.fullmatch(args.task):
        return f"--task must match {TASK_RE.pattern}"
    if not VE.POLICY_RE.fullmatch(args.expect_policy_version):
        return "--expect-policy-version is malformed"
    for flag, value, pattern in (("--expect-head", args.expect_head, OID_RE),
                                 ("--expect-base", args.expect_base, OID_RE),
                                 ("--expect-gate-sha256", args.expect_gate_sha256, SHA256_RE)):
        if value is not None and not pattern.fullmatch(value):
            return f"{flag} is malformed"
    if not args.repo.is_dir() or HG.toplevel(args.repo) is None:
        return f"--repo is not a git repository: {args.repo}"
    return None


def main(argv: Optional[Sequence[str]] = None) -> int:
    try:
        args = _parser().parse_args(argv)
    except SystemExit as exc:
        return 2 if exc.code else 0
    problem = _usage_problem(args)
    if problem:
        print(f"MERGE_GATE: USAGE {problem}", file=sys.stderr)
        return 2
    fields = {k: getattr(args, k) for k in ("task", "head_ref", "base_ref", "expect_head",
                                             "expect_base", "runs_root", "evidence_receipt",
                                             "review_receipt", "review_gate", "expect_gate_sha256",
                                             "identities", "ci_results", "workflow_runs",
                                             "integrated_ref", "registry", "catalog")}
    inputs = Inputs(repo=args.repo.resolve(), policy_version=args.expect_policy_version, **fields)
    try:
        document = evaluate(inputs)
    except Exception as exc:  # noqa: BLE001 - any unexpected failure is a refusal, never eligible
        document = Evaluation().document()
        document["reasons"] = [{"section": "evaluator", "code": "evaluator_error",
                                "detail": f"{type(exc).__name__}: {exc}"}]
    print(json.dumps(document, indent=2, sort_keys=True))
    codes = ",".join(sorted({r["code"] for r in document["reasons"]}))
    print(f"MERGE_GATE: {document['decision'].upper()} head={document['bound'].get('head')}"
          + (f" reasons={codes}" if codes else ""), file=sys.stderr)
    return 0 if document["decision"] == "eligible" else 1


if __name__ == "__main__":
    sys.exit(main())
