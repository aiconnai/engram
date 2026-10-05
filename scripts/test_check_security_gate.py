#!/usr/bin/env python3
"""Contract tests for the aggregate Security Gate checker (task Q5).

They pin the aggregate decision (pass / neutral / block), the matrix coverage, the runtime
``--results-json`` enforcement used by the ``security-gate`` job, the existing
``--self-test-failure`` / ``--self-test-unrequired`` modes, and the required-context chain:
removing ``security-gate`` from the ``needs`` of the required ``test`` job must fail the checker.

Missing files are failures, never skips.
"""

from __future__ import annotations

import copy
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-security-gate.py"
MATRIX = ROOT / "tests" / "fixtures" / "security_gate_matrix.json"
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"

LIVE_CONTEXTS = {
    "contexts": [
        "Format",
        "Clippy",
        "Documentation",
        "Test (ubuntu-latest)",
        "Security Audit",
        "Cargo Deny",
    ]
}
REQUIRED_CASES = {
    "all-pass",
    "constituent-failure",
    "cancelled",
    "timed-out",
    "missing",
    "unauthorized-skip",
    "allowed-skip-neutral",
}

SPEC = importlib.util.spec_from_file_location("check_security_gate", CHECKER)
assert SPEC is not None and SPEC.loader is not None
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def load_matrix() -> dict:
    return json.loads(MATRIX.read_text())


def success_payload(matrix: dict) -> dict:
    return {job: {"result": "success"} for job in matrix["constituents"]}


def run_cli(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(CHECKER), *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=60,
    )


def run_runtime(payload: dict, event: str = "pull_request") -> subprocess.CompletedProcess:
    return run_cli(
        [
            "--matrix",
            str(MATRIX),
            "--workflow",
            str(WORKFLOW),
            "--results-json",
            json.dumps(payload),
            "--event",
            event,
        ]
    )


class MatrixTests(unittest.TestCase):
    def test_matrix_covers_every_required_case(self) -> None:
        cases = {scenario.get("case") for scenario in load_matrix()["scenarios"]}
        self.assertEqual(REQUIRED_CASES - cases, set())

    def test_every_scenario_matches_the_declared_verdict(self) -> None:
        matrix = load_matrix()
        for scenario in matrix["scenarios"]:
            with self.subTest(scenario["name"]):
                allowed = set(scenario.get("allowed_skips", []))
                verdict, _ = GATE.verdict(scenario["results"], matrix["constituents"], allowed)
                self.assertEqual(verdict, scenario["expected"])

    def test_allowed_skip_is_neutral_not_pass(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["case"] == "allowed-skip-neutral")
        verdict, reasons = GATE.verdict(
            scenario["results"], matrix["constituents"], set(scenario["allowed_skips"])
        )
        self.assertEqual(verdict, "neutral")
        self.assertTrue(any("semgrep-security" in reason for reason in reasons))

    def test_scenario_cannot_invent_skip_authority(self) -> None:
        matrix = load_matrix()
        mutated = copy.deepcopy(matrix)
        scenario = next(s for s in mutated["scenarios"] if s["case"] == "allowed-skip-neutral")
        scenario["allowed_skips"] = ["audit"]
        scenario["results"]["audit"] = "skipped"
        with self.assertRaises(GATE.CheckError):
            GATE.validate_matrix(mutated)

    def test_pull_request_may_never_allow_skips(self) -> None:
        mutated = load_matrix()
        mutated["allowed_skips_by_event"]["pull_request"] = ["semgrep-security"]
        with self.assertRaises(GATE.CheckError):
            GATE.validate_matrix(mutated)

    def test_losing_a_required_case_is_rejected(self) -> None:
        mutated = load_matrix()
        mutated["scenarios"] = [s for s in mutated["scenarios"] if s["case"] != "cancelled"]
        with self.assertRaises(GATE.CheckError):
            GATE.validate_matrix(mutated)

    def test_wrong_declared_verdict_is_rejected(self) -> None:
        mutated = load_matrix()
        scenario = next(s for s in mutated["scenarios"] if s["case"] == "unauthorized-skip")
        scenario["expected"] = "pass"
        with self.assertRaises(GATE.CheckError):
            GATE.validate_matrix(mutated)


