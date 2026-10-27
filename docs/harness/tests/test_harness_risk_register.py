#!/usr/bin/env python3
"""Regression tests for docs/harness/bin/harness-risk-register.sh.

Covered:
  * next_risk_id reads zero-padded ids as decimal (RISK-0008 used to fail as octal);
  * validate accepts unquoted dates/timestamps under Psych 4+ (permitted_classes);
  * validate still rejects a wrong score, so the Psych fix did not disable validation.

Offline, stdlib only. Every write goes to a temporary file via HARNESS_RISK_FILE or --file;
the committed register is only read.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPT = REPO_ROOT / "docs" / "harness" / "bin" / "harness-risk-register.sh"
COMMITTED_REGISTER = REPO_ROOT / "docs" / "harness" / "risk-register.yaml"

HEADER = (
    "schema_version: harness-risk-register-v1\n"
    "updated_at: 2026-10-26T00:00:00Z\n"
    "entries:\n"
)


def _entry(risk_id: str, *, score: int = 1, date: str = "2026-10-01") -> str:
    return (
        f"\n  - id: {risk_id}\n"
        f"    date: {date}\n"
        "    status: open\n"
        '    description: "fixture"\n'
        "    probability: 1\n"
        "    impact: 1\n"
        f"    score: {score}\n"
        '    owner: "@team"\n'
        '    mitigation: "m"\n'
        '    contingency: ""\n'
        "    monitoring: []\n"
    )


def _run(args: list[str], *, env_file: Path | None = None) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    if env_file is not None:
        env["HARNESS_RISK_FILE"] = str(env_file)
    return subprocess.run(
        ["bash", str(SCRIPT), *args],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


@unittest.skipUnless(shutil.which("bash"), "bash is required")
class NextRiskIdTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.register = Path(self._tmp.name) / "risk-register.yaml"

    def _write(self, ids: list[str]) -> None:
        self.register.write_text(HEADER + "".join(_entry(i) for i in ids), encoding="utf-8")

    def test_next_id_after_zero_padded_eight_is_nine(self) -> None:
        self._write(["RISK-0001", "RISK-0007", "RISK-0008"])
        result = _run(["add", "--description", "d", "--probability", "1", "--impact", "1",
                       "--mitigation", "m"], env_file=self.register)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Added risk RISK-0009 ", result.stdout)

    def test_next_id_after_zero_padded_nine_is_ten(self) -> None:
        # "09" is not a valid octal literal either; both must parse as decimal.
        self._write(["RISK-0008", "RISK-0009"])
        result = _run(["add", "--description", "d", "--probability", "1", "--impact", "1",
                       "--mitigation", "m"], env_file=self.register)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Added risk RISK-0010 ", result.stdout)

    def test_sequential_adds_past_octal_boundary(self) -> None:
        self._write([])
        for _ in range(12):
            result = _run(["add", "--description", "d", "--probability", "1", "--impact", "1",
                           "--mitigation", "m"], env_file=self.register)
            self.assertEqual(result.returncode, 0, result.stderr)
        text = self.register.read_text(encoding="utf-8")
        self.assertIn("  - id: RISK-0012\n", text)
        self.assertEqual(text.count("  - id: RISK-"), 12)


@unittest.skipUnless(shutil.which("ruby"), "ruby is required for YAML validation")
class ValidateYamlTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.register = Path(self._tmp.name) / "risk-register.yaml"

    def test_committed_register_validates(self) -> None:
        result = _run(["validate", "--file", str(COMMITTED_REGISTER)])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_unquoted_date_and_timestamp_are_accepted(self) -> None:
        # Psych 4 rejects Date/Time scalars unless they are explicitly permitted.
        self.register.write_text(HEADER + _entry("RISK-0001", date="2026-10-01"),
                                 encoding="utf-8")
        result = _run(["validate", "--file", str(self.register)])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn("Psych::DisallowedClass", result.stdout + result.stderr)

    def test_wrong_score_is_still_rejected(self) -> None:
        self.register.write_text(HEADER + _entry("RISK-0001", score=7), encoding="utf-8")
        result = _run(["validate", "--file", str(self.register)])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid YAML content", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
