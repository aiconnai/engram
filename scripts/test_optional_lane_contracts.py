#!/usr/bin/env python3
"""Contract tests for the optional nightly lanes (task Q3): fuzz, mutation, Miri.

These tests run the real `scripts/run-fuzz-smoke.sh` and
`scripts/run-miri-smoke.sh` against a fake `cargo` placed first on PATH, so they
need neither nightly nor cargo-fuzz. They assert that every abnormal condition
(empty inventory, list failure, crash, missing infrastructure, inventory drift,
empty Miri filter, ...) ends in an explicit non-pass status and a non-zero exit
code, and that `.github/workflows/nightly.yml` cannot mask a failure and maps
`github.event.schedule` to the intended lane.

Run: python3 -m unittest scripts/test_optional_lane_contracts.py
"""

from __future__ import annotations

import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import textwrap
import tomllib
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
ROOT = SCRIPTS.parent
NIGHTLY = ROOT / ".github" / "workflows" / "nightly.yml"
FUZZ_SCRIPT = SCRIPTS / "run-fuzz-smoke.sh"
MIRI_SCRIPT = SCRIPTS / "run-miri-smoke.sh"
FUZZ_DIR = ROOT / "fuzz"

sys.path.insert(0, str(SCRIPTS))
import optional_lane_report as reporter  # noqa: E402

TARGETS = ["entity_extraction", "workspace_normalization", "text_util_boundaries"]

# Fake `cargo`: behaviour is selected through FAKE_* environment variables and
# every invocation is appended to $FAKE_CARGO_LOG.
FAKE_CARGO = r"""#!/usr/bin/env bash
echo "$*" >>"${FAKE_CARGO_LOG:-/dev/null}"
case "$1" in +*) echo "toolchain=$1" >>"${FAKE_CARGO_LOG:-/dev/null}"; shift ;; esac
sub="$1"; shift
case "$sub" in
  fuzz)
    action="$1"; shift
    case "$action" in
      --version)
        [ "${FAKE_INFRA:-}" = "missing" ] && { echo "error: no such command: fuzz" >&2; exit 101; }
        echo "cargo-fuzz 0.0.0-fake" ;;
      list)
        [ "${FAKE_LIST:-ok}" = "fail" ] && { echo "boom" >&2; exit 3; }
        [ "${FAKE_LIST:-ok}" = "empty" ] && exit 0
        if [ "${FAKE_LIST:-ok}" = "extra" ]; then printf '%s\n' ${FAKE_TARGETS} extra_unlisted; else printf '%s\n' ${FAKE_TARGETS}; fi ;;
      build)
        [ "$1" = "${FAKE_BUILD_FAIL:-}" ] && { echo "error[E0432]: unresolved import" >&2; exit 101; }
        echo "Finished release" ;;
      run)
        target="$1"
        if [ "$target" = "${FAKE_CRASH_TARGET:-}" ]; then
          mkdir -p "artifacts/$target"
          echo crash-bytes >"artifacts/$target/crash-deadbeef"
          echo "==1==ERROR: AddressSanitizer: heap-buffer-overflow"
          echo "Test unit written to artifacts/$target/crash-deadbeef"
          exit 77
        fi
        if [ "$target" = "${FAKE_NO_STATS_TARGET:-}" ]; then echo "INFO: ran nothing"; exit 0; fi
        [ -n "${FAKE_SLEEP:-}" ] && sleep "$FAKE_SLEEP"
        mkdir -p "corpus/$target"; echo new >"corpus/$target/found-1"
        echo "stat::number_of_executed_units: 4242"
        exit 0 ;;
    esac ;;
  miri)
    action="$1"; shift
    case "$action" in
      --version)
        [ "${FAKE_INFRA:-}" = "missing" ] && { echo "error: no such command: miri" >&2; exit 101; }
        echo "miri 0.0.0-fake" ;;
      test)
        filter="${@: -1}"
        case "$filter" in
          *nomatch*) printf 'running 0 tests\n\ntest result: ok. 0 passed; 0 failed\n' ;;
          *failing*) printf 'running 2 tests\ntest x ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
          *) printf 'running 3 tests\n\ntest result: ok. 3 passed; 0 failed\n' ;;
        esac ;;
    esac ;;
esac
exit 0
"""


