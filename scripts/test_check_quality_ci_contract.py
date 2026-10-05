#!/usr/bin/env python3
"""Contract tests for the CI/local quality-budget lane (task Q1a).

The lane validates the *historical baseline integrity* of the committed
retrieval floors and Criterion snapshot. It does not measure candidate
performance; that evidence arrives with Q7/Q1b. These tests pin:

* the checker's own contract (``--criterion`` is mandatory, bad inputs fail);
* the literal argv used by the GitHub workflow and by ``scripts/ci.sh``;
* agreement between the two lanes and the parity wrapper.

Missing tools or files are failures, never skips.
"""

from __future__ import annotations

import importlib.util
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

ROOT = Path(__file__).resolve().parent.parent
CHECKER_REL = "scripts/check-quality-budgets.py"
CHECKER = ROOT / CHECKER_REL
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
CI_SH = ROOT / "scripts" / "ci.sh"
POLICY = ROOT / "docs" / "quality" / "retrieval-performance-policy.md"

BUDGETS = "docs/quality/budgets.json"
RETRIEVAL = "tests/fixtures/retrieval_quality/baseline.json"
CRITERION = "benches/results/benchmark_results.txt"
LANE_LABEL = "historical baseline integrity"
NOT_PERFORMANCE = "not candidate performance"

BASE_ARGS = [
    CHECKER_REL,
    "--budgets",
    BUDGETS,
    "--retrieval",
    RETRIEVAL,
    "--criterion",
    CRITERION,
]
EXPECTED_ARGVS: List[List[str]] = [BASE_ARGS, BASE_ARGS + ["--self-test-degraded"]]

SPEC = importlib.util.spec_from_file_location("check_quality_budgets", CHECKER)
assert SPEC is not None and SPEC.loader is not None
CHECKER_MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER_MODULE)


def run_checker(args: Sequence[str]) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=120,
    )


