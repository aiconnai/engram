"""Candidate quality runner: fixed argv, verified checkout, reproducible report.

Contract (task Q7): ``--candidate-sha --corpus --features --criterion --output``.
No free shell command is accepted; the only child process is the approved
``cargo test`` argv built here, plus read-only toolchain version probes.
Any incomplete, stale, mismatched, non-finite or regressed evidence exits
non-zero and leaves a ``status: fail`` report (never a stale passing one).

Trust model: in CI, run this script (and capture) from a **trusted ref** (the
base branch checked out in a separate directory) and point ``--candidate-dir``
at the candidate checkout, so candidate code never executes in the verifier.
``--require-supervisor`` makes the supervisor binding non-vacuous and
``--floors-anchor`` supplies the trusted review anchor. Echo-back checks of the
test process (candidate SHA, corpus hash, counts) are plumbing consistency
checks, not proof against a malicious candidate.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

from . import criterion as crit
from . import floors as floors_mod
from .common import (
    BUDGETS_PATH,
    BUDGETS_SCRIPT,
    CRITERION_CEILING,
    DEFAULT_SUPERVISOR,
    LOCAL_SUPERVISORS,
    EVAL_MARKER,
    EVAL_SCHEMA,
    FEATURE_RE,
    FLOORS_PATH,
    HISTORICAL_CRITERION_PATH,
    METRICS,
    MODES,
    REPORT_SCHEMA,
    SUPERVISOR_ENV,
    CandidateError,
    CommandResult,
    Executor,
    child_env,
    format_utc,
    load_json_file,
    loads_strict,
    require_metric_map,
    sha256_bytes,
    sha256_file,
    subprocess_executor,
    utc_now,
)
from .gitstate import is_tracked, relative_to_repo, verify_checkout

TEST_TIMEOUT_SECONDS = 3600
PROBE_TIMEOUT_SECONDS = 60
TEST_NAME = "candidate_retrieval_metrics"
BUILD_ENV_KEYS = ("RUSTFLAGS", "RUSTC_WRAPPER", "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "RUSTUP_TOOLCHAIN")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def normalize_features(raw: str, repo: Path) -> List[str]:
    """Validate against Cargo.toml [features]; return a sorted unique list."""
    names = [item.strip() for item in raw.split(",") if item.strip()]
    for name in names:
        if not FEATURE_RE.fullmatch(name):
            raise CandidateError(f"--features: {name!r} is not a plain feature name")
    try:
        manifest = tomllib.loads((repo / "Cargo.toml").read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise CandidateError(f"cannot read Cargo.toml features: {exc}") from exc
    declared = set(manifest.get("features", {}))
    unknown = sorted(set(names) - declared)
    if unknown:
        raise CandidateError(f"--features: unknown feature(s) {', '.join(unknown)}")
    return sorted(set(names))


def approved_test_argv(features: Sequence[str]) -> List[str]:
    """The single approved retrieval command. Built from validated pieces only."""
    argv = ["cargo", "test", "--locked", "--test", "retrieval_quality", "--no-default-features"]
    if features:
        argv += ["--features", ",".join(features)]
    argv += ["--", "--exact", TEST_NAME, "--nocapture", "--test-threads=1"]
    return argv


def probe_toolchain(execute: Executor, repo: Path, env: Mapping[str, str]) -> Dict[str, str]:
    out = {}
    for tool in ("rustc", "cargo"):
        result = execute([tool, "-V"], repo, env, PROBE_TIMEOUT_SECONDS)
        if result.returncode != 0 or not result.stdout.strip():
            raise CandidateError(f"cannot determine toolchain: {tool} -V failed")
        out[tool] = result.stdout.strip().splitlines()[0]
    return out


def extract_eval_line(result: CommandResult) -> Dict[str, Any]:
    if result.returncode != 0:
        tail = (result.stderr or result.stdout).strip().splitlines()[-5:]
        raise CandidateError(
            f"approved retrieval test exited {result.returncode}: " + " | ".join(tail)
        )
    if "test result: ok. 1 passed; 0 failed" not in result.stdout:
        raise CandidateError(
            f"approved retrieval test did not run exactly 1 test (filter {TEST_NAME} matched "
            "nothing or the candidate lacks it)"
        )
    hits = [part for part in result.stdout.split(EVAL_MARKER)[1:]]
    if len(hits) != 1:
        raise CandidateError(f"expected exactly one {EVAL_MARKER} line, found {len(hits)}")
    payload = hits[0].splitlines()[0]
    return loads_strict(payload, "retrieval eval output")


def validate_eval(
    eval_json: Mapping[str, Any],
    *,
    candidate_sha: str,
    features: str,
    corpus_sha256: str,
    entry: Mapping[str, Any],
) -> Dict[str, Any]:
    """Cross-check what the test process says it evaluated against what we asked."""
    if eval_json.get("schema") != EVAL_SCHEMA:
        raise CandidateError(f"retrieval eval: schema must be {EVAL_SCHEMA}")
    expectations = {
        "candidate_sha": candidate_sha,
        "features": features,
        "corpus_sha256": corpus_sha256,
        "deterministic_seed": entry["deterministic_seed"],
        "memory_count": entry["memory_count"],
        "query_count": entry["query_count"],
    }
    for key, expected in expectations.items():
        if eval_json.get(key) != expected:
            raise CandidateError(
                f"retrieval eval {key} mismatch: process reported {eval_json.get(key)!r}, "
                f"expected {expected!r}"
            )
    provider = eval_json.get("provider")
    if not isinstance(provider, dict) or provider.get("network") is not False:
        raise CandidateError("retrieval eval: provider must be a recorded offline provider")
    modes = eval_json.get("modes")
    if not isinstance(modes, dict) or set(modes) != set(MODES):
        raise CandidateError(f"retrieval eval: modes must be exactly {', '.join(MODES)}")
    metrics = {}
    for mode in MODES:
        mode_json = modes[mode]
        if not isinstance(mode_json, dict):
            raise CandidateError(f"retrieval eval: mode {mode} must be an object")
        metrics[mode] = require_metric_map(mode_json.get("metrics"), f"retrieval eval {mode}.metrics")
    return metrics


def check_floors(metrics: Mapping[str, Mapping[str, float]], entry: Mapping[str, Any]) -> List[Dict[str, Any]]:
    rows = []
    failures = []
    for mode in MODES:
        for metric in METRICS:
            floor = float(entry["floors"][mode][metric])
            observed = metrics[mode][metric]
            rows.append({"mode": mode, "metric": metric, "floor": floor, "observed": observed})
            if observed < floor:
                failures.append(f"{mode}.{metric} observed {observed} below floor {floor}")
    if failures:
        raise CandidateError("retrieval floor regression: " + "; ".join(failures))
    return rows


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--features", required=True)
    parser.add_argument("--criterion", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--candidate-dir",
        type=Path,
        default=None,
        help="candidate checkout (default: the checkout containing this script)",
    )
    parser.add_argument(
        "--floors-anchor",
        default=None,
        help="trusted review anchor: a supervisor-chosen strict ancestor of the candidate",
    )
    parser.add_argument(
        "--require-supervisor",
        action="store_true",
        help=f"fail unless {SUPERVISOR_ENV} holds a supervisor-owned id (not empty/'local')",
    )
    return parser


def write_report(path: Path, report: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, path)


def evaluate(
    args: argparse.Namespace,
    repo: Path,
    trusted: Path,
    execute: Executor,
    environ: Mapping[str, str],
    report: Dict[str, Any],
) -> None:
    """Fill ``report`` step by step; raise CandidateError on the first failure.

    ``repo`` is the candidate checkout; ``trusted`` is where this runner (and the
    Q1a budgets module it imports) lives.
    """
    supervisor = environ.get(SUPERVISOR_ENV, DEFAULT_SUPERVISOR)
    report["supervisor"] = supervisor
    report["supervisor_required"] = bool(args.require_supervisor)
    if args.require_supervisor and supervisor.strip().lower() in LOCAL_SUPERVISORS:
        raise CandidateError(
            f"--require-supervisor: {SUPERVISOR_ENV} must be a supervisor-owned id, got {supervisor!r}"
        )
    candidate = verify_checkout(repo, args.candidate_sha)
    report["candidate"] = candidate

    features_list = normalize_features(args.features, repo)
    features = ",".join(features_list)
    report["features"] = features_list

    corpus_path = args.corpus if args.corpus.is_absolute() else Path.cwd() / args.corpus
    if not corpus_path.is_file():
        raise CandidateError(f"--corpus: {args.corpus} is not a file")
    corpus_rel = relative_to_repo(repo, corpus_path, "--corpus")
    if not is_tracked(repo, corpus_rel):
        raise CandidateError(f"--corpus: {corpus_rel} is not tracked in the candidate commit")
    corpus_sha = sha256_file(corpus_path)
    report["corpus"] = {"path": corpus_rel, "sha256": corpus_sha}

    floors_text = (repo / FLOORS_PATH).read_text(encoding="utf-8") if (repo / FLOORS_PATH).is_file() else ""
    if not floors_text:
        raise CandidateError(f"{FLOORS_PATH} is missing")
    floors_doc = floors_mod.parse_floors(floors_text)
    entry = floors_mod.select_entry(floors_doc, corpus_sha)
    anchor = floors_mod.assess_anchor(
        repo, candidate["sha"], entry, FLOORS_PATH, args.floors_anchor
    )
    budgets = load_json_file(repo / BUDGETS_PATH, "budgets")
    baseline_path = budgets.get("retrieval", {}).get("baseline_path")
    reconciled = False
    if isinstance(baseline_path, str) and (repo / baseline_path).is_file():
        baseline = load_json_file(repo / baseline_path, "retrieval baseline")
        reconciled = floors_mod.reconcile_with_budgets(entry, budgets, baseline)
    report["floors"] = {
        "path": FLOORS_PATH,
        "entry_digest": entry["entry_digest"],
        "anchored": anchor["anchored"],
        "accepted": anchor["accepted"],
        "anchor_revision": anchor["anchor_revision"],
        "anchor_reason": anchor["reason"],
        "reconciled_with_budgets": reconciled,
        "review": entry["review"],
        "values": entry["floors"],
    }

    env_base = child_env(environ)
    toolchain = probe_toolchain(execute, repo, env_base)
    report["toolchain"] = toolchain
    argv = approved_test_argv(features_list)
    report["argv"] = argv
    lock = repo / "Cargo.lock"
    if not lock.is_file():
        raise CandidateError("Cargo.lock is missing; the approved argv uses --locked")
    report["build_env"] = {
        "cargo_lock_sha256": sha256_file(lock),
        **{key: environ.get(key) for key in BUILD_ENV_KEYS},
    }
    run_env = child_env(
        environ,
        {
            "ENGRAM_RETRIEVAL_CORPUS": str(corpus_path.resolve()),
            "ENGRAM_CANDIDATE_SHA": candidate["sha"],
            "ENGRAM_CANDIDATE_FEATURES": features,
        },
    )
    result = execute(argv, repo, run_env, TEST_TIMEOUT_SECONDS)
    # The build/test run must not have changed the candidate it is describing.
    verify_checkout(repo, args.candidate_sha)
    eval_json = extract_eval_line(result)
    metrics = validate_eval(
        eval_json,
        candidate_sha=candidate["sha"],
        features=features,
        corpus_sha256=corpus_sha,
        entry=entry,
    )
    report["corpus"].update(
        {
            "name": eval_json.get("corpus_name"),
            "version": eval_json.get("corpus_version"),
            "deterministic_seed": eval_json["deterministic_seed"],
            "memory_count": eval_json["memory_count"],
            "query_count": eval_json["query_count"],
            "label_provenance": eval_json.get("label_provenance"),
        }
    )
    report["provider"] = eval_json["provider"]
    report["modes"] = {
        mode: {
            "metrics": metrics[mode],
            "categories": eval_json["modes"][mode].get("categories"),
            "discouraged": eval_json["modes"][mode].get("discouraged"),
        }
        for mode in MODES
    }
    report["retrieval_floor_checks"] = check_floors(metrics, entry)

    criterion_path = args.criterion if args.criterion.is_absolute() else Path.cwd() / args.criterion
    if not criterion_path.is_file():
        raise CandidateError(f"--criterion: {args.criterion} is not a file")
    criterion_rel = relative_to_repo(repo, criterion_path, "--criterion") if str(
        criterion_path.resolve()
    ).startswith(str(repo.resolve())) else None
    if criterion_rel is not None and (
        criterion_rel == HISTORICAL_CRITERION_PATH or is_tracked(repo, criterion_rel)
    ):
        raise CandidateError(
            f"--criterion: {criterion_rel} is a tracked historical file; candidate Criterion "
            "evidence must be freshly captured for this candidate"
        )
    criterion_text = criterion_path.read_text(encoding="utf-8")
    meta, body = crit.verify(
        criterion_text,
        {
            "candidate_sha": candidate["sha"],
            "candidate_tree": candidate["tree"],
            "features": features,
            "toolchain": toolchain["rustc"],
            "supervisor": supervisor,
        },
    )
    hot_paths = crit.compare_hot_paths(trusted, BUDGETS_SCRIPT, body, budgets)
    historical = repo / HISTORICAL_CRITERION_PATH
    report["criterion"] = {
        "file_sha256": sha256_bytes(criterion_text.encode("utf-8")),
        "body_sha256": meta["body_sha256"],
        "marker": {k: meta[k] for k in crit.REQUIRED_KEYS if k != "body_sha256"},
        "ceiling": CRITERION_CEILING,
        "hot_paths": hot_paths,
        "historical_context": {
            "path": HISTORICAL_CRITERION_PATH,
            "sha256": sha256_file(historical) if historical.is_file() else None,
            "role": "comparative context only; never accepted as candidate evidence",
        },
    }


def ensure_output_outside(output: Path, *roots: Path) -> None:
    """Never write (or delete) inside a checkout: tracked files must be safe."""
    resolved = output.resolve()
    for root in roots:
        try:
            resolved.relative_to(root.resolve())
        except ValueError:
            continue
        raise CandidateError(f"--output {output} is inside the checkout {root}; choose a path outside it")


def run(
    argv: Sequence[str],
    *,
    repo: Optional[Path] = None,
    execute: Executor = subprocess_executor,
    environ: Optional[Mapping[str, str]] = None,
) -> int:
    """``repo`` is the trusted root holding this runner (default: derived from the file)."""
    args = build_parser().parse_args(list(argv))
    trusted = repo or repo_root()
    candidate_dir = args.candidate_dir or trusted
    environ = os.environ if environ is None else environ
    output = args.output if args.output.is_absolute() else Path.cwd() / args.output
    try:
        ensure_output_outside(output, trusted, candidate_dir)
    except CandidateError as exc:
        print(json.dumps({"status": "fail", "errors": [str(exc)]}, sort_keys=True), file=sys.stderr)
        return 1
    # A stale passing report must never survive a failed run.
    try:
        output.unlink()
    except FileNotFoundError:
        pass
    report: Dict[str, Any] = {
        "schema_version": REPORT_SCHEMA,
        "status": "fail",
        "generated_at": format_utc(utc_now()),
        "hardware": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "cpu_count": os.cpu_count(),
        },
        "errors": [],
    }
    exit_code = 1
    try:
        evaluate(args, candidate_dir, trusted, execute, environ, report)
        report["status"] = "pass"
        exit_code = 0
    except CandidateError as exc:
        report["errors"].append(str(exc))
    write_report(output, report)
    if exit_code:
        print(json.dumps({"status": "fail", "errors": report["errors"]}, sort_keys=True), file=sys.stderr)
    else:
        print(json.dumps({"status": "pass", "output": str(output)}, sort_keys=True))
    return exit_code