def write_executable(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class FakeCargoCase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.tmp = Path(self._tmp.name)
        self.bin = self.tmp / "bin"
        self.bin.mkdir()
        write_executable(self.bin / "cargo", FAKE_CARGO)
        self.log = self.tmp / "cargo.log"
        self.out = self.tmp / "out"

    def run_script(self, script: Path, **env: str) -> subprocess.CompletedProcess:
        full_env = {
            **os.environ,
            "PATH": f"{self.bin}{os.pathsep}{os.environ['PATH']}",
            "FAKE_CARGO_LOG": str(self.log),
            **env,
        }
        return subprocess.run(
            ["bash", str(script)], capture_output=True, text=True, env=full_env, timeout=120, check=False
        )

    def read_report(self, lane: str) -> dict:
        return json.loads((self.out / f"{lane}-report.json").read_text(encoding="utf-8"))

    def cargo_calls(self) -> list[str]:
        return self.log.read_text(encoding="utf-8").splitlines() if self.log.exists() else []


class FuzzRunnerTests(FakeCargoCase):
    def make_fuzz_dir(self, inventory: list[str] | None = None, seeds: list[str] | None = None) -> Path:
        fuzz = self.tmp / "fuzz"
        fuzz.mkdir()
        names = TARGETS if inventory is None else inventory
        (fuzz / "targets.inventory").write_text("# declared\n" + "\n".join(names) + ("\n" if names else ""))
        for name in TARGETS if seeds is None else seeds:
            (fuzz / "seeds" / name).mkdir(parents=True)
            (fuzz / "seeds" / name / "seed-1").write_bytes(b"\x0f\x64@ronaldo")
        return fuzz

    def run_fuzz(self, fuzz: Path, **env: str) -> subprocess.CompletedProcess:
        base = {
            "FUZZ_DIR": str(fuzz),
            "FUZZ_SMOKE_OUT": str(self.out),
            "FAKE_TARGETS": " ".join(TARGETS),
            "FUZZ_SMOKE_OS": "Linux",
        }
        return self.run_script(FUZZ_SCRIPT, **{**base, **env})

    def test_all_targets_pass_with_default_bounds(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir())
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "pass")
        self.assertEqual(report["counts"]["pass"], len(TARGETS))
        self.assertEqual({t["target"] for t in report["targets"]}, set(TARGETS))
        for target in report["targets"]:
            self.assertEqual(target["executed"], 4242)
            self.assertEqual(target["seed_files"], 1)
            self.assertEqual(target["corpus_files"], 1)
        runs = [c for c in self.cargo_calls() if " run " in f" {c} "]
        self.assertEqual(len(runs), len(TARGETS))
        for call in runs:
            self.assertIn("-max_total_time=60", call)
            self.assertIn("-rss_limit_mb=2048", call)
        self.assertEqual(report["settings"]["max_total_time"], "60")

    def test_elapsed_is_recorded_and_default_grace_is_tight(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir())
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        report = self.read_report("fuzz")
        self.assertEqual(report["settings"]["wall_grace"], "120")
        for target in report["targets"]:
            self.assertIsInstance(target["elapsed_secs"], int)
            self.assertLessEqual(target["elapsed_secs"], 60 + 120)

    def test_elapsed_over_budget_fails_even_with_exit_zero(self) -> None:
        proc = self.run_fuzz(
            self.make_fuzz_dir(inventory=["entity_extraction"], seeds=["entity_extraction"]),
            FAKE_TARGETS="entity_extraction",
            FAKE_SLEEP="3",
            FUZZ_SMOKE_MAX_TOTAL_TIME="1",
            FUZZ_SMOKE_WALL_GRACE="0",
        )
        self.assertEqual(proc.returncode, 1, proc.stdout)
        report = self.read_report("fuzz")
        row = report["targets"][0]
        self.assertEqual((row["status"], row["exit_code"]), ("fail", "0"))
        self.assertGreaterEqual(row["elapsed_secs"], 3)
        self.assertIn("exceeds budget", row["note"])

    def test_toolchain_is_passed_through_and_recorded(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FUZZ_SMOKE_TOOLCHAIN="nightly-2026-04-21")
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual(self.read_report("fuzz")["settings"]["toolchain"], "nightly-2026-04-21")
        toolchains = {c for c in self.cargo_calls() if c.startswith("toolchain=")}
        self.assertEqual(toolchains, {"toolchain=+nightly-2026-04-21"})

    def test_empty_inventory_is_not_a_pass(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(inventory=[]))
        self.assertEqual(proc.returncode, 1)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "fail")
        self.assertTrue(any("inventory is empty" in e for e in report["lane_errors"]))
        self.assertFalse(any(" run " in f" {c} " for c in self.cargo_calls()))

    def test_list_failure_is_not_a_pass(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_LIST="fail")
        self.assertEqual(proc.returncode, 1)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "fail")
        self.assertEqual(report["counts"]["pass"], 0)
        self.assertTrue(any("list failed" in e for e in report["lane_errors"]))

    def test_empty_list_output_is_not_a_pass(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_LIST="empty")
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(self.read_report("fuzz")["status"], "fail")

    def test_inventory_drift_against_cargo_fuzz_list_fails_without_running(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_LIST="extra")
        self.assertEqual(proc.returncode, 1)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "fail")
        self.assertTrue(any("!= cargo fuzz list" in e for e in report["lane_errors"]))
        self.assertFalse(any(" run " in f" {c} " for c in self.cargo_calls()))

    def test_crash_fails_lane_preserves_reproducer_and_other_targets_still_run(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_CRASH_TARGET="workspace_normalization")
        self.assertEqual(proc.returncode, 1, proc.stdout)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "fail")
        by_target = {t["target"]: t for t in report["targets"]}
        crashed = by_target["workspace_normalization"]
        self.assertEqual(crashed["status"], "fail")
        self.assertEqual(crashed["exit_code"], "77")
        self.assertIn("crash-deadbeef", crashed["reproducer"])
        self.assertTrue((self.out / crashed["reproducer"]).is_file(), "reproducer must be preserved")
        self.assertEqual(report["counts"]["pass"], len(TARGETS) - 1)
        self.assertEqual(by_target["entity_extraction"]["status"], "pass")

    def test_infrastructure_unavailable_is_not_run_not_pass(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_INFRA="missing")
        self.assertEqual(proc.returncode, 2)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "not-run")
        self.assertEqual(report["counts"]["not-run"], len(TARGETS))
        self.assertEqual(report["counts"]["pass"], 0)

    def test_unsupported_platform_is_unsupported_not_pass(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FUZZ_SMOKE_OS="Windows_NT")
        self.assertEqual(proc.returncode, 3)
        report = self.read_report("fuzz")
        self.assertEqual(report["status"], "unsupported")
        self.assertEqual(report["counts"]["unsupported"], len(TARGETS))

    def test_build_failure_fails_that_target(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_BUILD_FAIL="entity_extraction")
        self.assertEqual(proc.returncode, 1)
        report = self.read_report("fuzz")
        row = {t["target"]: t for t in report["targets"]}["entity_extraction"]
        self.assertEqual((row["status"], row["stage"]), ("fail", "build"))
        self.assertTrue(any("compiled targets != declared" in e for e in report["lane_errors"]))

    def test_exit_zero_without_execution_evidence_fails(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(), FAKE_NO_STATS_TARGET="text_util_boundaries")
        self.assertEqual(proc.returncode, 1)
        row = {t["target"]: t for t in self.read_report("fuzz")["targets"]}["text_util_boundaries"]
        self.assertEqual(row["status"], "fail")

    def test_missing_seed_corpus_fails(self) -> None:
        proc = self.run_fuzz(self.make_fuzz_dir(seeds=["entity_extraction", "workspace_normalization"]))
        self.assertEqual(proc.returncode, 1)
        row = {t["target"]: t for t in self.read_report("fuzz")["targets"]}["text_util_boundaries"]
        self.assertEqual((row["status"], row["stage"]), ("fail", "seeds"))


