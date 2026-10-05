#!/usr/bin/env python3
"""Sandbox adapter and approved-check registry loader (task H3, ADR agent-harness-hardening-v1).

run_isolated(manifest_path, worktree_path, run_dir) executes the fixed argv vectors that a task
manifest selects BY CHECK ID inside one hardened, digest-pinned container and returns a RunOutcome.
The supervisor (this host-side process) is the only writer of run_dir; the container never sees it.

Boundary contract (fail-closed; there is NO host-execution fallback anywhere in this file):

  * Only the `docker` CLI is ever spawned on the host, and only with argv lists (never a shell).
    The check argv is handed to Docker as the container entrypoint + command; it is never executed
    on the host. Missing CLI, unreachable daemon or absent image -> status `unavailable`.
  * The engine endpoint is resolved ONCE (DOCKER_HOST, then DOCKER_CONTEXT, then the config's current context,
    via `docker context inspect`) and must be a local unix socket; remote engines (ssh://, tcp://, ...) are
    `unavailable`. Every later call runs with DOCKER_HOST=<that socket> and no DOCKER_CONTEXT, and the
    endpoint is recorded in the outcome.
  * Registry and argv policy live in the sibling module sandbox_registry.py.
  * Image pinned by digest (`name@sha256:<64 hex>`), `--pull never`: the adapter never fetches images.
  * `--network none`, `--cap-drop ALL`, `no-new-privileges`, non-root user, read-only root fs,
    tmpfs /tmp, memory / swap / pids / cpu limits, `--init`. No credential env, no HOME, no SSH agent,
    no Docker socket, no devices: the container env is a constant allowlist.
  * Mounts: the worktree at /work (writer-writable) and a supervisor-staged snapshot of the manifest,
    registry, catalog and tools at /tcb (read-only). run_dir and the host TCB are never mounted.
  * After `docker create` the adapter reads the configuration back with `docker inspect` and refuses
    to start if Docker did not actually apply a limit. `limits_enforced` reports only that read-back.
  * Timeouts `docker kill` + `docker rm -f` the container and verify that no container labelled with
    the run remains; a survivor is an `error`, never a pass.
  * The manifest must be a valid task-v2 document (validate-evidence.py) and every check ID it selects
    must exist in the registry AND the check catalog; unknown ID -> `refused`.

Statuses: passed | failed | timeout | refused | unavailable | error. Only `passed` is a pass.
Exit codes of the CLI: 0 passed, 1 failed/timeout/error, 2 refused, 3 unavailable.

Scope: fake writer only (ADR "not granted" list). This module does not grant a real coding agent
anything; credential brokerage, egress allowlists and image builds are out of scope.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import importlib.util
import json
import math
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple

HARNESS_DIR = Path(__file__).resolve().parent.parent


def _load_sibling(name: str) -> Any:
    """Load a module that sits next to this file (the hyphenated entry point is not importable)."""
    path = Path(__file__).resolve().parent / f"{name}.py"
    cached = sys.modules.get(name)
    if cached is not None and getattr(cached, "__file__", None) == str(path):
        return cached
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


_REG = _load_sibling("sandbox_registry")
Refusal, RegistryError = _REG.Refusal, _REG.RegistryError
SandboxConfig, CheckSpec, Registry = _REG.SandboxConfig, _REG.CheckSpec, _REG.Registry
parse_registry, load_registry, validate_argv, _validator = _REG.parse_registry, _REG.load_registry, _REG.validate_argv, _REG.validator
_is_int = _REG.is_int
DEFAULT_REGISTRY = HARNESS_DIR / "checks" / "registry.json"
DEFAULT_CATALOG = HARNESS_DIR / "schemas" / "check-catalog-v1.json"
SCHEMAS_DIR = HARNESS_DIR / "schemas"

TOOL_NAME_RE = re.compile(r"[A-Za-z0-9_.-]{1,64}")
UNSAFE_MOUNT_CHARS = set(',:"\n\r\0\\')

WORK_MOUNT = "/work"
TCB_MOUNT = "/tcb"
RUN_LABEL = "engram.sandbox.run"
# Constant container environment: nothing is inherited from the host.
CONTAINER_ENV: Dict[str, str] = {
    "HOME": "/tmp",
    "TMPDIR": "/tmp",
    "LANG": "C.UTF-8",
    "PYTHONDONTWRITEBYTECODE": "1",
    "ENGRAM_SANDBOX": "1",
}
# Host variables the docker CLI itself needs. They configure the CLI, never the container.
DOCKER_CLI_ENV = ("PATH", "HOME", "DOCKER_HOST", "DOCKER_CONTEXT", "DOCKER_CONFIG", "XDG_RUNTIME_DIR")

STATUS_PASSED, STATUS_FAILED, STATUS_TIMEOUT = "passed", "failed", "timeout"
STATUS_REFUSED, STATUS_UNAVAILABLE, STATUS_ERROR = "refused", "unavailable", "error"
EXIT_FOR_STATUS = {STATUS_PASSED: 0, STATUS_FAILED: 1, STATUS_TIMEOUT: 1, STATUS_ERROR: 1, STATUS_REFUSED: 2, STATUS_UNAVAILABLE: 3}

class Unavailable(Exception):
    """Required isolation is missing. Maps to status `unavailable`; never a fallback."""


# --- Outcome ---

def _now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


@dataclass(frozen=True)
class RunOutcome:
    """Supervisor-recorded result. `argv` lists the vectors actually started in a container."""

    status: str
    exit_code: Optional[int]
    argv: Tuple[Tuple[str, ...], ...]
    started_at: str
    finished_at: str
    log_paths: Tuple[str, ...]
    limits_enforced: Mapping[str, Any]
    reason: str = ""
    checks: Tuple[Mapping[str, Any], ...] = ()
    supervisor_limits: Mapping[str, Any] = field(default_factory=dict)
    image: str = ""
    run_id: str = ""
    task_id: str = ""
    manifest_target_sha: str = ""
    manifest_sha256: str = ""
    docker_endpoint: str = ""

    def to_dict(self) -> Dict[str, Any]:
        return {
            "outcome_version": "sandbox-outcome-v1",
            "status": self.status,
            "exit_code": self.exit_code,
            "argv": [list(a) for a in self.argv],
            "started_at": self.started_at,
            "finished_at": self.finished_at,
            "log_paths": list(self.log_paths),
            "limits_enforced": json.loads(json.dumps(self.limits_enforced)),
            "supervisor_limits": dict(self.supervisor_limits),
            "reason": self.reason,
            "checks": json.loads(json.dumps(list(self.checks))),
            "image": self.image,
            "run_id": self.run_id,
            "task_id": self.task_id,
            "manifest_target_sha": self.manifest_target_sha,
            "target_sha_bound": False,  # the adapter does not check the worktree HEAD against the manifest
            "manifest_sha256": self.manifest_sha256,
            "docker_endpoint": self.docker_endpoint,
            "executed_on_host": False,
        }


# --- Docker command construction and read-back verification ---

def _check_mount_source(path: Path) -> str:
    text = str(path)
    if not text.startswith("/") or any(ch in UNSAFE_MOUNT_CHARS for ch in text):
        raise Refusal(f"mount source {text!r} contains characters that could alter mount options")
    return text


def container_user() -> str:
    """Run as the invoking (non-root) uid so the worktree stays writable; never as root."""
    uid, gid = os.getuid(), os.getgid()
    return "65534:65534" if uid == 0 or gid == 0 else f"{uid}:{gid}"


def build_create_args(
    *, name: str, run_id: str, sandbox: SandboxConfig, argv: Sequence[str], worktree: Path, tcb_dir: Path, user: str,
) -> List[str]:
    """Argument vector (without the docker binary) of `docker create` for one check."""
    memory = f"{sandbox.memory_mb}m"
    args = [
        "create", "--name", name, "--label", f"{RUN_LABEL}={run_id}", "--pull", "never",
        "--network", "none", "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
        "--user", user, "--read-only", "--init",
        "--tmpfs", f"/tmp:rw,noexec,nosuid,nodev,size={sandbox.tmpfs_mb}m",
        "--memory", memory, "--memory-swap", memory, "--pids-limit", str(sandbox.pids_limit),
        "--cpus", f"{sandbox.cpus:g}", "--ulimit", "core=0",
        "--mount", f"type=bind,src={_check_mount_source(worktree)},dst={WORK_MOUNT}",
        "--mount", f"type=bind,src={_check_mount_source(tcb_dir)},dst={TCB_MOUNT},readonly",
        "--workdir", WORK_MOUNT,
    ]
    for key, value in CONTAINER_ENV.items():
        args += ["-e", f"{key}={value}"]
    args += ["--entrypoint", argv[0], sandbox.image, *argv[1:]]
    return args


def _env_names(entries: Any) -> List[str]:
    return sorted(str(e).split("=", 1)[0] for e in (entries or []))


def describe_container(info: Mapping[str, Any], image_env_names: Sequence[str]) -> Dict[str, Any]:
    """What Docker reports as configured for the container (the only source of `limits_enforced`)."""
    host = info.get("HostConfig") or {}
    config = info.get("Config") or {}
    mounts = [
        {"type": m.get("Type"), "destination": m.get("Destination"), "rw": bool(m.get("RW")), "source": m.get("Source")}
        for m in (info.get("Mounts") or [])
    ]
    return {
        "image_id": info.get("Image"),
        "network_mode": host.get("NetworkMode"),
        "cap_drop": list(host.get("CapDrop") or []),
        "cap_add": list(host.get("CapAdd") or []),
        "security_opt": list(host.get("SecurityOpt") or []),
        "no_new_privileges": "no-new-privileges" in (host.get("SecurityOpt") or [])
        or "no-new-privileges:true" in (host.get("SecurityOpt") or []),
        "pid_mode": host.get("PidMode") or "",
        "ipc_mode": host.get("IpcMode") or "",
        "uts_mode": host.get("UTSMode") or "",
        "devices": list(host.get("Devices") or []),
        "restart_policy": (host.get("RestartPolicy") or {}).get("Name") or "",
        "entrypoint": list(config.get("Entrypoint") or []),
        "cmd": list(config.get("Cmd") or []),
        "read_only_rootfs": bool(host.get("ReadonlyRootfs")),
        "user": config.get("User"),
        "memory_bytes": host.get("Memory"),
        "memory_swap_bytes": host.get("MemorySwap"),
        "pids_limit": host.get("PidsLimit"),
        "nano_cpus": host.get("NanoCpus"),
        "init": bool(host.get("Init")),
        "privileged": bool(host.get("Privileged")),
        "tmpfs": dict(host.get("Tmpfs") or {}),
        "mounts": mounts,
        "extra_env_names": sorted(set(_env_names(config.get("Env"))) - set(image_env_names)),
    }


def _tmpfs_problems(options: Optional[str], size_mb: int) -> List[str]:
    """/tmp must be a tmpfs that is rw, noexec, nosuid, nodev and size-capped as configured."""
    if options is None:
        return ["tmpfs: /tmp is not a tmpfs mount"]
    tokens = set(options.split(","))
    problems = [f"tmpfs: /tmp lacks option {flag!r}" for flag in ("noexec", "nosuid", "nodev") if flag not in tokens]
    if "ro" in tokens:
        problems.append("tmpfs: /tmp is read-only (the tmp directory must be writable)")
    size = next((t[5:] for t in tokens if t.startswith("size=")), None)
    unit = {"k": 1024, "m": 1024 ** 2, "g": 1024 ** 3}.get(size[-1].lower(), 1) if size else 1
    digits = size[:-1] if size and size[-1].lower() in "kmg" else size
    if not (digits and digits.isdigit() and int(digits) * unit == size_mb * 1024 * 1024):
        problems.append(f"tmpfs: /tmp size is {size!r}, policy requires {size_mb}m")
    return problems


def verify_container(described: Mapping[str, Any], sandbox: SandboxConfig, user: str, worktree: Path, tcb_dir: Path,
                     image_id: Optional[str], argv: Optional[Sequence[str]] = None) -> List[str]:
    """Compare Docker's read-back with the policy. Returns mismatches (empty = enforced)."""
    wanted_bytes = sandbox.memory_mb * 1024 * 1024
    problems: List[str] = []

    def expect(label: str, got: Any, want: Any) -> None:
        if got != want:
            problems.append(f"{label}: docker reports {got!r}, policy requires {want!r}")

    expect("network_mode", described["network_mode"], "none")
    expect("cap_drop", described["cap_drop"], ["ALL"])
    expect("cap_add", described["cap_add"], [])
    expect("no_new_privileges", described["no_new_privileges"], True)
    if described["security_opt"] not in (["no-new-privileges"], ["no-new-privileges:true"]):
        problems.append(f"security_opt: docker reports {described['security_opt']!r}; only no-new-privileges is allowed "
                        "(no seccomp/apparmor/label overrides)")
    for key in ("pid_mode", "ipc_mode", "uts_mode"):
        if described[key] not in (("", "private", "none") if key == "ipc_mode" else ("",)):
            problems.append(f"{key}: docker reports {described[key]!r}; sharing a host or foreign namespace is not allowed")
    expect("devices", described["devices"], [])
    if described["restart_policy"] not in ("", "no"):
        problems.append(f"restart_policy: docker reports {described['restart_policy']!r}; containers must not restart")
    if argv is not None:
        expect("entrypoint", described["entrypoint"], [argv[0]])
        expect("cmd", described["cmd"], list(argv[1:]))
    expect("read_only_rootfs", described["read_only_rootfs"], True)
    expect("user", described["user"], user)
    expect("memory_bytes", described["memory_bytes"], wanted_bytes)
    expect("memory_swap_bytes", described["memory_swap_bytes"], wanted_bytes)
    expect("pids_limit", described["pids_limit"], sandbox.pids_limit)
    expect("nano_cpus", described["nano_cpus"], int(round(sandbox.cpus * 1e9)))
    expect("init", described["init"], True)
    expect("privileged", described["privileged"], False)
    if image_id is not None:
        expect("image_id", described["image_id"], image_id)
    problems.extend(_tmpfs_problems(described["tmpfs"].get("/tmp"), sandbox.tmpfs_mb))
    mounts = sorted((m["destination"], m["rw"], m["type"]) for m in described["mounts"])
    expect("mounts", mounts, [(TCB_MOUNT, False, "bind"), (WORK_MOUNT, True, "bind")])
    sources = {m["destination"]: m["source"] for m in described["mounts"]}
    if sources.get(WORK_MOUNT) and os.path.realpath(sources[WORK_MOUNT]) != os.path.realpath(worktree):
        problems.append("mount /work does not point at the worktree")
    if sources.get(TCB_MOUNT) and os.path.realpath(sources[TCB_MOUNT]) != os.path.realpath(tcb_dir):
        problems.append("mount /tcb does not point at the staged TCB")
    extra = sorted(set(described["extra_env_names"]) - set(CONTAINER_ENV))
    if extra:
        problems.append(f"unexpected environment variables in the container: {extra}")
    return problems


