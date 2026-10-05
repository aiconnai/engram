#!/usr/bin/env python3
"""OFFLINE unit tests of docs/harness/bin/sandbox-adapter.py (task H3).

These tests never talk to a real Docker daemon: a recording fake `docker` executable stands in for
the CLI (sandbox_test_support.StubDocker), so the suite is deterministic and runs in the mandatory
offline lane on machines without Docker. What it proves: fail-closed refusal paths, argv
construction, that no check argv is ever executed on the host, truthful `limits_enforced`, timeout
and survivor handling, and supervisor-owned evidence. What it cannot prove is that the container
boundary really holds; that is the job of test_sandbox_smoke.py (real Docker, fake writer), run by
docs/harness/bin/run-sandbox-smoke.sh.
"""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

from sandbox_test_support import (  # noqa: E402
    ADAPTER_PATH, PROD_CATALOG, PROD_REGISTRY, TEST_IMAGE, SandboxEnv, StubDocker, default_sandbox, load_adapter,
)

SBX = load_adapter()
GOOD_ARGV = ["python3", "/tcb/tools/fake_writer.py", "benign"]


def run(env: SandboxEnv, stub: StubDocker, **extra):
    return SBX.run_isolated(env.manifest_path, env.worktree, env.run_dir,
                            **env.kwargs(docker_bin=str(stub.path), **extra))


class ModuleHygieneTests(unittest.TestCase):
    """A duplicated class name silently shadows the first definition and its tests never run."""

    def test_no_duplicate_test_case_classes_or_methods_in_the_sandbox_test_modules(self):
        import ast
        for name in ("test_sandbox_adapter.py", "test_sandbox_smoke.py"):
            tree = ast.parse((Path(__file__).resolve().parent / name).read_text(encoding="utf-8"))
            classes = [n.name for n in tree.body if isinstance(n, ast.ClassDef)]
            self.assertEqual(len(classes), len(set(classes)), f"{name}: duplicate class names {sorted(c for c in classes if classes.count(c) > 1)}")
            for node in (n for n in tree.body if isinstance(n, ast.ClassDef)):
                methods = [m.name for m in node.body if isinstance(m, ast.FunctionDef)]
                self.assertEqual(len(methods), len(set(methods)), f"{name}:{node.name} duplicate methods")


class RegistryTests(unittest.TestCase):
    def test_production_registry_loads_and_is_a_subset_of_the_catalog(self):
        registry = SBX.load_registry(PROD_REGISTRY, PROD_CATALOG)
        catalog = json.loads(PROD_CATALOG.read_text(encoding="utf-8"))["checks"]
        self.assertTrue(registry.checks)
        self.assertLessEqual(set(registry.checks), set(catalog))
        self.assertRegex(registry.sandbox.image, r"^[a-z0-9][a-z0-9._/-]*@sha256:[0-9a-f]{64}$")
        for check in registry.checks.values():
            self.assertIsInstance(check.argv, tuple)
            self.assertTrue(all(isinstance(a, str) for a in check.argv))

    def registry_error(self, mutate):
        env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        data = json.loads(env.registry_path.read_text(encoding="utf-8"))
        mutate(data)
        env.registry_path.write_text(json.dumps(data), encoding="utf-8")
        with self.assertRaises(SBX.RegistryError) as ctx:
            SBX.load_registry(env.registry_path, env.catalog_path)
        return str(ctx.exception)

    def test_shell_strings_and_shell_wrappers_are_not_registrable(self):
        bad = {
            "string": "python3 /tcb/tools/fake_writer.py benign",
            "empty": [],
            "bash -c": ["bash", "-c", "echo hi"],
            "sh -lc": ["/bin/sh", "-lc", "echo hi"],
            "bash --command": ["bash", "--command", "echo hi"],
            "python -c": ["python3", "-c", "print(1)"],
            "node -e": ["node", "-e", "1"],
            "eval": ["eval", "echo hi"],
            "sudo": ["sudo", "ls"],
            "docker": ["docker", "run", "x"],
            "python flag after the script": ["python3", "/tcb/tools/x.py", "-c", "print(1)"],
            "bash flag after the script": ["bash", "docs/x.sh", "-lc", "id"],
            "node -p": ["node", "-p", "1"],
            "node --eval": ["node", "--eval", "1"],
            "perl -E": ["perl", "-E", "say 1"],
            "perl -pe": ["perl", "-pe", "1"],
            "php -r": ["php", "-r", "echo 1;"],
            "timeout wrapper": ["timeout", "5", "bash", "x.sh"],
            "absolute-path timeout": ["/usr/bin/timeout", "5", "x"],
            "nice wrapper": ["nice", "bash", "x.sh"],
            "setsid wrapper": ["setsid", "x"],
            "stdbuf wrapper": ["stdbuf", "-o0", "x"],
            "env wrapper": ["env", "bash", "-c", "id"],
            "xargs wrapper": ["xargs", "x"],
            "awk program": ["awk", "BEGIN{system(\"id\")}"],
            "sed program": ["sed", "-e", "1e id"],
            "python attached -c": ["python3", "-cprint(1)"],
            "python attached -c quoted": ["python3", "-c'print(1)'"],
            "bash attached -c": ["bash", "-cid"],
            "perl attached -e": ["perl", "-eprint 1"],
            "perl attached -e quoted": ["perl", "-e'print 1'"],
            "ruby attached -e": ["ruby", "-eputs 1"],
            "ruby attached -e quoted": ["ruby", "-e'puts 1'"],
            "node attached -p": ["node", "-p1"],
            "node --eval=": ["node", "--eval=1"],
            "bash --command=": ["bash", "--command=id"],
            "non-string": ["python3", 7],
            "nul": ["python3", "a\0b"],
            "empty arg0": ["", "x"],
        }
        for label, argv in bad.items():
            with self.subTest(label):
                self.registry_error(lambda d, a=argv: d["checks"]["fake_ok"].__setitem__("argv", a))

    def test_unknown_catalog_id_unpinned_image_and_extra_keys_are_rejected(self):
        env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        data = json.loads(env.registry_path.read_text(encoding="utf-8"))
        data["checks"]["not_in_catalog"] = {"argv": GOOD_ARGV, "timeout_seconds": 5}
        env.registry_path.write_text(json.dumps(data), encoding="utf-8")
        with self.assertRaisesRegex(SBX.RegistryError, "catalog"):
            SBX.load_registry(env.registry_path, env.catalog_path)
        cases = {
            "tag only image": lambda d: d["sandbox"].__setitem__("image", "python:3.12-slim"),
            "latest tag with digest-like suffix": lambda d: d["sandbox"].__setitem__("image", "python:latest@sha256:xyz"),
            "short digest": lambda d: d["sandbox"].__setitem__("image", "python@sha256:abc"),
            "extra top-level key": lambda d: d.__setitem__("network", "bridge"),
            "extra sandbox key": lambda d: d["sandbox"].__setitem__("privileged", True),
            "extra check key": lambda d: d["checks"]["fake_ok"].__setitem__("env", {"A": "b"}),
            "wrong version": lambda d: d.__setitem__("registry_version", "check-registry-v9"),
            "bool timeout": lambda d: d["checks"]["fake_ok"].__setitem__("timeout_seconds", True),
            "zero timeout": lambda d: d["checks"]["fake_ok"].__setitem__("timeout_seconds", 0),
            "pids too high": lambda d: d["sandbox"].__setitem__("pids_limit", 100000),
            "memory missing": lambda d: d["sandbox"].pop("memory_mb"),
            "no checks": lambda d: d.__setitem__("checks", {}),
        }
        for label, mutate in cases.items():
            with self.subTest(label):
                self.registry_error(mutate)

    def test_duplicate_keys_and_garbage_are_rejected_strictly(self):
        env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        for text in ('{"registry_version": "a", "registry_version": "b"}', "not json", "[]", ""):
            with self.subTest(text[:20]):
                env.registry_path.write_text(text, encoding="utf-8")
                with self.assertRaises(SBX.RegistryError):
                    SBX.load_registry(env.registry_path, env.catalog_path)


