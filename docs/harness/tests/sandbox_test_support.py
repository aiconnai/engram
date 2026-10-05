#!/usr/bin/env python3
"""Shared fixtures for the sandbox adapter tests (task H3). Stdlib only, offline.

* load_adapter(): imports docs/harness/bin/sandbox-adapter.py (hyphenated filename).
* SandboxEnv: temporary worktree / run_dir / registry / catalog / manifest layout.
* STUB_DOCKER: a tiny fake `docker` CLI used by the OFFLINE unit tests. It records every call,
  synthesizes `docker inspect` output from the flags it was given (so the adapter's "limits actually
  enforced" check is exercised for real) and can be told to misbehave. The real-Docker smoke tests
  (test_sandbox_smoke.py) never use it.
"""

from __future__ import annotations

import importlib.util
import json
import shutil
import stat
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
ADAPTER_PATH = REPO_ROOT / "docs" / "harness" / "bin" / "sandbox-adapter.py"
FAKE_WRITER_PATH = REPO_ROOT / "docs" / "harness" / "tests" / "fake_writer.py"
PROD_REGISTRY = REPO_ROOT / "docs" / "harness" / "checks" / "registry.json"
PROD_CATALOG = REPO_ROOT / "docs" / "harness" / "schemas" / "check-catalog-v1.json"

TEST_IMAGE = "python@sha256:" + "a" * 64
POLICY_VERSION = "harness-hardening-v1"