# --- Docker CLI access (the only process the host ever spawns) ---

@dataclass(frozen=True)
class DockerCli:
    binary: str
    env: Mapping[str, str]
    endpoint: str = ""

    def run(self, args: Sequence[str], timeout: float) -> "subprocess.CompletedProcess[str]":
        return subprocess.run(
            [self.binary, *args], capture_output=True, text=True, timeout=timeout, env=dict(self.env),
            stdin=subprocess.DEVNULL, check=False,
        )

    def popen(self, args: Sequence[str]) -> "subprocess.Popen[bytes]":
        return subprocess.Popen(
            [self.binary, *args], stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            env=dict(self.env), bufsize=0,
        )


def _tail(text: str, limit: int = 300) -> str:
    text = " ".join((text or "").split())
    return text[-limit:]


def connect_docker(docker_bin: str) -> DockerCli:
    """Locate the CLI, resolve the engine endpoint ONCE and reach the daemon.

    Precedence is DOCKER_HOST, then DOCKER_CONTEXT, then the config's current context (resolved by the
    CLI itself). Only a local unix socket is accepted: bind-mount paths have no meaning on a remote engine
    and a remote daemon is not the sandbox the policy describes. Every later call runs with that socket in
    DOCKER_HOST and without DOCKER_CONTEXT, so the endpoint cannot change mid-run.
    """
    binary = shutil.which(docker_bin)
    if binary is None:
        raise Unavailable("docker CLI not found; there is no host-execution fallback")
    base_env = {k: os.environ[k] for k in DOCKER_CLI_ENV if k in os.environ}
    endpoint = base_env.get("DOCKER_HOST", "")
    if not endpoint:
        try:
            resolved = DockerCli(binary, base_env).run(["context", "inspect", "--format", "{{.Endpoints.docker.Host}}"], 30)
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise Unavailable(f"cannot resolve the docker endpoint: {type(exc).__name__}") from exc
        if resolved.returncode != 0:
            raise Unavailable("cannot resolve the docker endpoint: " + _tail(resolved.stderr))
        endpoint = resolved.stdout.strip()
    if not endpoint.startswith("unix:///"):
        raise Unavailable(f"docker endpoint {endpoint!r} is not a local unix socket; remote engines are refused")
    pinned = {k: v for k, v in base_env.items() if k not in ("DOCKER_HOST", "DOCKER_CONTEXT")}
    cli = DockerCli(binary, {**pinned, "DOCKER_HOST": endpoint}, endpoint)
    try:
        version = cli.run(["version", "--format", "{{.Server.Version}}"], 30)
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise Unavailable(f"docker CLI not usable: {type(exc).__name__}") from exc
    if version.returncode != 0:
        raise Unavailable("docker daemon unreachable: " + _tail(version.stderr))
    return cli


