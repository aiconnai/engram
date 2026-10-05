#!/usr/bin/env python3
"""Trusted runner tests (task H4): docs/harness/bin/run-task.py. OFFLINE: stub docker, no Docker.

The fake writer's file effects are applied by the stub to the /work mount; gate containers follow a
per-invocation script. Covered: happy path bound to the candidate commit, separate writer workspace and
gate checkout, writer-claimed PASS, scope refusal before any candidate commit, attempt / repair / wall
budgets, check timeout, unenforceable budgets, wrong repo / ref, dirty base, untrusted runs root,
writer concurrency, sandbox unavailable (no host fallback), gate mutating its checkout, kill switch and
cleanup (never a candidate ref, never another task).
"""

from __future__ import annotations

import fcntl
import json
import os
import subprocess
import sys
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

from runner_test_support import BIN, RunnerFixture, TempRepo, load_module, run_git  # noqa: E402

WRITE_NEW = [["write_file", "src/new.txt", "hello\n"]]


def run_dirs(fx, task="h4-task"):
    root = fx.runs_root / task
    return sorted(p for p in root.iterdir() if p.is_dir()) if root.exists() else []


def gate_mounts(fx):
    """(container cmd[0], /work source) per `docker create`, in order."""
    found = []
    for call in fx.stub.h3.calls():
        argv = call["argv"]
        if argv and argv[0] == "create":
            mounts = [argv[i + 1] for i, a in enumerate(argv) if a == "--mount"]
            work = next(m for m in mounts if "dst=/work" in m)
            found.append((argv[argv.index("--entrypoint") + 3], work.split("src=")[1].split(",")[0]))
    return found


class HappyPath(unittest.TestCase):
    def setUp(self):
        self.fx = RunnerFixture(self)
        self.summary = self.fx.run(WRITE_NEW)

    def test_pass_is_bound_to_a_verified_candidate_commit(self):
        s = self.summary
        self.assertEqual(s["status"], "passed", s)
        cand = s["result"]["candidate"]
        self.assertEqual(run_git(self.fx.repo.path, "rev-parse", cand["sha"] + "^").decode().strip(), self.fx.base)
        self.assertEqual(run_git(self.fx.repo.path, "show", cand["sha"] + ":src/new.txt"), b"hello\n")
        self.assertEqual(self.fx.runner_refs(), {cand["ref"]: cand["sha"]})
        self.assertEqual(self.fx.verify(s["result"]["receipt"], cand["sha"])["status"], "verified")

    def test_writer_workspace_and_gate_checkout_are_separate_plain_directories(self):
        mounts = gate_mounts(self.fx)
        self.assertEqual(len(mounts), 2, mounts)
        (writer_cmd, writer_src), (gate_cmd, gate_src) = mounts
        self.assertEqual(writer_cmd, "/tcb/tools/fake_writer.py")
        self.assertTrue(writer_src.endswith("/a1/ws"), writer_src)
        self.assertTrue(gate_src.endswith("/a1/c0"), gate_src)
        self.assertFalse(os.path.lexists(os.path.join(writer_src, ".git")))
        self.assertFalse(os.path.lexists(os.path.join(gate_src, ".git")))

    def test_evidence_binds_base_candidate_tree_policy_argv_and_writer_metadata(self):
        gate_root = Path(self.summary["result"]["receipt"]).parent
        evidence = json.loads((gate_root / "evidence.json").read_text())
        cand = self.summary["result"]["candidate"]["sha"]
        self.assertEqual((evidence["commit_sha"], evidence["base_sha"], evidence["policy_version"]),
                         (cand, self.fx.base, "harness-hardening-v1"))
        self.assertEqual(evidence["tree_sha"], run_git(self.fx.repo.path, "rev-parse", cand + "^{tree}").decode().strip())
        self.assertEqual(evidence["checks"][0]["command"], json.dumps(["python3", "/tcb/tools/pr_title_policy.py"]))
        env = evidence["environment"]
        self.assertEqual((env["model_requested"], env["effort_requested"]), ("fake-model", "low"))
        self.assertIn("unavailable", env["writer_identity_reported"])
        self.assertIn("no RTK", env["log_capture"])
        record = json.loads((gate_root / "run-record.json").read_text())
        self.assertEqual(record["changed_paths"], ["src/new.txt"])
        self.assertEqual(record["budgets"]["wall_seconds"], {"value": 2700, "enforced": True, "by": record["budgets"]["wall_seconds"]["by"]})
        self.assertFalse(record["budgets"]["turn_cap"]["enforced"])
        self.assertFalse(record["budgets"]["cost_cap_usd"]["enforced"])
        self.assertEqual(record["budgets"]["attempts"]["value"], 2)
        self.assertEqual(record["budgets"]["repair"]["value"], 1)
        self.assertEqual(set(record["tcb_sha256"]), {"run-task.py", "check-scope.py", "record-evidence.py", "harness_git.py",
                                                     "sandbox-adapter.py", "sandbox_registry.py", "validate-evidence.py",
                                                     "fake_writer.py"})

    def test_h2_validator_cli_accepts_the_evidence_with_require_expectations(self):
        gate_root = Path(self.summary["result"]["receipt"]).parent
        cand = self.summary["result"]["candidate"]["sha"]
        tree = run_git(self.fx.repo.path, "rev-parse", cand + "^{tree}").decode().strip()
        argv = [sys.executable, str(BIN / "validate-evidence.py"), str(gate_root / "evidence.json"),
                "--expect-candidate-sha", cand, "--expect-base-sha", self.fx.base, "--expect-tree-sha", tree,
                "--expect-policy-version", "harness-hardening-v1", "--catalog", str(self.fx.catalog),
                "--expect-catalog-sha256", self.fx.rec.sha256_file(self.fx.catalog),
                "--logs-dir", str(gate_root), "--task", str(gate_root / "inputs" / "gate-task.json"), "--require-expectations"]
        proc = subprocess.run(argv, capture_output=True, text=True, timeout=60)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        wrong = [a if a != cand else "f" * 40 for a in argv]
        self.assertNotEqual(subprocess.run(wrong, capture_output=True, text=True, timeout=60).returncode, 0)


