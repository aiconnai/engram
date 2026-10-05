#!/usr/bin/env python3
"""Contract tests for scripts/run-quality-candidate.py and its Criterion capture (Q7).

Each test builds a throw-away git repository holding copies of the real
fixtures, floors, budgets and runner, plus a fake `cargo`/`rustc` on PATH, so no
cargo build is needed. The cases required by the brief are covered: different
candidate, missing/NaN metric, corpus hash divergence, regression, edited floor,
and a Criterion file that is stale/historical/for another candidate.

Run: python3 -m unittest scripts/test_run_quality_candidate.py
"""

from __future__ import annotations

import contextlib
import copy
import io
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
ROOT = SCRIPTS.parent
sys.path.insert(0, str(SCRIPTS))

from quality_candidate import capture as capture_mod  # noqa: E402
from quality_candidate import criterion as crit  # noqa: E402
from quality_candidate import floors as floors_mod  # noqa: E402
from quality_candidate import runner as runner_mod  # noqa: E402
from quality_candidate.common import (  # noqa: E402
    EVAL_MARKER,
    METRICS,
    MODES,
    format_utc,
    sha256_file,
    utc_now,
)

CANDIDATE_CORPUS = "tests/fixtures/retrieval_quality/candidate_corpus.json"
V1_CORPUS = "tests/fixtures/retrieval_quality/corpus.json"
FEATURES = "openai,pdf"
TOOLCHAIN = "rustc 1.99.0 (fake 2026-01-01)"

COPY_FILES = [
    "Cargo.toml",
    "Cargo.lock",
    "scripts/check-quality-budgets.py",
    "scripts/run-quality-candidate.py",
    "scripts/capture-criterion-candidate.py",
    "docs/quality/budgets.json",
    "docs/quality/candidate-floors.json",
    "tests/fixtures/retrieval_quality/corpus.json",
    "tests/fixtures/retrieval_quality/candidate_corpus.json",
    "tests/fixtures/retrieval_quality/baseline.json",
    "benches/results/benchmark_results.txt",
]

FAKE_CARGO = r'''#!/usr/bin/env python3
import hashlib, json, os, sys
here = os.path.dirname(os.path.realpath(__file__))
cfg = json.load(open(os.path.join(here, "config.json")))
with open(os.path.join(here, "calls.log"), "a") as log:
    log.write(json.dumps(sys.argv[1:]) + "\n")
if sys.argv[1:2] == ["-V"]:
    print("cargo 1.99.0 (fake)")
    sys.exit(0)
if sys.argv[1] == "test":
    if cfg.get("dirty_after"):
        open(os.path.join(os.getcwd(), "build-artifact.txt"), "w").write("mutated\n")
    if cfg.get("test_exit"):
        print("error: could not compile", file=sys.stderr)
        sys.exit(cfg["test_exit"])
    ev = json.loads(json.dumps(cfg["eval"]))
    ev["candidate_sha"] = cfg.get("echo_sha", os.environ.get("ENGRAM_CANDIDATE_SHA"))
    ev["features"] = os.environ.get("ENGRAM_CANDIDATE_FEATURES")
    corpus = os.environ["ENGRAM_RETRIEVAL_CORPUS"]
    ev["corpus_sha256"] = cfg.get("echo_corpus_sha") or hashlib.sha256(open(corpus, "rb").read()).hexdigest()
    for key in cfg.get("drop_keys", []):
        ev.pop(key, None)
    line = EVAL_MARKER + json.dumps(ev)
    ran = cfg.get("tests_run", 1)
    print("running %d test" % ran)
    for _ in range(cfg.get("marker_lines", 1)):
        print("test candidate_retrieval_metrics ... " + line)
    print("test result: ok. %d passed; 0 failed; 0 ignored" % ran)
    sys.exit(0)
if sys.argv[1] == "bench":
    print(cfg["bench_stdout"])
    sys.exit(0)
sys.exit(97)
'''.replace("EVAL_MARKER", repr(EVAL_MARKER))

FAKE_RUSTC = '#!/usr/bin/env bash\necho "' + TOOLCHAIN + '"\n'


def run_git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@example.invalid",
         "-c", "commit.gpgsign=false", *args],
        capture_output=True, text=True, check=True,
    ).stdout.strip()


