#!/usr/bin/env python3
"""Real-Docker end-to-end smoke of the trusted runner with the FAKE writer (task H4).

NOT part of the mandatory offline lane: it needs a Docker daemon and the pinned image of the
production registry. Run it through docs/harness/bin/run-runner-smoke.sh, which reports a distinct
UNAVAILABLE state (exit 3, never a pass) when Docker or the image is missing. Run directly, an
unavailable environment makes the whole module SKIP, which is likewise not a pass.

The production registry and check catalog are used unchanged: the gate runs pr_title_policy from a clean
checkout of the candidate commit inside the hardened container; the fake writer edits its own plain
workspace inside another container. Nothing is executed on the host.

Usage:  python3 docs/harness/tests/test_runner_smoke.py --preflight   # exit 0 ready, 3 unavailable
"""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from runner_test_support import BASE_TREE, REPO_ROOT, TempRepo, load_module, run_git  # noqa: E402

SBX = load_module("sandbox-adapter")
RUNNER = load_module("run-task")
REC = load_module("record-evidence")
PROD_REGISTRY = REPO_ROOT / "docs" / "harness" / "checks" / "registry.json"
PROD_CATALOG = REPO_ROOT / "docs" / "harness" / "schemas" / "check-catalog-v1.json"
REAL_IMAGE = json.loads(PROD_REGISTRY.read_text(encoding="utf-8"))["sandbox"]["image"]
POLICY_SCRIPT = REPO_ROOT / "docs" / "harness" / "bin" / "pr-title-policy.sh"


def preflight():
    try:
        SBX.check_image(SBX.connect_docker("docker"), REAL_IMAGE)
    except SBX.Unavailable as exc:
        return False, str(exc)
    return True, "ok"


def setUpModule():
    ready, reason = preflight()
    if not ready:
        print(f"RUNNER_SMOKE_STATE=UNAVAILABLE reason={reason}", file=sys.stderr)
        raise unittest.SkipTest(f"UNAVAILABLE (not a pass): {reason}")


def containers_left(run_dir: Path):
    ids = []
    outcomes = [*run_dir.glob("a*/w[0-9]*/outcome.json"), *run_dir.glob("a*/g[0-9]*/run/outcome.json")]  # never ws/
    for outcome in outcomes:
        run_id = json.loads(outcome.read_text())["run_id"]
        proc = subprocess.run(["docker", "ps", "-aq", "--filter", f"label={SBX.RUN_LABEL}={run_id}"],
                              capture_output=True, text=True, timeout=30, check=False)
        ids += proc.stdout.split()
    return ids


class RunnerSmoke(unittest.TestCase):
    def setUp(self):
        self.repo = TempRepo(self, prefix="h4-smoke-")
        files = dict(BASE_TREE)
        files["docs/harness/bin/pr-title-policy.sh"] = ("exec", POLICY_SCRIPT.read_bytes())
        self.base = self.repo.commit(files)
        self.repo.checkout(self.base)
        self.runs_root = self.repo.root / "runs"

    def run_task(self, invocations, **over):
        request = {"request_version": "runner-request-v1", "task_id": "h4-smoke", "base_sha": self.base,
                   "policy_version": "harness-hardening-v1",
                   "writer": {"adapter": "fake", "invocations": invocations},
                   "required_checks": ["pr_title_policy"], "allowed_paths": ["src/"],
                   "budgets": {"wall_seconds": 600}}
        request.update(over)
        path = self.repo.root / "request.json"
        path.write_text(json.dumps(request), encoding="utf-8")
        summary = RUNNER.run_task(repo=self.repo.path, request_path=path, runs_root=self.runs_root,
                                  registry_path=PROD_REGISTRY, catalog_path=PROD_CATALOG)
        if summary.get("run_dir"):
            self.assertEqual(containers_left(Path(summary["run_dir"])), [], "a container of the run survived")
        return summary

    def verify(self, summary):
        return REC.verify(repo=self.repo.path, runs_root=self.runs_root, receipt_path=Path(summary["result"]["receipt"]),
                          expect_candidate=summary["result"]["candidate"]["sha"],
                          registry_path=PROD_REGISTRY, catalog_path=PROD_CATALOG)

    def test_fake_writer_edit_passes_the_production_gate_with_verified_evidence(self):
        s = self.run_task([["write_file", "src/new.txt", "hello from the sandbox\n"]])
        self.assertEqual(s["status"], "passed", json.dumps(s, indent=1))
        cand = s["result"]["candidate"]["sha"]
        self.assertEqual(run_git(self.repo.path, "show", cand + ":src/new.txt"), b"hello from the sandbox\n")
        self.assertEqual(self.verify(s)["status"], "verified")
        gate_root = Path(s["result"]["receipt"]).parent
        log = next((gate_root / "run" / "logs").glob("*.stdout.log")).read_text()
        self.assertIn("OK: PR title policy", log)
        record = json.loads((gate_root / "run-record.json").read_text())
        self.assertTrue(record["sandbox_outcome"]["limits_enforced"]["verified"])
        self.assertEqual(record["sandbox_outcome"]["limits_enforced"]["network_mode"], "none")

    def test_writer_claimed_pass_is_refused_by_scope_with_no_candidate(self):
        s = self.run_task([["claim_pass"]])
        self.assertEqual(s["status"], "scope_refused", json.dumps(s, indent=1))
        self.assertEqual(run_git(self.repo.path, "for-each-ref", "refs/engram-runner/"), b"")

    def test_failing_writer_exhausts_attempts_and_keeps_logs(self):
        s = self.run_task([["exit_code", "3"]])
        self.assertEqual(s["status"], "failed", json.dumps(s, indent=1))
        self.assertTrue(s["reason"].startswith("attempts_exhausted"), s["reason"])
        for attempt in ("a1", "a2"):
            self.assertTrue((Path(s["run_dir"]) / attempt / "w0" / "outcome.json").exists())

    def test_one_byte_changed_in_a_gate_log_is_refused(self):
        s = self.run_task([["write_file", "src/b.txt", "b\n"]])
        self.assertEqual(s["status"], "passed", json.dumps(s, indent=1))
        log = next((Path(s["result"]["receipt"]).parent / "run" / "logs").glob("*.stdout.log"))
        log.write_bytes(log.read_bytes().replace(b"OK", b"0K", 1))
        with self.assertRaises(REC.VerifyRefused):
            self.verify(s)


if __name__ == "__main__":
    if sys.argv[1:] == ["--preflight"]:
        ok, why = preflight()
        print(f"RUNNER_SMOKE_PREFLIGHT: {'READY' if ok else 'UNAVAILABLE'} reason={why}")
        sys.exit(0 if ok else 3)
    unittest.main(verbosity=2)