class ArgvPolicyAllowsPlainScripts(unittest.TestCase):
    def test_script_invocations_without_inline_code_are_registrable(self):
        for argv in (["bash", "docs/harness/bin/pr-title-policy.sh", "--title", "fix: clean title"],
                     ["python3", "/tcb/tools/fake_writer.py", "benign"], ["/usr/bin/python3.12", "-B", "x.py"]):
            with self.subTest(argv):
                self.assertEqual(SBX.validate_argv("fake_ok", argv), tuple(argv))


def good_inspect(wt: Path, tcb: Path, argv=GOOD_ARGV, user="1000:1000"):
    """A `docker inspect` document in which every policy value is exactly as required."""
    mib = 1024 * 1024
    return {
        "Image": "sha256:" + "b" * 64,
        "HostConfig": {
            "NetworkMode": "none", "CapDrop": ["ALL"], "CapAdd": None, "SecurityOpt": ["no-new-privileges"],
            "ReadonlyRootfs": True, "Memory": 128 * mib, "MemorySwap": 128 * mib, "PidsLimit": 32, "NanoCpus": 10 ** 9,
            "Init": True, "Privileged": False, "PidMode": "", "IpcMode": "private", "UTSMode": "", "Devices": [],
            "RestartPolicy": {"Name": "no", "MaximumRetryCount": 0},
            "Tmpfs": {"/tmp": "rw,noexec,nosuid,nodev,size=16m"},
        },
        "Mounts": [{"Type": "bind", "Source": str(wt), "Destination": "/work", "RW": True},
                   {"Type": "bind", "Source": str(tcb), "Destination": "/tcb", "RW": False}],
        "Config": {"User": user, "Env": ["PATH=/usr/bin"] + [f"{k}={v}" for k, v in SBX.CONTAINER_ENV.items()],
                   "Entrypoint": [argv[0]], "Cmd": list(argv[1:])},
    }