def criterion_body(ratio: float = 1.0) -> str:
    budgets = json.loads((ROOT / "docs/quality/budgets.json").read_text())
    units = {"ns": 1e-9, "us": 1e-6, "ms": 1e-3, "s": 1.0}
    parts = []
    for name, raw in budgets["criterion"]["hot_paths"].items():
        seconds = raw["baseline_value"] * units[raw["baseline_unit"]] * ratio
        parts.append(f"{name}\n                        time:   [{seconds} s {seconds} s {seconds} s]\n")
    return "\n".join(parts)


class Harness:
    """A temp candidate repo + fake toolchain + helpers to run the runner."""

    def __init__(self, case: unittest.TestCase) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="q7-"))
        case.addCleanup(shutil.rmtree, self.tmp, True)
        self.repo = self.tmp / "repo"
        self.bin = self.tmp / "bin"
        self.out = self.tmp / "out"
        self.out.mkdir()
        self.bin.mkdir()
        for rel in COPY_FILES:
            dst = self.repo / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / rel, dst)
        shutil.copytree(SCRIPTS / "quality_candidate", self.repo / "scripts/quality_candidate",
                        ignore=shutil.ignore_patterns("__pycache__"))
        run_git(self.repo, "init", "-q")
        self.commit("candidate")
        for name, body in (("cargo", FAKE_CARGO), ("rustc", FAKE_RUSTC)):
            path = self.bin / name
            path.write_text(body)
            path.chmod(path.stat().st_mode | stat.S_IEXEC)
        self.floors = json.loads((self.repo / "docs/quality/candidate-floors.json").read_text())
        self.config = {"eval": {}, "bench_stdout": criterion_body()}
        self.environ = {"PATH": f"{self.bin}{os.pathsep}{os.environ['PATH']}", "HOME": str(self.tmp)}
        self.use_corpus(CANDIDATE_CORPUS)

    # -- repo state -------------------------------------------------------
    def commit(self, message: str) -> None:
        run_git(self.repo, "add", "-A")
        run_git(self.repo, "commit", "-q", "--allow-empty", "-m", message)

    @property
    def sha(self) -> str:
        return run_git(self.repo, "rev-parse", "HEAD")

    @property
    def tree(self) -> str:
        return run_git(self.repo, "rev-parse", "HEAD^{tree}")

    def write_json(self, rel: str, value: object) -> None:
        (self.repo / rel).write_text(json.dumps(value, indent=2) + "\n")

    def reseal_and_commit(self, doc: dict, message: str) -> None:
        for entry in doc["corpora"]:
            entry["entry_digest"] = floors_mod.entry_digest(entry)
        self.write_json("docs/quality/candidate-floors.json", doc)
        self.commit(message)

    # -- fake toolchain output --------------------------------------------
    def entry_for(self, corpus: str) -> dict:
        sha = sha256_file(self.repo / corpus)
        return floors_mod.select_entry(floors_mod.parse_floors(
            (self.repo / "docs/quality/candidate-floors.json").read_text()), sha)

    def use_corpus(self, corpus: str) -> None:
        self.corpus = corpus
        entry = self.entry_for(corpus)
        self.config["eval"] = {
            "schema": "engram.retrieval-candidate-eval.v1",
            "corpus_name": entry["name"],
            "corpus_version": entry["version"],
            "deterministic_seed": entry["deterministic_seed"],
            "memory_count": entry["memory_count"],
            "query_count": entry["query_count"],
            "provider": {"embedder": "tfidf", "dimensions": 384, "network": False},
            "label_provenance": {"human_review": "none"},
            "modes": {mode: {"metrics": dict(entry["floors"][mode]), "categories": {}, "discouraged": {}}
                      for mode in MODES},
        }
        self.save_config()

    def save_config(self) -> None:
        (self.bin / "config.json").write_text(json.dumps(self.config))

    def set_metric(self, mode: str, metric: str, value: object) -> None:
        self.config["eval"]["modes"][mode]["metrics"][metric] = value
        self.save_config()

    def calls(self) -> list:
        log = self.bin / "calls.log"
        return [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []

    # -- criterion ---------------------------------------------------------
    def criterion(self, *, ratio: float = 1.0, name: str = "criterion.txt", **overrides) -> Path:
        meta = {
            "candidate_sha": self.sha, "candidate_tree": self.tree, "features": FEATURES,
            "toolchain": TOOLCHAIN, "supervisor": "local",
            "captured_at": format_utc(utc_now()),
            "bench_argv": [["cargo", "bench"]], "bench_params": {"sample_size": 100},
            "hardware": {"cpu_model": "fake"},
        }
        body = overrides.pop("body", criterion_body(ratio))
        meta.update(overrides)
        path = self.out / name
        path.write_text(crit.render(meta, body))
        return path

    # -- run ---------------------------------------------------------------
    def run(self, *, sha=None, corpus=None, features=FEATURES, criterion=None, output=None,
            anchor=None, require_supervisor=False, candidate_dir=None, environ=None):
        output = output or self.out / "report.json"
        criterion = criterion or self.criterion()
        cand = Path(candidate_dir) if candidate_dir else self.repo
        argv = ["--candidate-sha", sha or self.sha, "--corpus", str(cand / (corpus or self.corpus)),
                "--features", features, "--criterion", str(criterion), "--output", str(output)]
        if anchor:
            argv += ["--floors-anchor", anchor]
        if require_supervisor:
            argv += ["--require-supervisor"]
        if candidate_dir:
            argv += ["--candidate-dir", str(candidate_dir)]
        err, out = io.StringIO(), io.StringIO()
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(out):
            code = runner_mod.run(argv, repo=self.repo, environ=environ or self.environ)
        self.stderr = err.getvalue()
        report = json.loads(Path(output).read_text()) if Path(output).exists() else None
        return code, report


class RunnerBase(unittest.TestCase):
    def setUp(self) -> None:
        self.h = Harness(self)

    def assertFails(self, code, report, needle: str):
        self.assertNotEqual(code, 0, report)
        self.assertEqual(report["status"], "fail")
        self.assertTrue(any(needle in e for e in report["errors"]), report["errors"])


class HappyPathTests(RunnerBase):
    def test_passing_run_reports_candidate_corpus_features_and_criterion(self):
        code, report = self.h.run()
        self.assertEqual(code, 0, report)
        self.assertEqual(report["status"], "pass")
        self.assertEqual(report["candidate"], {"sha": self.h.sha, "tree": self.h.tree, "clean": True})
        self.assertEqual(report["features"], sorted(FEATURES.split(",")))
        self.assertEqual(report["toolchain"]["rustc"], TOOLCHAIN)
        corpus = report["corpus"]
        self.assertEqual(corpus["sha256"], sha256_file(self.h.repo / CANDIDATE_CORPUS))
        self.assertEqual(corpus["deterministic_seed"], 20261005)
        self.assertEqual(report["provider"]["network"], False)
        self.assertEqual(set(report["modes"]), set(MODES))
        self.assertEqual(len(report["retrieval_floor_checks"]), len(MODES) * len(METRICS))
        crit_report = report["criterion"]
        self.assertEqual(crit_report["ceiling"], 1.15)
        self.assertEqual(crit_report["marker"]["candidate_sha"], self.h.sha)
        self.assertIn("never accepted as candidate evidence", crit_report["historical_context"]["role"])
        self.assertEqual(
            crit_report["historical_context"]["sha256"],
            sha256_file(self.h.repo / "benches/results/benchmark_results.txt"),
        )

    def test_runs_only_the_fixed_approved_argv(self):
        code, report = self.h.run()
        self.assertEqual(code, 0, report)
        tests = [call for call in self.h.calls() if call[0] == "test"]
        self.assertEqual(len(tests), 1)
        self.assertEqual(tests[0], [
            "test", "--locked", "--test", "retrieval_quality", "--no-default-features",
            "--features", "openai,pdf", "--", "--exact", "candidate_retrieval_metrics",
            "--nocapture", "--test-threads=1",
        ])
        self.assertEqual(report["argv"][0], "cargo")
        # Nothing but cargo test and the two version probes was executed.
        self.assertTrue(all(call[0] in ("test", "-V") for call in self.h.calls()), self.h.calls())

    def test_v1_smoke_corpus_is_supported_and_reconciled_with_budgets(self):
        self.h.use_corpus(V1_CORPUS)
        code, report = self.h.run()
        self.assertEqual(code, 0, report)
        self.assertTrue(report["floors"]["reconciled_with_budgets"])

    def test_cli_wrapper_script_end_to_end(self):
        criterion = self.h.criterion()
        output = self.h.out / "cli.json"
        env = dict(self.h.environ)
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        completed = subprocess.run(
            [sys.executable, str(self.h.repo / "scripts/run-quality-candidate.py"),
             "--candidate-sha", self.h.sha, "--corpus", str(self.h.repo / CANDIDATE_CORPUS),
             "--features", FEATURES, "--criterion", str(criterion), "--output", str(output)],
            capture_output=True, text=True, env=env, cwd=self.h.tmp,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(json.loads(output.read_text())["status"], "pass")


class CandidateIdentityTests(RunnerBase):
    def test_different_candidate_sha_is_rejected(self):
        code, report = self.h.run(sha="0" * 40)
        self.assertFails(code, report, "candidate mismatch")
        self.assertEqual(self.h.calls(), [], "no command may run for the wrong candidate")

    def test_abbreviated_sha_is_rejected(self):
        code, report = self.h.run(sha=self.h.sha[:12])
        self.assertFails(code, report, "40-character")

    def test_dirty_checkout_is_rejected(self):
        (self.h.repo / "stray.txt").write_text("uncommitted\n")
        code, report = self.h.run()
        self.assertFails(code, report, "not clean")

    def test_unknown_or_shell_like_feature_is_rejected(self):
        for features in ("nonexistent-feature", "openai;touch pwned", "openai --release"):
            code, report = self.h.run(features=features)
            self.assertNotEqual(code, 0, features)
        self.assertFalse((self.h.tmp / "pwned").exists())
        self.assertEqual([c for c in self.h.calls() if c[0] == "test"], [])

    def test_untracked_corpus_outside_checkout_is_rejected(self):
        outside = self.h.out / "corpus.json"
        shutil.copy2(self.h.repo / CANDIDATE_CORPUS, outside)
        code, report = self.h.run(corpus=str(outside))
        self.assertFails(code, report, "outside the candidate checkout")


class MetricValidationTests(RunnerBase):
    def test_missing_metric_is_rejected(self):
        del self.h.config["eval"]["modes"]["hybrid_tfidf"]["metrics"]["ndcg@10"]
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "missing metric(s) ndcg@10")

    def test_missing_mode_is_rejected(self):
        del self.h.config["eval"]["modes"]["lexical_fts5"]
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "modes must be exactly")

    def test_nan_metric_is_rejected(self):
        self.h.set_metric("lexical_fts5", "mrr", float("nan"))
        code, report = self.h.run()
        self.assertFails(code, report, "non-finite")

    def test_out_of_range_metric_is_rejected(self):
        self.h.set_metric("lexical_fts5", "recall@10", 1.5)
        code, report = self.h.run()
        self.assertFails(code, report, "within [0, 1]")

    def test_string_metric_is_rejected(self):
        self.h.set_metric("hybrid_tfidf", "mrr", "0.9")
        code, report = self.h.run()
        self.assertFails(code, report, "must be a number")

    def test_zero_tests_executed_is_rejected(self):
        self.h.config["tests_run"] = 0
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "did not run exactly 1 test")

    def test_two_eval_lines_are_rejected(self):
        self.h.config["marker_lines"] = 2
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "exactly one")

    def test_failing_test_binary_is_rejected(self):
        self.h.config["test_exit"] = 101
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "exited 101")

    def test_process_reporting_a_different_candidate_is_rejected(self):
        self.h.config["echo_sha"] = "1" * 40
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "candidate_sha mismatch")

    def test_process_reporting_a_different_seed_is_rejected(self):
        self.h.config["eval"]["deterministic_seed"] = 7
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "deterministic_seed mismatch")

    def test_process_without_offline_provider_record_is_rejected(self):
        self.h.config["eval"]["provider"] = {"embedder": "openai", "network": True}
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "offline provider")


