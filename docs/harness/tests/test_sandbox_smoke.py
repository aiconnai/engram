#!/usr/bin/env python3
"""Real-Docker smoke test of the sandbox boundary with the FAKE writer (task H3).

NOT part of the mandatory offline lane: it needs a Docker daemon and the pinned image. Run it through
docs/harness/bin/run-sandbox-smoke.sh, which reports a distinct UNAVAILABLE state (exit 3, never a
pass) when Docker or the image is missing. Run directly, an unavailable environment makes the whole
module SKIP, which is likewise not a pass.

Every escape attempt of docs/harness/tests/fake_writer.py runs INSIDE the container; each test asserts
from the host side that it was refused or contained: writes outside the worktree, TCB/manifest
tampering, network egress, host credentials, fork bombs, outliving the timeout, surviving children,
forged evidence in run_dir, memory exhaustion and log flooding.

Usage:  python3 docs/harness/tests/test_sandbox_smoke.py --preflight   # exit 0 ready, 3 unavailable
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

from sandbox_test_support import (  # noqa: E402
    FAKE_WRITER_PATH, PROD_CATALOG, PROD_REGISTRY, REPO_ROOT, SandboxEnv, default_sandbox, load_adapter, task_manifest,
)

SBX = load_adapter()
REAL_IMAGE = json.loads(PROD_REGISTRY.read_text(encoding="utf-8"))["sandbox"]["image"]
TOOLS = {"fake_writer.py": FAKE_WRITER_PATH}


def preflight():
    """(ready, reason). Ready means the docker CLI, the daemon and the pinned image are all present."""
    try:
        cli = SBX.connect_docker("docker")
        SBX.check_image(cli, REAL_IMAGE)
    except SBX.Unavailable as exc:
        return False, str(exc)
    return True, "ok"


def setUpModule():
    ready, reason = preflight()
    if not ready:
        print(f"SANDBOX_SMOKE_STATE=UNAVAILABLE reason={reason}", file=sys.stderr)
        raise unittest.SkipTest(f"UNAVAILABLE (not a pass): {reason}")


def attempts(outcome):
    """Parse the fake writer's JSON lines from the supervisor-captured stdout log."""
    found = {}
    for line in Path(outcome.log_paths[0]).read_text(encoding="utf-8").splitlines():
        try:
            record = json.loads(line)
        except ValueError:
            continue
        found[record["attempt"]] = record
    return found


def containers_left(run_id):
    proc = subprocess.run(["docker", "ps", "-aq", "--filter", f"label={SBX.RUN_LABEL}={run_id}"],
                          capture_output=True, text=True, timeout=30, check=False)
    return proc.stdout.split()


class SmokeBase(unittest.TestCase):
    def sandbox_run(self, args, timeout=60, check_timeout=30, **sandbox_over):
        """Run `fake_writer.py <args>` as an approved check; returns (outcome, env)."""
        env = SandboxEnv(self, {"fake_check": {"argv": ["python3", "/tcb/tools/fake_writer.py", *map(str, args)],
                                                "timeout_seconds": check_timeout}},
                         sandbox=default_sandbox(image=REAL_IMAGE, **sandbox_over), timeout=timeout)
        outcome = SBX.run_isolated(env.manifest_path, env.worktree, env.run_dir, **env.kwargs(tcb_files=TOOLS))
        self.assertTrue(outcome.checks, f"nothing ran: {outcome.status} {outcome.reason}")
        self.assertEqual(containers_left(outcome.run_id), [], "a container of the run survived")
        return outcome, env

    def assert_refused(self, found, names):
        for name in names:
            self.assertIn(name, found, f"attempt {name} missing from fake writer output")
            self.assertEqual(found[name]["outcome"], "refused", f"{name} was not refused: {found[name]}")