def check_image(cli: DockerCli, image: str) -> Tuple[str, List[str]]:
    """Return (image id, image env names). Absent or digest-mismatched image -> Unavailable (no pull)."""
    try:
        proc = cli.run(["image", "inspect", image], 30)
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise Unavailable(f"image inspect failed: {type(exc).__name__}") from exc
    if proc.returncode != 0:
        raise Unavailable(f"pinned image {image} is not present locally (the adapter never pulls): {_tail(proc.stderr)}")
    try:
        info = json.loads(proc.stdout)[0]
    except (ValueError, IndexError, TypeError) as exc:
        raise Unavailable("image inspect returned unreadable output") from exc
    digest = image.split("@", 1)[1]
    if not any(isinstance(d, str) and (d == image or d.endswith("@" + digest)) for d in info.get("RepoDigests") or []):
        raise Unavailable(f"local image does not carry the pinned digest {digest}")
    return str(info.get("Id")), _env_names((info.get("Config") or {}).get("Env"))


# --- Log capture (supervisor-owned files under run_dir) ---

class LogPump(threading.Thread):
    """Drain a pipe into a capped file; keep draining past the cap so the child never blocks.

    Any I/O problem is kept in `error` (the pipe is still drained) and makes the check an `error`:
    evidence that could not be recorded is never a pass.
    """

    def __init__(self, stream: Any, path: Path, cap: int):
        super().__init__(daemon=True)
        self.stream, self.path, self.cap = stream, path, cap
        self.truncated, self.size, self.digest, self.error = False, 0, hashlib.sha256(), ""

    def run(self) -> None:
        handle: Any = None
        try:
            handle = os.fdopen(os.open(self.path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb")
        except OSError as exc:
            self.error = f"cannot create {self.path.name}: {exc.strerror or type(exc).__name__}"
        try:
            while True:
                chunk = self.stream.read(65536)
                if not chunk:
                    break
                room = self.cap - self.size
                if handle is None or room <= 0:
                    self.truncated = self.truncated or room <= 0
                    continue
                if len(chunk) > room:
                    chunk, self.truncated = chunk[:room], True
                try:
                    handle.write(chunk)
                except OSError as exc:
                    self.error = f"cannot write {self.path.name}: {exc.strerror or type(exc).__name__}"
                    handle.close()
                    handle = None
                    continue
                self.digest.update(chunk)
                self.size += len(chunk)
        except Exception as exc:  # noqa: BLE001 - surfaced as a check error by the supervisor
            self.error = self.error or f"log capture failed: {type(exc).__name__}"
        finally:
            if handle is not None:
                try:
                    handle.close()
                except OSError as exc:
                    self.error = self.error or f"cannot close {self.path.name}: {exc.strerror or type(exc).__name__}"


# --- Execution of one check ---

@dataclass
class _Run:
    cli: DockerCli
    run_id: str
    sandbox: SandboxConfig
    worktree: Path
    tcb_dir: Path
    user: str
    logs_dir: Path
    image_id: str
    image_env: List[str]
    limits: Dict[str, Any] = field(default_factory=dict)


def _inspect(run: _Run, name: str) -> Optional[Mapping[str, Any]]:
    proc = run.cli.run(["inspect", name], 30)
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout)[0]
    except (ValueError, IndexError, TypeError):
        return None


def _remove_and_verify(run: _Run, name: str) -> bool:
    """Force-remove the container and confirm no container of this run remains."""
    for _ in range(2):
        try:
            run.cli.run(["kill", name], 30)
        except (OSError, subprocess.TimeoutExpired):
            pass
        try:
            run.cli.run(["rm", "-f", "-v", name], 60)
            listed = run.cli.run(["ps", "-aq", "--filter", f"label={RUN_LABEL}={run.run_id}"], 30)
        except (OSError, subprocess.TimeoutExpired):
            continue
        if listed.returncode == 0 and not listed.stdout.strip() and _inspect(run, name) is None:
            return True
    return False


def _run_check(run: _Run, index: int, spec: CheckSpec, budget: float) -> Tuple[Dict[str, Any], str, str]:
    """Run one check in a fresh container. Returns (record, status, reason)."""
    name = f"engram-sbx-{run.run_id}-{index}"
    stem = f"{index:02d}-{spec.id}"
    out_path, err_path = run.logs_dir / f"{stem}.stdout.log", run.logs_dir / f"{stem}.stderr.log"
    record: Dict[str, Any] = {
        "id": spec.id, "argv": list(spec.argv), "container_name": name, "status": STATUS_ERROR, "exit_code": None,
        "started_at": None, "finished_at": None, "stdout_log": str(out_path), "stderr_log": str(err_path),
        "log_sha256": {}, "log_bytes": {}, "truncated": {}, "oom_killed": None, "container_removed": False,
        "timeout_seconds": min(spec.timeout_seconds, max(1, math.ceil(budget))), "limits_enforced": {},
    }
    status, reason, started = STATUS_ERROR, "", False
    try:
        create = run.cli.run(build_create_args(name=name, run_id=run.run_id, sandbox=run.sandbox, argv=spec.argv,
                                               worktree=run.worktree, tcb_dir=run.tcb_dir, user=run.user), 60)
        if create.returncode != 0:
            return record, STATUS_ERROR, "docker create failed: " + _tail(create.stderr)
        info = _inspect(run, name)
        if info is None:
            return record, STATUS_ERROR, "docker inspect failed after create"
        described = describe_container(info, run.image_env)
        problems = verify_container(described, run.sandbox, run.user, run.worktree, run.tcb_dir, run.image_id, spec.argv)
        run.limits.clear()
        run.limits.update({**described, "verified": not problems, "mismatches": problems,
                           "source": "docker inspect after create, before start"})
        record["limits_enforced"] = json.loads(json.dumps(run.limits))
        if problems:
            return record, STATUS_ERROR, "limit not enforced by docker: " + "; ".join(problems)

        record["started_at"], started = _now(), True
        proc = run.cli.popen(["start", "-a", name])
        pumps = (LogPump(proc.stdout, out_path, run.sandbox.log_cap_bytes), LogPump(proc.stderr, err_path, run.sandbox.log_cap_bytes))
        for pump in pumps:
            pump.start()
        timed_out = False
        try:
            proc.wait(timeout=record["timeout_seconds"])
        except subprocess.TimeoutExpired:
            timed_out = True
            run.cli.run(["kill", name], 30)
            try:
                proc.wait(timeout=15)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=15)
        for pump in pumps:
            pump.join(timeout=15)
        for stream in (proc.stdout, proc.stderr):
            stream.close()
        log_errors = [p.error or f"{p.path.name} writer did not finish" for p in pumps if p.error or p.is_alive()]
        record["finished_at"] = _now()
        after = _inspect(run, name) or {}
        state = after.get("State") or {}
        exit_code = state.get("ExitCode")
        record["exit_code"] = exit_code if _is_int(exit_code) else None
        record["oom_killed"] = bool(state.get("OOMKilled"))
        if timed_out:
            status, reason = STATUS_TIMEOUT, f"check '{spec.id}' exceeded {record['timeout_seconds']}s; container killed"
        elif state.get("Status") != "exited" or record["exit_code"] is None:
            status, reason = STATUS_ERROR, f"container ended in unexpected state {state.get('Status')!r}"
        elif record["exit_code"] == 0:
            status = STATUS_PASSED
        else:
            status, reason = STATUS_FAILED, f"check '{spec.id}' exited {record['exit_code']}"
        for label, pump in zip(("stdout", "stderr"), pumps):
            record["log_sha256"][label] = pump.digest.hexdigest()
            record["log_bytes"][label] = pump.size
            record["truncated"][label] = pump.truncated
        if log_errors:
            status, reason = STATUS_ERROR, "evidence capture failed: " + "; ".join(log_errors)
    except (OSError, subprocess.TimeoutExpired) as exc:
        status, reason = STATUS_ERROR, f"docker call failed: {type(exc).__name__}"
        record["finished_at"] = record["finished_at"] or (_now() if started else None)
    finally:
        record["container_removed"] = _remove_and_verify(run, name)
    if not record["container_removed"]:
        return {**record, "status": STATUS_ERROR}, STATUS_ERROR, "container survived removal; run is not trustworthy"
    return {**record, "status": status}, status, reason