class MiriRunnerTests(FakeCargoCase):
    def run_miri(self, filters: list[str], **env: str) -> subprocess.CompletedProcess:
        inventory = self.tmp / "miri-inventory.txt"
        inventory.write_text("# nominal\n" + "\n".join(filters) + ("\n" if filters else ""))
        return self.run_script(
            MIRI_SCRIPT,
            MIRI_INVENTORY=str(inventory),
            MIRI_SMOKE_OUT=str(self.out),
            MIRI_SMOKE_OS="Linux",
            **env,
        )

    def test_matching_filters_pass_with_counts(self) -> None:
        proc = self.run_miri(["a::tests::one", "b::tests::two"])
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        report = self.read_report("miri")
        self.assertEqual(report["status"], "pass")
        self.assertEqual([t["executed"] for t in report["targets"]], [3, 3])

    def test_empty_inventory_is_not_a_pass(self) -> None:
        proc = self.run_miri([])
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(self.read_report("miri")["status"], "fail")

    def test_filter_matching_zero_tests_is_not_a_pass(self) -> None:
        proc = self.run_miri(["a::tests::one", "workspace::nomatch"])
        self.assertEqual(proc.returncode, 1)
        report = self.read_report("miri")
        self.assertEqual(report["status"], "fail")
        row = {t["target"]: t for t in report["targets"]}["workspace::nomatch"]
        self.assertEqual((row["status"], row["stage"], row["executed"]), ("fail", "filter", 0))

    def test_failing_test_is_not_a_pass(self) -> None:
        proc = self.run_miri(["a::failing"])
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(self.read_report("miri")["status"], "fail")

    def test_missing_miri_is_not_run(self) -> None:
        proc = self.run_miri(["a::tests::one"], FAKE_INFRA="missing")
        self.assertEqual(proc.returncode, 2)
        self.assertEqual(self.read_report("miri")["status"], "not-run")

    def test_repo_inventory_is_non_empty_and_excludes_sqlite_tests(self) -> None:
        lines = [
            re.sub(r"#.*", "", line).strip()
            for line in (SCRIPTS / "miri-inventory.txt").read_text(encoding="utf-8").splitlines()
        ]
        filters = [line for line in lines if line]
        self.assertGreater(len(filters), 0)
        scoping = (ROOT / "src" / "storage" / "scoping.rs").read_text(encoding="utf-8")
        sqlite_tests = re.findall(r"fn (test_(?:set|get|search|move|scope_tree|list)\w*)", scoping)
        prefix = "storage::scoping::tests::"
        self.assertGreater(len(sqlite_tests), 0)
        for item in (f for f in filters if f.startswith(prefix)):
            for name in sqlite_tests:
                self.assertFalse(
                    name.startswith(item[len(prefix) :]),
                    f"Miri filter {item!r} would select SQLite-backed test {name}",
                )


