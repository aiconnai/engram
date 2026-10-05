#!/usr/bin/env python3
"""OFFLINE unit tests of docs/harness/bin/run-standing-checks.py (task O4).

Read-only standing checks with ownership: a goal selects an approved check by ID, never a command. These tests
use the recording fake `docker` CLI of the H3 tests (sandbox_test_support.StubDocker) and synthetic temporary git
repositories: no Docker daemon, no network, no credentials. What they prove: strict goals validation (unknown
check, extra shell/command field, timeout above policy, missing owner), the per-goal concurrency lock, that
timeout / failure / missing artifact / mutated checkout / missing Docker are never a pass, the manual /
dispatched / scheduled selection rules, a receipt that binds SHA, policy, toolchain and log hashes, a LOCAL alert
that names owner and runbook (never delivered), that nothing is written to the repository and no writer is
started, and the read-only contract of .github/workflows/standing-checks.yml. What they cannot prove is that the
container boundary holds (H3 smoke) or that a real scheduled GitHub run works (NOT RUN here).
"""

from __future__ import annotations

import ast
import copy
import fcntl
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

from sandbox_test_support import POLICY_VERSION, PROD_CATALOG, PROD_REGISTRY, StubDocker, default_sandbox  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
RUNNER_PATH = Path(os.environ.get("ENGRAM_STANDING_RUNNER_UNDER_TEST", REPO_ROOT / "docs" / "harness" / "bin" / "run-standing-checks.py"))
PROD_GOALS = REPO_ROOT / "docs" / "harness" / "goals" / "registry.json"
PROD_SCHEMA = REPO_ROOT / "docs" / "harness" / "schemas" / "goal-v1.schema.json"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "standing-checks.yml"
SUPPLY_CHAIN = REPO_ROOT / "scripts" / "check-workflow-supply-chain.py"
CHECK_ARGV = ["bash", "docs/harness/bin/pr-title-policy.sh", "--title", "fix: clean title"]
RUNBOOK = "docs/harness/goals/README.md"


def load_runner():
    # The runner loads its siblings from its own directory, so an override must live next to the real tools.
    spec = importlib.util.spec_from_file_location("run_standing_checks_under_test", RUNNER_PATH)
    if spec is None or spec.loader is None:  # pragma: no cover
        raise ImportError(f"cannot load {RUNNER_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules["run_standing_checks_under_test"] = module
    spec.loader.exec_module(module)
    return module


RUN = load_runner()


def git(repo: Path, *args: str) -> str:
    env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid",
           "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull,
           "GIT_CONFIG_SYSTEM": os.devnull}
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, check=True, env=env).stdout.strip()


def goal(goal_id="daily-title", check_id="pr_title_policy", schedule="daily", timeout=20, **over):
    base = {"id": goal_id, "check_id": check_id, "owner": "harness-maintainers", "schedule": schedule,
            "timeout_seconds": timeout, "on_failure": "alert_only", "runbook": RUNBOOK}
    base.update(over)
    return base


class Env:
    """Temporary git repo + registry + catalog + goals + runs root + stub docker, owned by one test."""

    def __init__(self, test: unittest.TestCase, goals=None, **stub):
        self.test = test
        self.root = Path(tempfile.mkdtemp(prefix="o4-")).resolve()
        test.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.repo = self.root / "repo"
        (self.repo / "docs/harness/goals").mkdir(parents=True)
        (self.repo / RUNBOOK).write_text("runbook\n", encoding="utf-8")
        (self.repo / "README.md").write_text("synthetic\n", encoding="utf-8")
        git(self.repo, "init", "-q", "-b", "main")
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-q", "-m", "init")
        self.sha = git(self.repo, "rev-parse", "HEAD")
        self.runs_root = self.root / "runs"
        self.alerts = self.runs_root / "alerts"
        self.registry_path = self.root / "registry.json"
        self.catalog_path = self.root / "catalog.json"
        self.goals_path = self.root / "goals.json"
        self.stub = StubDocker(test, **stub)
        self.write_registry()
        self.write_goals(goals if goals is not None else [goal()])

    def write_registry(self, checks=None, sandbox=None):
        checks = checks or {"pr_title_policy": {"argv": CHECK_ARGV, "timeout_seconds": 30},
                            "second_check": {"argv": CHECK_ARGV, "timeout_seconds": 30}}
        registry = {"registry_version": "check-registry-v1", "policy_version": POLICY_VERSION,
                    "sandbox": sandbox or default_sandbox(), "checks": checks}
        self.registry_path.write_text(json.dumps(registry), encoding="utf-8")
        catalog = {"catalog_version": "check-catalog-v1", "checks": {c: {"description": f"synthetic {c}"} for c in checks}}
        self.catalog_path.write_text(json.dumps(catalog), encoding="utf-8")

    def write_goals(self, goals, **over):
        doc = {"schema_version": "goal-v1", "policy_version": POLICY_VERSION, "goals": goals}
        doc.update(over)
        self.goals_path.write_text(json.dumps(doc), encoding="utf-8")

    def write_goals_text(self, text: str):
        self.goals_path.write_text(text, encoding="utf-8")

    def run(self, mode="scheduled", schedule="daily", goal_ids=(), **extra):
        kwargs = dict(repo=self.repo, runs_root=self.runs_root, mode=mode, schedule=schedule, goal_ids=list(goal_ids),
                      goals_path=self.goals_path, schema_path=PROD_SCHEMA, registry_path=self.registry_path,
                      catalog_path=self.catalog_path, docker=str(self.stub.path))
        kwargs.update(extra)
        return RUN.run_standing_checks(**kwargs)

    def alerts_written(self):
        return sorted(self.alerts.glob("*.json")) if self.alerts.exists() else []

    def repo_state(self):
        return {"head": git(self.repo, "rev-parse", "HEAD"), "refs": git(self.repo, "for-each-ref"),
                "status": git(self.repo, "status", "--porcelain", "--untracked-files=all"),
                "objects": git(self.repo, "count-objects", "-v")}