# --- Staging, path policy and the public entry point ---

def _is_within(child: Path, parent: Path) -> bool:
    return child == parent or parent in child.parents


def prepare_paths(worktree_path: Path, run_dir: Path, tcb_inputs: Sequence[Path]) -> Tuple[Path, Path]:
    try:
        worktree = Path(worktree_path).resolve(strict=True)
    except (OSError, RuntimeError) as exc:
        raise Refusal(f"worktree does not exist: {worktree_path}") from exc
    if not worktree.is_dir():
        raise Refusal("worktree is not a directory")
    if worktree == Path(worktree.anchor) or _is_within(Path.home().resolve(), worktree):
        raise Refusal("worktree must not be the filesystem root, the home directory or an ancestor of it")
    _check_mount_source(worktree)
    run_path = Path(run_dir).resolve(strict=False)
    if _is_within(run_path, worktree) or _is_within(worktree, run_path):
        raise Refusal("run_dir and worktree must be disjoint: evidence must stay outside the writer-writable mount")
    for path in tcb_inputs:
        if _is_within(Path(path).resolve(strict=False), worktree):
            raise Refusal(f"TCB input {path} is inside the writer-writable worktree")
    if _is_within(Path(__file__).resolve(), worktree):
        raise Refusal("the active sandbox adapter lives inside the worktree; the writer could modify the TCB")
    if run_path.exists():
        _check_run_dir_owner(run_path)
        if any(run_path.iterdir()):
            raise Refusal("run_dir must not exist or must be empty; evidence is never overwritten")
    return worktree, run_path


