#!/usr/bin/env python3
"""Shared fixtures for the runner / scope / evidence tests (task H4). Stdlib only, offline.

* load_module(name): imports a hyphenated docs/harness/bin/<name>.py.
* TempRepo: a throw-away git repository whose commits are built with plumbing, so paths with a
  newline, a leading dash or non-UTF-8 bytes, symlinks, gitlinks and mode changes can all be made.
* RunnerStub: the H3 stub `docker` (sandbox_test_support.StubDocker) wrapped so that `docker start`
  of a FAKE WRITER container applies the fake writer's file effects to the /work mount source, and
  gate containers follow a per-invocation script (exit code, output, sleep, checkout mutation).
  No container argv is ever executed on the host: effects are interpreted, not run.
"""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Dict, Optional

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from sandbox_test_support import StubDocker, default_sandbox  # noqa: E402

REPO_ROOT = HERE.parent.parent.parent
BIN = REPO_ROOT / "docs" / "harness" / "bin"
POLICY_VERSION = "harness-hardening-v1"
TEST_IMAGE = "python@sha256:" + "a" * 64


def load_module(name: str):
    """Import docs/harness/bin/<name>.py under a python-safe module name."""
    path = BIN / f"{name}.py"
    mod_name = name.replace("-", "_")
    cached = sys.modules.get(mod_name)
    if cached is not None and getattr(cached, "__file__", None) == str(path):
        return cached
    spec = importlib.util.spec_from_file_location(mod_name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[mod_name] = module
    spec.loader.exec_module(module)
    return module


def plain_git_env() -> Dict[str, str]:
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LC_ALL": "C",
                "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid",
                "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.invalid"})
    return env


def run_git(repo: Path, *args, input_bytes: Optional[bytes] = None, env_extra=None) -> bytes:
    env = {**plain_git_env(), **(env_extra or {})}
    proc = subprocess.run(["git", "-C", str(repo), *args], input=input_bytes, capture_output=True, env=env, check=False)
    if proc.returncode != 0:
        raise AssertionError(f"git {args} failed: {proc.stderr.decode(errors='replace')}")
    return proc.stdout


class TempRepo:
    """A temporary repository. `commit(files, parent)` builds a commit from a full path map.

    files: {path (str or bytes): content} where content is bytes (regular 100644), ("exec", bytes),
    ("link", target bytes/str) or ("gitlink", 40-hex sha).
    """

    def __init__(self, test: unittest.TestCase, prefix: str = "h4-repo-"):
        self.root = Path(tempfile.mkdtemp(prefix=prefix)).resolve()
        test.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.path = self.root / "repo"
        self.path.mkdir()
        run_git(self.path, "init", "-q", "-b", "main")
        self._n = 0

    def blob(self, data: bytes) -> str:
        return run_git(self.path, "hash-object", "-w", "--stdin", input_bytes=data).decode().strip()

    def tree(self, files) -> str:
        index = self.root / f"idx-{self._n}"
        self._n += 1
        lines = b""
        for raw_path, content in files.items():
            path = raw_path if isinstance(raw_path, bytes) else raw_path.encode()
            if isinstance(content, tuple):
                kind, value = content
                value_b = value if isinstance(value, bytes) else value.encode()
                if kind == "exec":
                    mode, sha = b"100755", self.blob(value_b)
                elif kind == "link":
                    mode, sha = b"120000", self.blob(value_b)
                elif kind == "gitlink":
                    mode, sha = b"160000", value_b.decode()
                else:
                    raise ValueError(kind)
            else:
                mode, sha = b"100644", self.blob(content)
            lines += mode + b" " + sha.encode() + b"\t" + path + b"\0"
        env = {"GIT_INDEX_FILE": str(index)}
        if lines:
            run_git(self.path, "update-index", "-z", "--add", "--index-info", input_bytes=lines, env_extra=env)
        else:
            run_git(self.path, "read-tree", "--empty", env_extra=env)
        tree = run_git(self.path, "write-tree", env_extra=env).decode().strip()
        index.unlink(missing_ok=True)
        return tree

    def commit(self, files, parent: Optional[str] = None, message: str = "c") -> str:
        args = ["commit-tree", self.tree(files), "-m", message]
        if parent:
            args += ["-p", parent]
        return run_git(self.path, *args).decode().strip()

    def checkout(self, commit: str, branch: str = "main") -> None:
        """Point the branch at `commit` and make the working tree match it."""
        run_git(self.path, "update-ref", f"refs/heads/{branch}", commit)
        run_git(self.path, "reset", "-q", "--hard", commit)