class WriterClaims(unittest.TestCase):
    def test_writer_claimed_pass_outside_scope_is_refused_before_any_candidate(self):
        fx = RunnerFixture(self)
        s = fx.run([["claim_pass"]])
        self.assertEqual(s["status"], "scope_refused", s)
        self.assertIn("path_not_allowed", s["reason"])
        self.assertEqual(fx.runner_refs(), {})
        self.assertEqual(fx.stub.count("gate"), 0)
        ws = Path(s["run_dir"]) / "a1" / "ws"
        self.assertTrue((ws / "receipt.json").exists())
        with self.assertRaises(fx.rec.VerifyRefused):
            fx.verify(ws / "receipt.json", fx.base)

    def test_writer_claimed_pass_never_overrides_a_failing_gate(self):
        fx = RunnerFixture(self, gate_default={"exit_code": 1, "stdout": "FAIL\n"})
        s = fx.run([["claim_pass"]], allowed_paths=["evidence.json", "receipt.json", "outcome.json"],
                   budgets={"attempts": 1, "repair": 0})
        self.assertEqual(s["status"], "failed", s)
        self.assertEqual(s["result"], {})
        gate_root = Path(s["run_dir"]) / "a1" / "g0"
        self.assertEqual(json.loads((gate_root / "evidence.json").read_text())["verdict"], "fail")
        cand = s["events"][0]["candidate"]
        with self.assertRaises(fx.rec.VerifyRefused) as ctx:
            fx.verify(gate_root / "receipt.json", cand)
        self.assertEqual(ctx.exception.code, "evidence_invalid")

    def test_cli_reports_status_line_and_exit_codes(self):
        fx = RunnerFixture(self)
        base_args = ["run", "--repo", str(fx.repo.path), "--runs-root", str(fx.runs_root), "--registry", str(fx.registry),
                     "--catalog", str(fx.catalog), "--docker", str(fx.stub.path)]
        ok = subprocess.run([sys.executable, str(BIN / "run-task.py"), *base_args, "--request", str(fx.request(WRITE_NEW))],
                            capture_output=True, text=True, timeout=120)
        self.assertEqual(ok.returncode, 0, ok.stdout + ok.stderr)
        self.assertRegex(ok.stdout.strip().splitlines()[-1], r"^RUN_STATUS: PASSED task=h4-task run=\S+ candidate=[0-9a-f]{40} ")
        bad = subprocess.run([sys.executable, str(BIN / "run-task.py"), *base_args,
                              "--request", str(fx.request([["write_file", "docs/harness/bin/x.sh", "evil"]],
                                                          allowed_paths=["src/", "docs/"]))],
                             capture_output=True, text=True, timeout=120)
        self.assertEqual(bad.returncode, 4, bad.stdout)
        self.assertIn("RUN_STATUS: SCOPE_REFUSED", bad.stdout)


