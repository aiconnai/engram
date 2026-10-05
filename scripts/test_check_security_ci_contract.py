#!/usr/bin/env python3
"""Contract tests that pin the security wiring of ``.github/workflows/ci.yml`` (task Q5).

Without these, deleting the CodeQL findings-policy step, or editing its identity arguments, would
silently revert the gate to "scanner exit 0 means clean". They run in the ``security-gate`` job
next to the other security contract tests.

Missing files, jobs or steps are failures, never skips.
"""

from __future__ import annotations

import re
import shlex
import unittest
from pathlib import Path
from typing import Dict, List, Optional

ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
FINDINGS = "scripts/check-security-findings.py"

EXPECTED_FINDINGS_ARGV = {
    "--scanner": "codeql",
    "--tool": "CodeQL",
    "--sarif": "sarif-results/rust.sarif",
    "--expected-sha": "${{ github.sha }}",
    "--scanner-exit": "0",
    "--exceptions": "docs/security/finding-exceptions.toml",
    "--checkout-dir": ".",
}
GATE_COMMANDS = (
    "python3 scripts/test_check_security_gate.py",
    "python3 scripts/test_check_security_findings.py",
    "python3 scripts/test_check_workflow_supply_chain.py",
    "python3 scripts/test_check_security_ci_contract.py",
    "python3 scripts/test_check_security_exceptions.py",
    "python3 scripts/check-workflow-supply-chain.py",
)


def job_block(text: str, job: str) -> str:
    match = re.search(rf"^  {re.escape(job)}:\s*$", text, re.MULTILINE)
    if match is None:
        raise AssertionError(f"job {job} not found in ci.yml")
    rest = text[match.end() :]
    following = re.search(r"^  [A-Za-z0-9_-]+:\s*$", rest, re.MULTILINE)
    return rest[: following.start()] if following else rest


def steps(block: str) -> List[str]:
    parts = re.split(r"^      - ", block, flags=re.MULTILINE)
    return ["      - " + part for part in parts[1:]]


def run_text(step: str) -> Optional[str]:
    """Return the step's `run:` command (folded or literal blocks joined), or None."""
    lines = step.splitlines()
    for index, line in enumerate(lines):
        match = re.match(r"^\s+(?:- )?run:\s*(.*)$", line)
        if not match:
            continue
        inline = match.group(1).strip()
        if inline and inline not in (">-", ">", "|", "|-"):
            return inline
        folded = inline.startswith(">")
        body: List[str] = []
        for follower in lines[index + 1 :]:
            if follower.strip() and not follower.startswith("          "):
                break
            body.append(follower.strip())
        return (" " if folded else "\n").join(part for part in body if part)
    return None


def argv_of(step: str) -> List[str]:
    command = run_text(step)
    assert command is not None
    return shlex.split(command.replace("\\\n", " "))


def argv_pairs(argv: List[str]) -> Dict[str, str]:
    return {flag: argv[i + 1] for i, flag in enumerate(argv[:-1]) if flag.startswith("--")}


class CodeqlFindingsPolicyPin(unittest.TestCase):
    def setUp(self) -> None:
        self.text = WORKFLOW.read_text()
        self.block = job_block(self.text, "codeql-security")
        self.steps = steps(self.block)

    def findings_steps(self) -> List[str]:
        return [step for step in self.steps if FINDINGS in (run_text(step) or "")]

    def test_exactly_one_findings_policy_step(self) -> None:
        self.assertEqual(len(self.findings_steps()), 1, "CodeQL findings-policy step missing or duplicated")

    def test_findings_policy_argv_is_pinned(self) -> None:
        argv = argv_of(self.findings_steps()[0])
        self.assertEqual(argv[:2], ["python3", FINDINGS])
        pairs = argv_pairs(argv)
        for flag, value in EXPECTED_FINDINGS_ARGV.items():
            with self.subTest(flag):
                self.assertEqual(pairs.get(flag), value)
        self.assertEqual(set(pairs) - set(EXPECTED_FINDINGS_ARGV), set(), "unexpected extra flags")

    def test_findings_policy_step_cannot_be_skipped_or_softened(self) -> None:
        step = self.findings_steps()[0]
        self.assertIsNone(re.search(r"^\s+if:", step, re.MULTILINE), "findings step is conditional")
        self.assertNotIn("continue-on-error", self.block)
        self.assertNotIn("|| true", self.block)

    def test_analyze_step_does_not_upload_and_writes_sarif_where_the_policy_reads_it(self) -> None:
        analyze = [s for s in self.steps if "codeql-action/analyze@" in s]
        self.assertEqual(len(analyze), 1)
        self.assertIn("upload: never", analyze[0])
        self.assertIn("output: sarif-results", analyze[0])

    def test_findings_policy_runs_after_analyze(self) -> None:
        indexes = {
            "analyze": next(i for i, s in enumerate(self.steps) if "codeql-action/analyze@" in s),
            "policy": next(i for i, s in enumerate(self.steps) if FINDINGS in (run_text(s) or "")),
        }
        self.assertLess(indexes["analyze"], indexes["policy"])

    def test_job_is_read_only(self) -> None:
        self.assertIsNone(re.search(r":\s*write\b", self.block), "codeql-security has a write permission")
        self.assertIn("contents: read", self.block)

    def test_sarif_is_retained(self) -> None:
        self.assertIn("name: codeql-security-sarif", self.block)
        self.assertIn("retention-days: 30", self.block)