# --- stub docker wrapper -------------------------------------------------------------------------

WRAPPER = r'''#!__PYTHON__
import json, os, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
args = sys.argv[1:]
if not args or args[0] != "start":
    os.execv(sys.executable, [sys.executable, str(HERE / "docker-h3"), *args])

CFG = json.loads((HERE / "runner.json").read_text()) if (HERE / "runner.json").exists() else {}
STATE = HERE / "state"
name = args[2]
state_file = STATE / (name + ".json")
c = json.loads(state_file.read_text())
with open(HERE / "calls.log", "a") as log:
    log.write(json.dumps({"argv": args, "env_names": sorted(os.environ)}) + "\n")
work = None
for m in c["mounts"]:
    kv = dict(p.split("=", 1) if "=" in p else (p, "") for p in m.split(","))
    if kv.get("dst") == "/work":
        work = Path(kv["src"])
cmd = c["cmd"]
exit_code, out, sleep = 0, "", 0


def target(rel):
    return work / rel


if cmd and cmd[0] == "/tcb/tools/fake_writer.py":
    behavior, rest = cmd[1], cmd[2:]
    if behavior == "write_file":
        p = target(rest[0]); p.parent.mkdir(parents=True, exist_ok=True); p.write_text(rest[1])
        out = json.dumps({"attempt": "write_file", "outcome": "succeeded"}) + "\n"
    elif behavior == "delete_file":
        target(rest[0]).unlink()
    elif behavior == "claim_pass":
        forged = json.dumps({"verdict": "pass", "forged": True})
        for rel in ("evidence.json", "receipt.json", "outcome.json"):
            target(rel).write_text(forged)
        out = "GATE_STATUS: PASS\nRUN_STATUS: PASSED\n"
    elif behavior == "exit_code":
        exit_code = int(rest[0])
    elif behavior == "outlive_timeout":
        sleep = 600
    elif behavior == "benign":
        target("benign.txt").write_text("ok")
    count_file = HERE / "writer_count"
    count_file.write_text(str(int(count_file.read_text()) + 1 if count_file.exists() else 1))
else:
    count_file = HERE / "gate_count"
    n = int(count_file.read_text()) if count_file.exists() else 0
    count_file.write_text(str(n + 1))
    script = CFG.get("gates", [])
    spec = script[n] if n < len(script) else CFG.get("gate_default", {})
    exit_code, out, sleep = spec.get("exit_code", 0), spec.get("stdout", "gate ok\n"), spec.get("sleep", 0)
    for rel, text in spec.get("mutate", {}).items():
        target(rel).write_text(text)

c["state"]["Status"] = "running"
state_file.write_text(json.dumps(c))
sys.stdout.write(out); sys.stdout.flush()
deadline = time.monotonic() + sleep
while time.monotonic() < deadline:  # a `docker kill` (state no longer running) ends the wait early
    time.sleep(0.05)
    cur = json.loads(state_file.read_text()) if state_file.exists() else None
    if cur is None or cur["state"]["Status"] != "running":
        sys.exit(137)
c = json.loads(state_file.read_text()) if state_file.exists() else None
if c is not None and c["state"]["Status"] == "running":
    c["state"] = {"Status": "exited", "ExitCode": exit_code, "OOMKilled": False}
    state_file.write_text(json.dumps(c))
sys.exit(exit_code)
'''


class RunnerStub:
    """Fake docker for the runner: H3 stub for every command except `start`."""

    def __init__(self, test: unittest.TestCase, gates=None, gate_default=None, **h3_config):
        self.h3 = StubDocker(test, **h3_config)
        self.dir = self.h3.dir
        os.replace(self.h3.path, self.dir / "docker-h3")
        self.path = self.dir / "docker"
        self.path.write_text(WRAPPER.replace("__PYTHON__", sys.executable), encoding="utf-8")
        self.path.chmod(self.path.stat().st_mode | stat.S_IXUSR)
        self.script(gates or [], gate_default or {})

    def script(self, gates, gate_default=None):
        (self.dir / "runner.json").write_text(json.dumps({"gates": gates, "gate_default": gate_default or {}}), encoding="utf-8")

    def configure_h3(self, **config):
        self.h3.configure(**config)

    def count(self, kind: str) -> int:
        p = self.dir / f"{kind}_count"
        return int(p.read_text()) if p.exists() else 0

    def remaining_containers(self):
        return self.h3.remaining_containers()