class RepoInventoryTests(unittest.TestCase):
    def test_declared_inventory_matches_cargo_bins_files_and_seeds(self) -> None:
        declared = [
            re.sub(r"#.*", "", line).strip()
            for line in (FUZZ_DIR / "targets.inventory").read_text(encoding="utf-8").splitlines()
        ]
        declared = sorted(line for line in declared if line)
        self.assertGreaterEqual(len(declared), 2, "inventory must be non-empty")
        for required in ("entity_extraction", "workspace_normalization"):
            self.assertIn(required, declared)

        manifest = tomllib.loads((FUZZ_DIR / "Cargo.toml").read_text(encoding="utf-8"))
        bins = sorted(b["name"] for b in manifest["bin"])
        self.assertEqual(bins, declared)
        files = sorted(p.stem for p in (FUZZ_DIR / "fuzz_targets").glob("*.rs"))
        self.assertEqual(files, declared)
        for name in declared:
            seeds = [p for p in (FUZZ_DIR / "seeds" / name).glob("*") if p.is_file()]
            self.assertGreater(len(seeds), 0, f"{name} needs a seed corpus")
        self.assertEqual(manifest["workspace"]["members"], ["."], "fuzz must be its own workspace")

    def test_root_workspace_does_not_include_fuzz(self) -> None:
        root = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertNotIn("fuzz", root["workspace"]["members"])
        self.assertNotIn("libfuzzer-sys", (ROOT / "Cargo.lock").read_text(encoding="utf-8"))