class CorpusHashTests(RunnerBase):
    def test_edited_corpus_without_reviewed_floors_entry_is_rejected(self):
        path = self.h.repo / CANDIDATE_CORPUS
        doc = json.loads(path.read_text())
        doc["queries"][0]["relevance"] = {"acme-pricing": 3}  # relabel to move a metric
        path.write_text(json.dumps(doc, indent=2) + "\n")
        self.h.commit("relabel")
        code, report = self.h.run()
        self.assertFails(code, report, "corpus hash divergence")

    def test_process_evaluating_another_corpus_is_rejected(self):
        self.h.config["echo_corpus_sha"] = sha256_file(self.h.repo / V1_CORPUS)
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "corpus_sha256 mismatch")

    def test_counts_that_diverge_from_reviewed_entry_are_rejected(self):
        self.h.config["eval"]["query_count"] += 1
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "query_count mismatch")


class RegressionTests(RunnerBase):
    def test_metric_below_floor_is_rejected_and_named(self):
        entry = self.h.entry_for(CANDIDATE_CORPUS)
        self.h.set_metric("hybrid_tfidf", "ndcg@10", entry["floors"]["hybrid_tfidf"]["ndcg@10"] - 0.01)
        code, report = self.h.run()
        self.assertFails(code, report, "retrieval floor regression: hybrid_tfidf.ndcg@10")

    def test_metric_exactly_at_floor_passes_and_above_passes(self):
        self.h.set_metric("hybrid_tfidf", "ndcg@10", 1.0)
        code, report = self.h.run()
        self.assertEqual(code, 0, report)

    def test_criterion_regression_above_ceiling_is_rejected(self):
        code, report = self.h.run(criterion=self.h.criterion(ratio=1.16))
        self.assertFails(code, report, "criterion regression")

    def test_criterion_at_ceiling_passes(self):
        code, report = self.h.run(criterion=self.h.criterion(ratio=1.15))
        self.assertEqual(code, 0, report)

    def test_ceiling_edit_in_budgets_is_rejected(self):
        budgets = json.loads((self.h.repo / "docs/quality/budgets.json").read_text())
        budgets["criterion"]["maximum_regression_ratio"] = 1.5
        self.h.write_json("docs/quality/budgets.json", budgets)
        self.h.commit("relax ceiling")
        code, report = self.h.run(criterion=self.h.criterion(ratio=1.3))
        self.assertFails(code, report, "expected 1.15")

    def test_failed_run_removes_a_previous_passing_report(self):
        code, report = self.h.run()
        self.assertEqual(code, 0)
        code, report = self.h.run(sha="2" * 40)
        self.assertEqual(report["status"], "fail")
        self.assertNotIn("modes", report)