class ScopeAndWorkspace(unittest.TestCase):
    def test_protected_change_stops_the_run_without_creating_a_candidate(self):
        fx = RunnerFixture(self)
        s = fx.run([["write_file", "docs/harness/bin/evil.sh", "rm -rf /"]], allowed_paths=["src/", "docs/"])
        self.assertEqual(s["status"], "scope_refused", s)
        self.assertEqual(fx.runner_refs(), {})
        report = json.loads((Path(s["run_dir"]) / "a1" / "scope-0.json").read_text())
        self.assertIn("protected_path", {f["code"] for f in report["findings"]})
        self.assertEqual(fx.stub.count("writer"), 1)  # no retry around a security refusal

    def test_git_metadata_written_by_the_writer_is_refused(self):
        fx = RunnerFixture(self)
        s = fx.run([["write_file", "src/.git/config", "[core]\n"]])
        self.assertEqual(s["status"], "scope_refused", s)
        self.assertIn("git_metadata_in_workspace", s["reason"])
        self.assertEqual(fx.runner_refs(), {})


class Budgets(unittest.TestCase):
    def test_writer_failure_then_success_uses_the_second_attempt(self):
        fx = RunnerFixture(self)
        s = fx.run([["exit_code", "3"], ["write_file", "src/b.txt", "b"]])
        self.assertEqual(s["status"], "passed", s)
        self.assertEqual([e["attempt"] for e in s["events"]], [1, 2])
        self.assertTrue(s["events"][0]["result"].startswith("writer_failed"))

    def test_attempt_cap_is_nonzero_and_writer_logs_survive(self):
        fx = RunnerFixture(self)
        s = fx.run([["exit_code", "3"]])
        self.assertEqual(s["status"], "failed", s)
        self.assertTrue(s["reason"].startswith("attempts_exhausted"), s["reason"])
        self.assertEqual(fx.stub.count("writer"), 2)
        for attempt in ("a1", "a2"):
            logs = Path(s["run_dir"]) / attempt / "w0" / "logs"
            self.assertTrue(any(logs.iterdir()), logs)
            self.assertTrue((Path(s["run_dir"]) / attempt / "w0" / "outcome.json").exists())
        self.assertTrue((Path(s["run_dir"]) / "summary.json").exists())

    def test_repair_round_fixes_a_failing_gate_and_keeps_the_failed_candidate(self):
        fx = RunnerFixture(self, gates=[{"exit_code": 1}, {"exit_code": 0}])
        s = fx.run([["write_file", "src/x.txt", "1"], ["write_file", "src/x.txt", "2"]])
        self.assertEqual(s["status"], "passed", s)
        self.assertEqual([(e["attempt"], e["round"]) for e in s["events"]], [(1, 0), (1, 1)])
        refs = fx.runner_refs()
        self.assertEqual(len(refs), 2)
        self.assertEqual(sorted(r.rsplit("/", 1)[1] for r in refs), ["a1r0", "a1r1"])
        failed_gate = Path(s["run_dir"]) / "a1" / "g0"
        self.assertEqual(json.loads((failed_gate / "evidence.json").read_text())["verdict"], "fail")

    def test_repair_and_attempt_caps_bound_the_number_of_gate_runs(self):
        fx = RunnerFixture(self, gate_default={"exit_code": 1})
        s = fx.run([["write_file", "src/x.txt", "1"], ["write_file", "src/x.txt", "2"]])
        self.assertEqual(s["status"], "failed", s)
        self.assertTrue(s["reason"].startswith("attempts_exhausted"), s["reason"])
        self.assertEqual(fx.stub.count("gate"), 3)  # a1r0, a1r1, a2r0; a2r1 reproduces a2r0's tree -> no change, no gate
        self.assertEqual(len(fx.runner_refs()), 3)
        self.assertEqual(s["events"][-1]["result"], "writer_no_changes")

    def test_check_timeout_is_a_nonzero_failure_with_logs(self):
        fx = RunnerFixture(self, check_timeout=1, gate_default={"sleep": 30})
        s = fx.run(WRITE_NEW, budgets={"attempts": 1, "repair": 0})
        self.assertEqual(s["status"], "failed", s)
        self.assertIn("gate_timeout", s["events"][0]["result"])
        evidence = json.loads((Path(s["run_dir"]) / "a1" / "g0" / "evidence.json").read_text())
        self.assertEqual(evidence["checks"][0]["status"], "timeout")
        self.assertTrue((Path(s["run_dir"]) / "a1" / "g0" / "run" / "logs").is_dir())

    def test_wall_budget_is_enforced_and_logs_survive(self):
        fx = RunnerFixture(self)
        s = fx.run([["outlive_timeout"]], budgets={"wall_seconds": 3})
        self.assertEqual(s["status"], "failed", s)
        self.assertEqual(s["reason"], "wall_budget_exhausted")
        self.assertTrue((Path(s["run_dir"]) / "a1" / "w0" / "outcome.json").exists())
        self.assertEqual(json.loads((Path(s["run_dir"]) / "a1" / "w0" / "outcome.json").read_text())["status"], "timeout")

    def test_unenforceable_or_oversized_budgets_are_refused_before_anything_runs(self):
        fx = RunnerFixture(self)
        cases = {
            "turn cap required": ({"require_enforced": ["turn_cap"]}, "budget_not_enforceable"),
            "cost cap set": ({"budgets": {"cost_cap_usd": 5}}, "budget_not_enforceable"),
            "attempts above pilot": ({"budgets": {"attempts": 3}}, "budget_above_pilot_cap"),
            "wall above pilot": ({"budgets": {"wall_seconds": 2701}}, "budget_above_pilot_cap"),
            "turn cap above pilot": ({"budgets": {"turn_cap": 21}}, "budget_above_pilot_cap"),
            "concurrency 2": ({"budgets": {"writer_concurrency": 2}}, "budget_above_pilot_cap"),
            "real writer": ({"writer": {"adapter": "claude-code", "invocations": [["benign"]]}}, "writer_not_granted"),
            "unexposed behavior": ({"writer": {"adapter": "fake", "invocations": [["network"]]}}, "request_invalid"),
            "policy mismatch": ({"policy_version": "other-policy"}, "policy_mismatch"),
        }
        for label, (over, code) in cases.items():
            with self.subTest(label):
                s = fx.run(WRITE_NEW, **over)
                self.assertEqual(s["status"], "refused", s)
                self.assertTrue(s["reason"].startswith(code), s["reason"])
        self.assertEqual(fx.stub.h3.calls(), [])
        enforced = fx.run(WRITE_NEW, require_enforced=["wall_seconds", "attempts", "check_timeout"])
        self.assertEqual(enforced["status"], "passed", enforced)