def _check_run_dir_owner(run_path: Path) -> None:
    """The evidence directory must be ours alone: a real directory, owned by us, not group/world writable."""
    info = os.lstat(run_path)
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.geteuid() or info.st_mode & 0o022:
        raise Refusal("run_dir must be a directory owned by the current user and not writable by group or others")


def _create_run_dir(run_path: Path) -> None:
    if not run_path.exists():
        run_path.mkdir(parents=True, mode=0o700)
    run_path.chmod(0o700)
    _check_run_dir_owner(run_path)


def _stage_tcb(tcb_dir: Path, manifest: bytes, registry: bytes, catalog: bytes, tools: Mapping[str, Path]) -> None:
    (tcb_dir / "tools").mkdir()
    files = {"manifest.json": manifest, "registry.json": registry, "catalog.json": catalog}
    for name, source in tools.items():
        if not TOOL_NAME_RE.fullmatch(name):
            raise Refusal(f"invalid tool name {name!r}")
        files[f"tools/{name}"] = Path(source).read_bytes()
    for rel, data in files.items():
        target = tcb_dir / rel
        target.write_bytes(data)
        target.chmod(0o444)
    for directory in (tcb_dir, tcb_dir / "tools"):
        directory.chmod(0o755)


def _validate_manifest(staged_manifest: Path, staged_catalog: Path, registry: Registry) -> Dict[str, Any]:
    validator = _validator()
    result = validator.validate_file_detailed(
        staged_manifest, "task-v2", SCHEMAS_DIR,
        expectations=validator.Expectations(policy_version=registry.policy_version, catalog_path=staged_catalog),
    )
    if not result.ok:
        raise Refusal("manifest invalid: " + "; ".join(str(e) for e in result.errors[:3]))
    data = json.loads(staged_manifest.read_text(encoding="utf-8"))
    caps = set(data["allowed_capabilities"])
    if not {"sandbox_container", "network_none"} <= caps:
        raise Refusal("manifest must grant sandbox_container and network_none")
    unknown = [c for c in data["required_checks"] if c not in registry.checks]
    if unknown:
        raise Refusal(f"check id(s) not in the approved registry: {unknown}")
    return data