class BoundaryTests(SmokeBase):
    def test_benign_control_is_non_root_writes_only_the_worktree_and_limits_are_read_back(self):
        outcome, env = self.sandbox_run(["benign"])
        self.assertEqual(outcome.status, "passed", outcome.reason)
        found = attempts(outcome)
        self.assertEqual(found["write_worktree"]["outcome"], "succeeded")
        self.assertEqual(found["write_tmp"]["outcome"], "succeeded")
        self.assertNotEqual(found["identity"]["uid"], 0)
        self.assertEqual((env.worktree / "benign.txt").read_text(encoding="utf-8"), "ok")
        limits = outcome.limits_enforced
        self.assertTrue(limits["verified"], limits.get("mismatches"))
        self.assertEqual((limits["network_mode"], limits["cap_drop"], limits["memory_bytes"], limits["pids_limit"]),
                         ("none", ["ALL"], 128 * 1024 * 1024, 32))
        self.assertEqual([m for m in outcome.argv], [("python3", "/tcb/tools/fake_writer.py", "benign")])
        recorded = json.loads((env.run_dir / "outcome.json").read_text(encoding="utf-8"))
        self.assertEqual(recorded["status"], "passed")
        self.assertTrue(recorded["docker_endpoint"].startswith("unix:///"))
        self.assertEqual(recorded["checks"][0]["limits_enforced"]["network_mode"], "none")

    def test_writes_outside_the_worktree_are_refused_and_nothing_leaks_to_the_host(self):
        outcome, env = self.sandbox_run(["write_outside"])
        self.assertEqual(outcome.status, "passed", outcome.reason)
        found = attempts(outcome)
        self.assertEqual(found["write_worktree_control"]["outcome"], "succeeded")
        self.assert_refused(found, ["write:/etc/engram-escape", "write:/usr/local/engram-escape", "write:/engram-escape",
                                    "write:/work/../engram-escape", "write:/var/engram-escape", "write:/var/tmp/engram-escape", "write:/root/engram-escape",
                                    "write:/proc/sys/kernel/hostname", "write:/sys/kernel/engram",
                                    "write_via_symlink_to_etc", "chmod_system_file", "unlink_system_file"])
        self.assertFalse((env.root / "engram-escape").exists())
        self.assertFalse(Path("/etc/engram-escape").exists())

    def test_privilege_escalation_is_impossible(self):
        outcome, _ = self.sandbox_run(["privilege"])
        found = attempts(outcome)
        self.assertEqual(found["proc_status"]["CapEff"].strip("0"), "")
        self.assertEqual(found["proc_status"]["CapBnd"].strip("0"), "")
        self.assertEqual(found["proc_status"]["NoNewPrivs"], "1")
        self.assertNotEqual(found["identity"]["uid"], 0)
        self.assert_refused(found, ["setuid_0", "mknod_dev", "chroot", "mount_tmpfs", "exec_from_tmpfs"])

    def test_tcb_and_manifest_are_read_only_and_host_copies_are_untouched(self):
        env_probe = SandboxEnv(self, {"fake_check": ["python3", "/tcb/tools/fake_writer.py", "tamper_tcb"]},
                               sandbox=default_sandbox(image=REAL_IMAGE))
        before = {p: hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in (env_probe.manifest_path, env_probe.registry_path, env_probe.catalog_path, FAKE_WRITER_PATH)}
        outcome = SBX.run_isolated(env_probe.manifest_path, env_probe.worktree, env_probe.run_dir,
                                   **env_probe.kwargs(tcb_files=TOOLS))
        self.assertEqual(outcome.status, "passed", outcome.reason)
        found = attempts(outcome)
        self.assertEqual(found["read_manifest_control"]["outcome"], "succeeded")
        self.assert_refused(found, ["write_manifest", "append_registry", "unlink_manifest", "rename_manifest",
                                    "chmod_manifest", "create_in_tcb", "write_tool", "truncate_manifest"])
        after = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in before}
        self.assertEqual(before, after)
        self.assertEqual(limits_mount(outcome, "/tcb"), False)

    def test_network_egress_is_impossible(self):
        outcome, _ = self.sandbox_run(["network"], check_timeout=60)
        self.assertEqual(outcome.status, "passed", outcome.reason)
        found = attempts(outcome)
        self.assert_refused(found, ["tcp_connect_external", "udp_dns_external", "resolve_hostname", "connect_docker_socket_tcp"])
        self.assertEqual(found["interfaces"]["interfaces"], ["lo"])
        self.assertFalse(found["docker_sock_present"]["present"])

    def test_host_credentials_home_and_ssh_are_not_visible(self):
        canary = "H3-CANARY-7f3a9c"
        ssh_dir = str(Path.home() / ".ssh")
        leaks = {"SSH_AUTH_SOCK": f"/tmp/{canary}", "GIT_ASKPASS": f"/tmp/{canary}-askpass", "GITHUB_TOKEN": canary,
                 "AWS_SECRET_ACCESS_KEY": canary, "ANTHROPIC_API_KEY": canary, "OPENAI_API_KEY": canary}
        with mock.patch.dict(os.environ, leaks):
            outcome, _ = self.sandbox_run(["read_host_secrets", canary, ssh_dir, str(Path.home())])
        self.assertEqual(outcome.status, "passed", outcome.reason)
        found = attempts(outcome)
        self.assertEqual(found["credential_env"]["present"], [])
        self.assertFalse(found["canary_in_env"]["leaked"])
        self.assertEqual(found["home"]["home"], "/tmp")
        self.assertFalse(found[f"host_path_visible:{ssh_dir}"]["visible"])
        self.assertFalse(found[f"host_path_visible:{Path.home()}"]["visible"])
        # inside the container "~" is HOME=/tmp, so the writer probes /tmp/.ssh; each probe must exist and be empty-handed
        for path in ("/tmp/.ssh", "/root/.ssh", "/var/run/docker.sock", "/run/secrets"):
            self.assertIn(f"path_visible:{path}", found, f"probe for {path} missing from the fake writer output")
            self.assertFalse(found[f"path_visible:{path}"]["visible"], path)
        image = subprocess.run(["docker", "image", "inspect", REAL_IMAGE, "--format", "{{json .Config.Env}}"],
                               capture_output=True, text=True, timeout=30, check=True)
        allowed = {e.split("=", 1)[0] for e in json.loads(image.stdout)} | set(SBX.CONTAINER_ENV) | {"HOSTNAME"}
        self.assertLessEqual(set(found["env_names"]["names"]), allowed)

    def test_fork_bomb_is_stopped_by_the_pids_cap_and_leaves_no_process(self):
        outcome, _ = self.sandbox_run(["forkbomb", 300], pids_limit=32)
        self.assertEqual(outcome.status, "passed", outcome.reason)
        record = attempts(outcome)["fork_until_refused"]
        self.assertEqual(record["outcome"], "refused")
        self.assertEqual(record["errno"], "EAGAIN")
        self.assertLess(record["forked"], 32)
        self.assertGreater(record["forked"], 0)

    def test_process_that_outlives_the_timeout_is_killed_and_the_container_removed(self):
        began = time.monotonic()
        outcome, _ = self.sandbox_run(["outlive_timeout"], timeout=3, check_timeout=3)
        self.assertLess(time.monotonic() - began, 40)
        self.assertEqual(outcome.status, "timeout", outcome.reason)
        self.assertTrue(outcome.checks[0]["container_removed"])
        self.assertEqual(outcome.exit_code, 137)

    def test_a_daemonized_child_does_not_survive_the_run(self):
        outcome, env = self.sandbox_run(["daemon_child"])
        self.assertEqual(outcome.status, "passed", outcome.reason)
        heartbeat = env.worktree / "heartbeat"
        self.assertTrue(heartbeat.exists(), "child never ran, so the survivor check proves nothing")
        first = heartbeat.read_text(encoding="utf-8")
        time.sleep(1.5)
        self.assertEqual(heartbeat.read_text(encoding="utf-8"), first, "a child kept running after the container was removed")

    def test_forged_evidence_cannot_reach_run_dir_and_the_supervisor_record_wins(self):
        env = SandboxEnv(self, {"fake_check": ["python3", "/tcb/tools/fake_writer.py", "tamper_run_dir", "PLACEHOLDER"]},
                         sandbox=default_sandbox(image=REAL_IMAGE))
        registry = json.loads(env.registry_path.read_text(encoding="utf-8"))
        registry["checks"]["fake_check"]["argv"][-1] = str(env.run_dir)
        env.registry_path.write_text(json.dumps(registry), encoding="utf-8")
        outcome = SBX.run_isolated(env.manifest_path, env.worktree, env.run_dir, **env.kwargs(tcb_files=TOOLS))
        self.assertEqual(outcome.status, "passed", outcome.reason)
        found = attempts(outcome)
        self.assertEqual(found["forge_outcome_in_worktree"]["outcome"], "succeeded")
        self.assert_refused(found, ["write_host_run_dir_outcome", "write_host_run_dir_log", "mkdir_host_run_dir"])
        self.assertFalse(found["host_run_dir_visible"]["visible"])
        self.assertIn("forged", (env.worktree / "outcome.json").read_text(encoding="utf-8"))
        recorded = json.loads((env.run_dir / "outcome.json").read_text(encoding="utf-8"))
        self.assertNotIn("forged", recorded)
        self.assertEqual(sorted(p.name for p in env.run_dir.iterdir()), ["logs", "outcome.json"])
        self.assertFalse(any("forged" in p.name for p in (env.run_dir / "logs").iterdir()))
        for label, path in zip(("stdout", "stderr"), outcome.log_paths):
            self.assertEqual(hashlib.sha256(Path(path).read_bytes()).hexdigest(), outcome.checks[0]["log_sha256"][label])

    def test_memory_cap_kills_a_memory_bomb(self):
        outcome, _ = self.sandbox_run(["memory_bomb", 600], memory_mb=128)
        self.assertEqual(outcome.status, "failed", outcome.reason)
        self.assertEqual(outcome.exit_code, 137)
        self.assertTrue(outcome.checks[0]["oom_killed"])

    def test_nonzero_exit_is_a_failure_and_log_output_is_capped(self):
        outcome, _ = self.sandbox_run(["exit_code", 7])
        self.assertEqual((outcome.status, outcome.exit_code), ("failed", 7))
        flood, _ = self.sandbox_run(["spam_output", 400 * 1024], log_cap_bytes=65536)
        self.assertEqual(flood.status, "passed")
        self.assertLessEqual(Path(flood.log_paths[0]).stat().st_size, 65536)
        self.assertTrue(flood.checks[0]["truncated"]["stdout"])