class InputsAndEnvironment(unittest.TestCase):
    def test_wrong_repo_wrong_ref_and_dirty_base_are_refused(self):
        fx = RunnerFixture(self)
        other = TempRepo(self)
        foreign = other.commit({"x": b"1\n"})
        self.assertTrue(fx.run(WRITE_NEW, base_sha=foreign)["reason"].startswith("wrong_repo"))
        child = fx.repo.commit({"src/a.txt": b"other\n"}, parent=fx.base)
        self.assertTrue(fx.run(WRITE_NEW, base_ref=child)["reason"].startswith("base_ref_mismatch"))
        self.assertTrue(fx.run(WRITE_NEW, base_ref="refs/heads/missing")["reason"].startswith("base_ref_mismatch"))
        (fx.repo.path / "src" / "a.txt").write_text("dirty\n")
        s = fx.run(WRITE_NEW)
        self.assertEqual(s["status"], "refused")
        self.assertTrue(s["reason"].startswith("dirty_base"), s["reason"])
        self.assertEqual(fx.stub.h3.calls(), [])

    def test_runs_root_inside_the_repository_or_relative_is_refused(self):
        fx = RunnerFixture(self)
        inside = fx.run(WRITE_NEW, runs_root=fx.repo.path / "runs")
        self.assertTrue(inside["reason"].startswith("untrusted_runs_root"), inside)
        self.assertFalse((fx.repo.path / "runs").exists())
        relative = fx.run(WRITE_NEW, runs_root=Path("relative-runs"))
        self.assertTrue(relative["reason"].startswith("untrusted_runs_root"), relative)

    def test_second_concurrent_writer_is_refused_even_from_another_runs_root(self):
        fx = RunnerFixture(self)
        common = run_git(fx.repo.path, "rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip()
        fd = os.open(Path(common) / "engram-runner.lock", os.O_RDWR | os.O_CREAT, 0o600)
        self.addCleanup(os.close, fd)
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)  # a run holding the per-repository writer lock
        for root in (fx.runs_root, fx.repo.root / "runs-b"):
            with self.subTest(str(root)):
                s = fx.run(WRITE_NEW, runs_root=root)
                self.assertEqual(s["status"], "refused", s)
                self.assertTrue(s["reason"].startswith("writer_concurrency_cap"), s["reason"])
        self.assertEqual(fx.stub.h3.calls(), [])
        fcntl.flock(fd, fcntl.LOCK_UN)
        self.assertEqual(fx.run(WRITE_NEW, runs_root=fx.repo.root / "runs-b")["status"], "passed")

    def test_unusable_repository_lock_is_a_refusal_not_an_exception(self):
        fx = RunnerFixture(self)
        common = Path(run_git(fx.repo.path, "rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip())
        os.chmod(common, 0o500)
        self.addCleanup(os.chmod, common, 0o755)
        s = fx.run(WRITE_NEW)
        self.assertEqual(s["status"], "refused", s)
        self.assertTrue(s["reason"].startswith("runner_lock_unavailable"), s["reason"])
        self.assertEqual(fx.stub.h3.calls(), [])

    def test_existing_non_empty_directory_is_not_adopted_as_a_runs_root(self):
        fx = RunnerFixture(self)
        foreign = fx.repo.root / "someone-elses-dir"
        foreign.mkdir(mode=0o700)
        (foreign / "data.txt").write_text("keep\n")
        s = fx.run(WRITE_NEW, runs_root=foreign)
        self.assertEqual(s["status"], "refused", s)
        self.assertTrue(s["reason"].startswith("untrusted_runs_root"), s["reason"])
        self.assertFalse((foreign / ".engram-runs-root").exists())
        empty = fx.repo.root / "empty-dir"
        empty.mkdir(mode=0o700)
        self.assertEqual(fx.run(WRITE_NEW, runs_root=empty)["status"], "passed")

    def test_unavailable_sandbox_is_a_distinct_status_with_no_host_fallback(self):
        fx = RunnerFixture(self, daemon_down=True)
        s = fx.run(WRITE_NEW)
        self.assertEqual(s["status"], "unavailable", s)
        self.assertFalse((Path(s["run_dir"]) / "a1" / "ws" / "src" / "new.txt").exists())
        self.assertEqual(fx.runner_refs(), {})

    def test_gate_that_mutates_its_checkout_is_never_a_pass(self):
        fx = RunnerFixture(self, gate_default={"mutate": {"src/generated.txt": "x"}})
        s = fx.run(WRITE_NEW)
        self.assertEqual(s["status"], "failed", s)
        self.assertTrue(s["reason"].startswith("gate_mutated_checkout"), s["reason"])
        gate_root = Path(s["run_dir"]) / "a1" / "g0"
        with self.assertRaises(fx.rec.VerifyRefused) as ctx:
            fx.verify(gate_root / "receipt.json", s["events"][0]["candidate"])
        self.assertEqual(ctx.exception.code, "gate_mutated_checkout")

    def test_kill_switch_refuses_without_touching_anything(self):
        fx = RunnerFixture(self)
        with mock.patch.dict(os.environ, {"ENGRAM_RUNNER_DISABLED": "1"}):
            s = fx.run(WRITE_NEW)
        self.assertEqual(s["status"], "refused")
        self.assertIn("runner_disabled", s["reason"])
        self.assertFalse(fx.runs_root.exists())


class WorkspaceBounds(unittest.TestCase):
    def setUp(self):
        self.repo = TempRepo(self)
        self.commit = self.repo.commit({"x": b"1\n"})
        self.ws = self.repo.root / "ws"
        (self.ws / "d").mkdir(parents=True)
        for i in range(3):
            (self.ws / "d" / f"f{i}.txt").write_text("x" * 100)
        self.hg = load_module("harness_git")

    def test_file_count_and_byte_caps_refuse_an_oversized_workspace(self):
        with self.assertRaises(self.hg.WorkspaceError) as ctx:
            self.hg.build_tree(self.repo.path, self.ws, self.repo.root, max_files=2)
        self.assertEqual(ctx.exception.code, "workspace_too_large")
        with self.assertRaises(self.hg.WorkspaceError) as ctx:
            self.hg.build_tree(self.repo.path, self.ws, self.repo.root, max_bytes=250)
        self.assertEqual(ctx.exception.code, "workspace_too_large")
        self.assertEqual(len(self.hg.build_tree(self.repo.path, self.ws, self.repo.root, max_files=3, max_bytes=300)), 40)

    def test_deadline_bounds_export_and_build(self):
        past = time.monotonic() - 1
        with self.assertRaises(self.hg.WorkspaceError) as ctx:
            self.hg.build_tree(self.repo.path, self.ws, self.repo.root, deadline=past)
        self.assertEqual(ctx.exception.code, "deadline_exceeded")
        dest = self.repo.root / "export"
        dest.mkdir()
        with self.assertRaises(self.hg.WorkspaceError) as ctx:
            self.hg.export_tree(self.repo.path, self.commit, dest, deadline=past)
        self.assertEqual(ctx.exception.code, "deadline_exceeded")


class Cleanup(unittest.TestCase):
    def test_cleanup_removes_only_scratch_of_one_run_and_never_refs_or_other_tasks(self):
        fx = RunnerFixture(self)
        first = fx.run(WRITE_NEW)
        second = fx.run([["write_file", "src/other.txt", "o"]], task_id="h4-other")
        self.assertEqual((first["status"], second["status"]), ("passed", "passed"))
        refs_before = fx.runner_refs()
        run_dir = Path(first["run_dir"])
        removed = fx.runner.cleanup(repo=fx.repo.path, runs_root=fx.runs_root, task_id="h4-task", run_id=run_dir.name)
        self.assertEqual(sorted(removed), sorted([f"h4-task/{run_dir.name}/a1/c0", f"h4-task/{run_dir.name}/a1/ws"]))
        self.assertEqual(fx.runner_refs(), refs_before)
        self.assertTrue((run_dir / "a1" / "g0" / "evidence.json").exists())
        self.assertTrue((run_dir / "a1" / "w0" / "logs").is_dir())
        cand = first["result"]["candidate"]["sha"]
        self.assertEqual(fx.verify(first["result"]["receipt"], cand)["status"], "verified")
        other = Path(second["run_dir"])
        self.assertTrue((other / "a1" / "ws").is_dir() and (other / "a1" / "c0").is_dir())
        for bad in (("../h4-other", run_dir.name), ("h4-task", "../../x")):
            with self.subTest(bad), self.assertRaises(fx.runner.Refused):
                fx.runner.cleanup(repo=fx.repo.path, runs_root=fx.runs_root, task_id=bad[0], run_id=bad[1])
        missing = fx.repo.root / "no-such-runs-root"
        with self.assertRaises(fx.runner.Refused):
            fx.runner.cleanup(repo=fx.repo.path, runs_root=missing, task_id="h4-task", run_id=run_dir.name)
        self.assertFalse(missing.exists())  # cleanup never creates directories


if __name__ == "__main__":
    unittest.main(verbosity=2)