class ModuleHygieneTests(unittest.TestCase):
    def test_no_duplicate_test_case_classes_or_methods(self):
        tree = ast.parse(Path(__file__).read_text(encoding="utf-8"))
        classes = [n.name for n in tree.body if isinstance(n, ast.ClassDef)]
        self.assertEqual(len(classes), len(set(classes)))
        for node in (n for n in tree.body if isinstance(n, ast.ClassDef)):
            methods = [m.name for m in node.body if isinstance(m, ast.FunctionDef)]
            self.assertEqual(len(methods), len(set(methods)), f"{node.name} duplicate methods")

    def test_runner_has_no_network_message_or_process_imports(self):
        tree = ast.parse(RUNNER_PATH.read_text(encoding="utf-8"))
        imported = {a.name.split(".")[0] for n in ast.walk(tree) if isinstance(n, ast.Import) for a in n.names}
        imported |= {(n.module or "").split(".")[0] for n in ast.walk(tree) if isinstance(n, ast.ImportFrom)}
        forbidden = {"subprocess", "socket", "urllib", "http", "smtplib", "requests", "ssl", "ftplib", "shutil"}
        self.assertEqual(imported & forbidden, set(), "the scheduler may only reach docker/git through the H3/H4 modules")


class GoalValidationTests(unittest.TestCase):
    def refused(self, mutate, expect_code):
        env = Env(self)
        doc = json.loads(env.goals_path.read_text(encoding="utf-8"))
        mutate(doc)
        env.goals_path.write_text(json.dumps(doc), encoding="utf-8")
        summary = env.run()
        self.assertEqual(summary["status"], "refused", summary)
        self.assertEqual(summary["exit_code"], 2)
        self.assertIn(expect_code, summary["reason"])
        self.assertEqual(env.stub.calls(), [], "a refused registry must not reach docker")
        self.assertFalse((env.runs_root / "goals").exists())

    def test_production_goals_load_against_the_production_registry(self):
        registry = RUN.SBX.load_registry(PROD_REGISTRY, PROD_CATALOG)
        goals = RUN.parse_goals(PROD_GOALS.read_bytes(), PROD_SCHEMA.read_bytes(), registry, lambda rel: (REPO_ROOT / rel).is_file())
        self.assertTrue(goals)
        for g in goals:
            self.assertIn(g.check_id, registry.checks)
            self.assertEqual(g.on_failure, "alert_only")
            self.assertTrue(g.owner)

    def test_unknown_check_is_refused(self):
        self.refused(lambda d: d["goals"][0].__setitem__("check_id", "not_a_registered_check"), "unknown_check")

    def test_check_in_catalog_but_not_in_registry_is_refused(self):
        self.refused(lambda d: d["goals"][0].__setitem__("check_id", "fmt"), "unknown_check")

    def test_extra_shell_or_command_fields_are_refused(self):
        for field, value in (("command", "rm -rf /"), ("shell", "bash -c id"), ("argv", ["bash", "x"]), ("script", "x.sh"),
                             ("env", {"A": "b"}), ("run", "make"), ("args", ["--x"]), ("cmd", "ls")):
            with self.subTest(field=field):
                self.refused(lambda d, f=field, v=value: d["goals"][0].__setitem__(f, v), "unexpected_property")

    def test_extra_top_level_field_is_refused(self):
        self.refused(lambda d: d.__setitem__("hooks", {"pre": "x"}), "unexpected_property")

    def test_timeout_above_check_policy_is_refused(self):
        self.refused(lambda d: d["goals"][0].__setitem__("timeout_seconds", 31), "timeout_above_policy")

    def test_timeout_above_sandbox_maximum_is_refused(self):
        env = Env(self)
        env.write_registry(checks={"pr_title_policy": {"argv": CHECK_ARGV, "timeout_seconds": 600}, "second_check": {"argv": CHECK_ARGV, "timeout_seconds": 600}},
                           sandbox=default_sandbox(max_total_timeout_seconds=120))
        env.write_goals([goal(timeout=121)])
        summary = env.run()
        self.assertEqual(summary["status"], "refused")
        self.assertIn("timeout_above_policy", summary["reason"])

    def test_timeout_must_be_a_plain_bounded_integer(self):
        for bad in (True, 20.0, "20", 0, 1, -5, 10 ** 9, None):
            with self.subTest(timeout=bad):
                self.refused(lambda d, b=bad: d["goals"][0].__setitem__("timeout_seconds", b), "goals_invalid")

    def test_missing_or_blank_owner_is_refused(self):
        def drop(d):
            del d["goals"][0]["owner"]
        self.refused(drop, "missing_required")
        for owner in ("", " ", "a", None, 7, "has space", "semi;colon", "x" * 65):
            with self.subTest(owner=owner):
                self.refused(lambda d, o=owner: d["goals"][0].__setitem__("owner", o), "goals_invalid")

    def test_on_failure_must_be_alert_only(self):
        for value in ("remediate", "auto_fix", "open_pr", "", None):
            with self.subTest(on_failure=value):
                self.refused(lambda d, v=value: d["goals"][0].__setitem__("on_failure", v), "goals_invalid")

        def drop(d):
            del d["goals"][0]["on_failure"]
        self.refused(drop, "missing_required")

    def test_schedule_must_be_an_enum_value(self):
        for value in ("hourly", "", "DAILY", "daily ", None, 1):
            with self.subTest(schedule=value):
                self.refused(lambda d, v=value: d["goals"][0].__setitem__("schedule", v), "goals_invalid")

    def test_goal_id_cannot_traverse_or_collide(self):
        for value in ("../x", "a/b", "A", "", " ", "a" * 65, ".hidden"):
            with self.subTest(goal_id=value):
                self.refused(lambda d, v=value: d["goals"][0].__setitem__("id", v), "goals_invalid")
        self.refused(lambda d: d["goals"].append(copy.deepcopy(d["goals"][0])), "duplicate goal id")

    def test_runbook_must_exist_in_the_run_commit_and_stay_inside_the_repo(self):
        self.refused(lambda d: d["goals"][0].__setitem__("runbook", "docs/harness/goals/absent.md"), "runbook_missing")
        for value in ("/etc/passwd", "../outside.md", "a/../../b"):
            with self.subTest(runbook=value):
                self.refused(lambda d, v=value: d["goals"][0].__setitem__("runbook", v), "goals_invalid")

    def test_policy_version_and_schema_version_must_match(self):
        self.refused(lambda d: d.__setitem__("policy_version", "other-policy-v9"), "policy_mismatch")
        self.refused(lambda d: d.__setitem__("schema_version", "goal-v2"), "goals_invalid")
        self.refused(lambda d: d.__setitem__("goals", []), "goals_invalid")

    def test_duplicate_json_keys_and_non_object_roots_are_refused(self):
        env = Env(self)
        for text in ('{"schema_version":"goal-v1","schema_version":"goal-v1","policy_version":"%s","goals":[]}' % POLICY_VERSION,
                     "[]", "not json", '{"goals": NaN}'):
            with self.subTest(text=text[:30]):
                env.write_goals_text(text)
                summary = env.run()
                self.assertEqual((summary["status"], summary["exit_code"]), ("refused", 2), summary)

    def test_symlinked_goals_file_is_refused(self):
        env = Env(self)
        link = env.root / "goals-link.json"
        link.symlink_to(env.goals_path)
        summary = env.run(goals_path=link)
        self.assertEqual(summary["status"], "refused")
        self.assertIn("input_unsafe", summary["reason"])

    def test_validate_cli_reports_ok_and_fail(self):
        ok = subprocess.run([sys.executable, str(RUNNER_PATH), "validate"], capture_output=True, text=True, timeout=60)
        self.assertEqual((ok.returncode, ok.stdout.startswith("GOALS: OK")), (0, True), ok.stderr)
        env = Env(self)
        env.write_goals([goal(check_id="nope")])
        bad = subprocess.run([sys.executable, str(RUNNER_PATH), "validate", "--goals", str(env.goals_path), "--registry", str(env.registry_path),
                              "--catalog", str(env.catalog_path)], capture_output=True, text=True, timeout=60)
        self.assertEqual(bad.returncode, 2)
        self.assertIn("unknown_check", bad.stderr)