def workflow_budget_steps(text: str) -> List[Tuple[str, List[str]]]:
    """Return (step name, argv) for every workflow step invoking the checker."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines) if line.lstrip().startswith("- name:")]
    steps: List[Tuple[str, List[str]]] = []
    for pos, start in enumerate(starts):
        end = starts[pos + 1] if pos + 1 < len(starts) else len(lines)
        block = lines[start:end]
        name = block[0].split("- name:", 1)[1].strip().strip("'\"")
        run_text = _extract_run(block)
        if run_text is not None and "check-quality-budgets.py" in run_text:
            steps.append((name, shlex.split(run_text)))
    return steps


def workflow_steps(text: str) -> List[Dict[str, str]]:
    """Return every workflow step as {job, name, run}, in file order."""
    steps: List[Dict[str, str]] = []
    job = ""
    in_jobs = False
    lines = text.splitlines()
    starts: List[Tuple[int, str]] = []
    for i, line in enumerate(lines):
        if line.startswith("jobs:"):
            in_jobs = True
        elif in_jobs and re.match(r"^  [A-Za-z0-9_-]+:\s*$", line):
            job = line.strip().rstrip(":")
        elif in_jobs and re.match(r"^ {6}- ", line):
            starts.append((i, job))
    for pos, (start, step_job) in enumerate(starts):
        end = starts[pos + 1][0] if pos + 1 < len(starts) else len(lines)
        block = [lines[start].replace("- ", "  ", 1), *lines[start + 1 : end]]
        name = ""
        for line in block:
            if line.strip().startswith("name:"):
                name = line.split("name:", 1)[1].strip().strip("'\"")
                break
        steps.append({"job": step_job, "name": name, "run": _extract_run(block) or ""})
    return steps


CONTRACT_TEST_ARGV = ["python3", "scripts/test_check_quality_ci_contract.py"]


def workflow_runs_contract_suite(text: str) -> bool:
    """True when the job that runs the budget checker then runs this suite, fail-closed."""
    steps = workflow_steps(text)
    budget = [i for i, s in enumerate(steps) if "check-quality-budgets.py" in s["run"]]
    if not budget:
        return False
    job = steps[budget[0]]["job"]
    for i, step in enumerate(steps):
        if (
            step["job"] == job
            and i > budget[-1]
            and shlex.split(step["run"]) == CONTRACT_TEST_ARGV
        ):
            return True
    return False


def ci_sh_runs_contract_suite(text: str) -> bool:
    """True when ci.sh calls the suite at top level, after the lane call, not inside it.

    The suite invokes ``ci.sh quality-budgets``; running it inside the lane function
    would recurse forever, so it must be a separate top-level statement.
    """
    lines = text.splitlines()
    call = [i for i, line in enumerate(lines) if line == "run_quality_budget_lane"]
    suite = [
        i
        for i, line in enumerate(lines)
        if line == 'python3 "$SCRIPT_DIR/test_check_quality_ci_contract.py"'
    ]
    return len(call) == 1 and len(suite) == 1 and suite[0] > call[0]


def _extract_run(block: Sequence[str]) -> Optional[str]:
    for i, line in enumerate(block):
        stripped = line.strip()
        if not stripped.startswith("run:"):
            continue
        value = stripped[len("run:") :].strip()
        if value and value[0] not in ">|":
            return value
        indent = len(line) - len(line.lstrip())
        body: List[str] = []
        for follow in block[i + 1 :]:
            if follow.strip() and len(follow) - len(follow.lstrip()) <= indent:
                break
            body.append(follow.strip())
        return " ".join(part for part in body if part)
    return None


def record_local_lane(extra_env: Optional[Dict[str, str]] = None) -> Tuple[int, List[List[str]], str]:
    """Run ``scripts/ci.sh quality-budgets`` against a recording fake python3."""
    with tempfile.TemporaryDirectory() as tmp:
        fake_bin = Path(tmp) / "bin"
        fake_bin.mkdir()
        log = Path(tmp) / "calls.log"
        fake = fake_bin / "python3"
        fake.write_text(
            "#!/bin/sh\n"
            f'printf "%s\\n" "$@" >> "{log}"\n'
            f'printf "%s\\n" "--END--" >> "{log}"\n'
            'exit "${FAKE_PYTHON_EXIT:-0}"\n',
            encoding="utf-8",
        )
        fake.chmod(0o755)
        env = dict(os.environ)
        env["PATH"] = f"{fake_bin}{os.pathsep}{env['PATH']}"
        env.update(extra_env or {})
        done = subprocess.run(
            ["bash", str(CI_SH), "quality-budgets"],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=60,
        )
        calls: List[List[str]] = []
        if log.exists():
            current: List[str] = []
            for line in log.read_text(encoding="utf-8").splitlines():
                if line == "--END--":
                    calls.append(current)
                    current = []
                else:
                    current.append(line)
        return done.returncode, calls, done.stdout + done.stderr


class CheckerContract(unittest.TestCase):
    """The checker itself: mandatory argument and non-zero on bad input."""

    def budgets(self) -> dict:
        return json.loads((ROOT / BUDGETS).read_text(encoding="utf-8"))

    def criterion_text(self, scale_first: float = 1.0) -> str:
        hot_paths = self.budgets()["criterion"]["hot_paths"]
        units = CHECKER_MODULE.UNITS_TO_SECONDS
        chunks = []
        for index, (name, raw) in enumerate(hot_paths.items()):
            seconds = raw["baseline_value"] * units[raw["baseline_unit"]]
            if index == 0:
                seconds *= scale_first
            chunks.append(
                f"{name}\n                        time:   [{seconds} s {seconds} s {seconds} s]\n"
            )
        return "".join(chunks)

    def run_with_criterion(self, text: str, *extra: str) -> subprocess.CompletedProcess:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "criterion.txt"
            path.write_text(text, encoding="utf-8")
            return run_checker(
                [CHECKER_REL, "--budgets", BUDGETS, "--retrieval", RETRIEVAL,
                 "--criterion", str(path), *extra]
            )

    def test_missing_criterion_argument_exits_2(self) -> None:
        done = run_checker([CHECKER_REL, "--budgets", BUDGETS, "--retrieval", RETRIEVAL])
        self.assertEqual(done.returncode, 2, done.stderr)
        self.assertIn("--criterion", done.stderr)

    def test_missing_criterion_file_is_non_zero(self) -> None:
        done = run_checker(
            [CHECKER_REL, "--budgets", BUDGETS, "--retrieval", RETRIEVAL,
             "--criterion", "benches/results/does-not-exist.txt"]
        )
        self.assertNotEqual(done.returncode, 0, done.stdout)

    def test_invalid_unit_is_non_zero(self) -> None:
        names = list(self.budgets()["criterion"]["hot_paths"])
        text = "".join(
            f"{name}\n                        time:   [1 xs 1 xs 1 xs]\n" for name in names
        )
        done = self.run_with_criterion(text)
        self.assertNotEqual(done.returncode, 0, done.stdout)

    def test_116_percent_regression_is_non_zero(self) -> None:
        done = self.run_with_criterion(self.criterion_text(scale_first=1.16))
        self.assertNotEqual(done.returncode, 0, done.stdout)
        self.assertIn("criterion regression", done.stdout)

    def test_valid_temp_criterion_at_baseline_passes(self) -> None:
        done = self.run_with_criterion(self.criterion_text())
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)

    def test_full_call_with_repo_snapshot_passes(self) -> None:
        done = run_checker(BASE_ARGS)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)

    def test_self_test_with_repo_snapshot_passes(self) -> None:
        done = run_checker(EXPECTED_ARGVS[1])
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn("engram.quality-budget-self-test.v1", done.stdout)


class WorkflowArgvContract(unittest.TestCase):
    """The required GitHub job passes the complete checker argv."""

    def steps(self) -> List[Tuple[str, List[str]]]:
        return workflow_budget_steps(WORKFLOW.read_text(encoding="utf-8"))

    def test_workflow_argv_are_exactly_the_contract(self) -> None:
        argvs = [argv[1:] for _, argv in self.steps()]
        self.assertEqual(argvs, EXPECTED_ARGVS)
        for _, argv in self.steps():
            self.assertEqual(argv[0], "python3")

    def test_workflow_argv_satisfy_checker_parser(self) -> None:
        for name, argv in self.steps():
            with self.subTest(step=name):
                parsed = CHECKER_MODULE.parse_args(argv[2:])
                self.assertEqual(parsed.criterion, Path(CRITERION))

    def test_workflow_steps_are_labelled_historical_integrity(self) -> None:
        steps = self.steps()
        self.assertTrue(steps, "workflow has no quality-budget step")
        for name, _ in steps:
            with self.subTest(step=name):
                self.assertIn(LANE_LABEL, name.lower())
                self.assertIn(NOT_PERFORMANCE, name.lower())
                self.assertNotIn("performance budgets", name.lower())


class WorkflowRealRun(unittest.TestCase):
    """Execute the workflow argv literally; mirrors what the runner does."""

    def test_workflow_argv_run_green(self) -> None:
        steps = workflow_budget_steps(WORKFLOW.read_text(encoding="utf-8"))
        self.assertTrue(steps, "workflow has no quality-budget step")
        for name, argv in steps:
            with self.subTest(step=name):
                done = subprocess.run(
                    [sys.executable, *argv[1:]],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    timeout=120,
                )
                self.assertEqual(done.returncode, 0, done.stdout + done.stderr)


class LocalLaneContract(unittest.TestCase):
    """``scripts/ci.sh`` (``make ci``) runs the same argv, fail-closed."""

    def test_local_lane_argv_are_exactly_the_contract(self) -> None:
        code, calls, output = record_local_lane()
        self.assertEqual(code, 0, output)
        self.assertEqual(calls, EXPECTED_ARGVS, output)

    def test_local_lane_is_labelled_historical_integrity(self) -> None:
        _, _, output = record_local_lane()
        headers = [line for line in output.lower().splitlines() if line.startswith("==>")]
        self.assertEqual(len(headers), 2, output)
        for header in headers:
            self.assertIn(LANE_LABEL, header)
            self.assertIn(NOT_PERFORMANCE, header)
        self.assertNotIn("performance budgets", output.lower())

    def test_local_lane_propagates_checker_failure(self) -> None:
        code, calls, output = record_local_lane({"FAKE_PYTHON_EXIT": "1"})
        self.assertNotEqual(code, 0, output)
        self.assertEqual(len(calls), 1, "must stop at the first failing call")

    def test_local_lane_without_python3_is_non_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            bin_dir = Path(tmp)
            for tool in ("bash", "dirname"):
                found = shutil.which(tool)
                self.assertIsNotNone(found, f"{tool} is required for the test")
                (bin_dir / tool).symlink_to(str(found))
            done = subprocess.run(
                [str(bin_dir / "bash"), str(CI_SH), "quality-budgets"],
                cwd=ROOT,
                env={"PATH": str(bin_dir), "HOME": tmp},
                capture_output=True,
                text=True,
                timeout=60,
            )
        self.assertNotEqual(done.returncode, 0, done.stdout)
        self.assertIn("python3", done.stdout + done.stderr)

    def test_make_ci_delegates_to_ci_sh(self) -> None:
        make = shutil.which("make")
        self.assertIsNotNone(make, "make is required to verify `make ci`")
        done = subprocess.run(
            [str(make), "-n", "ci"], cwd=ROOT, capture_output=True, text=True, timeout=60
        )
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("scripts/ci.sh", done.stdout)


class LaneAgreement(unittest.TestCase):
    def test_local_and_ci_argv_agree(self) -> None:
        ci = [argv[1:] for _, argv in workflow_budget_steps(WORKFLOW.read_text(encoding="utf-8"))]
        code, local, output = record_local_lane()
        self.assertEqual(code, 0, output)
        self.assertEqual(local, ci)


class GateWiring(unittest.TestCase):
    """The contract suite itself runs in the required job and in scripts/ci.sh."""

    def workflow(self) -> str:
        return WORKFLOW.read_text(encoding="utf-8")

    def ci_sh(self) -> str:
        return CI_SH.read_text(encoding="utf-8")

    def test_required_job_runs_contract_suite(self) -> None:
        self.assertTrue(workflow_runs_contract_suite(self.workflow()))

    def test_workflow_without_suite_step_is_detected(self) -> None:
        kept = [
            line
            for line in self.workflow().splitlines()
            if "run: python3 scripts/test_check_quality_ci_contract.py" not in line
        ]
        self.assertLess(len(kept), len(self.workflow().splitlines()))
        self.assertFalse(workflow_runs_contract_suite("\n".join(kept)))

    def test_workflow_suite_in_other_job_is_detected(self) -> None:
        text = self.workflow().replace(
            "run: python3 scripts/test_check_quality_ci_contract.py",
            "run: python3 scripts/test_check_pdf_worker_packaging.py",
        )
        self.assertFalse(workflow_runs_contract_suite(text))

    def test_workflow_suite_with_swallowed_failure_is_detected(self) -> None:
        text = self.workflow().replace(
            "run: python3 scripts/test_check_quality_ci_contract.py",
            "run: python3 scripts/test_check_quality_ci_contract.py || true",
        )
        self.assertFalse(workflow_runs_contract_suite(text))

    def test_ci_sh_runs_contract_suite(self) -> None:
        self.assertTrue(ci_sh_runs_contract_suite(self.ci_sh()))

    def test_ci_sh_without_suite_call_is_detected(self) -> None:
        kept = [
            line
            for line in self.ci_sh().splitlines()
            if "test_check_quality_ci_contract.py" not in line
        ]
        self.assertFalse(ci_sh_runs_contract_suite("\n".join(kept)))

    def test_ci_sh_suite_inside_lane_function_is_detected(self) -> None:
        text = self.ci_sh().replace(
            'python3 "$SCRIPT_DIR/test_check_quality_ci_contract.py"',
            '  python3 "$SCRIPT_DIR/test_check_quality_ci_contract.py"',
        )
        self.assertFalse(ci_sh_runs_contract_suite(text))


class DocsNaming(unittest.TestCase):
    def test_policy_names_lane_as_historical_integrity(self) -> None:
        text = POLICY.read_text(encoding="utf-8").lower()
        self.assertIn(LANE_LABEL, text)
        self.assertIn("not candidate performance", text)


class ParityScript(unittest.TestCase):
    """``ci-parity-check.sh`` passes on the repo and fails if --criterion is lost."""

    ARTIFACTS = (
        "scripts",
        ".github/workflows/ci.yml",
        "Makefile",
        "justfile",
        "docs/quality",
    )

    def run_parity(self, root: Path) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["bash", str(root / "scripts" / "ci-parity-check.sh")],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=300,
        )

    def test_parity_passes_on_repository(self) -> None:
        done = self.run_parity(ROOT)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)

    def test_parity_fails_when_workflow_drops_criterion(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp)
            for rel in self.ARTIFACTS:
                src = ROOT / rel
                dst = copy / rel
                dst.parent.mkdir(parents=True, exist_ok=True)
                if src.is_dir():
                    shutil.copytree(src, dst)
                else:
                    shutil.copy2(src, dst)
            workflow = copy / ".github" / "workflows" / "ci.yml"
            text = workflow.read_text(encoding="utf-8")
            self.assertIn(f"--criterion {CRITERION}", text)
            workflow.write_text(text.replace(f"--criterion {CRITERION}", ""), encoding="utf-8")
            done = self.run_parity(copy)
        self.assertNotEqual(done.returncode, 0, done.stdout)
        self.assertIn("criterion", (done.stdout + done.stderr).lower())


if __name__ == "__main__":
    unittest.main()
