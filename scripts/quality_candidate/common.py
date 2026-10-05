"""Shared constants, errors and small helpers for the candidate quality runner."""

from __future__ import annotations

import hashlib
import json
import math
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Mapping, Optional, Sequence

REPORT_SCHEMA = "engram.quality-candidate-report.v1"
FLOORS_SCHEMA = "engram.quality-candidate-floors.v1"
EVAL_SCHEMA = "engram.retrieval-candidate-eval.v1"
EVAL_MARKER = "ENGRAM_RETRIEVAL_EVAL_JSON="

METRICS = ("recall@10", "mrr", "ndcg@10")
# Evaluation modes emitted by tests/retrieval_quality.rs (candidate_retrieval_metrics).
MODES = ("lexical_fts5", "hybrid_tfidf")
FLOORS_PATH = "docs/quality/candidate-floors.json"
BUDGETS_PATH = "docs/quality/budgets.json"
BUDGETS_SCRIPT = "scripts/check-quality-budgets.py"
HISTORICAL_CRITERION_PATH = "benches/results/benchmark_results.txt"
# Existing PR performance ceiling (ci.yml alert-threshold 115%). Never relaxed here.
CRITERION_CEILING = 1.15
# A Criterion capture older than this is stale for a candidate run.
MAX_CRITERION_AGE_SECONDS = 24 * 3600
FUTURE_SKEW_SECONDS = 300
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
FEATURE_RE = re.compile(r"^[A-Za-z0-9_-]+$")
SUPERVISOR_ENV = "ENGRAM_QUALITY_SUPERVISOR"
DEFAULT_SUPERVISOR = "local"
# Values that mean "no supervisor-owned id" (rejected by --require-supervisor).
LOCAL_SUPERVISORS = ("", "local")
# Environment variables forwarded to approved child commands. Nothing else.
ENV_ALLOWLIST = (
    "PATH",
    "HOME",
    "USER",
    "LANG",
    "LC_ALL",
    "TMPDIR",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "RUSTUP_TOOLCHAIN",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_JOBS",
    "RUSTFLAGS",
    "RUSTC_WRAPPER",
    "SDKROOT",
    "CC",
    "CXX",
)


class CandidateError(ValueError):
    """The candidate evidence is incomplete, stale, mismatched or regressed."""


@dataclass(frozen=True)
class CommandResult:
    returncode: int
    stdout: str
    stderr: str


# An executor runs one fixed argv list (never a shell string).
Executor = Callable[[Sequence[str], Path, Mapping[str, str], int], CommandResult]


def subprocess_executor(
    argv: Sequence[str], cwd: Path, env: Mapping[str, str], timeout: int
) -> CommandResult:
    try:
        completed = subprocess.run(
            list(argv),
            cwd=str(cwd),
            env=dict(env),
            capture_output=True,
            text=True,
            timeout=timeout,
            shell=False,
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise CandidateError(f"command timed out after {timeout}s: {argv[0]} {argv[1] if len(argv) > 1 else ''}") from exc
    except OSError as exc:
        raise CandidateError(f"cannot execute {argv[0]}: {exc}") from exc
    return CommandResult(completed.returncode, completed.stdout, completed.stderr)


def child_env(source: Mapping[str, str], extra: Optional[Mapping[str, str]] = None) -> dict:
    env = {key: source[key] for key in ENV_ALLOWLIST if key in source}
    env.update(extra or {})
    return env


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def _reject_constant(token: str) -> Any:
    raise CandidateError(f"non-finite JSON number {token} is not valid evidence")


def loads_strict(text: str, label: str) -> Any:
    """Parse JSON refusing NaN/Infinity literals."""
    try:
        return json.loads(text, parse_constant=_reject_constant)
    except json.JSONDecodeError as exc:
        raise CandidateError(f"{label}: invalid JSON: {exc}") from exc


def load_json_file(path: Path, label: str) -> dict:
    try:
        value = loads_strict(path.read_text(encoding="utf-8"), label)
    except OSError as exc:
        raise CandidateError(f"{label}: cannot read {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise CandidateError(f"{label}: root must be an object")
    return value


def utc_now() -> datetime:
    return datetime.now(timezone.utc).replace(microsecond=0)


def format_utc(moment: datetime) -> str:
    return moment.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_utc(raw: Any, label: str) -> datetime:
    if not isinstance(raw, str):
        raise CandidateError(f"{label}: timestamp must be a string")
    try:
        return datetime.strptime(raw, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    except ValueError as exc:
        raise CandidateError(f"{label}: timestamp must be YYYY-MM-DDTHH:MM:SSZ") from exc


def require_metric_map(raw: Any, label: str, keys: Sequence[str] = METRICS) -> dict:
    """Validate an exact-key map of finite numbers within [0, 1]."""
    if not isinstance(raw, dict):
        raise CandidateError(f"{label}: must be an object")
    missing = [key for key in keys if key not in raw]
    if missing:
        raise CandidateError(f"{label}: missing metric(s) {', '.join(missing)}")
    extra = sorted(set(raw) - set(keys))
    if extra:
        raise CandidateError(f"{label}: unexpected key(s) {', '.join(extra)}")
    out = {}
    for key in keys:
        value = raw[key]
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise CandidateError(f"{label}.{key}: must be a number, got {value!r}")
        number = float(value)
        if not math.isfinite(number) or not 0.0 <= number <= 1.0:
            raise CandidateError(f"{label}.{key}: must be finite within [0, 1], got {value!r}")
        out[key] = number
    return out