class FloorEditTests(RunnerBase):
    def lower(self, doc: dict, corpus: str, mode: str, metric: str, value: float) -> None:
        sha = sha256_file(self.h.repo / corpus)
        for entry in doc["corpora"]:
            if entry["corpus_sha256"] == sha:
                entry["floors"][mode][metric] = value

    def test_floor_edited_without_reseal_is_rejected(self):
        doc = json.loads((self.h.repo / "docs/quality/candidate-floors.json").read_text())
        self.lower(doc, CANDIDATE_CORPUS, "hybrid_tfidf", "mrr", 0.1)
        self.h.write_json("docs/quality/candidate-floors.json", doc)
        self.h.commit("edit floor")
        self.h.set_metric("hybrid_tfidf", "mrr", 0.2)
        code, report = self.h.run()
        self.assertFails(code, report, "entry_digest mismatch")

    # -- trusted anchor (supervisor supplied) ------------------------------
    def accept_and_anchor(self, status="accepted-independent-review"):
        """Commit an entry with the given review status, then a later candidate commit."""
        doc = json.loads((self.h.repo / "docs/quality/candidate-floors.json").read_text())
        sha = sha256_file(self.h.repo / CANDIDATE_CORPUS)
        for entry in doc["corpora"]:
            if entry["corpus_sha256"] == sha:
                entry["review"]["status"] = status
        self.h.reseal_and_commit(doc, "review accepted")
        anchor = self.h.sha
        (self.h.repo / "CHANGELOG.md").write_text("candidate change\n")
        self.h.commit("candidate")
        return anchor

    def test_no_supervisor_anchor_means_not_anchored_and_not_accepted(self):
        code, report = self.h.run()
        self.assertEqual(code, 0, report)
        self.assertFalse(report["floors"]["anchored"])
        self.assertFalse(report["floors"]["accepted"])
        self.assertIn("no supervisor", report["floors"]["anchor_reason"])

    def test_trusted_ancestor_with_accepted_review_is_anchored_and_accepted(self):
        anchor = self.accept_and_anchor()
        code, report = self.h.run(anchor=anchor)
        self.assertEqual(code, 0, report)
        self.assertTrue(report["floors"]["anchored"])
        self.assertTrue(report["floors"]["accepted"])
        self.assertEqual(report["floors"]["anchor_revision"], anchor)

    def test_anchored_but_review_not_accepted_is_not_accepted(self):
        anchor = self.accept_and_anchor(status="proposed-pending-independent-review")
        code, report = self.h.run(anchor=anchor)
        self.assertEqual(code, 0, report)
        self.assertTrue(report["floors"]["anchored"])
        self.assertFalse(report["floors"]["accepted"])
        self.assertIn("not 'accepted-independent-review'", report["floors"]["anchor_reason"])

    def test_anchor_equal_to_candidate_commit_is_not_anchored(self):
        self.accept_and_anchor()
        code, report = self.h.run(anchor=self.h.sha)
        self.assertEqual(code, 0, report)
        self.assertFalse(report["floors"]["anchored"])
        self.assertFalse(report["floors"]["accepted"])
        self.assertIn("own commit", report["floors"]["anchor_reason"])

    def test_anchor_off_the_candidate_ancestry_is_not_anchored(self):
        base = self.h.sha
        run_git(self.h.repo, "checkout", "-q", "-b", "side")
        doc = json.loads((self.h.repo / "docs/quality/candidate-floors.json").read_text())
        for entry in doc["corpora"]:
            entry["review"]["status"] = "accepted-independent-review"
        self.h.reseal_and_commit(doc, "side branch accepted")
        side = self.h.sha
        run_git(self.h.repo, "checkout", "-q", "-")
        (self.h.repo / "CHANGELOG.md").write_text("x\n")
        self.h.commit("candidate")
        code, report = self.h.run(anchor=side)
        self.assertEqual(code, 0, report)
        self.assertFalse(report["floors"]["anchored"])
        self.assertIn("not an ancestor", report["floors"]["anchor_reason"])
        self.assertNotEqual(base, side)

    def test_unknown_anchor_commit_is_not_anchored(self):
        code, report = self.h.run(anchor="9" * 40)
        self.assertEqual(code, 0, report)
        self.assertFalse(report["floors"]["anchored"])

    def test_malformed_anchor_is_rejected(self):
        code, report = self.h.run(anchor="abc123")
        self.assertFails(code, report, "40-character")

    def test_forged_anchor_revision_in_candidate_floors_file_is_ignored(self):
        doc = json.loads((self.h.repo / "docs/quality/candidate-floors.json").read_text())
        doc["anchor_revision"] = self.h.sha
        for entry in doc["corpora"]:
            entry["review"]["status"] = "accepted-independent-review"
        self.h.reseal_and_commit(doc, "forge acceptance")
        code, report = self.h.run()
        self.assertEqual(code, 0, report)
        self.assertFalse(report["floors"]["anchored"])
        self.assertFalse(report["floors"]["accepted"])

    def test_floor_lower_than_trusted_anchor_is_rejected_even_when_resealed(self):
        anchor = self.accept_and_anchor()
        doc = json.loads((self.h.repo / "docs/quality/candidate-floors.json").read_text())
        self.lower(doc, CANDIDATE_CORPUS, "lexical_fts5", "recall@10", 0.1)
        self.h.reseal_and_commit(doc, "relax")
        self.h.set_metric("lexical_fts5", "recall@10", 0.2)
        code, report = self.h.run(anchor=anchor)
        self.assertFails(code, report, "floor relaxed vs anchor")

    def test_v1_floor_below_budgets_floor_is_rejected_even_when_resealed(self):
        doc = json.loads((self.h.repo / "docs/quality/candidate-floors.json").read_text())
        self.lower(doc, V1_CORPUS, "lexical_fts5", "mrr", 0.5)
        self.h.reseal_and_commit(doc, "relax v1 lexical floor")
        self.h.use_corpus(V1_CORPUS)
        self.h.set_metric("lexical_fts5", "mrr", 0.6)
        code, report = self.h.run()
        self.assertFails(code, report, "lane threshold relaxed")

    def test_seal_cli_recomputes_digest(self):
        path = self.h.repo / "docs/quality/candidate-floors.json"
        doc = json.loads(path.read_text())
        doc["corpora"][0]["floors"]["lexical_fts5"]["mrr"] = 0.5
        path.write_text(json.dumps(doc))
        with self.assertRaises(Exception):
            floors_mod.parse_floors(path.read_text())
        floors_mod.seal(path)
        floors_mod.parse_floors(path.read_text())