def run_isolated(
    manifest_path: Path,
    worktree_path: Path,
    run_dir: Path,
    *,
    registry_path: Path = DEFAULT_REGISTRY,
    catalog_path: Path = DEFAULT_CATALOG,
    docker_bin: str = "docker",
    tcb_files: Optional[Mapping[str, Path]] = None,
) -> RunOutcome:
    """Run the manifest's approved checks in the sandbox. Never raises for policy/availability problems."""
    started_at, run_id = _now(), uuid.uuid4().hex[:16]
    run_path: Optional[Path] = None
    stage: Optional[Path] = None
    ran: List[Mapping[str, Any]] = []
    limits: Mapping[str, Any] = {}
    meta: Dict[str, str] = {"image": "", "task_id": "", "manifest_target_sha": "", "manifest_sha256": "", "docker_endpoint": ""}
    supervisor: Dict[str, Any] = {}
    status, reason, exit_code = STATUS_ERROR, "", None
    try:
        worktree, run_path = prepare_paths(worktree_path, run_dir, [manifest_path, registry_path, catalog_path])
        manifest_bytes, registry_bytes, catalog_bytes = (Path(p).read_bytes() for p in (manifest_path, registry_path, catalog_path))
        meta["manifest_sha256"] = hashlib.sha256(manifest_bytes).hexdigest()
        _create_run_dir(run_path)
        stage = Path(tempfile.mkdtemp(prefix="engram-tcb-")).resolve()
        if _is_within(stage, worktree):
            raise Refusal("the TCB staging directory would be inside the writer-writable worktree (check TMPDIR)")
        _stage_tcb(stage, manifest_bytes, registry_bytes, catalog_bytes, tcb_files or {})
        registry = load_registry(stage / "registry.json", stage / "catalog.json")
        manifest = _validate_manifest(stage / "manifest.json", stage / "catalog.json", registry)
        meta.update(image=registry.sandbox.image, task_id=manifest["task_id"], manifest_target_sha=manifest["target_sha"])

        cli = connect_docker(docker_bin)
        meta["docker_endpoint"] = cli.endpoint
        if _is_within(Path(cli.endpoint[len("unix://"):]).resolve(strict=False), worktree):
            raise Refusal("the docker socket lives inside the worktree; the writer could reach the engine")
        image_id, image_env = check_image(cli, registry.sandbox.image)
        logs_dir = run_path / "logs"
        logs_dir.mkdir(mode=0o700)
        run = _Run(cli, run_id, registry.sandbox, worktree, stage, container_user(), logs_dir, image_id, image_env)

        budget = min(manifest["timeout_seconds"], registry.sandbox.max_total_timeout_seconds)
        supervisor = {"total_timeout_seconds": budget, "log_cap_bytes": registry.sandbox.log_cap_bytes,
                      "enforced_by": "supervisor (not docker)"}
        deadline = time.monotonic() + budget
        status = STATUS_PASSED
        for index, check_id in enumerate(manifest["required_checks"]):
            remaining = deadline - time.monotonic()
            if remaining < 1:
                status, reason = STATUS_TIMEOUT, f"manifest timeout of {budget}s exhausted before '{check_id}'"
                break
            record, check_status, check_reason = _run_check(run, index, registry.checks[check_id], remaining)
            ran.append(record)
            exit_code = record["exit_code"]
            if check_status != STATUS_PASSED:
                status, reason = check_status, check_reason
                break
        limits = dict(run.limits)
    except Refusal as exc:
        status, reason = STATUS_REFUSED, str(exc)
    except Unavailable as exc:
        status, reason = STATUS_UNAVAILABLE, str(exc)
    except Exception as exc:  # noqa: BLE001 - an unexpected supervisor failure must be a visible non-pass
        status, reason = STATUS_ERROR, f"internal error: {type(exc).__name__}: {exc}"
    finally:
        if stage is not None:
            shutil.rmtree(stage, ignore_errors=True)
    if status in (STATUS_REFUSED, STATUS_UNAVAILABLE):
        exit_code = None
    started = [r for r in ran if r.get("started_at")]
    outcome = RunOutcome(
        status=status, exit_code=exit_code, argv=tuple(tuple(r["argv"]) for r in started), started_at=started_at,
        finished_at=_now(), log_paths=tuple(p for r in started for p in (r["stdout_log"], r["stderr_log"])),
        limits_enforced=limits, reason=reason, checks=tuple(ran), supervisor_limits=supervisor, run_id=run_id, **meta,
    )
    return _persist(outcome, run_path)