class ReadBackTests(unittest.TestCase):
    """verify_container judges Docker's own read-back; crafted inspect documents prove each rule bites."""

    def setUp(self):
        import tempfile
        import shutil
        self.root = Path(tempfile.mkdtemp(prefix="h3-rb-")).resolve()
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.wt, self.tcb = self.root / "wt", self.root / "tcb"
        self.wt.mkdir()
        self.tcb.mkdir()
        self.sandbox = SBX.SandboxConfig(image=TEST_IMAGE, memory_mb=128, pids_limit=32, cpus=1.0, tmpfs_mb=16,
                                         log_cap_bytes=65536, max_total_timeout_seconds=120)

    def problems(self, info):
        described = SBX.describe_container(info, ["PATH"])
        return SBX.verify_container(described, self.sandbox, "1000:1000", self.wt, self.tcb, "sha256:" + "b" * 64, GOOD_ARGV)

    def test_a_fully_enforced_container_has_no_problems(self):
        self.assertEqual(self.problems(good_inspect(self.wt, self.tcb)), [])

    def test_tmpfs_size_reported_in_bytes_is_equivalent(self):
        info = good_inspect(self.wt, self.tcb)
        info["HostConfig"]["Tmpfs"]["/tmp"] = "rw,noexec,nosuid,nodev,size=16777216"
        self.assertEqual(self.problems(info), [])

    def test_each_policy_violation_is_reported(self):
        def host(**kw):
            return lambda i: i["HostConfig"].update(kw)

        cases = {
            "seccomp unconfined": (host(SecurityOpt=["no-new-privileges", "seccomp=unconfined"]), "security_opt"),
            "apparmor unconfined": (host(SecurityOpt=["no-new-privileges", "apparmor=unconfined"]), "security_opt"),
            "label disabled": (host(SecurityOpt=["no-new-privileges", "label=disable"]), "security_opt"),
            "no security opt": (host(SecurityOpt=[]), "security_opt"),
            "host pid namespace": (host(PidMode="host"), "pid_mode"),
            "container pid namespace": (host(PidMode="container:abc"), "pid_mode"),
            "host ipc namespace": (host(IpcMode="host"), "ipc_mode"),
            "host uts namespace": (host(UTSMode="host"), "uts_mode"),
            "device passthrough": (host(Devices=[{"PathOnHost": "/dev/kvm"}]), "devices"),
            "restart always": (host(RestartPolicy={"Name": "always"}), "restart_policy"),
            "privileged": (host(Privileged=True), "privileged"),
            "network bridge": (host(NetworkMode="bridge"), "network_mode"),
            "cap_add": (host(CapAdd=["SYS_ADMIN"]), "cap_add"),
            "tmpfs without noexec": (host(Tmpfs={"/tmp": "rw,nosuid,nodev,size=16m"}), "noexec"),
            "tmpfs without nosuid": (host(Tmpfs={"/tmp": "rw,noexec,nodev,size=16m"}), "nosuid"),
            "tmpfs without nodev": (host(Tmpfs={"/tmp": "rw,noexec,nosuid,size=16m"}), "nodev"),
            "tmpfs without size": (host(Tmpfs={"/tmp": "rw,noexec,nosuid,nodev"}), "size"),
            "tmpfs wrong size": (host(Tmpfs={"/tmp": "rw,noexec,nosuid,nodev,size=1g"}), "size"),
            "tmpfs missing": (host(Tmpfs={}), "tmpfs"),
            "entrypoint rewritten": (lambda i: i["Config"].update(Entrypoint=["/bin/sh"]), "entrypoint"),
            "cmd rewritten": (lambda i: i["Config"].update(Cmd=["-c", "id"]), "cmd"),
            "extra env": (lambda i: i["Config"]["Env"].append("AWS_SECRET_ACCESS_KEY=x"), "environment"),
            "tcb mounted rw": (lambda i: i["Mounts"][1].update(RW=True), "mounts"),
            "worktree source swapped": (lambda i: i["Mounts"][0].update(Source="/etc"), "worktree"),
            "extra mount": (lambda i: i["Mounts"].append({"Type": "bind", "Source": "/", "Destination": "/host", "RW": True}), "mounts"),
            "different image": (lambda i: i.update(Image="sha256:" + "c" * 64), "image_id"),
        }
        for label, (mutate, needle) in cases.items():
            with self.subTest(label):
                info = good_inspect(self.wt, self.tcb)
                mutate(info)
                found = self.problems(info)
                self.assertTrue(found, f"{label} was not detected")
                self.assertTrue(any(needle in item for item in found), found)

    def test_stub_level_unconfined_and_namespace_modes_refuse_to_start(self):
        stub = StubDocker(self)
        env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        for mode in ("seccomp_unconfined", "apparmor_unconfined", "pid_host", "device", "restart_always"):
            with self.subTest(mode):
                stub.configure(inspect_mode=mode)
                env.run_dir = env.root / "runs" / mode
                outcome = run(env, stub)
                self.assertEqual(outcome.status, "error", outcome.reason)
                self.assertEqual(outcome.checks[0]["limits_enforced"]["verified"], False)
        self.assertNotIn("start", stub.commands())


def docker_env(**variables):
    """Patch os.environ with DOCKER_HOST/DOCKER_CONTEXT removed unless given."""
    patcher = mock.patch.dict(os.environ, {k: v for k, v in variables.items() if v is not None})
    patcher.start()
    for key in ("DOCKER_HOST", "DOCKER_CONTEXT"):
        if variables.get(key) is None:
            os.environ.pop(key, None)
    return patcher