class CriterionBindingTests(RunnerBase):
    def test_bare_criterion_file_without_marker_is_rejected(self):
        bare = self.h.out / "bare.txt"
        bare.write_text(criterion_body())
        code, report = self.h.run(criterion=bare)
        self.assertFails(code, report, "no candidate marker")

    def test_tracked_historical_file_is_rejected_even_with_a_marker(self):
        code, report = self.h.run(criterion=self.h.repo / "benches/results/benchmark_results.txt")
        self.assertFails(code, report, "tracked historical file")

    def test_criterion_captured_for_another_candidate_is_rejected(self):
        code, report = self.h.run(criterion=self.h.criterion(candidate_sha="3" * 40))
        self.assertFails(code, report, "different candidate_sha")

    def test_criterion_for_another_tree_is_rejected(self):
        code, report = self.h.run(criterion=self.h.criterion(candidate_tree="4" * 40))
        self.assertFails(code, report, "different candidate_tree")

    def test_criterion_after_a_new_commit_is_stale(self):
        old = self.h.criterion(name="old.txt")
        (self.h.repo / "CHANGELOG.md").write_text("change\n")
        self.h.commit("next candidate")
        code, report = self.h.run(criterion=old)
        self.assertFails(code, report, "different candidate_sha")

    def test_old_capture_is_stale(self):
        old = format_utc(utc_now().replace(year=utc_now().year - 1))
        code, report = self.h.run(criterion=self.h.criterion(captured_at=old))
        self.assertFails(code, report, "stale")

    def test_future_capture_is_rejected(self):
        future = format_utc(utc_now().replace(year=utc_now().year + 1))
        code, report = self.h.run(criterion=self.h.criterion(captured_at=future))
        self.assertFails(code, report, "future")

    def test_numbers_edited_after_capture_are_rejected(self):
        path = self.h.criterion()
        path.write_text(path.read_text().replace("time:", "time:  ", 1))
        code, report = self.h.run(criterion=path)
        self.assertFails(code, report, "edited after capture")

    def test_different_features_toolchain_or_supervisor_are_rejected(self):
        for key, value in (("features", "pdf"), ("toolchain", "rustc 1.0.0"), ("supervisor", "ci-77")):
            code, report = self.h.run(criterion=self.h.criterion(**{key: value}))
            self.assertFails(code, report, f"different {key}")

    def test_missing_hot_path_is_rejected(self):
        code, report = self.h.run(criterion=self.h.criterion(body="unrelated/bench\n   time:   [1 s 1 s 1 s]\n"))
        self.assertFails(code, report, "missing named hot path")

    def test_nan_or_negative_timing_is_rejected(self):
        budgets = json.loads((ROOT / "docs/quality/budgets.json").read_text())
        for value in ("-1", "0", "inf"):
            body = "".join(
                f"{name}\n                        time:   [{value} s {value} s {value} s]\n"
                for name in budgets["criterion"]["hot_paths"]
            )
            code, report = self.h.run(criterion=self.h.criterion(body=body))
            self.assertNotEqual(code, 0, value)