class RuntimeTests(unittest.TestCase):
    def test_all_success_reports_pass(self) -> None:
        proc = run_runtime(success_payload(load_matrix()))
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("PASS", proc.stdout)
        self.assertNotIn("NEUTRAL", proc.stdout)

    def test_each_bad_state_for_each_constituent_blocks(self) -> None:
        matrix = load_matrix()
        for job in matrix["constituents"]:
            for state in ("failure", "cancelled", "timed_out", "skipped", "bogus"):
                with self.subTest(job=job, state=state):
                    payload = success_payload(matrix)
                    payload[job] = {"result": state}
                    proc = run_runtime(payload)
                    self.assertEqual(proc.returncode, 1, proc.stdout)

    def test_missing_constituent_blocks(self) -> None:
        matrix = load_matrix()
        for job in matrix["constituents"]:
            with self.subTest(job=job):
                payload = success_payload(matrix)
                del payload[job]
                self.assertEqual(run_runtime(payload).returncode, 1)

    def test_malformed_constituent_entry_blocks(self) -> None:
        payload = success_payload(load_matrix())
        payload["audit"] = "success"
        self.assertEqual(run_runtime(payload).returncode, 1)

    def test_skip_is_neutral_only_when_the_event_allows_it(self) -> None:
        payload = success_payload(load_matrix())
        payload["semgrep-security"] = {"result": "skipped"}
        blocked = run_runtime(payload, event="pull_request")
        self.assertEqual(blocked.returncode, 1)
        neutral = run_runtime(payload, event="not_applicable_fixture")
        self.assertEqual(neutral.returncode, 0, neutral.stderr)
        self.assertIn("NEUTRAL", neutral.stdout)
        self.assertNotIn("PASS", neutral.stdout)

    def test_non_object_results_are_rejected(self) -> None:
        proc = run_cli(["--matrix", str(MATRIX), "--results-json", "[]"])
        self.assertEqual(proc.returncode, 1)


class SelfTestModes(unittest.TestCase):
    def test_failure_self_test_passes(self) -> None:
        proc = run_cli(["--matrix", str(MATRIX), "--self-test-failure"])
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("failure self-test: PASS", proc.stdout)

    def test_unrequired_self_test_passes(self) -> None:
        proc = run_cli(["--matrix", str(MATRIX), "--self-test-unrequired"])
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("unrequired-context self-test: PASS", proc.stdout)

    def test_live_required_contexts_contract_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            contexts = Path(tmp) / "contexts.json"
            contexts.write_text(json.dumps(LIVE_CONTEXTS))
            proc = run_cli(["--matrix", str(MATRIX), "--required-contexts", str(contexts)])
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("security-gate contract: PASS", proc.stdout)


class WorkflowChainTests(unittest.TestCase):
    def run_with_workflow(self, text: str, *extra: str) -> subprocess.CompletedProcess:
        with tempfile.TemporaryDirectory() as tmp:
            workflow = Path(tmp) / "ci.yml"
            workflow.write_text(text)
            contexts = Path(tmp) / "contexts.json"
            contexts.write_text(json.dumps(LIVE_CONTEXTS))
            return run_cli(
                [
                    "--matrix",
                    str(MATRIX),
                    "--workflow",
                    str(workflow),
                    "--required-contexts",
                    str(contexts),
                    *extra,
                ]
            )

    def test_unmodified_workflow_passes(self) -> None:
        proc = self.run_with_workflow(WORKFLOW.read_text())
        self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_removing_security_gate_from_required_test_job_fails(self) -> None:
        text = WORKFLOW.read_text()
        needle = "needs: [fmt, clippy, security-gate]"
        self.assertEqual(text.count(needle), 1, "ci.yml no longer has the expected needs line")
        proc = self.run_with_workflow(text.replace(needle, "needs: [fmt, clippy]"))
        self.assertEqual(proc.returncode, 1)
        self.assertIn("security-gate", proc.stderr)

    def test_removing_the_dependency_also_fails_every_self_test_mode(self) -> None:
        text = WORKFLOW.read_text().replace(
            "needs: [fmt, clippy, security-gate]", "needs: [fmt, clippy]"
        )
        for mode in ("--self-test-failure", "--self-test-unrequired"):
            with self.subTest(mode=mode):
                self.assertEqual(self.run_with_workflow(text, mode).returncode, 1)

    def test_dropping_a_constituent_from_the_aggregate_fails(self) -> None:
        text = WORKFLOW.read_text()
        needle = "      - agentshield-security\n"
        self.assertEqual(text.count(needle), 1)
        self.assertEqual(self.run_with_workflow(text.replace(needle, "")).returncode, 1)

    def test_aggregate_must_run_always(self) -> None:
        text = WORKFLOW.read_text()
        needle = "    if: always()\n    needs:\n      - audit"
        self.assertEqual(text.count(needle), 1)
        mutated = text.replace(needle, "    needs:\n      - audit")
        self.assertEqual(self.run_with_workflow(mutated).returncode, 1)

    def test_no_live_required_context_reaching_the_gate_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            contexts = Path(tmp) / "contexts.json"
            contexts.write_text(json.dumps({"contexts": ["Format", "Clippy"]}))
            proc = run_cli(["--matrix", str(MATRIX), "--required-contexts", str(contexts)])
        self.assertEqual(proc.returncode, 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