class ModeSelectionTests(unittest.TestCase):
    def setUp(self):
        self.env = Env(self, goals=[goal("daily-a"), goal("weekly-b", "second_check", "weekly"), goal("manual-c", schedule="manual")])

    def ran(self, summary):
        return sorted(r["goal_id"] for r in summary["results"])

    def test_scheduled_runs_only_the_matching_schedule_and_never_manual_goals(self):
        daily = self.env.run("scheduled", "daily")
        self.assertEqual((daily["status"], self.ran(daily)), ("pass", ["daily-a"]), daily)
        weekly = self.env.run("scheduled", "weekly")
        self.assertEqual(self.ran(weekly), ["weekly-b"])

    def test_scheduled_refuses_goal_names_missing_schedule_and_manual_schedule(self):
        for kwargs in ({"schedule": "daily", "goal_ids": ["daily-a"]}, {"schedule": None}, {"schedule": "manual"}, {"schedule": "hourly"}):
            with self.subTest(kwargs=kwargs):
                summary = self.env.run("scheduled", **kwargs)
                self.assertEqual((summary["status"], summary["exit_code"]), ("refused", 2), summary)
                self.assertIn("usage", summary["reason"])
        self.assertEqual(self.env.stub.calls(), [])

    def test_manual_needs_an_explicit_goal_and_can_run_a_manual_schedule_goal(self):
        refused = self.env.run("manual", None)
        self.assertEqual(refused["status"], "refused")
        both = self.env.run("manual", "daily", ["daily-a"])
        self.assertEqual(both["status"], "refused")
        ran = self.env.run("manual", None, ["manual-c"])
        self.assertEqual((ran["status"], self.ran(ran), ran["mode"]), ("pass", ["manual-c"], "manual"), ran)

    def test_dispatched_takes_exactly_one_of_goal_or_schedule(self):
        for kwargs in ({"schedule": None, "goal_ids": []}, {"schedule": "daily", "goal_ids": ["daily-a"]}):
            with self.subTest(kwargs=kwargs):
                self.assertEqual(self.env.run("dispatched", **kwargs)["status"], "refused")
        by_goal = self.env.run("dispatched", None, ["weekly-b", "manual-c"])
        self.assertEqual(self.ran(by_goal), ["manual-c", "weekly-b"])
        by_schedule = self.env.run("dispatched", "weekly")
        self.assertEqual(self.ran(by_schedule), ["weekly-b"])

    def test_every_mode_is_recorded_in_the_receipt(self):
        for mode, kwargs in (("manual", {"schedule": None, "goal_ids": ["daily-a"]}), ("dispatched", {"schedule": "daily"}), ("scheduled", {"schedule": "daily"})):
            with self.subTest(mode=mode):
                summary = self.env.run(mode, **kwargs)
                receipt = json.loads(Path(summary["results"][0]["receipt"]).read_text(encoding="utf-8"))
                self.assertEqual(receipt["mode"], mode)

    def test_unknown_goal_and_unknown_mode_are_refused(self):
        self.assertIn("unknown_goal", self.env.run("manual", None, ["nope"])["reason"])
        self.assertEqual(self.env.run("cron", "daily")["status"], "refused")

    def test_nothing_selected_is_not_run_never_pass(self):
        env = Env(self, goals=[goal("only-manual", schedule="manual")])
        summary = env.run("scheduled", "daily")
        self.assertEqual((summary["status"], summary["exit_code"], summary["selected"], summary["passed"]), ("not-run", 4, 0, 0), summary)