class SupervisorAndTrustTests(RunnerBase):
    def env_with(self, **extra):
        env = dict(self.h.environ)
        env.update(extra)
        return env

    def test_require_supervisor_rejects_missing_and_local(self):
        for env in (self.h.environ, self.env_with(ENGRAM_QUALITY_SUPERVISOR="local"),
                    self.env_with(ENGRAM_QUALITY_SUPERVISOR=" ")):
            code, report = self.h.run(require_supervisor=True, environ=env)
            self.assertFails(code, report, "supervisor-owned id")
        self.assertEqual(self.h.calls(), [], "no command may run without a supervisor id")

    def test_require_supervisor_accepts_a_real_id_and_binds_the_marker(self):
        env = self.env_with(ENGRAM_QUALITY_SUPERVISOR="77-1")
        code, report = self.h.run(require_supervisor=True, environ=env,
                                  criterion=self.h.criterion(supervisor="77-1"))
        self.assertEqual(code, 0, report)
        self.assertTrue(report["supervisor_required"])
        self.assertEqual(report["supervisor"], "77-1")
        code, report = self.h.run(require_supervisor=True, environ=env,
                                  criterion=self.h.criterion(supervisor="other-run"))
        self.assertFails(code, report, "different supervisor")

    def test_capture_require_supervisor(self):
        output = self.h.out / "cap.txt"
        with contextlib.redirect_stderr(io.StringIO()), contextlib.redirect_stdout(io.StringIO()):
            code = capture_mod.run(["--candidate-sha", self.h.sha, "--features", FEATURES,
                                    "--output", str(output), "--require-supervisor"],
                                   repo=self.h.repo, environ=self.h.environ)
        self.assertNotEqual(code, 0)
        self.assertFalse(output.exists())

    def test_runner_runs_from_trusted_dir_and_never_imports_candidate_code(self):
        # Candidate checkout is a clone whose budgets module would write a sentinel.
        cand = self.h.tmp / "candidate"
        subprocess.run(["git", "clone", "-q", str(self.h.repo), str(cand)], check=True)
        sentinel = self.h.tmp / "executed-candidate-code"
        (cand / "scripts/check-quality-budgets.py").write_text(
            f"open({str(sentinel)!r}, 'w').write('pwned')\nraise SystemExit(0)\n")
        run_git(cand, "add", "-A")
        run_git(cand, "commit", "-q", "-m", "malicious budgets module")
        cand_sha = run_git(cand, "rev-parse", "HEAD")
        meta_tree = run_git(cand, "rev-parse", "HEAD^{tree}")
        criterion = self.h.criterion(candidate_sha=cand_sha, candidate_tree=meta_tree)
        code, report = self.h.run(sha=cand_sha, candidate_dir=cand, criterion=criterion)
        self.assertEqual(code, 0, report)
        self.assertFalse(sentinel.exists(), "candidate code ran inside the verifier")
        self.assertEqual(report["candidate"]["sha"], cand_sha)

    def test_output_inside_checkout_is_refused_and_tracked_file_survives(self):
        tracked = self.h.repo / "docs/quality/budgets.json"
        before = tracked.read_bytes()
        code, report = self.h.run(output=tracked)
        self.assertNotEqual(code, 0)
        self.assertEqual(tracked.read_bytes(), before)
        self.assertIn("inside the checkout", self.h.stderr)
        code, report = self.h.run(output=self.h.repo / "new-report.json")
        self.assertNotEqual(code, 0)
        self.assertFalse((self.h.repo / "new-report.json").exists())

    def test_capture_refuses_output_inside_checkout(self):
        target = self.h.repo / "docs/quality/budgets.json"
        before = target.read_bytes()
        with contextlib.redirect_stderr(io.StringIO()), contextlib.redirect_stdout(io.StringIO()):
            code = capture_mod.run(["--candidate-sha", self.h.sha, "--features", FEATURES,
                                    "--output", str(target)], repo=self.h.repo, environ=self.h.environ)
        self.assertNotEqual(code, 0)
        self.assertEqual(target.read_bytes(), before)

    def test_checkout_changed_by_the_cargo_run_is_rejected(self):
        self.h.config["dirty_after"] = True
        self.h.save_config()
        code, report = self.h.run()
        self.assertFails(code, report, "not clean")

    def test_report_records_build_env_and_lock_hash(self):
        env = self.env_with(RUSTFLAGS="-C target-cpu=native", CARGO_TARGET_DIR="/tmp/t")
        code, report = self.h.run(environ=env)
        self.assertEqual(code, 0, report)
        build = report["build_env"]
        self.assertEqual(build["RUSTFLAGS"], "-C target-cpu=native")
        self.assertEqual(build["CARGO_TARGET_DIR"], "/tmp/t")
        self.assertIsNone(build["RUSTC_WRAPPER"])
        self.assertEqual(build["cargo_lock_sha256"], sha256_file(self.h.repo / "Cargo.lock"))