class EndpointTests(unittest.TestCase):
    """The engine endpoint is resolved once (DOCKER_HOST, DOCKER_CONTEXT, config) and must be a local socket."""

    def setUp(self):
        self.stub = StubDocker(self)
        self.env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})

    def go(self, **variables):
        patcher = docker_env(**variables)
        self.addCleanup(patcher.stop)
        return run(self.env, self.stub)

    def test_remote_contexts_and_config_selected_remote_engines_are_unavailable(self):
        cases = {
            "DOCKER_CONTEXT ssh": ({"DOCKER_CONTEXT": "remote"}, {"contexts": {"remote": "ssh://deploy@build-host"}}),
            "DOCKER_CONTEXT tcp": ({"DOCKER_CONTEXT": "remote"}, {"contexts": {"remote": "tcp://10.0.0.5:2376"}}),
            "config currentContext ssh": ({}, {"context_host": "ssh://deploy@build-host"}),
            "config currentContext tcp": ({}, {"context_host": "tcp://10.0.0.5:2375"}),
            "config currentContext npipe": ({}, {"context_host": "npipe:////./pipe/docker_engine"}),
            "DOCKER_HOST ssh": ({"DOCKER_HOST": "ssh://deploy@build-host"}, {}),
            "DOCKER_HOST tcp": ({"DOCKER_HOST": "tcp://10.0.0.5:2375"}, {}),
            "DOCKER_HOST relative": ({"DOCKER_HOST": "unix://relative.sock"}, {}),
        }
        for label, (variables, config) in cases.items():
            with self.subTest(label):
                self.stub.configure(**config)
                self.env.run_dir = self.env.root / "runs" / label.replace(" ", "_")
                outcome = self.go(**variables)
                self.assertEqual(outcome.status, "unavailable", outcome.reason)
                self.assertIn("local unix socket", outcome.reason)
                self.assertEqual(outcome.docker_endpoint, "")
        self.assertNotIn("create", self.stub.commands())
        self.assertNotIn("version", self.stub.commands())

    def test_endpoint_is_resolved_once_pinned_and_recorded(self):
        self.stub.configure(contexts={"local": "unix:///run/engram-test/docker.sock"})
        outcome = self.go(DOCKER_CONTEXT="local")
        self.assertEqual(outcome.status, "passed", outcome.reason)
        self.assertEqual(outcome.docker_endpoint, "unix:///run/engram-test/docker.sock")
        self.assertEqual(self.stub.commands().count("context"), 1)
        later = [c for c in self.stub.calls() if c["argv"][0] != "context"]
        self.assertTrue(later)
        for call in later:
            self.assertEqual(call["docker_host"], "unix:///run/engram-test/docker.sock")
            self.assertIsNone(call["docker_context"])
        self.assertEqual(json.loads((self.env.run_dir / "outcome.json").read_text(encoding="utf-8"))["docker_endpoint"],
                         "unix:///run/engram-test/docker.sock")

    def test_docker_host_wins_and_skips_context_resolution(self):
        self.stub.configure(contexts={"remote": "ssh://deploy@build-host"})
        outcome = self.go(DOCKER_HOST="unix:///run/engram-test/other.sock", DOCKER_CONTEXT="remote")
        self.assertEqual(outcome.status, "passed", outcome.reason)
        self.assertEqual(outcome.docker_endpoint, "unix:///run/engram-test/other.sock")
        self.assertNotIn("context", self.stub.commands())
        self.assertTrue(all(c["docker_context"] is None for c in self.stub.calls()))

    def test_unresolvable_context_is_unavailable(self):
        stub_bin = self.stub.path
        original = stub_bin.read_text(encoding="utf-8")
        stub_bin.write_text(original.replace('print(contexts.get(', 'fail("context not found"); print(contexts.get(', 1), encoding="utf-8")
        outcome = self.go()
        self.assertEqual(outcome.status, "unavailable", outcome.reason)
        self.assertIn("cannot resolve", outcome.reason)

    def test_a_docker_socket_inside_the_worktree_is_refused(self):
        self.stub.configure(context_host=f"unix://{self.env.worktree}/docker.sock")
        outcome = self.go()
        self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertIn("socket", outcome.reason)
        self.assertNotIn("create", self.stub.commands())


class RunDirAndStagingTests(unittest.TestCase):
    def setUp(self):
        self.stub = StubDocker(self)
        self.env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})

    def test_run_dir_must_be_private_to_the_current_user(self):
        self.env.run_dir.mkdir(parents=True)
        self.env.run_dir.chmod(0o777)
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertIn("owned by the current user", outcome.reason)
        self.env.run_dir.chmod(0o775)
        self.assertEqual(run(self.env, self.stub).status, "refused")
        self.env.run_dir.chmod(0o755)
        self.assertEqual(run(self.env, self.stub).status, "passed")
        self.assertEqual(self.env.run_dir.stat().st_mode & 0o777, 0o700)

    def test_run_dir_owned_by_someone_else_is_refused(self):
        self.env.run_dir.mkdir(parents=True)
        with mock.patch.object(SBX.os, "geteuid", return_value=os.geteuid() + 1):
            outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertEqual(self.stub.calls(), [])

    def test_run_dir_that_is_a_regular_file_is_refused(self):
        regular = self.env.root / "a-file"
        regular.write_text("x", encoding="utf-8")
        outcome = SBX.run_isolated(self.env.manifest_path, self.env.worktree, regular, **self.env.kwargs(docker_bin=str(self.stub.path)))
        self.assertEqual(outcome.status, "refused", outcome.reason)

    def test_tcb_staging_inside_the_worktree_is_refused(self):
        import tempfile
        with mock.patch.object(tempfile, "tempdir", str(self.env.worktree)):
            outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertIn("staging", outcome.reason)
        self.assertEqual(self.stub.calls(), [])
        self.assertEqual([p.name for p in self.env.worktree.iterdir()], ["README.txt"], "staging dir was not cleaned up")

    def test_worktree_that_is_the_home_directory_or_an_ancestor_of_it_is_refused(self):
        home = Path.home().resolve()
        for label, worktree in (("home", home), ("parent of home", home.parent)):
            with self.subTest(label):
                outcome = SBX.run_isolated(self.env.manifest_path, worktree, self.env.root / "runs" / label.replace(" ", "_"),
                                           **self.env.kwargs(docker_bin=str(self.stub.path)))
                self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertEqual(self.stub.calls(), [])


class EvidenceCaptureTests(unittest.TestCase):
    def test_a_log_file_that_cannot_be_written_makes_the_check_an_error_not_a_pass(self):
        stub = StubDocker(self, start={"stdout": "some output\n"})
        env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        real_open = os.open

        def refusing_open(path, *args, **kwargs):
            if str(path).endswith(".stdout.log"):
                raise PermissionError(13, "Permission denied")
            return real_open(path, *args, **kwargs)

        with mock.patch.object(SBX.os, "open", refusing_open):
            outcome = run(env, stub)
        self.assertEqual(outcome.status, "error", outcome.reason)
        self.assertIn("evidence capture failed", outcome.reason)
        self.assertEqual(stub.remaining_containers(), [])


