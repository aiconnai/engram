#!/usr/bin/env python3
"""Contract tests for the Q1 consumption rule (scripts/consume-quality-candidate.py, task Q1b).

A candidate report is accepted evidence only when every rule of
docs/quality/retrieval-performance-policy.md ("Consuming a report") holds. The
consumer separates three outcomes so CI can stay report-only while the floors
are pending independent review:

* rejected (exit 1): integrity failure (status, SHA, supervisor binding);
* not-accepted (exit 3): integrity holds but ``floors.accepted`` is not true;
* accepted (exit 0).

Run: python3 -m unittest scripts/test_quality_candidate_consume.py
"""

from __future__ import annotations

import contextlib
import copy
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
WORKFLOW = SCRIPTS.parent / ".github" / "workflows" / "quality-candidate.yml"
sys.path.insert(0, str(SCRIPTS))

from quality_candidate import consume  # noqa: E402

SHA = "a" * 40
OTHER_SHA = "b" * 40
SUPERVISOR = "123456789-1"
CLI = SCRIPTS / "consume-quality-candidate.py"


def passing_report() -> dict:
    return {
        "schema_version": "engram.quality-candidate-report.v1",
        "status": "pass",
        "errors": [],
        "supervisor": SUPERVISOR,
        "supervisor_required": True,
        "candidate": {"sha": SHA, "tree": "c" * 40},
        "floors": {
            "anchored": True,
            "accepted": True,
            "anchor_revision": OTHER_SHA,
            "anchor_reason": "anchored",
        },
        "criterion": {"marker": {"candidate_sha": SHA, "supervisor": SUPERVISOR}},
    }


class AssessRule(unittest.TestCase):
    def assess(self, report, sha=SHA, supervisor=SUPERVISOR):
        return consume.assess(report, candidate_sha=sha, supervisor=supervisor)

    def test_fully_bound_report_with_accepted_floors_is_accepted(self):
        verdict = self.assess(passing_report())
        self.assertEqual(verdict.decision, consume.ACCEPTED, verdict.reasons)
        self.assertEqual(verdict.reasons, [])

    def test_unaccepted_floors_are_not_accepted_but_not_an_integrity_failure(self):
        report = passing_report()
        report["floors"].update(
            {"anchored": False, "accepted": False, "anchor_reason": "no anchor supplied"}
        )
        verdict = self.assess(report)
        self.assertEqual(verdict.decision, consume.NOT_ACCEPTED)
        self.assertTrue(any("floors.accepted" in r for r in verdict.reasons), verdict.reasons)
        self.assertTrue(any("no anchor supplied" in r for r in verdict.reasons), verdict.reasons)

    def test_anchored_but_unreviewed_floors_are_not_accepted(self):
        report = passing_report()
        report["floors"]["accepted"] = False
        self.assertEqual(self.assess(report).decision, consume.NOT_ACCEPTED)

    def test_truthy_non_boolean_accepted_flag_is_not_accepted(self):
        report = passing_report()
        report["floors"]["accepted"] = "true"
        self.assertEqual(self.assess(report).decision, consume.NOT_ACCEPTED)

    def test_failed_status_is_rejected(self):
        report = passing_report()
        report["status"] = "fail"
        report["errors"] = ["criterion regression"]
        verdict = self.assess(report)
        self.assertEqual(verdict.decision, consume.REJECTED)
        self.assertTrue(any("criterion regression" in r for r in verdict.reasons), verdict.reasons)

    def test_other_candidate_sha_is_rejected(self):
        verdict = self.assess(passing_report(), sha=OTHER_SHA)
        self.assertEqual(verdict.decision, consume.REJECTED)
        self.assertTrue(any("candidate.sha" in r for r in verdict.reasons), verdict.reasons)

    def test_criterion_marker_for_another_candidate_is_rejected(self):
        report = passing_report()
        report["criterion"]["marker"]["candidate_sha"] = OTHER_SHA
        self.assertEqual(self.assess(report).decision, consume.REJECTED)

    def test_supervisor_of_another_job_is_rejected(self):
        verdict = self.assess(passing_report(), supervisor="999-1")
        self.assertEqual(verdict.decision, consume.REJECTED)
        self.assertTrue(any("supervisor" in r for r in verdict.reasons), verdict.reasons)

    def test_marker_supervisor_mismatch_is_rejected(self):
        report = passing_report()
        report["criterion"]["marker"]["supervisor"] = "other-job"
        self.assertEqual(self.assess(report).decision, consume.REJECTED)

    def test_local_or_blank_expected_supervisor_is_rejected(self):
        for value in ("", "local", "  "):
            report = passing_report()
            report["supervisor"] = value
            report["criterion"]["marker"]["supervisor"] = value
            with self.subTest(value=value):
                self.assertEqual(self.assess(report, supervisor=value).decision, consume.REJECTED)

    def test_supervisor_not_required_is_rejected(self):
        report = passing_report()
        report["supervisor_required"] = False
        self.assertEqual(self.assess(report).decision, consume.REJECTED)

    def test_missing_sections_are_rejected_not_crashing(self):
        for key in ("candidate", "floors", "criterion", "supervisor", "status"):
            report = copy.deepcopy(passing_report())
            del report[key]
            with self.subTest(missing=key):
                self.assertEqual(self.assess(report).decision, consume.REJECTED)

    def test_non_object_report_is_rejected(self):
        self.assertEqual(self.assess([]).decision, consume.REJECTED)

    def test_malformed_expected_sha_is_rejected(self):
        self.assertEqual(self.assess(passing_report(), sha="HEAD").decision, consume.REJECTED)