class CaptureTests(RunnerBase):
    def capture(self, *extra: str, features: str = FEATURES, sha: str = None):
        output = self.h.out / "captured.txt"
        err = io.StringIO()
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
            code = capture_mod.run(
                ["--candidate-sha", sha or self.h.sha, "--features", features,
                 "--output", str(output), *extra],
                repo=self.h.repo, environ=self.h.environ,
            )
        return code, output

    def test_captured_file_is_accepted_by_the_runner(self):
        code, output = self.capture("--sample-size", "20", "--warm-up-time", "1", "--measurement-time", "2")
        self.assertEqual(code, 0)
        bench = [c for c in self.h.calls() if c[0] == "bench"][0]
        self.assertEqual(bench, [
            "bench", "--locked", "--no-default-features", "--features", "openai,pdf", "--bench",
            "entity_extraction", "--", "--sample-size", "20", "--warm-up-time", "1",
            "--measurement-time", "2", "--noplot",
        ])
        meta, _ = crit.parse(output.read_text())
        self.assertEqual(meta["candidate_sha"], self.h.sha)
        self.assertEqual(meta["bench_params"]["sample_size"], 20)
        code, report = self.h.run(criterion=output)
        self.assertEqual(code, 0, report)

    def test_capture_refuses_wrong_candidate_and_unapproved_bench(self):
        code, output = self.capture(sha="5" * 40)
        self.assertNotEqual(code, 0)
        self.assertFalse(output.exists())
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            self.capture("--bench", "rm -rf /")

    def test_capture_refuses_out_of_bounds_parameters(self):
        code, output = self.capture("--sample-size", "5")
        self.assertNotEqual(code, 0)
        code, output = self.capture("--measurement-time", "100000")
        self.assertNotEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