class CreateArgsTests(unittest.TestCase):
    def build(self, **over):
        registry_sandbox = SBX.SandboxConfig(image=TEST_IMAGE, memory_mb=128, pids_limit=32, cpus=1.0, tmpfs_mb=16,
                                             log_cap_bytes=65536, max_total_timeout_seconds=120)
        kwargs = dict(name="engram-sbx-r1-0", run_id="r1", sandbox=registry_sandbox, argv=("python3", "/tcb/x.py", "--flag"),
                      worktree=Path("/wt"), tcb_dir=Path("/tcb-stage"), user="1000:1000")
        kwargs.update(over)
        return SBX.build_create_args(**kwargs)

    def test_every_hardening_flag_is_present(self):
        args = self.build()
        joined = " ".join(args)
        for needle in ("--pull never", "--network none", "--cap-drop ALL", "--security-opt no-new-privileges",
                       "--user 1000:1000", "--read-only", "--init", "--memory 128m", "--memory-swap 128m",
                       "--pids-limit 32", "--cpus 1", "--workdir /work", "--entrypoint python3"):
            self.assertIn(needle, joined)
        self.assertIn("/tmp:rw,noexec,nosuid,nodev,size=16m", joined)
        self.assertEqual(args[0], "create")

    def test_image_comes_last_before_the_command_and_argv_is_passed_verbatim(self):
        args = self.build()
        self.assertEqual(args[args.index(TEST_IMAGE) + 1:], ["/tcb/x.py", "--flag"])

    def test_exactly_two_mounts_worktree_rw_and_tcb_readonly(self):
        args = self.build()
        mounts = [args[i + 1] for i, a in enumerate(args) if a == "--mount"]
        self.assertEqual(len(mounts), 2)
        self.assertEqual(mounts[0], "type=bind,src=/wt,dst=/work")
        self.assertEqual(mounts[1], "type=bind,src=/tcb-stage,dst=/tcb,readonly")
        for forbidden in ("-v", "--volume", "--privileged", "--env-file", "--device", "--cap-add", "--network=host",
                          "--pid", "--userns", "--add-host", "--publish", "-p"):
            self.assertNotIn(forbidden, args)

    def test_only_allowlisted_env_is_passed_and_values_are_constants(self):
        args = self.build()
        envs = [args[i + 1] for i, a in enumerate(args) if a == "-e"]
        self.assertEqual(sorted(e.split("=", 1)[0] for e in envs), sorted(SBX.CONTAINER_ENV))
        self.assertTrue(all("=" in e for e in envs))
        self.assertIn("HOME=/tmp", envs)

    def test_mount_sources_that_could_smuggle_extra_mount_options_are_rejected(self):
        for bad in ("/wt,dst=/etc", "/wt\nx", '/w"t', "/w:t"):
            with self.subTest(bad):
                with self.assertRaises(SBX.Refusal):
                    self.build(worktree=Path(bad))