def load_adapter():
    spec = importlib.util.spec_from_file_location("sandbox_adapter", ADAPTER_PATH)
    if spec is None or spec.loader is None:  # pragma: no cover - only if the file is missing
        raise ImportError(f"cannot load {ADAPTER_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules["sandbox_adapter"] = module
    spec.loader.exec_module(module)
    return module


def default_sandbox(**over):
    base = {
        "image": TEST_IMAGE, "memory_mb": 128, "pids_limit": 32, "cpus": 1, "tmpfs_mb": 16,
        "log_cap_bytes": 65536, "max_total_timeout_seconds": 120,
    }
    base.update(over)
    return base


def task_manifest(check_ids, timeout_seconds=60, **over):
    manifest = {
        "schema_version": "task-v2", "policy_version": POLICY_VERSION, "task_id": "h3-sandbox-test",
        "target_sha": "0123456789abcdef0123456789abcdef01234567",
        "base_sha": "abcdef0123456789abcdef0123456789abcdef01",
        "allowed_capabilities": ["workspace_read", "workspace_write", "test_exec", "sandbox_container", "network_none"],
        "required_checks": list(check_ids), "timeout_seconds": timeout_seconds,
        "allowed_paths": ["src/"], "protected_paths": [".git/"],
    }
    manifest.update(over)
    return manifest


class SandboxEnv:
    """Temporary layout owned by one test. `checks` maps check id -> argv (or a full entry dict)."""

    def __init__(self, test: unittest.TestCase, checks, sandbox=None, manifest_over=None, check_ids=None, timeout=60):
        self.root = Path(tempfile.mkdtemp(prefix="h3-sbx-")).resolve()
        test.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.worktree = self.root / "worktree"
        self.worktree.mkdir()
        (self.worktree / "README.txt").write_text("synthetic worktree\n", encoding="utf-8")
        self.run_dir = self.root / "runs" / "run1"
        self.registry_path = self.root / "registry.json"
        self.catalog_path = self.root / "catalog.json"
        self.manifest_path = self.root / "manifest.json"
        self.write_registry(checks, sandbox)
        ids = list(check_ids) if check_ids is not None else list(checks)
        self.write_manifest(ids, timeout, manifest_over)

    def write_registry(self, checks, sandbox=None):
        entries = {cid: (v if isinstance(v, dict) else {"argv": v, "timeout_seconds": 30}) for cid, v in checks.items()}
        registry = {"registry_version": "check-registry-v1", "policy_version": POLICY_VERSION,
                    "sandbox": sandbox or default_sandbox(), "checks": entries}
        self.registry_path.write_text(json.dumps(registry, indent=2), encoding="utf-8")
        catalog = {"catalog_version": "check-catalog-v1",
                   "checks": {cid: {"description": f"synthetic {cid}"} for cid in entries}}
        self.catalog_path.write_text(json.dumps(catalog, indent=2), encoding="utf-8")

    def write_manifest(self, check_ids, timeout=60, over=None):
        self.manifest_path.write_text(json.dumps(task_manifest(check_ids, timeout, **(over or {})), indent=2), encoding="utf-8")

    def kwargs(self, **extra):
        return {"registry_path": self.registry_path, "catalog_path": self.catalog_path, **extra}


STUB_DOCKER = r'''#!__PYTHON__
import json, os, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
CFG = json.loads((HERE / "stub.json").read_text()) if (HERE / "stub.json").exists() else {}
STATE = HERE / "state"
STATE.mkdir(exist_ok=True)
with open(HERE / "calls.log", "a") as log:
    log.write(json.dumps({"argv": sys.argv[1:], "env_names": sorted(os.environ),
                          "docker_host": os.environ.get("DOCKER_HOST"), "docker_context": os.environ.get("DOCKER_CONTEXT")}) + "\n")

args = sys.argv[1:]
VALUE_FLAGS = {"--name", "--label", "--pull", "--network", "--cap-drop", "--security-opt", "--user", "--tmpfs",
               "--memory", "--memory-swap", "--pids-limit", "--cpus", "--mount", "--workdir", "-e", "--entrypoint",
               "--ulimit", "--ipc", "--hostname"}
BOOL_FLAGS = {"--read-only", "--init"}


def to_bytes(text):
    unit = {"k": 1024, "m": 1024 ** 2, "g": 1024 ** 3}.get(text[-1].lower())
    return int(text[:-1]) * unit if unit else int(text)


def load(name):
    p = STATE / (name + ".json")
    return json.loads(p.read_text()) if p.exists() else None


def save(name, data):
    (STATE / (name + ".json")).write_text(json.dumps(data))


def fail(msg, code=1):
    sys.stderr.write(msg + "\n")
    sys.exit(code)


cmd = args[0] if args else ""
if cmd == "context" and args[1] == "inspect":
    # Emulates the CLI's own endpoint resolution: DOCKER_CONTEXT, else the config's current context.
    contexts = CFG.get("contexts", {})
    print(contexts.get(os.environ.get("DOCKER_CONTEXT", ""), CFG.get("context_host", "unix:///var/run/docker.sock")))
elif cmd == "version":
    if CFG.get("daemon_down"):
        fail("Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?")
    print("29.4.0")
elif cmd == "image" and args[1] == "inspect":
    if CFG.get("image_absent"):
        fail("Error: No such image: " + args[2])
    digests = CFG.get("repo_digests", [args[2]])
    print(json.dumps([{"Id": "sha256:" + "b" * 64, "RepoDigests": digests, "Config": {"Env": ["PATH=/usr/local/bin:/usr/bin"]}}]))
elif cmd == "create":
    if CFG.get("create_fail"):
        fail("create failed")
    c = {"flags": {}, "mounts": [], "env": [], "tmpfs": [], "security_opt": [], "cap_drop": [], "bool": [], "cmd": []}
    i = 1
    while i < len(args):
        a = args[i]
        if a in BOOL_FLAGS:
            c["bool"].append(a); i += 1
        elif a in VALUE_FLAGS:
            v = args[i + 1]
            if a == "--mount": c["mounts"].append(v)
            elif a == "-e": c["env"].append(v)
            elif a == "--tmpfs": c["tmpfs"].append(v)
            elif a == "--security-opt": c["security_opt"].append(v)
            elif a == "--cap-drop": c["cap_drop"].append(v)
            else: c["flags"][a] = v
            i += 2
        else:
            c["image"] = a; c["cmd"] = args[i + 1:]; break
    c["state"] = {"Status": "created", "ExitCode": 0, "OOMKilled": False}
    save(c["flags"]["--name"], c)
    print("c" * 64)
elif cmd == "inspect":
    c = load(args[1])
    if c is None:
        fail("Error: No such object: " + args[1])
    f, mode = c["flags"], CFG.get("inspect_mode", "")
    mounts = []
    for m in c["mounts"]:
        kv = dict(p.split("=", 1) if "=" in p else (p, "") for p in m.split(","))
        mounts.append({"Type": kv["type"], "Source": kv["src"], "Destination": kv["dst"], "RW": "readonly" not in kv})
    if mode == "extra_mount":
        mounts.append({"Type": "bind", "Source": "/", "Destination": "/host", "RW": True})
    memory = 0 if mode == "ignore_memory" else to_bytes(f["--memory"])
    secopt = [] if mode == "no_nnp" else list(c["security_opt"])
    if mode in ("seccomp_unconfined", "apparmor_unconfined"):
        secopt.append(mode.split("_")[0] + "=unconfined")
    host = {"NetworkMode": "bridge" if mode == "net_bridge" else f["--network"], "CapDrop": c["cap_drop"], "CapAdd": None,
            "SecurityOpt": secopt, "ReadonlyRootfs": "--read-only" in c["bool"], "UTSMode": "",
            "RestartPolicy": {"Name": "always" if mode == "restart_always" else "no"},
            "Memory": memory, "MemorySwap": to_bytes(f["--memory-swap"]), "PidsLimit": int(f["--pids-limit"]),
            "NanoCpus": int(float(f["--cpus"]) * 1e9), "Init": "--init" in c["bool"], "Privileged": mode == "privileged",
            "PidMode": "host" if mode == "pid_host" else "", "IpcMode": "private", "Binds": None,
            "Devices": [{"PathOnHost": "/dev/kvm"}] if mode == "device" else [],
            "Tmpfs": {t.split(":", 1)[0]: t.split(":", 1)[1] for t in c["tmpfs"]}}
    env = ["PATH=/usr/local/bin:/usr/bin"] + c["env"]
    print(json.dumps([{"Id": "c" * 64, "Name": "/" + args[1], "Image": "sha256:" + "b" * 64, "State": c["state"],
                       "HostConfig": host, "Mounts": mounts,
                       "Config": {"User": f["--user"], "Env": env, "Image": c["image"],
                                  "Entrypoint": [f["--entrypoint"]], "Cmd": c["cmd"] or None}}]))
elif cmd == "start":
    c = load(args[2])
    if c is None:
        fail("no such container")
    s = CFG.get("start", {})
    c["state"]["Status"] = "running"; save(args[2], c)
    sys.stdout.write(s.get("stdout", "")); sys.stdout.flush()
    if s.get("stdout_bytes"):
        chunk = "y" * 1023 + "\n"
        for _ in range(s["stdout_bytes"] // 1024):
            sys.stdout.write(chunk)
        sys.stdout.flush()
    sys.stderr.write(s.get("stderr", "")); sys.stderr.flush()
    if s.get("sleep"):
        time.sleep(s["sleep"])
    c = load(args[2])
    if c is not None:
        c["state"] = {"Status": "exited", "ExitCode": s.get("exit_code", 0), "OOMKilled": bool(s.get("oom"))}
        save(args[2], c)
    sys.exit(s.get("exit_code", 0))
elif cmd == "kill":
    c = load(args[1])
    if c is not None:
        c["state"] = {"Status": "exited", "ExitCode": 137, "OOMKilled": False}; save(args[1], c)
elif cmd == "rm":
    if CFG.get("rm_fail"):
        fail("cannot remove container")
    p = STATE / (args[-1] + ".json")
    if p.exists():
        p.unlink()
elif cmd == "ps":
    for p in sorted(STATE.glob("*.json")):
        print(p.stem)
else:
    fail("stub docker: unsupported command " + cmd, 2)
'''


class StubDocker:
    """A fake `docker` executable plus its configuration and recorded calls."""

    def __init__(self, test: unittest.TestCase, **config):
        self.dir = Path(tempfile.mkdtemp(prefix="h3-stub-")).resolve()
        test.addCleanup(shutil.rmtree, self.dir, ignore_errors=True)
        self.path = self.dir / "docker"
        self.path.write_text(STUB_DOCKER.replace("__PYTHON__", sys.executable), encoding="utf-8")
        self.path.chmod(self.path.stat().st_mode | stat.S_IXUSR)
        self.configure(**config)

    def configure(self, **config):
        (self.dir / "stub.json").write_text(json.dumps(config), encoding="utf-8")

    def calls(self):
        log = self.dir / "calls.log"
        if not log.exists():
            return []
        return [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines() if line.strip()]

    def commands(self):
        return [c["argv"][0] for c in self.calls() if c["argv"]]

    def remaining_containers(self):
        return sorted(p.stem for p in (self.dir / "state").glob("*.json")) if (self.dir / "state").exists() else []