def jobs_block(text: str) -> dict[str, str]:
    """Split the `jobs:` mapping of a workflow into {job id: raw text}."""
    body = text.split("\njobs:\n", 1)[1]
    jobs: dict[str, list[str]] = {}
    current = None
    for line in body.splitlines():
        match = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if match:
            current = match.group(1)
            jobs[current] = []
        elif current is not None:
            jobs[current].append(line)
    return {name: "\n".join(lines) for name, lines in jobs.items()}


def plan_run_block(text: str) -> str:
    plan = jobs_block(text)["plan"]
    lines = plan.splitlines()
    start = next(i for i, line in enumerate(lines) if re.match(r"^\s+run: \|\s*$", line))
    indent = len(lines[start]) - len(lines[start].lstrip()) + 2
    block = []
    for line in lines[start + 1 :]:
        if line.strip() and len(line) - len(line.lstrip()) < indent:
            break
        block.append(line[indent:] if line.strip() else "")
    return "\n".join(block)


def schedule_crons(text: str) -> list[str]:
    section = text.split("\n  schedule:\n", 1)[1].split("\n  workflow_dispatch", 1)[0]
    return re.findall(r"^\s+- cron: '([^']+)'", section, flags=re.MULTILINE)


class NightlyWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = NIGHTLY.read_text(encoding="utf-8")
        cls.jobs = jobs_block(cls.text)

    def resolve_lane(self, event_name: str, schedule: str) -> tuple[int, dict[str, str]]:
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "github_output"
            output.write_text("")
            proc = subprocess.run(
                ["bash", "-c", plan_run_block(self.text)],
                env={
                    **os.environ,
                    "EVENT_NAME": event_name,
                    "EVENT_SCHEDULE": schedule,
                    "GITHUB_OUTPUT": str(output),
                },
                capture_output=True,
                text=True,
                check=False,
            )
            values = dict(
                line.split("=", 1) for line in output.read_text(encoding="utf-8").splitlines() if "=" in line
            )
            return proc.returncode, values

    def test_schedule_crons_cover_each_weekday_exactly_once(self) -> None:
        crons = schedule_crons(self.text)
        self.assertEqual(len(crons), 2, crons)
        covered: list[int] = []
        for cron in crons:
            minute, hour, dom, month, dow = cron.split()
            self.assertEqual((minute, hour, dom, month), ("0", "3", "*", "*"))
            for part in dow.split(","):
                if "-" in part:
                    low, high = part.split("-")
                    covered.extend(range(int(low), int(high) + 1))
                else:
                    covered.append(int(part))
        self.assertEqual(sorted(covered), list(range(7)))

    def test_every_cron_maps_to_a_lane_and_weekly_runs_mutants(self) -> None:
        crons = schedule_crons(self.text)
        results = {}
        for cron in crons:
            code, values = self.resolve_lane("schedule", cron)
            self.assertEqual(code, 0, cron)
            results[cron] = values
        lanes = {values["lane"]: values["run_mutants"] for values in results.values()}
        self.assertEqual(lanes, {"daily": "false", "weekly": "true"})
        weekly = next(c for c, v in results.items() if v["lane"] == "weekly")
        self.assertEqual(weekly.split()[4], "0", "weekly lane must be the Sunday cron")

    def test_manual_dispatch_runs_everything(self) -> None:
        code, values = self.resolve_lane("workflow_dispatch", "")
        self.assertEqual(code, 0)
        self.assertEqual(values, {"lane": "manual", "run_mutants": "true"})

    def test_unmapped_trigger_fails_instead_of_guessing(self) -> None:
        for event_name, schedule in (("schedule", "0 3 * * *"), ("schedule", ""), ("push", ""), ("workflow_dispatch", "0 3 * * 0")):
            code, values = self.resolve_lane(event_name, schedule)
            self.assertNotEqual(code, 0, (event_name, schedule))
            self.assertEqual(values, {})

    def test_mutants_job_is_gated_by_the_resolved_lane(self) -> None:
        mutants = self.jobs["mutants"]
        self.assertRegex(mutants, r"needs:\s*plan")
        self.assertIn("needs.plan.outputs.run_mutants == 'true'", mutants)
        conditions = [l for l in mutants.splitlines() if l.startswith("    if:")]  # job-level only
        self.assertEqual(len(conditions), 1)
        self.assertNotIn("github.event", conditions[0], "gate must not compare the raw cron string")

    def test_q3_lanes_cannot_mask_failures(self) -> None:
        for name in ("fuzz", "mutants", "miri"):
            body = self.jobs[name]
            self.assertNotIn("continue-on-error", body, name)
            for line in body.splitlines():
                code = line.split("#", 1)[0]
                self.assertNotRegex(code, r"\|\|\s*(true|:)\b", f"{name}: {line.strip()}")
        for script in (FUZZ_SCRIPT, MIRI_SCRIPT):
            for line in script.read_text(encoding="utf-8").splitlines():
                code = line.split("#", 1)[0]
                self.assertNotRegex(code, r"\|\|\s*(true|:)\b", f"{script.name}: {line.strip()}")

    def test_tools_are_pinned_and_locked(self) -> None:
        for name in ("fuzz", "mutants"):
            installs = [l for l in self.jobs[name].splitlines() if "cargo install" in l]
            self.assertGreaterEqual(len(installs), 1, name)
            for install in installs:
                self.assertRegex(install, r"--version \d+\.\d+\.\d+")
                self.assertIn("--locked", install)

    def test_lane_jobs_run_the_contract_scripts_and_upload_reports_always(self) -> None:
        self.assertIn("bash scripts/run-fuzz-smoke.sh", self.jobs["fuzz"])
        self.assertIn("bash scripts/run-miri-smoke.sh", self.jobs["miri"])
        for name in ("fuzz", "mutants", "miri"):
            self.assertRegex(self.jobs[name], r"if: always\(\)\s*\n\s+uses: actions/upload-artifact@")

    def test_fuzz_job_runs_sixty_seconds_per_target_with_tight_grace(self) -> None:
        self.assertIn("FUZZ_SMOKE_MAX_TOTAL_TIME: '60'", self.jobs["fuzz"])
        self.assertIn("FUZZ_SMOKE_WALL_GRACE: '120'", self.jobs["fuzz"])

    def test_nightly_toolchain_is_a_dated_pin_used_by_fuzz_and_miri(self) -> None:
        self.assertRegex(self.text, r"(?m)^  NIGHTLY_TOOLCHAIN: nightly-\d{4}-\d{2}-\d{2}\s*$")
        for name, env_name in (("fuzz", "FUZZ_SMOKE_TOOLCHAIN"), ("miri", "MIRI_SMOKE_TOOLCHAIN")):
            body = self.jobs[name]
            self.assertIn("toolchain: ${{ env.NIGHTLY_TOOLCHAIN }}", body, name)
            self.assertIn(f"{env_name}: ${{{{ env.NIGHTLY_TOOLCHAIN }}}}", body, name)

    def test_fuzz_job_audits_the_separate_fuzz_lockfile(self) -> None:
        body = self.jobs["fuzz"]
        self.assertRegex(body, r"cargo install cargo-audit --version \d+\.\d+\.\d+ --locked")
        self.assertIn("cargo audit --file fuzz/Cargo.lock", body)

    def test_plan_job_runs_these_contract_tests(self) -> None:
        self.assertIn("python3 -m unittest scripts/test_optional_lane_contracts.py", self.jobs["plan"])
        self.assertIn("actions/checkout@", self.jobs["plan"])

    def test_mutants_job_is_scoped_and_fail_closed(self) -> None:
        body = self.jobs["mutants"]
        self.assertNotIn("continue-on-error", body)
        timeout = int(re.search(r"timeout-minutes:\s*(\d+)", body).group(1))
        self.assertLessEqual(timeout, 300)
        run = re.search(r"run: (cargo mutants[^\n]*)", body).group(1)
        self.assertNotIn("--in-place", run)
        config = tomllib.loads((ROOT / ".cargo" / "mutants.toml").read_text(encoding="utf-8"))
        globs = config["examine_globs"]
        self.assertGreaterEqual(len(globs), 1, "mutants must be scoped by examine_globs")
        self.assertLessEqual(len(globs), 8, "scope grew: re-measure the job duration before widening")
        total_lines = 0
        for glob in globs:
            self.assertNotIn("*", glob, "list explicit module files, not wildcards")
            path = ROOT / glob
            self.assertTrue(path.is_file(), glob)
            total_lines += len(path.read_text(encoding="utf-8").splitlines())
        self.assertLess(total_lines, 3000, "whole-lib-sized scope cannot finish in the job timeout")
        for glob in ("src/text_util.rs", "src/storage/scoping.rs", "src/intelligence/entity_extraction.rs", "src/auth/transport_principal.rs"):
            self.assertIn(glob, globs)
        for key in ("exclude_globs", "skip_calls"):
            self.assertNotIn(key, config, "config must not silently hide mutants")
        text = (ROOT / ".cargo" / "mutants.toml").read_text(encoding="utf-8")
        self.assertIn("Owner: Ronaldo", text)
        self.assertIn("How to widen", text)


class ReporterTests(unittest.TestCase):
    @staticmethod
    def row(status: str) -> dict:
        return {"target": "t", "status": status, "stage": "run", "exit_code": "0", "elapsed_secs": 1, "executed": 1,
                "corpus_files": 0, "seed_files": 1, "reproducer": "", "note": ""}

    def test_precedence_and_empty_lane(self) -> None:
        pass_row = self.row("pass")
        self.assertEqual(reporter.overall_status([pass_row], [], None), "pass")
        self.assertEqual(reporter.overall_status([], [], None), "fail")
        self.assertEqual(reporter.overall_status([pass_row], ["drift"], None), "fail")
        self.assertEqual(reporter.overall_status([pass_row, self.row("fail")], [], None), "fail")
        self.assertEqual(reporter.overall_status([pass_row, self.row("not-run")], [], None), "not-run")
        self.assertEqual(reporter.overall_status([self.row("unsupported")], [], "unsupported"), "unsupported")

    def test_malformed_rows_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            reporter.parse_rows("only\ttwo\n")
        with self.assertRaises(ValueError):
            reporter.parse_rows("\t".join(["t", "green", "run", "0", "1", "1", "0", "1", "", ""]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
