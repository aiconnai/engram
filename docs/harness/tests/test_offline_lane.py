#!/usr/bin/env python3
"""Fail-closed contract of docs/harness/bin/run-offline-lane.sh (task H2).

The runner is copied into a temporary repository skeleton whose component suites are stubs, so
every failure mode can be provoked deterministically: zero tests, skipped tests, failing exit,
missing summary, count below the tripwire floor, NOT RUN checks, a deleted component and a
missing jsonschema. Offline; no network.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
LANE = REPO_ROOT / "docs" / "harness" / "bin" / "run-offline-lane.sh"

# Mirrors the exact floors of run-offline-lane.sh (the real suites' current counts).
GOOD_COUNTS = {"unit": 76, "selftest": 18, "fixtures": 39, "live": 40, "gate": 39, "lane": 19,
               "sandbox": 54, "context": 49, "runner": 75, "merge": 47, "retention": 21,
               "standing": 52}


def unittest_source(n: int, skip: bool = False) -> str:
    body = "".join(
        f"    def test_{i}(self):\n        self.assertTrue(True)\n" for i in range(n)
    )
    decorator = "    @unittest.skip('stub skip')\n    def test_skipped(self):\n        pass\n" if skip else ""
    return f"import unittest\n\nclass T(unittest.TestCase):\n{body or '    pass'}\n{decorator}\nif __name__ == '__main__':\n    unittest.main()\n"


class OfflineLaneContract(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="h2-lane-"))
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        (self.root / "docs/harness/bin").mkdir(parents=True)
        (self.root / "docs/harness/tests").mkdir(parents=True)
        shutil.copy(LANE, self.root / "docs/harness/bin/run-offline-lane.sh")
        self.counts = dict(GOOD_COUNTS)
        self.write_stubs()

    # -- stub builders ---------------------------------------------------------------------

    def put(self, rel: str, text: str, executable: bool = False) -> None:
        p = self.root / rel
        p.write_text(text, encoding="utf-8")
        if executable:
            p.chmod(0o755)

    def write_stubs(self, **over):
        c = {**self.counts, **over}
        self.put("docs/harness/tests/test_validate_evidence.py", unittest_source(c["unit"], over.get("skip", False)))
        self.put("docs/harness/tests/test_offline_lane.py", unittest_source(c["lane"]))
        self.put("docs/harness/tests/test_sandbox_adapter.py", unittest_source(c["sandbox"], over.get("sandbox_skip", False)))
        self.put("docs/harness/tests/test_context_budget.py", unittest_source(c["context"], over.get("context_skip", False)))
        self.put("docs/harness/tests/test_runner.py", unittest_source(c["runner"], over.get("runner_skip", False)))
        self.put("docs/harness/tests/test_merge_gate.py",
                 unittest_source(c["merge"], over.get("merge_skip", False)))
        self.put("docs/harness/tests/test_retention.py", unittest_source(c["retention"], over.get("retention_skip", False)))
        self.put("docs/harness/tests/test_standing_checks.py", unittest_source(c["standing"], over.get("standing_skip", False)))
        self.put("docs/harness/tests/test_scope.py", unittest_source(0))
        self.put("docs/harness/tests/test_evidence_integrity.py", unittest_source(0))
        passed, failed = c["selftest"], over.get("self_failed", 0)
        self.put("docs/harness/bin/validate-evidence.py",
                 f"print('SELF_TEST_RESULT: passed={passed} failed={failed}')\n")
        self.put("docs/harness/bin/test-fixtures.sh",
                 f"#!/usr/bin/env bash\necho 'Passed: {c['fixtures']}'\necho 'Failed: 0'\necho 'ALL TESTS PASSED'\n", True)
        self.put("docs/harness/bin/test-check-live-state.sh",
                 f"#!/usr/bin/env bash\necho 'PASS check-live-state regression suite (assertions: {c['live']})'\n", True)
        not_run = over.get("not_run", 0)
        self.put("docs/harness/bin/test-review-gate.sh",
                 f"#!/usr/bin/env bash\necho 'Tests: {c['gate']}  Assertions passed: 200  Assertions failed: 0  Not run: {not_run}'\n", True)

    def run_lane(self, *args, env=None):
        full_env = {**os.environ, **(env or {})}
        proc = subprocess.run(["bash", str(self.root / "docs/harness/bin/run-offline-lane.sh"), *args],
                              capture_output=True, text=True, timeout=120, env=full_env)
        return proc.returncode, proc.stdout, proc.stderr

    # -- tests -----------------------------------------------------------------------------

    def test_all_components_green_passes_and_reports_counts(self):
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 0, out + err)
        self.assertIn("OFFLINE_LANE: PASS components=12 checks=", out)

    def test_zero_tests_is_a_failure(self):
        self.write_stubs(unit=0)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("OFFLINE_LANE: FAIL", err)

    def test_skipped_tests_are_a_failure(self):
        self.write_stubs(skip=True)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("validator_unit", err)

    def test_count_below_the_tripwire_floor_is_a_failure(self):
        self.write_stubs(unit=5)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("below the tripwire floor", err)

    def test_failing_selftest_and_zero_selftest_checks_fail(self):
        for over in ({"self_failed": 1}, {"selftest": 0}):
            with self.subTest(over):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn("validator_self", err)

    def test_sandbox_adapter_component_is_mandatory_and_skips_or_low_counts_fail(self):
        for label, over, needle in (("skip", {"sandbox_skip": True}, "sandbox_unit"),
                                    ("below floor", {"sandbox": 5}, "below the tripwire floor"),
                                    ("zero", {"sandbox": 0}, "sandbox_unit")):
            with self.subTest(label):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(needle, err)
        self.write_stubs()
        (self.root / "docs/harness/tests/test_sandbox_adapter.py").unlink()
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("sandbox_unit", err)

    def test_context_budget_component_is_mandatory_and_skips_or_low_counts_fail(self):
        for label, over, needle in (("skip", {"context_skip": True}, "context_budget"),
                                    ("below floor", {"context": 5}, "below the tripwire floor"),
                                    ("zero", {"context": 0}, "context_budget")):
            with self.subTest(label):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(needle, err)
        self.write_stubs()
        (self.root / "docs/harness/tests/test_context_budget.py").unlink()
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("context_budget", err)

    def test_runner_component_is_mandatory_and_skips_low_counts_or_a_missing_file_fail(self):
        for label, over, needle in (("skip", {"runner_skip": True}, "runner_unit"),
                                    ("below floor", {"runner": 5}, "below the tripwire floor"),
                                    ("zero", {"runner": 0}, "runner_unit")):
            with self.subTest(label):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(needle, err)
        for name in ("test_runner.py", "test_scope.py", "test_evidence_integrity.py"):
            with self.subTest(f"missing {name}"):
                self.write_stubs()
                (self.root / "docs/harness/tests" / name).unlink()
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn("runner_unit", err)

    def test_every_component_fails_one_below_its_exact_floor(self):
        names = {"unit": "validator_unit", "selftest": "validator_self", "fixtures": "fixtures",
                 "live": "live_state", "gate": "review_gate", "lane": "lane_contract",
                 "sandbox": "sandbox_unit", "context": "context_budget", "runner": "runner_unit",
                 "merge": "merge_gate", "retention": "retention", "standing": "standing_checks"}
        self.assertEqual(sorted(names), sorted(GOOD_COUNTS))
        for key, component in names.items():
            with self.subTest(component):
                self.write_stubs(**{key: GOOD_COUNTS[key] - 1})
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(component, err)
                self.assertIn("below the tripwire floor", err)

    def test_review_gate_count_below_its_exact_floor_fails(self):
        self.write_stubs(gate=38)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("below the tripwire floor", err)

    def test_merge_gate_component_is_mandatory_and_skips_low_counts_or_a_missing_file_fail(self):
        for label, over, needle in (("skip", {"merge_skip": True}, "merge_gate"),
                                    ("below floor", {"merge": 46}, "below the tripwire floor"),
                                    ("zero", {"merge": 0}, "merge_gate")):
            with self.subTest(label):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(needle, err)
        self.write_stubs()
        (self.root / "docs/harness/tests/test_merge_gate.py").unlink()
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("merge_gate", err)

    def test_retention_component_is_mandatory_and_skips_low_counts_or_a_missing_file_fail(self):
        for label, over, needle in (("skip", {"retention_skip": True}, "retention"),
                                    ("below floor", {"retention": 20}, "below the tripwire floor"),
                                    ("zero", {"retention": 0}, "retention")):
            with self.subTest(label):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(needle, err)
        self.write_stubs()
        (self.root / "docs/harness/tests/test_retention.py").unlink()
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("retention", err)

    def test_standing_checks_component_is_mandatory_and_skips_low_counts_or_a_missing_file_fail(self):
        for label, over, needle in (("skip", {"standing_skip": True}, "standing_checks"),
                                    ("below floor", {"standing": 51}, "below the tripwire floor"),
                                    ("zero", {"standing": 0}, "standing_checks")):
            with self.subTest(label):
                self.write_stubs(**over)
                rc, out, err = self.run_lane()
                self.assertEqual(rc, 1, out + err)
                self.assertIn(needle, err)
        self.write_stubs()
        (self.root / "docs/harness/tests/test_standing_checks.py").unlink()
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("standing_checks", err)

    def test_nonzero_exit_of_a_component_fails_even_with_a_good_summary(self):
        self.put("docs/harness/bin/test-fixtures.sh",
                 "#!/usr/bin/env bash\necho 'Passed: 99'\necho 'Failed: 0'\necho 'ALL TESTS PASSED'\nexit 1\n", True)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("exit-1", out + err)

    def test_missing_summary_line_fails(self):
        self.put("docs/harness/bin/test-check-live-state.sh", "#!/usr/bin/env bash\necho 'looks fine'\n", True)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("live_state", err)

    def test_deleted_component_fails(self):
        (self.root / "docs/harness/bin/test-review-gate.sh").unlink()
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("review_gate", err)

    def test_not_run_checks_fail_unless_explicitly_tolerated(self):
        self.write_stubs(not_run=1)
        rc, out, err = self.run_lane()
        self.assertEqual(rc, 1, out + err)
        self.assertIn("NOT RUN", err)
        rc, out, err = self.run_lane(env={"HARNESS_LANE_MAX_NOT_RUN": "1"})
        self.assertEqual(rc, 0, out + err)

    def test_missing_jsonschema_fails_the_lane_instead_of_skipping(self):
        shim = self.root / "shim"
        shim.mkdir()
        (shim / "jsonschema.py").write_text("raise ImportError('blocked for the test')\n", encoding="utf-8")
        rc, out, err = self.run_lane(env={"PYTHONPATH": str(shim)})
        self.assertEqual(rc, 1, out + err)
        self.assertIn("reason=jsonschema-missing", err)

    def test_arguments_are_a_usage_error(self):
        rc, _, err = self.run_lane("--skip-review-gate")
        self.assertEqual(rc, 2, err)


if __name__ == "__main__":
    unittest.main(verbosity=2)