class Cli(unittest.TestCase):
    def run_cli(self, report, *extra, sha=SHA, supervisor=SUPERVISOR):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "report.json"
            summary = Path(tmp) / "summary.md"
            if isinstance(report, str):
                path.write_text(report, encoding="utf-8")
            elif report is not None:
                path.write_text(json.dumps(report), encoding="utf-8")
            argv = [
                "--report", str(path), "--candidate-sha", sha,
                "--supervisor", supervisor, "--summary", str(summary), *extra,
            ]
            out, err = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                code = consume.main(argv)
            text = summary.read_text(encoding="utf-8") if summary.exists() else ""
            return code, out.getvalue() + err.getvalue(), text

    def test_exit_codes_map_to_decisions(self):
        self.assertEqual(self.run_cli(passing_report())[0], 0)
        unaccepted = passing_report()
        unaccepted["floors"]["accepted"] = False
        self.assertEqual(self.run_cli(unaccepted)[0], consume.EXIT_NOT_ACCEPTED)
        failed = passing_report()
        failed["status"] = "fail"
        self.assertEqual(self.run_cli(failed)[0], 1)

    def test_missing_or_invalid_report_is_rejected(self):
        self.assertEqual(self.run_cli(None)[0], 1)
        self.assertEqual(self.run_cli("{not json")[0], 1)
        self.assertEqual(self.run_cli('{"status": NaN}')[0], 1)

    def test_summary_names_the_decision_and_says_report_only(self):
        unaccepted = passing_report()
        unaccepted["floors"]["accepted"] = False
        code, _, summary = self.run_cli(unaccepted)
        self.assertEqual(code, consume.EXIT_NOT_ACCEPTED)
        self.assertIn("NOT ACCEPTED", summary)
        self.assertIn(SHA, summary)
        self.assertIn("report-only", summary)

    def test_summary_is_appended_not_truncated(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "report.json"
            path.write_text(json.dumps(passing_report()), encoding="utf-8")
            summary = Path(tmp) / "summary.md"
            summary.write_text("previous step\n", encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                consume.main([
                    "--report", str(path), "--candidate-sha", SHA,
                    "--supervisor", SUPERVISOR, "--summary", str(summary),
                ])
            text = summary.read_text(encoding="utf-8")
        self.assertTrue(text.startswith("previous step\n"))
        self.assertIn("ACCEPTED", text)

    def test_wrapper_script_runs_without_writing_bytecode(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "report.json"
            path.write_text(json.dumps(passing_report()), encoding="utf-8")
            done = subprocess.run(
                [sys.executable, str(CLI), "--report", str(path), "--candidate-sha", SHA,
                 "--supervisor", SUPERVISOR],
                capture_output=True, text=True, check=False,
            )
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn("ACCEPTED", done.stdout)


def workflow_step_script(name_prefix: str) -> str:
    """The literal `run: |` block of a quality-candidate.yml step (no YAML dependency)."""
    lines = WORKFLOW.read_text(encoding="utf-8").splitlines()
    start = next(i for i, line in enumerate(lines) if line.strip().startswith(f"- name: {name_prefix}"))
    run = next(i for i in range(start + 1, len(lines)) if lines[i].strip() == "run: |")
    indent = len(lines[run + 1]) - len(lines[run + 1].lstrip())
    body = []
    for line in lines[run + 1:]:
        if line.strip() and len(line) - len(line.lstrip()) < indent:
            break
        body.append(line[indent:])
    return "\n".join(body) + "\n"


class WorkflowVerdictStep(unittest.TestCase):
    """The consumer step maps exit 3 to a warning and fails on anything else, under `bash -e`.

    GitHub runs `run:` blocks with `bash -e {0}`, so a bare `rc=$?` after a failing command
    never executes; this pins the mapping against the real step text.
    """

    def run_step(self, consumer_exit: int):
        script = workflow_step_script("Apply the Q1 consumption rule")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fake = root / "trusted" / "scripts" / "consume-quality-candidate.py"
            fake.parent.mkdir(parents=True)
            fake.write_text(f"import sys\nsys.exit({consumer_exit})\n", encoding="utf-8")
            env = {
                "PATH": "/usr/bin:/bin:/usr/local/bin:/opt/homebrew/bin",
                "RUNNER_TEMP": str(root / "tmp"),
                "GITHUB_STEP_SUMMARY": str(root / "summary.md"),
                "CANDIDATE_SHA": SHA,
                "ENGRAM_QUALITY_SUPERVISOR": SUPERVISOR,
            }
            return subprocess.run(["bash", "-e", "-c", script], cwd=root, env=env,
                                  capture_output=True, text=True, check=False)

    def test_accepted_passes(self):
        self.assertEqual(self.run_step(0).returncode, 0)

    def test_not_accepted_is_a_warning_not_a_failure(self):
        done = self.run_step(consume.EXIT_NOT_ACCEPTED)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn("::warning::", done.stdout)

    def test_rejected_and_usage_errors_fail_the_job(self):
        for code in (1, 2):
            with self.subTest(code=code):
                self.assertEqual(self.run_step(code).returncode, code)

    def test_step_has_no_continue_on_error_or_true_mask(self):
        keys = [line.strip() for line in WORKFLOW.read_text(encoding="utf-8").splitlines()]
        self.assertFalse([k for k in keys if k.startswith("continue-on-error:")])
        self.assertNotIn("|| true", workflow_step_script("Apply the Q1 consumption rule"))


if __name__ == "__main__":
    unittest.main()