def _persist(outcome: RunOutcome, run_path: Optional[Path]) -> RunOutcome:
    """Write outcome.json (supervisor only, exclusive create). Failure to persist downgrades a pass."""
    if run_path is None or not run_path.is_dir():
        return outcome
    try:
        fd = os.open(run_path / "outcome.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(outcome.to_dict(), handle, indent=2, sort_keys=True)
            handle.write("\n")
    except OSError as exc:
        return RunOutcome(**{**outcome.__dict__, "status": STATUS_ERROR if outcome.status == STATUS_PASSED else outcome.status,
                             "reason": f"{outcome.reason} (outcome.json not written: {type(exc).__name__})".strip()})
    return outcome


# --- CLI ---

def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Hardened sandbox adapter (fake writer only; see ADR agent-harness-hardening-v1)")
    sub = parser.add_subparsers(dest="command", required=True)
    run_cmd = sub.add_parser("run", help="run a manifest's approved checks in the sandbox; prints the outcome JSON")
    run_cmd.add_argument("--manifest", required=True, type=Path)
    run_cmd.add_argument("--worktree", required=True, type=Path)
    run_cmd.add_argument("--run-dir", required=True, type=Path)
    run_cmd.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    run_cmd.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    run_cmd.add_argument("--docker", default="docker", help="docker CLI to use (default: docker on PATH)")
    val = sub.add_parser("validate-registry", help="strictly validate a registry against the check catalog")
    val.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    val.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.command == "validate-registry":
        try:
            registry = load_registry(args.registry, args.catalog)
        except Refusal as exc:
            print(f"REGISTRY: FAIL {exc}", file=sys.stderr)
            return 2
        print(f"REGISTRY: OK checks={len(registry.checks)} image={registry.sandbox.image}")
        return 0
    outcome = run_isolated(args.manifest, args.worktree, args.run_dir, registry_path=args.registry,
                           catalog_path=args.catalog, docker_bin=args.docker)
    print(json.dumps(outcome.to_dict(), indent=2, sort_keys=True))
    return EXIT_FOR_STATUS.get(outcome.status, 1)


if __name__ == "__main__":
    sys.exit(main())