class ExecutionTests(unittest.TestCase):
    def setUp(self):
        self.stub = StubDocker(self)
        self.env = SandboxEnv(self, {"fake_ok": GOOD_ARGV, "fake_other": ["python3", "/tcb/tools/fake_writer.py", "exit_code", "3"]})
        self.env.write_manifest(["fake_ok"])

    # ---- success path and outcome contract -----------------------------------------------
    def test_success_outcome_contract_and_supervisor_owned_evidence(self):
        self.stub.configure(start={"stdout": "hello\n", "stderr": "warn\n", "exit_code": 0})
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "passed", outcome.reason)
        self.assertEqual(outcome.exit_code, 0)
        self.assertEqual(outcome.argv, (tuple(GOOD_ARGV),))
        self.assertRegex(outcome.started_at, r"^\d{4}-\d\d-\d\dT.*Z$")
        self.assertRegex(outcome.finished_at, r"^\d{4}-\d\d-\d\dT.*Z$")
        self.assertLessEqual(outcome.started_at, outcome.finished_at)
        self.assertEqual(len(outcome.log_paths), 2)
        for path in outcome.log_paths:
            self.assertTrue(Path(path).is_file())
            self.assertTrue(Path(path).resolve().is_relative_to(self.env.run_dir.resolve()))
            self.assertFalse(Path(path).resolve().is_relative_to(self.env.worktree.resolve()))
        self.assertEqual(Path(outcome.log_paths[0]).read_text(encoding="utf-8"), "hello\n")
        persisted = json.loads((self.env.run_dir / "outcome.json").read_text(encoding="utf-8"))
        self.assertEqual(persisted, outcome.to_dict())
        for key in ("status", "exit_code", "argv", "started_at", "finished_at", "log_paths", "limits_enforced"):
            self.assertIn(key, persisted)
        self.assertIs(persisted["executed_on_host"], False)
        self.assertEqual(persisted["manifest_target_sha"], "0123456789abcdef0123456789abcdef01234567")
        self.assertNotIn("target_sha", persisted)
        self.assertIs(persisted["target_sha_bound"], False)
        self.assertTrue(persisted["docker_endpoint"].startswith("unix:///"))
        self.assertEqual(persisted["checks"][0]["limits_enforced"]["network_mode"], "none")
        self.assertTrue(persisted["checks"][0]["limits_enforced"]["verified"])
        self.assertEqual(persisted["checks"][0]["id"], "fake_ok")
        self.assertEqual(len(persisted["checks"][0]["log_sha256"]["stdout"]), 64)
        self.assertEqual(self.stub.remaining_containers(), [])

    def test_limits_enforced_is_read_back_from_docker_inspect(self):
        outcome = run(self.env, self.stub)
        limits = outcome.limits_enforced
        self.assertEqual(limits["network_mode"], "none")
        self.assertEqual(limits["cap_drop"], ["ALL"])
        self.assertTrue(limits["no_new_privileges"])
        self.assertTrue(limits["read_only_rootfs"])
        self.assertEqual(limits["memory_bytes"], 128 * 1024 * 1024)
        self.assertEqual(limits["memory_swap_bytes"], 128 * 1024 * 1024)
        self.assertEqual(limits["pids_limit"], 32)
        self.assertEqual(limits["nano_cpus"], 1_000_000_000)
        self.assertFalse(limits["privileged"])
        self.assertNotIn("0", limits["user"].split(":")[:1])
        self.assertEqual(sorted((m["destination"], m["rw"]) for m in limits["mounts"]), [("/tcb", False), ("/work", True)])
        self.assertEqual(outcome.supervisor_limits["log_cap_bytes"], 65536)

    def test_outcome_file_is_never_overwritten(self):
        self.assertEqual(run(self.env, self.stub).status, "passed")
        before = (self.env.run_dir / "outcome.json").read_bytes()
        again = run(self.env, self.stub)
        self.assertEqual(again.status, "refused")
        self.assertEqual((self.env.run_dir / "outcome.json").read_bytes(), before)

    # ---- refusals: nothing may reach docker ----------------------------------------------
    def assert_refused_without_docker(self, outcome, needle):
        self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertIn(needle, outcome.reason)
        self.assertNotIn("create", self.stub.commands())
        self.assertNotIn("start", self.stub.commands())
        self.assertIsNone(outcome.exit_code)

    def test_unknown_check_id_is_refused_before_any_docker_call(self):
        self.env.write_manifest(["fake_ok", "no_such_check"])
        outcome = run(self.env, self.stub)
        self.assert_refused_without_docker(outcome, "no_such_check")
        self.assertEqual(self.stub.calls(), [])

    def test_id_in_catalog_but_not_in_registry_is_refused(self):
        catalog = json.loads(self.env.catalog_path.read_text(encoding="utf-8"))
        catalog["checks"]["cataloged_only"] = {"description": "no argv registered"}
        self.env.catalog_path.write_text(json.dumps(catalog), encoding="utf-8")
        self.env.write_manifest(["cataloged_only"])
        self.assert_refused_without_docker(run(self.env, self.stub), "cataloged_only")

    def test_manifest_must_be_a_valid_task_v2_with_isolation_capabilities(self):
        cases = {
            "missing network_none": {"allowed_capabilities": ["workspace_read", "sandbox_container"]},
            "missing sandbox_container": {"allowed_capabilities": ["workspace_read", "network_none"]},
            "host exec capability": {"allowed_capabilities": ["workspace_read", "sandbox_container", "network_none", "host_exec"]},
            "wrong policy": {"policy_version": "some-other-policy"},
            "wrong schema": {"schema_version": "task-v1"},
            "extra key": {"command": "rm -rf /"},
        }
        for label, over in cases.items():
            with self.subTest(label):
                self.env.run_dir = self.env.root / "runs" / label.replace(" ", "_")
                self.env.write_manifest(["fake_ok"], over=over)
                outcome = run(self.env, self.stub)
                self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertEqual(self.stub.calls(), [])

    def test_malformed_manifest_is_refused(self):
        for text in ("not json", "[]", '{"a":1,"a":2}'):
            with self.subTest(text):
                self.env.run_dir = self.env.root / "runs" / f"m{abs(hash(text))}"
                self.env.manifest_path.write_text(text, encoding="utf-8")
                self.assertEqual(run(self.env, self.stub).status, "refused")
        self.assertEqual(self.stub.calls(), [])

    def test_path_layout_violations_are_refused(self):
        wt = self.env.worktree
        cases = {
            "run_dir inside worktree": (wt, wt / "evidence"),
            "worktree inside run_dir": (self.env.root / "runs" / "outer" / "wt", self.env.root / "runs" / "outer"),
            "worktree missing": (self.env.root / "does-not-exist", self.env.root / "runs" / "x"),
            "worktree is a file": (self.env.root / "README-file", self.env.root / "runs" / "y"),
            "worktree is filesystem root": (Path("/"), self.env.root / "runs" / "z"),
            "worktree path with comma": (self.env.root / "a,b", self.env.root / "runs" / "c"),
        }
        (self.env.root / "README-file").write_text("x", encoding="utf-8")
        (self.env.root / "a,b").mkdir()
        (self.env.root / "runs" / "outer" / "wt").mkdir(parents=True)
        for label, (worktree, run_dir) in cases.items():
            with self.subTest(label):
                outcome = SBX.run_isolated(self.env.manifest_path, worktree, run_dir,
                                           **self.env.kwargs(docker_bin=str(self.stub.path)))
                self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertEqual(self.stub.calls(), [])

    def test_the_writer_writable_mount_must_not_contain_the_tcb(self):
        # Manifest inside the worktree, and a worktree that holds the live adapter, both let the writer edit the TCB.
        inside = self.env.worktree / "manifest.json"
        inside.write_bytes(self.env.manifest_path.read_bytes())
        outcome = SBX.run_isolated(inside, self.env.worktree, self.env.root / "runs" / "a",
                                   **self.env.kwargs(docker_bin=str(self.stub.path)))
        self.assertEqual(outcome.status, "refused", outcome.reason)
        self.assertIn("TCB", outcome.reason)
        live = SBX.run_isolated(self.env.manifest_path, ADAPTER_PATH.parents[3], self.env.root / "runs" / "b",
                                **self.env.kwargs(docker_bin=str(self.stub.path)))
        self.assertEqual(live.status, "refused", live.reason)
        self.assertIn("adapter", live.reason)
        self.assertEqual(self.stub.calls(), [])

    def test_symlinked_run_dir_pointing_into_the_worktree_is_refused(self):
        link = self.env.root / "link-to-worktree"
        link.symlink_to(self.env.worktree)
        outcome = SBX.run_isolated(self.env.manifest_path, self.env.worktree, link / "evidence",
                                   **self.env.kwargs(docker_bin=str(self.stub.path)))
        self.assertEqual(outcome.status, "refused", outcome.reason)

    # ---- unavailable: never fall back to the host -----------------------------------------
    def test_missing_docker_binary_is_unavailable_not_pass_and_never_runs_on_host(self):
        spawned = []
        real_popen = subprocess.Popen

        def spy(argv, *a, **k):
            spawned.append(list(argv))
            return real_popen(argv, *a, **k)

        with mock.patch.object(SBX.subprocess, "Popen", spy), mock.patch.object(SBX.subprocess, "run", side_effect=AssertionError("host exec")):
            outcome = SBX.run_isolated(self.env.manifest_path, self.env.worktree, self.env.run_dir,
                                       **self.env.kwargs(docker_bin=str(self.env.root / "no-such-docker")))
        self.assertEqual(outcome.status, "unavailable", outcome.reason)
        self.assertNotEqual(outcome.status, "passed")
        self.assertIsNone(outcome.exit_code)
        self.assertEqual(spawned, [])
        persisted = json.loads((self.env.run_dir / "outcome.json").read_text(encoding="utf-8"))
        self.assertEqual(persisted["status"], "unavailable")
        self.assertEqual(persisted["argv"], [])

    def test_daemon_unreachable_is_unavailable(self):
        self.stub.configure(daemon_down=True)
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "unavailable")
        self.assertNotIn("create", self.stub.commands())

    def test_image_absent_or_digest_mismatch_is_unavailable_and_nothing_is_pulled(self):
        for config in ({"image_absent": True}, {"repo_digests": ["python@sha256:" + "f" * 64]}):
            with self.subTest(config):
                self.stub.configure(**config)
                self.env.run_dir = self.env.root / "runs" / f"img{len(config)}{list(config)[0]}"
                outcome = run(self.env, self.stub)
                self.assertEqual(outcome.status, "unavailable", outcome.reason)
        self.assertNotIn("pull", self.stub.commands())
        self.assertNotIn("create", self.stub.commands())

    def test_remote_docker_engine_is_refused(self):
        with mock.patch.dict(os.environ, {"DOCKER_HOST": "tcp://10.0.0.1:2375"}):
            outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "unavailable", outcome.reason)
        self.assertEqual(self.stub.calls(), [])

    # ---- truthful limits ------------------------------------------------------------------
    def test_limit_not_actually_enforced_by_docker_refuses_to_start(self):
        for mode in ("ignore_memory", "net_bridge", "no_nnp", "privileged", "extra_mount"):
            with self.subTest(mode):
                self.stub.configure(inspect_mode=mode)
                self.env.run_dir = self.env.root / "runs" / mode
                outcome = run(self.env, self.stub)
                self.assertEqual(outcome.status, "error", outcome.reason)
                self.assertIn("limit", outcome.reason)
                self.assertNotIn("start", self.stub.commands())
                self.assertEqual(outcome.limits_enforced.get("verified"), False)
        self.assertEqual(self.stub.remaining_containers(), [])

    # ---- process results ------------------------------------------------------------------
    def test_nonzero_exit_is_failed_and_stops_before_later_checks(self):
        self.stub.configure(start={"exit_code": 3})
        self.env.write_manifest(["fake_other", "fake_ok"])
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "failed")
        self.assertEqual(outcome.exit_code, 3)
        self.assertEqual(len(outcome.checks), 1)
        self.assertEqual(self.stub.commands().count("start"), 1)

    def test_oom_kill_is_reported(self):
        self.stub.configure(start={"exit_code": 137, "oom": True})
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "failed")
        self.assertTrue(outcome.checks[0]["oom_killed"])

    def test_output_is_capped_and_marked_truncated(self):
        self.stub.configure(start={"stdout_bytes": 512 * 1024})
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "passed")
        self.assertLessEqual(Path(outcome.log_paths[0]).stat().st_size, 65536)
        self.assertTrue(outcome.checks[0]["truncated"]["stdout"])

    def test_timeout_kills_removes_and_verifies_no_survivor(self):
        self.stub.configure(start={"sleep": 30})
        self.env.write_manifest(["fake_ok"], timeout=2)
        began = time.monotonic()
        outcome = run(self.env, self.stub)
        self.assertLess(time.monotonic() - began, 20)
        self.assertEqual(outcome.status, "timeout", outcome.reason)
        self.assertIn("kill", self.stub.commands())
        self.assertIn("rm", self.stub.commands())
        self.assertEqual(self.stub.remaining_containers(), [])
        self.assertTrue(outcome.checks[0]["container_removed"])
        self.assertEqual(outcome.exit_code, 137)

    def test_container_that_survives_removal_is_an_error_never_a_pass(self):
        self.stub.configure(rm_fail=True, start={"exit_code": 0})
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "error", outcome.reason)
        self.assertIn("survive", outcome.reason)

    def test_total_manifest_timeout_bounds_the_whole_run(self):
        # Each check alone is far inside its own 30 s limit; only the 3 s manifest budget can stop the second.
        self.stub.configure(start={"sleep": 2})
        self.env.write_manifest(["fake_ok", "fake_other"], timeout=3)
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "timeout", outcome.reason)
        self.assertEqual(len(outcome.checks), 1)
        self.assertEqual(outcome.checks[0]["status"], "passed")
        self.assertEqual(self.stub.commands().count("start"), 1)
        self.assertIn("manifest timeout of 3s", outcome.reason)
        self.assertIn("fake_other", outcome.reason)
        self.assertEqual(outcome.supervisor_limits["total_timeout_seconds"], 3)

    # ---- host isolation of the supervisor itself ------------------------------------------
    def test_only_the_docker_cli_is_ever_spawned_and_secrets_are_not_forwarded(self):
        canaries = {"SSH_AUTH_SOCK": "/tmp/ssh-canary", "GIT_ASKPASS": "/tmp/askpass-canary",
                    "GITHUB_TOKEN": "ghp_canary_h3", "AWS_SECRET_ACCESS_KEY": "aws-canary-h3", "ANTHROPIC_API_KEY": "sk-canary-h3"}
        spawned = []
        real_popen, real_run = subprocess.Popen, subprocess.run

        def popen_spy(argv, *a, **k):
            spawned.append(list(argv))
            return real_popen(argv, *a, **k)

        def run_spy(argv, *a, **k):
            spawned.append(list(argv))
            return real_run(argv, *a, **k)

        with mock.patch.dict(os.environ, canaries), mock.patch.object(SBX.subprocess, "Popen", popen_spy), \
                mock.patch.object(SBX.subprocess, "run", run_spy):
            outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "passed", outcome.reason)
        self.assertTrue(spawned)
        self.assertTrue(all(argv[0] == str(self.stub.path) for argv in spawned), spawned)
        self.assertFalse(any(a in ("python3", "bash", "sh") for argv in spawned for a in argv[:1]))
        flat = json.dumps(self.stub.calls())
        for name, value in canaries.items():
            self.assertNotIn(value, flat)
            for call in self.stub.calls():
                self.assertNotIn(name, call["env_names"])

    def test_adapter_source_has_no_shell_or_host_exec_constructs(self):
        for path in (ADAPTER_PATH, ADAPTER_PATH.parent / "sandbox_registry.py"):
            source = path.read_text(encoding="utf-8")
            for forbidden in ("shell=True", "os.system", "os.popen", "os.exec", "os.spawn", "pty."):
                self.assertNotIn(forbidden, source, path.name)
        self.assertNotIn("subprocess", (ADAPTER_PATH.parent / "sandbox_registry.py").read_text(encoding="utf-8"),
                         "the registry module is pure policy and must not spawn anything")

    def test_staged_tcb_is_a_snapshot_so_later_manifest_edits_do_not_matter(self):
        # The manifest is copied and validated once; changing the file afterwards cannot alter the run.
        self.stub.configure(start={"stdout": "x"})
        before = self.env.manifest_path.read_bytes()
        outcome = run(self.env, self.stub)
        self.assertEqual(outcome.status, "passed")
        self.assertEqual(self.env.manifest_path.read_bytes(), before)
        self.assertEqual(outcome.manifest_sha256, __import__("hashlib").sha256(before).hexdigest())