class RunTests(unittest.TestCase):
    def test_pass_records_sha_policy_toolchain_and_log_hashes_and_raises_no_alert(self):
        env = Env(self, start={"stdout": "title ok\n"})
        before = env.repo_state()
        summary = env.run()
        self.assertEqual((summary["status"], summary["exit_code"], summary["passed"]), ("pass", 0, 1), summary)
        result = summary["results"][0]
        self.assertEqual(result["status"], "pass")
        self.assertIsNone(result["alert"])
        self.assertEqual(env.alerts_written(), [])
        receipt = json.loads(Path(result["receipt"]).read_text(encoding="utf-8"))
        self.assertEqual(receipt["run_sha"], env.sha)
        self.assertEqual(receipt["tree_sha"], git(env.repo, "rev-parse", "HEAD^{tree}"))
        self.assertEqual(receipt["policy_version"], POLICY_VERSION)
        self.assertTrue(receipt["toolchain"]["python"] and receipt["toolchain"]["git"] and receipt["toolchain"]["docker_endpoint"])
        self.assertTrue(receipt["toolchain"]["image"].startswith("python@sha256:"))
        for key in ("goals", "goal_schema", "check_registry", "check_catalog"):
            self.assertRegex(receipt["inputs_sha256"][key], r"^[0-9a-f]{64}$")
        self.assertEqual(set(receipt["tcb_sha256"]), set(RUN.TCB_FILES))
        self.assertEqual({r["stream"] for r in receipt["logs"]}, {"stdout", "stderr"})
        for log in receipt["logs"]:
            self.assertRegex(log["sha256"], r"^[0-9a-f]{64}$")
        self.assertRegex(receipt["outcome"]["outcome_sha256"], r"^[0-9a-f]{64}$")
        self.assertTrue(receipt["checkout_unchanged"])
        self.assertEqual(receipt["authority"], {"writer_started": False, "repo_mutated": False, "network": "none", "credentials": "none",
                                                "on_failure": "alert_only", "delivery": "local-only"})
        self.assertEqual(env.repo_state(), before, "a standing check must not change the repository")

    def test_only_the_approved_argv_runs_with_read_only_capabilities_and_no_writer(self):
        env = Env(self)
        summary = env.run()
        run_dir = Path(summary["results"][0]["receipt"]).parent
        manifest = json.loads((run_dir / "inputs" / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["allowed_capabilities"], ["workspace_read", "test_exec", "sandbox_container", "network_none"])
        self.assertNotIn("workspace_write", manifest["allowed_capabilities"])
        self.assertEqual(manifest["required_checks"], ["pr_title_policy"])
        creates = [c["argv"] for c in env.stub.calls() if c["argv"][:1] == ["create"]]
        self.assertEqual(len(creates), 1)
        self.assertEqual(creates[0][-len(CHECK_ARGV) + 1:], CHECK_ARGV[1:], "the registry argv is the only command handed to docker")
        self.assertNotIn("fake_writer", " ".join(creates[0]))
        self.assertTrue(all(c["argv"][0] in {"context", "version", "image", "create", "inspect", "start", "kill", "rm", "ps"} for c in env.stub.calls()))
        outcome = json.loads((run_dir / "sandbox" / "outcome.json").read_text(encoding="utf-8"))
        self.assertFalse(outcome["executed_on_host"])

    def test_failing_check_is_a_fail_with_a_local_alert_naming_owner_and_runbook(self):
        env = Env(self, start={"exit_code": 1, "stdout": "bad title\n"})
        before = env.repo_state()
        summary = env.run()
        self.assertEqual((summary["status"], summary["exit_code"], summary["passed"]), ("fail", 1, 0), summary)
        result = summary["results"][0]
        self.assertEqual(result["status"], "fail")
        alerts = env.alerts_written()
        self.assertEqual([str(a) for a in alerts], [result["alert"]])
        alert = json.loads(alerts[0].read_text(encoding="utf-8"))
        self.assertEqual((alert["owner"], alert["runbook"], alert["goal_id"], alert["status"]), ("harness-maintainers", RUNBOOK, "daily-title", "fail"))
        self.assertEqual(alert["run_sha"], env.sha)
        self.assertFalse(alert["delivery"]["sent"])
        self.assertIsNone(alert["delivery"]["channel"])
        self.assertEqual(alert["remediation"], "none (alert_only)")
        self.assertEqual(alert["receipt_sha256"], RUN._file_sha(Path(alert["receipt_path"])))
        self.assertTrue(alert["log_sha256"])
        self.assertEqual(env.repo_state(), before)

    def test_timeout_is_never_a_pass(self):
        env = Env(self, goals=[goal(timeout=2)], start={"sleep": 30})
        summary = env.run()
        self.assertEqual((summary["status"], summary["exit_code"]), ("fail", 1), summary)
        self.assertEqual(summary["results"][0]["status"], "timeout")
        self.assertEqual(len(env.alerts_written()), 1)
        self.assertGreaterEqual(env.stub.commands().count("kill"), 1)
        self.assertEqual(list((env.stub.dir / "state").glob("*.json")), [], "the timed-out container must be removed")

    def test_missing_docker_is_unavailable_never_pass(self):
        env = Env(self)
        for label, run_kwargs in (("missing binary", {"docker": str(env.root / "no-such-docker")}), ("daemon down", {})):
            with self.subTest(label=label):
                if label == "daemon down":
                    env.stub.configure(daemon_down=True)
                summary = env.run(**run_kwargs)
                self.assertEqual((summary["status"], summary["exit_code"], summary["passed"]), ("unavailable", 3, 0), summary)
                self.assertEqual(summary["results"][0]["status"], "unavailable")
        self.assertEqual(len(env.alerts_written()), 2)

    def test_absent_image_is_unavailable(self):
        env = Env(self, image_absent=True)
        summary = env.run()
        self.assertEqual((summary["status"], summary["results"][0]["status"]), ("unavailable", "unavailable"))

    def _run_with_tamper(self, env, tamper):
        real = RUN.SBX.run_isolated

        def wrapped(manifest, worktree, run_dir, **kw):
            outcome = real(manifest, worktree, run_dir, **kw)
            tamper(outcome, Path(worktree), Path(run_dir))
            return outcome

        with mock.patch.object(RUN.SBX, "run_isolated", side_effect=wrapped):
            return env.run()

    def test_missing_outcome_log_or_mismatched_hash_is_never_a_pass(self):
        cases = {
            "outcome.json deleted": lambda o, w, r: (r / "outcome.json").unlink(),
            "stdout log deleted": lambda o, w, r: Path(o.checks[0]["stdout_log"]).unlink(),
            "stderr log replaced by a symlink": lambda o, w, r: (Path(o.checks[0]["stderr_log"]).unlink(), Path(o.checks[0]["stderr_log"]).symlink_to(r / "outcome.json")),
            "log content changed after capture": lambda o, w, r: Path(o.checks[0]["stdout_log"]).write_text("forged\n", encoding="utf-8"),
            "outcome.json says failed": lambda o, w, r: (r / "outcome.json").write_text(json.dumps({"status": "failed", "run_id": o.run_id}), encoding="utf-8"),
        }
        for label, tamper in cases.items():
            with self.subTest(label=label):
                env = Env(self, start={"stdout": "ok\n"})
                summary = self._run_with_tamper(env, tamper)
                self.assertNotEqual(summary["results"][0]["status"], "pass", summary)
                self.assertEqual(summary["results"][0]["status"], "error")
                self.assertEqual(summary["exit_code"], 1)
                self.assertEqual(len(env.alerts_written()), 1)

    def test_outcome_that_did_not_run_the_approved_argv_is_never_a_pass(self):
        env = Env(self)
        real = RUN.SBX.run_isolated

        def wrapped(manifest, worktree, run_dir, **kw):
            outcome = real(manifest, worktree, run_dir, **kw)
            return RUN.SBX.RunOutcome(**{**outcome.__dict__, "argv": (("bash", "-c", "id"),)})

        with mock.patch.object(RUN.SBX, "run_isolated", side_effect=wrapped):
            summary = env.run()
        self.assertEqual(summary["results"][0]["status"], "error")
        self.assertIn("argv", summary["results"][0]["reason"])

    def test_a_check_that_changes_the_checkout_is_a_failure_even_with_exit_zero(self):
        env = Env(self)
        for label, tamper in (("new file", lambda o, w, r: (w / "dropped.txt").write_text("x", encoding="utf-8")),
                              ("edited file", lambda o, w, r: (w / "README.md").write_text("changed\n", encoding="utf-8")),
                              ("mode change", lambda o, w, r: os.chmod(w / "README.md", 0o755))):
            with self.subTest(label=label):
                summary = self._run_with_tamper(env, tamper)
                result = summary["results"][0]
                self.assertEqual(result["status"], "fail", result)
                self.assertIn("gate_mutated_checkout", result["reason"])
        self.assertEqual(env.repo_state()["status"], "")

    def test_internal_errors_are_a_visible_non_pass(self):
        env = Env(self)
        with mock.patch.object(RUN.SBX, "run_isolated", side_effect=RuntimeError("boom")):
            summary = env.run()
        self.assertEqual((summary["results"][0]["status"], summary["exit_code"]), ("error", 1))
        self.assertIn("boom", summary["results"][0]["reason"])
        self.assertEqual(len(env.alerts_written()), 1)

    def test_a_refused_sandbox_run_is_a_refusal_not_a_pass(self):
        env = Env(self)
        env.write_registry(sandbox=default_sandbox(max_total_timeout_seconds=1))
        summary = env.run()
        self.assertNotEqual(summary["status"], "pass", summary)

    def test_alerts_exist_only_for_non_pass_goals(self):
        env = Env(self, goals=[goal("good"), goal("bad", "second_check")])
        real = RUN.SBX.run_isolated
        calls = {"n": 0}

        def flaky(manifest, worktree, run_dir, **kw):
            calls["n"] += 1
            env.stub.configure(start={"exit_code": 0 if calls["n"] == 1 else 3})
            return real(manifest, worktree, run_dir, **kw)

        with mock.patch.object(RUN.SBX, "run_isolated", side_effect=flaky):
            summary = env.run()
        self.assertEqual([r["status"] for r in summary["results"]], ["pass", "fail"])
        self.assertEqual((summary["status"], summary["passed"], summary["selected"]), ("fail", 1, 2))
        alert = json.loads(env.alerts_written()[0].read_text(encoding="utf-8"))
        self.assertEqual((len(env.alerts_written()), alert["goal_id"]), (1, "bad"))

    def test_kill_switch_refuses_everything(self):
        env = Env(self)
        with mock.patch.dict(os.environ, {"ENGRAM_STANDING_CHECKS_DISABLED": "1"}):
            summary = env.run()
        self.assertEqual((summary["status"], summary["exit_code"]), ("refused", 2))
        self.assertIn("standing_checks_disabled", summary["reason"])
        self.assertEqual(env.stub.calls(), [])

    def test_unknown_ref_and_non_repo_are_refused(self):
        env = Env(self)
        self.assertEqual(env.run(ref="refs/heads/missing")["status"], "refused")
        not_repo = env.root / "plain"
        not_repo.mkdir()
        summary = env.run(repo=not_repo)
        self.assertEqual(summary["status"], "refused")

    def test_runs_root_and_alerts_dir_must_be_outside_the_repository(self):
        env = Env(self)
        for label, kwargs in (("runs root in repo", {"runs_root": env.repo / "runs"}), ("relative runs root", {"runs_root": Path("runs-rel")}),
                              ("alerts in repo", {"alerts_dir": env.repo / "alerts"})):
            with self.subTest(label=label):
                summary = env.run(**kwargs)
                self.assertEqual(summary["status"], "refused", summary)
                self.assertIn("untrusted_dir", summary["reason"])
        self.assertEqual(env.stub.calls(), [])
        self.assertEqual(env.repo_state()["status"], "")

    def test_group_writable_runs_root_is_refused(self):
        env = Env(self)
        env.runs_root.mkdir(mode=0o777)
        os.chmod(env.runs_root, 0o777)
        self.assertEqual(env.run()["status"], "refused")


class ConcurrencyTests(unittest.TestCase):
    def hold(self, env, goal_id):
        lock_dir = Path(git(env.repo, "rev-parse", "--path-format=absolute", "--git-common-dir")) / RUN.LOCK_DIR
        lock_dir.mkdir(mode=0o700, exist_ok=True)
        fd = os.open(lock_dir / f"{goal_id}.lock", os.O_RDWR | os.O_CREAT, 0o600)
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        self.addCleanup(os.close, fd)
        return lock_dir

    def test_a_concurrent_run_of_the_same_goal_is_refused_and_never_reaches_docker(self):
        env = Env(self, goals=[goal("held"), goal("free", "second_check")])
        lock_dir = self.hold(env, "held")
        summary = env.run("dispatched", None, ["held"])
        result = summary["results"][0]
        self.assertEqual((summary["status"], summary["exit_code"], result["status"]), ("refused", 2, "refused"), summary)
        self.assertIn("goal_locked", result["reason"])
        self.assertEqual(env.stub.calls(), [])
        self.assertEqual(len(env.alerts_written()), 1)
        self.assertEqual(lock_dir.parent.name, git(env.repo, "rev-parse", "--git-common-dir").split("/")[-1])
        self.assertFalse(str(lock_dir).startswith(str(env.runs_root)), "the lock lives outside the runs root")

    def test_a_different_goal_is_not_blocked_and_the_lock_is_released_after_the_run(self):
        env = Env(self, goals=[goal("held"), goal("free", "second_check")])
        self.hold(env, "held")
        free = env.run("dispatched", None, ["free"])
        self.assertEqual(free["status"], "pass", free)
        again = env.run("dispatched", None, ["free"])
        self.assertEqual(again["status"], "pass", "the lock must be released when a run ends")

    def test_the_lock_is_released_after_a_failing_run(self):
        env = Env(self, start={"exit_code": 1})
        self.assertEqual(env.run()["status"], "fail")
        env.stub.configure(start={"exit_code": 0})
        self.assertEqual(env.run()["status"], "pass")

    def test_the_lock_is_not_reachable_from_the_checkout_mounted_into_the_container(self):
        env = Env(self)
        summary = env.run()
        checkout = Path(summary["results"][0]["receipt"]).parent / "checkout"
        self.assertEqual(sorted(p.name for p in checkout.iterdir()), ["README.md", "docs"])
        self.assertFalse(any(RUN.LOCK_DIR in str(p) for p in checkout.rglob("*")))


class CliTests(unittest.TestCase):
    def cli(self, env, *extra):
        cmd = [sys.executable, str(RUNNER_PATH), "run", "--repo", str(env.repo), "--runs-root", str(env.runs_root), "--goals", str(env.goals_path),
               "--schema", str(PROD_SCHEMA), "--registry", str(env.registry_path), "--catalog", str(env.catalog_path), "--docker", str(env.stub.path), *extra]
        return subprocess.run(cmd, capture_output=True, text=True, timeout=120)

    def test_exit_codes_and_final_line(self):
        env = Env(self)
        ok = self.cli(env, "--mode", "scheduled", "--schedule", "daily")
        self.assertEqual(ok.returncode, 0, ok.stderr)
        self.assertRegex(ok.stdout.splitlines()[-1], r"^STANDING_CHECKS: PASS mode=scheduled selected=1 passed=1 summary=")
        env.stub.configure(start={"exit_code": 1})
        bad = self.cli(env, "--mode", "dispatched", "--goal", "daily-title")
        self.assertEqual(bad.returncode, 1)
        self.assertRegex(bad.stdout.splitlines()[-1], r"^STANDING_CHECKS: FAIL mode=dispatched selected=1 passed=0")
        self.assertIn("ALERT: goal=daily-title status=fail", bad.stderr)
        usage = self.cli(env, "--mode", "scheduled", "--goal", "daily-title")
        self.assertEqual(usage.returncode, 2)
        self.assertRegex(usage.stdout.splitlines()[-1], r"^STANDING_CHECKS: REFUSED")


class WorkflowContractTests(unittest.TestCase):
    def setUp(self):
        self.text = WORKFLOW.read_text(encoding="utf-8")
        self.code = "\n".join(line for line in self.text.splitlines() if not line.lstrip().startswith("#"))

    def test_triggers_are_only_schedule_and_workflow_dispatch(self):
        on_block = re.search(r"^on:\n((?:[ ]{2}.*\n|\n)+)", self.code, re.M)
        self.assertIsNotNone(on_block)
        self.assertEqual(sorted(re.findall(r"^  ([a-z_]+):", on_block.group(1), re.M)), ["schedule", "workflow_dispatch"])
        self.assertNotRegex(self.code, r"pull_request|push:|workflow_run|issue_comment")

    def test_permissions_are_read_only_without_secrets_or_write_commands(self):
        self.assertRegex(self.code, r"(?m)^permissions:\n  contents: read\n(?!  )")
        self.assertEqual(self.code.count("permissions:"), 1, "no job-level permission override")
        self.assertNotRegex(self.code, r"secrets\.|: write|write-all|id-token|GH_TOKEN|GITHUB_TOKEN")
        self.assertNotRegex(self.code, r"git (commit|push|tag|am|apply)|gh (pr|issue|api|release)|curl |wget |slack|webhook", re.I)
        self.assertNotRegex(self.code, r"--writer|fake_writer|run-task\.py|merge-gate|auto-merge")

    def test_every_action_is_pinned_by_sha_and_credentials_are_not_persisted(self):
        uses = re.findall(r"uses:\s*(\S+)", self.code)
        self.assertTrue(uses)
        for ref in uses:
            self.assertRegex(ref, r"^[^@\s]+@[0-9a-f]{40}$")
        self.assertIn("persist-credentials: false", self.code)
        self.assertEqual(sorted({u.split("@")[0] for u in uses}), ["actions/checkout", "actions/upload-artifact"])

    def test_it_runs_the_alert_only_runner_and_uploads_reports_only(self):
        self.assertIn("docs/harness/bin/run-standing-checks.py run --mode scheduled --schedule daily", self.code)
        self.assertIn("--mode dispatched", self.code)
        self.assertNotIn("${{ github.event.inputs.goal }}", re.sub(r"GOAL_INPUT: \$\{\{ github\.event\.inputs\.goal \}\}", "", self.code), "the dispatch input reaches the shell only through env")
        self.assertIn("if: always()", self.code)
        self.assertIn("!${{ runner.temp }}/standing-checks/goals/*/*/checkout/", self.code, "exported checkouts are not uploaded")

    def test_the_workflow_passes_the_supply_chain_check(self):
        proc = subprocess.run([sys.executable, str(SUPPLY_CHAIN)], capture_output=True, text=True, cwd=REPO_ROOT, timeout=120)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("PASS", proc.stdout)


if __name__ == "__main__":
    unittest.main()