class ProductionRegistryAndAvailabilityTests(SmokeBase):
    def test_a_real_registry_check_runs_in_the_sandbox(self):
        root = Path(tempfile.mkdtemp(prefix="h3-prod-")).resolve()
        self.addCleanup(shutil.rmtree, root, ignore_errors=True)
        worktree, manifest = root / "worktree", root / "manifest.json"
        (worktree / "docs" / "harness" / "bin").mkdir(parents=True)
        shutil.copy(REPO_ROOT / "docs/harness/bin/pr-title-policy.sh", worktree / "docs/harness/bin/pr-title-policy.sh")
        manifest.write_text(json.dumps(task_manifest(["pr_title_policy"])), encoding="utf-8")
        outcome = SBX.run_isolated(manifest, worktree, root / "run", registry_path=PROD_REGISTRY, catalog_path=PROD_CATALOG)
        self.assertEqual(outcome.status, "passed", outcome.reason)
        self.assertIn("OK: PR title policy", Path(outcome.log_paths[0]).read_text(encoding="utf-8"))
        self.assertEqual(containers_left(outcome.run_id), [])

    def test_missing_pinned_image_is_unavailable_and_never_pulled(self):
        env = SandboxEnv(self, {"fake_check": ["python3", "/tcb/tools/fake_writer.py", "benign"]},
                         sandbox=default_sandbox(image="python@sha256:" + "0" * 64))
        outcome = SBX.run_isolated(env.manifest_path, env.worktree, env.run_dir, **env.kwargs(tcb_files=TOOLS))
        self.assertEqual(outcome.status, "unavailable", outcome.reason)
        self.assertEqual(outcome.checks, ())
        self.assertFalse((env.worktree / "benign.txt").exists())


def limits_mount(outcome, destination):
    for mount in outcome.limits_enforced["mounts"]:
        if mount["destination"] == destination:
            return mount["rw"]
    raise AssertionError(f"mount {destination} not reported")


if __name__ == "__main__":
    if "--preflight" in sys.argv:
        ready, why = preflight()
        print(("SANDBOX_PREFLIGHT: READY" if ready else f"SANDBOX_PREFLIGHT: UNAVAILABLE reason={why}"))
        sys.exit(0 if ready else 3)
    unittest.main(verbosity=2)