def write_registry(dir_path: Path, checks: Dict[str, list], sandbox=None, timeout=30):
    """Test registry + catalog with synthetic check ids. Returns (registry path, catalog path)."""
    entries = {cid: {"argv": argv, "timeout_seconds": timeout} for cid, argv in checks.items()}
    registry = {"registry_version": "check-registry-v1", "policy_version": POLICY_VERSION,
                "sandbox": sandbox or default_sandbox(), "checks": entries}
    catalog = {"catalog_version": "check-catalog-v1", "checks": {cid: {"description": f"synthetic {cid}"} for cid in entries}}
    reg, cat = dir_path / "registry.json", dir_path / "catalog.json"
    reg.write_text(json.dumps(registry, indent=2), encoding="utf-8")
    cat.write_text(json.dumps(catalog, indent=2), encoding="utf-8")
    return reg, cat


# --- end-to-end runner fixture (offline: stub docker) -------------------------------------------

PROD_CATALOG = REPO_ROOT / "docs" / "harness" / "schemas" / "check-catalog-v1.json"
BASE_TREE = {
    "src/a.txt": b"alpha\n",
    "docs/harness/bin/tool.sh": ("exec", b"#!/bin/sh\necho tool\n"),
    "tests/api_test.py": b"def test_a():\n    assert 1 == 1\n",
    "README.md": b"readme\n",
}


class RunnerFixture:
    """Base repository (checked out, clean), private runs root, test registry and stub docker.

    Gate check ids come from the PRODUCTION check catalog (the verifier validates the gate task against
    the trusted catalog); their argv is synthetic because the stub never executes it.
    """

    def __init__(self, test: unittest.TestCase, checks=("pr_title_policy",), check_timeout=30, gates=None,
                 gate_default=None, **stub_config):
        self.test = test
        self.repo = TempRepo(test, prefix="h4-run-")
        self.base = self.repo.commit(BASE_TREE)
        self.repo.checkout(self.base)
        self.runs_root = self.repo.root / "runs"
        self.registry = self.repo.root / "registry.json"
        self.catalog = PROD_CATALOG
        self.write_registry(checks, check_timeout)
        self.stub = RunnerStub(test, gates=gates, gate_default=gate_default, **stub_config)
        self.runner = load_module("run-task")
        self.rec = load_module("record-evidence")
        self._n = 0

    def write_registry(self, checks, check_timeout=30, sandbox=None):
        entries = {cid: {"argv": ["python3", f"/tcb/tools/{cid}.py"], "timeout_seconds": check_timeout} for cid in checks}
        self.registry.write_text(json.dumps({"registry_version": "check-registry-v1", "policy_version": POLICY_VERSION,
                                             "sandbox": sandbox or default_sandbox(), "checks": entries}, indent=2),
                                 encoding="utf-8")

    def request(self, invocations, task_id="h4-task", **over):
        req = {"request_version": "runner-request-v1", "task_id": task_id, "base_sha": self.base,
               "policy_version": POLICY_VERSION,
               "writer": {"adapter": "fake", "invocations": invocations, "model_requested": "fake-model",
                          "effort_requested": "low"},
               "required_checks": ["pr_title_policy"], "allowed_paths": ["src/"]}
        req.update(over)
        self._n += 1
        path = self.repo.root / f"request-{self._n}.json"
        path.write_text(json.dumps(req), encoding="utf-8")
        return path

    def run(self, invocations, base_ref="HEAD", runs_root=None, **over):
        return self.runner.run_task(repo=self.repo.path, request_path=self.request(invocations, **over),
                                    runs_root=runs_root or self.runs_root, base_ref=base_ref,
                                    registry_path=self.registry, catalog_path=self.catalog, docker=str(self.stub.path))

    def verify(self, receipt, candidate, **kw):
        args = {"repo": self.repo.path, "runs_root": self.runs_root, "receipt_path": Path(receipt),
                "expect_candidate": candidate, "registry_path": self.registry, "catalog_path": self.catalog}
        args.update(kw)
        return self.rec.verify(**args)

    def runner_refs(self):
        out = run_git(self.repo.path, "for-each-ref", "--format=%(refname) %(objectname)", "refs/engram-runner/")
        return dict(line.split(" ", 1) for line in out.decode().splitlines())