class CliTests(unittest.TestCase):
    def cli(self, env, stub, *extra, environ=None):
        return subprocess.run(
            [sys.executable, str(ADAPTER_PATH), "run", "--manifest", str(env.manifest_path), "--worktree", str(env.worktree),
             "--run-dir", str(env.run_dir), "--registry", str(env.registry_path), "--catalog", str(env.catalog_path),
             "--docker", str(stub.path), *extra],
            capture_output=True, text=True, timeout=60, env={**os.environ, **(environ or {})})

    def test_exit_codes_distinguish_pass_refusal_and_unavailable(self):
        stub = StubDocker(self)
        env = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        proc = self.cli(env, stub)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertEqual(json.loads(proc.stdout)["status"], "passed")

        env2 = SandboxEnv(self, {"fake_ok": GOOD_ARGV}, check_ids=["fake_ok", "ghost"])
        proc = self.cli(env2, stub)
        self.assertEqual(proc.returncode, 2, proc.stderr)
        self.assertEqual(json.loads(proc.stdout)["status"], "refused")

        stub.configure(daemon_down=True)
        env3 = SandboxEnv(self, {"fake_ok": GOOD_ARGV})
        proc = self.cli(env3, stub)
        self.assertEqual(proc.returncode, 3, proc.stderr)
        self.assertEqual(json.loads(proc.stdout)["status"], "unavailable")

    def test_smoke_runner_reports_unavailable_distinctly_when_docker_is_missing(self):
        # PATH holds only python3 and dirname: no docker anywhere, so nothing may run and the state is
        # UNAVAILABLE (exit 3), which is neither a pass nor a generic failure.
        import shutil
        import tempfile
        bin_dir = Path(tempfile.mkdtemp(prefix="h3-nodocker-"))
        self.addCleanup(shutil.rmtree, bin_dir, ignore_errors=True)
        (bin_dir / "python3").symlink_to(sys.executable)
        (bin_dir / "dirname").symlink_to(shutil.which("dirname"))
        runner = ADAPTER_PATH.parent / "run-sandbox-smoke.sh"
        proc = subprocess.run([shutil.which("bash"), str(runner)], capture_output=True, text=True, timeout=120,
                              env={"PATH": str(bin_dir), "HOME": str(bin_dir)})
        self.assertEqual(proc.returncode, 3, proc.stdout + proc.stderr)
        self.assertIn("SANDBOX_SMOKE: UNAVAILABLE", proc.stdout)
        self.assertNotIn("SANDBOX_SMOKE: PASS", proc.stdout)

    def test_offline_lane_does_not_depend_on_docker(self):
        lane = (ADAPTER_PATH.parent / "run-offline-lane.sh").read_text(encoding="utf-8")
        active = "\n".join(line for line in lane.splitlines() if not line.lstrip().startswith("#"))
        for token in ("docker", "run-sandbox-smoke", "test_sandbox_smoke"):
            self.assertNotIn(token, active)
        self.assertIn("test_sandbox_adapter.py", active)

    def test_validate_registry_subcommand_checks_the_production_registry(self):
        proc = subprocess.run([sys.executable, str(ADAPTER_PATH), "validate-registry"], capture_output=True, text=True, timeout=60)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("REGISTRY: OK", proc.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)