class ExitCodeScannerPins(unittest.TestCase):
    """Semgrep and Gitleaks have no findings step: their exit code IS the policy, so pin it."""

    def setUp(self) -> None:
        self.text = WORKFLOW.read_text()

    def test_semgrep_fails_on_any_finding(self) -> None:
        block = job_block(self.text, "semgrep-security")
        self.assertIn("semgrep --config p/ci --error", block)
        self.assertNotIn("|| true", block)
        self.assertNotIn("continue-on-error", block)

    def test_gitleaks_fails_on_any_leak_and_is_digest_pinned(self) -> None:
        block = job_block(self.text, "gitleaks-security")
        self.assertIn("--exit-code 1", block)
        self.assertRegex(block, r"zricethezav/gitleaks:v[0-9.]+@sha256:[0-9a-f]{64}")
        self.assertNotIn("|| true", block)
        self.assertNotIn("continue-on-error", block)

    def test_agentshield_fails_on_high(self) -> None:
        block = job_block(self.text, "agentshield-security")
        self.assertIn("fail-on: high", block)
        self.assertNotIn("continue-on-error", block)


class SecurityGateJobPins(unittest.TestCase):
    def setUp(self) -> None:
        self.block = job_block(WORKFLOW.read_text(), "security-gate")

    def test_gate_job_runs_every_security_contract_command(self) -> None:
        commands = "\n".join(filter(None, (run_text(step) for step in steps(self.block))))
        for command in GATE_COMMANDS:
            with self.subTest(command):
                self.assertIn(command, commands)

    def test_gate_job_still_enforces_the_runtime_matrix(self) -> None:
        self.assertIn("scripts/check-security-gate.py", self.block)
        self.assertIn("--results-json", self.block)
        self.assertIn("if: always()", self.block)

    def test_aggregate_depends_on_codeql(self) -> None:
        self.assertRegex(self.block, r"(?m)^\s+- codeql-security\s*$")

    def test_no_continue_on_error_in_security_jobs(self) -> None:
        for job in ("audit", "deny", "security-exceptions", "security-gate"):
            with self.subTest(job):
                self.assertNotIn("continue-on-error", job_block(WORKFLOW.read_text(), job))


class PinSelfCheck(unittest.TestCase):
    """The pin must actually detect the regressions it exists for."""

    def mutated(self, old: str, new: str) -> str:
        text = WORKFLOW.read_text()
        self.assertIn(old, text)
        return text.replace(old, new)

    def policy_steps(self, text: str) -> List[str]:
        return [s for s in steps(job_block(text, "codeql-security")) if FINDINGS in (run_text(s) or "")]

    def test_deleting_the_step_is_detected(self) -> None:
        text = self.mutated("--exceptions docs/security/finding-exceptions.toml", "--exceptions x")
        self.assertNotEqual(
            argv_pairs(argv_of(self.policy_steps(text)[0]))["--exceptions"],
            EXPECTED_FINDINGS_ARGV["--exceptions"],
        )
        text = self.mutated("python3 scripts/check-security-findings.py", "python3 scripts/other.py")
        self.assertEqual(self.policy_steps(text), [])

    def test_changing_identity_args_is_detected(self) -> None:
        text = self.mutated('--expected-sha "${{ github.sha }}"', "--expected-sha deadbeef")
        self.assertNotEqual(
            argv_pairs(argv_of(self.policy_steps(text)[0]))["--expected-sha"],
            EXPECTED_FINDINGS_ARGV["--expected-sha"],
        )
        text = self.mutated("--scanner-exit 0", "--scanner-exit not-run")
        self.assertNotEqual(
            argv_pairs(argv_of(self.policy_steps(text)[0]))["--scanner-exit"],
            EXPECTED_FINDINGS_ARGV["--scanner-exit"],
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
