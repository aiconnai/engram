"""Capture Criterion output bound to one candidate (fixed argv, no free shell).

Produces the marked file ``run-quality-candidate.py --criterion`` accepts. The
approved bench list and numeric Criterion parameters are the only knobs.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
from pathlib import Path
from typing import List, Mapping, Optional, Sequence

from . import criterion as crit
from .common import (
    DEFAULT_SUPERVISOR,
    LOCAL_SUPERVISORS,
    SUPERVISOR_ENV,
    CandidateError,
    Executor,
    child_env,
    subprocess_executor,
    utc_now,
    format_utc,
)
from .gitstate import verify_checkout
from .runner import ensure_output_outside, normalize_features, probe_toolchain, repo_root

# Hot paths in docs/quality/budgets.json live in entity_extraction. The others are
# available for evidence runs but are not required by the gate.
APPROVED_BENCHES = ("entity_extraction", "memory_ops", "mcp_dispatch", "search")
DEFAULT_BENCHES = ("entity_extraction",)
BENCH_TIMEOUT_SECONDS = 3 * 3600


def approved_bench_argv(
    bench: str, features: Sequence[str], sample_size: int, warm_up: float, measurement: float
) -> List[str]:
    argv = ["cargo", "bench", "--locked", "--no-default-features"]
    if features:
        argv += ["--features", ",".join(features)]
    argv += [
        "--bench",
        bench,
        "--",
        "--sample-size",
        str(sample_size),
        "--warm-up-time",
        f"{warm_up:g}",
        "--measurement-time",
        f"{measurement:g}",
        "--noplot",
    ]
    return argv


def cpu_model(execute: Executor, repo: Path, env: Mapping[str, str]) -> str:
    if platform.system() == "Darwin":
        result = execute(["sysctl", "-n", "machdep.cpu.brand_string"], repo, env, 30)
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip()
    cpuinfo = Path("/proc/cpuinfo")
    if cpuinfo.is_file():
        for line in cpuinfo.read_text(encoding="utf-8", errors="replace").splitlines():
            if line.lower().startswith("model name"):
                return line.partition(":")[2].strip()
    return platform.processor() or "unknown"


def run(
    argv: Sequence[str],
    *,
    repo: Optional[Path] = None,
    execute: Executor = subprocess_executor,
    environ: Optional[Mapping[str, str]] = None,
) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--features", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--candidate-dir", type=Path, default=None)
    parser.add_argument("--require-supervisor", action="store_true")
    parser.add_argument("--bench", action="append", choices=APPROVED_BENCHES)
    parser.add_argument("--sample-size", type=int, default=100)
    parser.add_argument("--warm-up-time", type=float, default=3.0)
    parser.add_argument("--measurement-time", type=float, default=5.0)
    args = parser.parse_args(list(argv))
    trusted = repo or repo_root()
    repo = args.candidate_dir or trusted
    environ = os.environ if environ is None else environ
    output = args.output if args.output.is_absolute() else Path.cwd() / args.output
    try:
        ensure_output_outside(output, trusted, repo)
    except CandidateError as exc:
        print(json.dumps({"status": "fail", "errors": [str(exc)]}), file=sys.stderr)
        return 1
    try:
        output.unlink()
    except FileNotFoundError:
        pass
    try:
        supervisor = environ.get(SUPERVISOR_ENV, DEFAULT_SUPERVISOR)
        if args.require_supervisor and supervisor.strip().lower() in LOCAL_SUPERVISORS:
            raise CandidateError(
                f"--require-supervisor: {SUPERVISOR_ENV} must be a supervisor-owned id, got {supervisor!r}"
            )
        if not 10 <= args.sample_size <= 1000:
            raise CandidateError("--sample-size must be within [10, 1000]")
        if not 0.1 <= args.warm_up_time <= 60 or not 0.5 <= args.measurement_time <= 600:
            raise CandidateError("--warm-up-time/--measurement-time outside approved bounds")
        candidate = verify_checkout(repo, args.candidate_sha)
        features = normalize_features(args.features, repo)
        env = child_env(environ)
        toolchain = probe_toolchain(execute, repo, env)
        benches = list(dict.fromkeys(args.bench or DEFAULT_BENCHES))
        argvs, body = [], []
        for bench in benches:
            bench_argv = approved_bench_argv(
                bench, features, args.sample_size, args.warm_up_time, args.measurement_time
            )
            result = execute(bench_argv, repo, env, BENCH_TIMEOUT_SECONDS)
            if result.returncode != 0:
                tail = (result.stderr or result.stdout).strip().splitlines()[-5:]
                raise CandidateError(f"bench {bench} exited {result.returncode}: " + " | ".join(tail))
            argvs.append(bench_argv)
            body.append(result.stdout)
        verify_checkout(repo, args.candidate_sha)  # benches must not have altered the candidate
        meta = {
            "candidate_sha": candidate["sha"],
            "candidate_tree": candidate["tree"],
            "features": ",".join(features),
            "toolchain": toolchain["rustc"],
            "supervisor": supervisor,
            "captured_at": format_utc(utc_now()),
            "bench_argv": argvs,
            "bench_params": {
                "benches": benches,
                "sample_size": args.sample_size,
                "warm_up_time_s": args.warm_up_time,
                "measurement_time_s": args.measurement_time,
            },
            "hardware": {
                "platform": platform.platform(),
                "machine": platform.machine(),
                "cpu_count": os.cpu_count(),
                "cpu_model": cpu_model(execute, repo, env),
            },
        }
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(crit.render(meta, "\n".join(body)), encoding="utf-8")
    except CandidateError as exc:
        print(json.dumps({"status": "fail", "errors": [str(exc)]}), file=sys.stderr)
        return 1
    print(json.dumps({"status": "captured", "output": str(output)}))
    return 0
