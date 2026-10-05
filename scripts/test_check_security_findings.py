#!/usr/bin/env python3
"""Tests for the mechanical security-findings policy (task Q5).

Every scenario in ``tests/fixtures/security_findings/matrix.json`` is run through the real CLI.
The policy must be fail-closed: a scanner exit code of 0 is not absence of findings, a SARIF
upload is not a clean scan, payload self-declarations (SARIF suppressions, embedded SHAs) are
never authority, and a skip is only neutral when the supervisor explicitly allowed it.

Missing files or tools are failures, never skips.
"""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-security-findings.py"
FIXTURES = ROOT / "tests" / "fixtures" / "security_findings"
MATRIX = FIXTURES / "matrix.json"

REQUIRED_CASES = {
    "clean",
    "high-finding-exit0",
    "self-declaration",
    "approved-exception",
    "expired-exception",
    "invalid-exception",
    "sarif-missing",
    "sarif-stale",
    "sarif-malformed",
    "wrong-tool",
    "scanner-not-run",
    "scanner-failed",
    "allowed-skip",
}


def load_matrix() -> dict:
    return json.loads(MATRIX.read_text())


def run_cli(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(CHECKER), *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=60,
    )


def scenario_args(matrix: dict, scenario: dict) -> list[str]:
    args = [
        "--scanner",
        matrix["scanner"],
        "--tool",
        matrix["tool"],
        "--sarif",
        str(FIXTURES / scenario["sarif"]) if scenario["sarif"] else str(FIXTURES / "unset.sarif"),
        "--expected-sha",
        matrix["expected_sha"],
        "--scanner-exit",
        scenario["scanner_exit"],
        "--today",
        matrix["today"],
    ]
    if scenario["exceptions"]:
        args += ["--exceptions", str(FIXTURES / scenario["exceptions"])]
    if scenario["allowed_skip"]:
        args += ["--allowed-skip", scenario["allowed_skip"]]
    return args


class MatrixTests(unittest.TestCase):
    def test_matrix_covers_every_required_case(self) -> None:
        cases = {scenario["case"] for scenario in load_matrix()["scenarios"]}
        self.assertEqual(REQUIRED_CASES - cases, set(), "matrix lost required cases")

    def test_every_scenario_decides_as_declared(self) -> None:
        matrix = load_matrix()
        self.assertGreaterEqual(len(matrix["scenarios"]), 20)
        for scenario in matrix["scenarios"]:
            with self.subTest(scenario["name"]):
                proc = run_cli(scenario_args(matrix, scenario) + ["--json"])
                verdict = json.loads(proc.stdout)["verdict"]
                self.assertEqual(
                    verdict, scenario["expected"], f"{proc.stdout}\n{proc.stderr}"
                )
                self.assertEqual(proc.returncode, 1 if scenario["expected"] == "block" else 0)

    def test_neutral_is_reported_as_neutral_never_pass(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["expected"] == "neutral")
        proc = run_cli(scenario_args(matrix, scenario))
        self.assertEqual(proc.returncode, 0)
        self.assertIn("NEUTRAL", proc.stdout)
        self.assertNotIn("PASS", proc.stdout)

    def test_block_names_a_reason(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["case"] == "high-finding-exit0")
        proc = run_cli(scenario_args(matrix, scenario) + ["--json"])
        reasons = json.loads(proc.stdout)["reasons"]
        self.assertTrue(any("high" in reason for reason in reasons), reasons)


class DiagnosticsTests(unittest.TestCase):
    def test_missing_provenance_block_carries_a_fix_hint(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["name"] == "no-provenance-blocks")
        proc = run_cli(scenario_args(matrix, scenario) + ["--json"])
        reasons = " ".join(json.loads(proc.stdout)["reasons"])
        self.assertIn("versionControlProvenance", reasons)
        self.assertIn("codeql-security-sarif", reasons)


class RepositoryExceptionsTests(unittest.TestCase):
    """The governed file CI passes to the policy must parse and must not approve anything yet."""

    REPO_EXCEPTIONS = ROOT / "docs" / "security" / "finding-exceptions.toml"

    def test_repository_exceptions_file_is_valid_and_empty(self) -> None:
        self.assertTrue(self.REPO_EXCEPTIONS.is_file())
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["name"] == "clean-passes")
        args = scenario_args(matrix, scenario) + ["--exceptions", str(self.REPO_EXCEPTIONS)]
        self.assertEqual(run_cli(args).returncode, 0)

    def test_repository_exceptions_file_approves_no_high_finding(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["case"] == "high-finding-exit0")
        args = scenario_args(matrix, scenario) + ["--exceptions", str(self.REPO_EXCEPTIONS)]
        self.assertEqual(run_cli(args).returncode, 1)


class IdentityTests(unittest.TestCase):
    def test_expected_sha_must_be_a_full_hex_sha(self) -> None:
        matrix = load_matrix()
        scenario = matrix["scenarios"][0]
        args = scenario_args(matrix, scenario)
        args[args.index("--expected-sha") + 1] = "HEAD"
        proc = run_cli(args)
        self.assertEqual(proc.returncode, 2, proc.stderr)

    def test_identity_comes_from_the_cli_not_the_payload(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["name"] == "clean-passes")
        args = scenario_args(matrix, scenario)
        args[args.index("--expected-sha") + 1] = "f" * 40
        proc = run_cli(args + ["--json"])
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(json.loads(proc.stdout)["verdict"], "block")

    def test_required_arguments_are_not_defaulted(self) -> None:
        for flag in ("--scanner-exit", "--expected-sha", "--sarif", "--scanner", "--tool"):
            with self.subTest(flag):
                matrix = load_matrix()
                args = scenario_args(matrix, matrix["scenarios"][0])
                index = args.index(flag)
                del args[index : index + 2]
                self.assertEqual(run_cli(args).returncode, 2)

    def test_invalid_scanner_exit_value_is_a_usage_error(self) -> None:
        matrix = load_matrix()
        args = scenario_args(matrix, matrix["scenarios"][0])
        args[args.index("--scanner-exit") + 1] = "maybe"
        self.assertEqual(run_cli(args).returncode, 2)

    def test_empty_allowed_skip_reason_is_not_authorization(self) -> None:
        matrix = load_matrix()
        scenario = next(s for s in matrix["scenarios"] if s["case"] == "scanner-not-run")
        args = scenario_args(matrix, scenario) + ["--allowed-skip", "  "]
        proc = run_cli(args + ["--json"])
        self.assertEqual(json.loads(proc.stdout)["verdict"], "block")


if __name__ == "__main__":
    unittest.main(verbosity=2)
