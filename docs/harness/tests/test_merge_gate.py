#!/usr/bin/env python3
"""Merge-policy evaluator tests (task H5): docs/harness/bin/merge-gate.py, agent-evidence.yml.

OFFLINE. Fixtures live in merge_gate_test_support.py: a real trusted-runner run (H4, stub docker,
fake writer) and operator-owned inputs outside the repository (receipt from the H1 producer
`review-gate.sh receipt-template`, review-v2, identity allowlist, check runs, workflow runs).
Only the complete, consistent set is `eligible`; every brief case is `refused`: wrong task / SHA /
policy, reviewer unavailable / forged / unauthorized, replayed approval of a previous candidate,
malformed or prose PASS, PASS with a blocking finding, missing log, later FAIL, stale base, head
swapped during evaluation, lineage swap, CI results a PR could have produced itself (changed CI
files, foreign suites, unbound workflow) and an integrated (merge-queue) tree without evidence.
"""

from __future__ import annotations

import datetime
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from merge_gate_test_support import (BIN, CI_YML, MG, OPERATOR, POLICY_VERSION,  # noqa: E402
                                     REPO_ROOT, REVIEWER, SUITE, WORKFLOW, Base, Operator,
                                     World, receipt_dict, run, run_git, sha256,
                                     workflow_contract_errors)


class Eligibility(Base):
    def test_complete_trusted_evidence_is_eligible_and_bound_to_the_exact_candidate(self):
        doc = self.evaluate(self.op.inputs())
        self.assertEligible(doc)
        w = self.world
        evaluator_sha = sha256((BIN / "merge-gate.py").read_bytes())
        self.assertEqual(doc["bound"], {"task": "h4-task", "base": w.base, "head": w.head,
                                        "tree": w.tree, "policy": POLICY_VERSION,
                                        "merge_policy": "merge-policy-v1",
                                        "ci_policy_sha256": MG.CI.policy_fingerprint(),
                                        "evaluator_sha256": evaluator_sha})
        self.assertTrue(all(v == "pass" for v in doc["sections"].values()), doc["sections"])
        self.assertIn(w.head, doc["recheck_before_merge"])
        self.assertIn("nothing was approved, merged", doc["authority"])

    def test_evaluation_writes_nothing_to_the_repository_or_the_runs_root(self):
        w = self.world
        inputs = self.op.inputs()

        def snapshot():
            refs = run_git(w.repo, "for-each-ref", "--format=%(refname) %(objectname)")
            status = run_git(w.repo, "status", "--porcelain", "--ignored")
            runs = sorted(str(p) for p in w.fx.runs_root.rglob("*"))
            return refs, status, runs

        before = snapshot()
        self.assertEligible(self.evaluate(inputs))
        self.assertEqual(snapshot(), before)

    def test_a_section_left_unevaluated_is_never_eligible(self):
        ev = MG.Evaluation()
        for name in MG.SECTIONS[:-1]:
            ev.passed(name)
        doc = ev.document()
        self.assertEqual(doc["decision"], "refused")
        self.assertEqual(doc["reasons"][0]["code"], "incomplete_evaluation")

    def test_cli_eligible_refused_and_usage_exit_codes(self):
        inputs = self.op.inputs()
        bare = ["--repo", str(inputs.repo), "--task", inputs.task, "--head-ref", inputs.head_ref,
                "--base-ref", inputs.base_ref, "--expect-policy-version", POLICY_VERSION]
        argv = bare + ["--runs-root", str(inputs.runs_root),
                       "--evidence-receipt", str(inputs.evidence_receipt),
                       "--review-receipt", str(inputs.review_receipt),
                       "--review-gate", str(inputs.review_gate),
                       "--expect-gate-sha256", inputs.expect_gate_sha256,
                       "--identities", str(inputs.identities),
                       "--ci-results", str(inputs.ci_results),
                       "--workflow-runs", str(inputs.workflow_runs),
                       "--registry", str(inputs.registry), "--catalog", str(inputs.catalog)]

        def cli(args):
            return subprocess.run([sys.executable, str(BIN / "merge-gate.py"), *args],
                                  capture_output=True, text=True, timeout=180)

        proc = cli(argv)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual(json.loads(proc.stdout)["decision"], "eligible")
        self.assertIn("MERGE_GATE: ELIGIBLE", proc.stderr)
        proc = cli(bare)  # no operator inputs at all (the CI situation): refused, not usage
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        codes = {r["code"] for r in json.loads(proc.stdout)["reasons"]}
        self.assertTrue({"evidence_unavailable", "review_receipt_unavailable",
                         "reviewer_unavailable", "ci_results_unavailable"} <= codes, codes)
        for bad in (["--task", "../x"], ["--expect-head", "abc"],
                    ["--expect-policy-version", "Bad Policy"]):
            with self.subTest(bad):
                proc = cli(bare + bad)
                self.assertEqual(proc.returncode, 2, proc.stdout + proc.stderr)
        self.assertEqual(cli(["--repo", str(inputs.repo)]).returncode, 2)


class WrongBindings(Base):
    def test_wrong_task_is_refused(self):
        doc = self.evaluate(self.op.inputs(task="other-task"))
        self.assertRefused(doc, "evidence_task_mismatch", "review_task_mismatch")

    def test_wrong_head_sha_is_refused(self):
        w = self.world
        other = w.commit({"README.md": b"other\n"})
        doc = self.evaluate(self.op.inputs(head_ref=w.head_ref(other)))
        self.assertRefused(doc, "evidence_refused", "review_head_mismatch")
        doc = self.evaluate(self.op.inputs(expect_head=other))
        self.assertRefused(doc, "head_mismatch")

    def test_wrong_policy_version_is_refused(self):
        doc = self.evaluate(self.op.inputs(policy_version="harness-hardening-v2"))
        self.assertRefused(doc, "policy_mismatch", "policy_unsupported", "review_policy_mismatch",
                           "evidence_policy_mismatch")
        self.assertIsNone(doc["bound"]["merge_policy"])
        doc = self.evaluate(self.op.inputs(receipt={"POLICY_VERSION": "harness-hardening-v0"}))
        self.assertRefused(doc, "review_policy_mismatch")

    def test_replayed_approval_of_a_previous_candidate_is_refused(self):
        w = self.world
        newer = w.commit({"src/new.txt": b"hello!\n"})
        doc = self.evaluate(self.op.inputs(head_ref=w.head_ref(newer)))  # receipt binds old head
        self.assertRefused(doc, "review_head_mismatch", "evidence_refused")

    def test_stale_base_is_refused(self):
        w = self.world
        advanced = w.commit({"README.md": b"readme v2\n"})
        doc = self.evaluate(self.op.inputs(base_ref=w.base_ref(advanced)))
        self.assertRefused(doc, "stale_base", "review_base_mismatch")
        doc = self.evaluate(self.op.inputs(expect_base=advanced))
        self.assertRefused(doc, "base_mismatch")

    def test_head_base_or_integrated_swapped_during_evaluation_is_refused(self):
        w = self.world
        for field, target, code in (("head_ref", w.base, "head_changed_during_evaluation"),
                                    ("base_ref", w.head, "base_changed_during_evaluation")):
            with self.subTest(field):
                inputs = self.op.inputs()
                ref = getattr(inputs, field)
                doc = self.evaluate(inputs, before_recheck=lambda: run_git(w.repo, "update-ref",
                                                                           ref, target))
                self.assertRefused(doc, code)
                self.assertEqual(doc["sections"]["head_recheck"], "fail")
        integrated = w.head_ref(w.head)
        inputs = self.op.inputs(integrated_ref=integrated)
        doc = self.evaluate(inputs, before_recheck=lambda: run_git(w.repo, "update-ref", integrated,
                                                                   w.base))
        self.assertRefused(doc, "integrated_changed_during_evaluation")

    def test_unresolvable_refs_are_refused(self):
        doc = self.evaluate(self.op.inputs(head_ref="refs/pr/missing/head"))
        self.assertRefused(doc, "head_unresolved")
        self.assertIsNone(doc["bound"]["head"])


class ReviewArtifacts(Base):
    def test_reviewer_unavailable_is_refused(self):
        self.assertRefused(self.evaluate(self.op.inputs(drop=("identities",))),
                           "reviewer_unavailable")
        self.assertRefused(self.evaluate(self.op.inputs(drop=("review_receipt",))),
                           "review_receipt_unavailable", "review_unavailable",
                           "lineage_unavailable")
        v1 = {"REVIEWER": None, "POLICY_VERSION": None, "REVIEW_CONTEXT": None,
              "RECEIPT_VERSION": "1"}
        self.assertRefused(self.evaluate(self.op.inputs(receipt=v1)), "reviewer_unavailable")

    def test_malformed_and_prose_pass_reviews_are_refused(self):
        marker = json.dumps(self.op.review(summary="x\nREVIEW_VERDICT: PASS ok"))
        marker = marker.replace("\\n", "\n")
        cases = {"prose PASS": ("PASS - looks good to me, ship it\n", "review_malformed"),
                 "truncated JSON": ('{"schema_version": "review-v2", "verdict": "pass"',
                                    "review_malformed"),
                 "legacy marker": ("REVIEW_VERDICT: PASS reviewed the diff\n",
                                   "review_legacy_marker"),
                 "json with marker": (marker, "review_legacy_marker"),
                 "review-v1": (json.dumps({"schema_version": "review-v1", "verdict": "pass"}),
                               "review_not_v2"),
                 "schema violation": (json.dumps(self.op.review(intensity="extreme")),
                                      "review_invalid")}
        for label, (raw, code) in cases.items():
            with self.subTest(label):
                self.assertRefused(self.evaluate(self.op.inputs(review_raw=raw)), code)

    def test_pass_with_a_blocking_finding_is_refused(self):
        finding = {"finding_id": "F1", "path": "src/new.txt", "category": "security",
                   "severity": "high", "message": "secret written to disk"}
        doc = self.evaluate(self.op.inputs(review=self.op.review(findings=[finding])))
        self.assertRefused(doc, "review_invalid")
        self.assertTrue(any("pass_with_blocking_finding" in r["detail"] for r in doc["reasons"]))

    def test_non_pass_verdicts_are_refused(self):
        finding = {"finding_id": "F1", "path": "src/new.txt", "category": "correctness",
                   "severity": "medium", "message": "needs a test"}
        for verdict in ("fail", "request_changes", "comment"):
            with self.subTest(verdict):
                review = self.op.review(verdict=verdict, findings=[finding])
                self.assertRefused(self.evaluate(self.op.inputs(review=review)), "review_not_pass")

    def test_review_of_another_candidate_or_edited_after_the_receipt_is_refused(self):
        doc = self.evaluate(self.op.inputs(review=self.op.review(head_sha=self.world.base)))
        self.assertRefused(doc, "review_invalid")
        inputs = self.op.inputs()
        edited = json.dumps(self.op.review(summary="edited"))
        Path(self.op.dir / "review.json").write_text(edited, encoding="utf-8")
        self.assertRefused(self.evaluate(inputs), "review_hash_mismatch")
        os.unlink(self.op.dir / "review.json")
        self.assertRefused(self.evaluate(inputs), "review_missing")

    def test_receipt_scope_and_context_mismatches_are_refused(self):
        cases = {"diff": ({"DIFF_SHA256": "0" * 64}, "review_diff_mismatch"),
                 "tree": ({"TREE_SHA": "1" * 40}, "review_tree_mismatch"),
                 "context": ({"REVIEW_CONTEXT": "writer-session"}, "review_context_unattested"),
                 "gate": ({"GATE_SCRIPT_SHA256": "2" * 64}, "review_gate_mismatch")}
        for label, (over, code) in cases.items():
            with self.subTest(label):
                self.assertRefused(self.evaluate(self.op.inputs(receipt=over)), code)
        self.assertRefused(self.evaluate(self.op.inputs(expect_gate_sha256="3" * 64)),
                           "review_gate_mismatch")
        self.assertRefused(self.evaluate(self.op.inputs(drop=("review_gate",))),
                           "review_scope_unavailable")

    def test_malformed_receipts_are_refused(self):
        values = self.op.receipt_values(self.op.write("review.json", "{}"))
        good = "".join(f"{k}={v}\n" for k, v in values.items())
        cases = {"unknown key": good + "APPROVED=yes\n",
                 "duplicate key": good + f"REVIEWER={REVIEWER}\n",
                 "missing key": good.replace("REVIEW_CONTEXT=fresh-readonly\n", ""),
                 "not key=value": good + "PASS\n",
                 "bad version": good.replace("RECEIPT_VERSION=2", "RECEIPT_VERSION=3"),
                 "bad sha": good.replace(f"HEAD_SHA={self.world.head}", "HEAD_SHA=HEAD"),
                 "unfilled template": self.world.template}
        for label, raw in cases.items():
            with self.subTest(label):
                self.assertRefused(self.evaluate(self.op.inputs(receipt_raw=raw)),
                                   "review_receipt_malformed")

    def fill(self, template: str) -> str:
        """The operator's part: identities, policy and the fresh-context attestation."""
        values = {"OPERATOR": OPERATOR, "REVIEWER": REVIEWER, "POLICY_VERSION": POLICY_VERSION,
                  "REVIEW_CONTEXT": "fresh-readonly"}
        lines = []
        for line in template.splitlines():
            key = line.split("=", 1)[0]
            lines.append(f"{key}={values[key]}" if key in values else line)
        return "\n".join(lines) + "\n"

    def test_template_parses_in_both_consumers_but_each_needs_its_own_review(self):
        """One producer, two consumers: H1 post judges a marker review, H5 a review-v2 JSON.

        The same receipt format is parsed by both, but a receipt binds ONE review artifact, so the
        H1 flow and the H5 flow need separate review artifacts and separate receipts."""
        w = self.world
        self.assertIn("REVIEW_CONTEXT=<", w.template)  # the attestation is never pre-filled
        marker = self.op.write("review.md", "REVIEW_VERDICT: PASS fresh-context review\n")
        h1_text = self.fill(w.receipt_template(review_file=marker))
        self.assertNotIn("=<", h1_text)
        h1_receipt = self.op.write("h1.env", h1_text)
        post = subprocess.run(["bash", str(w.gate), "post", "h4-task", "--repo", str(w.repo),
                               "--base", w.base, "--head", w.head, "--review-file", str(marker),
                               "--receipt", str(h1_receipt), "--expect-tree", w.tree,
                               "--expect-diff-sha256", receipt_dict(h1_text)["DIFF_SHA256"],
                               "--expect-gate-sha256", w.gate_sha, "--operator", OPERATOR],
                              capture_output=True, text=True, timeout=120)
        self.assertEqual(post.returncode, 0, post.stdout + post.stderr)
        self.assertIn("GATE_STATUS: PASS", post.stdout)
        doc = self.evaluate(replace(self.op.inputs(), review_receipt=h1_receipt))
        self.assertEqual(doc["sections"]["review_receipt"], "pass", doc["reasons"])
        self.assertEqual(doc["sections"]["review_scope"], "pass", doc["reasons"])
        self.assertRefused(doc, "review_legacy_marker")  # the H1 review never satisfies H5
        review_v2 = self.op.write("review-v2.json", json.dumps(self.op.review()))
        h5_receipt = self.op.write("h5.env", self.fill(w.receipt_template(review_file=review_v2)))
        self.assertEligible(self.evaluate(replace(self.op.inputs(), review_receipt=h5_receipt)))


class TrustedLocations(Base):
    def test_operator_inputs_inside_the_repository_are_refused(self):
        w = self.world
        inputs = self.op.inputs()
        inside = w.repo / "receipt.env"
        shutil.copy(inputs.review_receipt, inside)
        self.addCleanup(inside.unlink)
        self.assertRefused(self.evaluate(replace(inputs, review_receipt=inside)),
                           "review_receipt_untrusted")
        gate_inside = w.repo / "review-gate.sh"
        shutil.copy(w.gate, gate_inside)
        self.addCleanup(gate_inside.unlink)
        self.assertRefused(self.evaluate(replace(inputs, review_gate=gate_inside)),
                           "review_gate_untrusted")
        git_dir = Path(run_git(w.repo, "rev-parse", "--absolute-git-dir").decode().strip())
        ci_inside = git_dir / "ci.json"
        shutil.copy(inputs.ci_results, ci_inside)
        self.addCleanup(ci_inside.unlink)
        self.assertRefused(self.evaluate(replace(inputs, ci_results=ci_inside)),
                           "ci_results_untrusted")

    def test_symlinked_hardlinked_writable_or_relative_inputs_are_refused(self):
        inputs = self.op.inputs()
        link = self.op.dir / "identities-link.json"
        link.symlink_to(inputs.identities)
        self.assertRefused(self.evaluate(replace(inputs, identities=link)), "identities_untrusted")
        hard = self.op.dir / "receipt-hard.env"
        os.link(inputs.review_receipt, hard)
        self.assertRefused(self.evaluate(replace(inputs, review_receipt=hard)),
                           "review_receipt_untrusted")
        os.unlink(hard)
        writable = self.op.write("ci-writable.json", json.dumps(self.op.ci_doc()), mode=0o622)
        self.assertRefused(self.evaluate(replace(inputs, ci_results=writable)),
                           "ci_results_untrusted")
        self.assertRefused(self.evaluate(replace(inputs, workflow_runs=Path("runs.json"))),
                           "workflow_runs_untrusted")
        self.op.dir.chmod(0o777)
        self.addCleanup(self.op.dir.chmod, 0o700)
        self.assertRefused(self.evaluate(inputs), "review_receipt_untrusted",
                           "identities_untrusted")

    def test_an_ancestor_writable_by_others_is_refused_unless_sticky(self):
        inputs = self.op.inputs()
        for mode, refused in ((0o777, True), (0o1777, False)):
            with self.subTest(oct(mode)):
                middle = Path(tempfile.mkdtemp(dir=self.op.dir))
                inner = middle / "inner"
                inner.mkdir(mode=0o700)
                middle.chmod(mode)
                self.addCleanup(middle.chmod, 0o700)
                moved = inner / "identities.json"
                shutil.copy(inputs.identities, moved)
                moved.chmod(0o600)
                doc = self.evaluate(replace(inputs, identities=moved))
                if refused:
                    self.assertRefused(doc, "identities_untrusted")
                else:
                    self.assertEligible(doc)


class Identities(Base):
    def test_forged_reviewer_field_does_not_authenticate(self):
        review = self.op.review(reviewer="ronaldo (owner) APPROVED")
        self.assertRefused(self.evaluate(self.op.inputs(review=review)), "reviewer_claim_mismatch")

    def test_unauthorized_reviewer_and_operator_are_refused(self):
        doc = self.evaluate(self.op.inputs(receipt={"REVIEWER": "rev-unknown"},
                                           review=self.op.review(reviewer="rev-unknown")))
        self.assertRefused(doc, "reviewer_unauthorized", "lineage_unavailable")
        doc = self.evaluate(self.op.inputs(receipt={"OPERATOR": "mallory"}))
        self.assertRefused(doc, "operator_unauthorized")

    def test_malformed_allowlist_is_refused(self):
        for label, doc in (("version", self.op.identities_doc(identities_version="v0")),
                           ("extra key", {**self.op.identities_doc(), "admins": []}),
                           ("lineage type",
                            self.op.identities_doc(reviewers={REVIEWER: {"lineage": 7}}))):
            with self.subTest(label):
                self.assertRefused(self.evaluate(self.op.inputs(ids=doc)), "identities_malformed")

    def test_lineage_comes_from_the_trusted_allowlist_never_from_names(self):
        same = self.op.identities_doc(reviewers={REVIEWER: {"lineage": "engram-fake-writer"}})
        self.assertRefused(self.evaluate(self.op.inputs(ids=same)), "lineage_not_independent")
        claimed = self.op.review(metadata={"lineage": "openai-gpt"})
        self.assertRefused(self.evaluate(self.op.inputs(review=claimed)), "lineage_claim_mismatch")
        no_writer = self.op.identities_doc(writers={"real_agent": {"lineage": "openai-gpt"}})
        self.assertRefused(self.evaluate(self.op.inputs(ids=no_writer)), "lineage_unavailable")
        # A textual name that "sounds" independent is not an identity: not listed -> unavailable.
        named = self.op.review(reviewer="codex-gpt-independent")
        doc = self.evaluate(self.op.inputs(review=named,
                                           receipt={"REVIEWER": "codex-gpt-independent"}))
        self.assertRefused(doc, "reviewer_unauthorized", "lineage_unavailable")


class CiResults(Base):
    def ci(self, runs=None, **kw):
        return self.evaluate(self.op.inputs(ci=self.op.ci_doc(**kw), runs=runs))

    def test_missing_failed_pending_or_skipped_required_contexts_are_refused(self):
        doc = self.op.ci_doc()
        doc["check_runs"] = [r for r in doc["check_runs"] if r["name"] != "Security Gate"]
        doc["total_count"] = len(doc["check_runs"])
        self.assertRefused(self.evaluate(self.op.inputs(ci=doc)), "ci_blocked")
        for name, over in (("Format", {"conclusion": "failure"}),
                           ("Clippy", {"conclusion": "skipped"}),
                           ("Documentation", {"status": "in_progress", "conclusion": None,
                                              "completed_at": None}),
                           ("Cargo Deny", {"conclusion": "neutral"})):
            with self.subTest(name):
                self.assertRefused(self.ci(overrides={name: over}), "ci_blocked")

    def test_reruns_count_only_inside_the_single_ci_suite(self):
        head = self.world.head
        later_fail = run(800, "Format", "failure", "2026-10-05T11:00:00Z", head)
        self.assertRefused(self.ci(extra=[later_fail]), "ci_blocked")  # later FAIL wins
        rerun = run(801, "Format", "success", "2026-10-05T12:00:00Z", head)
        self.assertEligible(self.ci(extra=[later_fail, rerun]))  # true re-run, same suite
        queued = run(802, "Format", None, None, head, status="queued")
        self.assertRefused(self.ci(extra=[later_fail, rerun, queued]), "ci_blocked")

    def test_a_same_named_job_in_another_suite_cannot_launder_a_failure(self):
        head = self.world.head
        failed = {"Format": {"conclusion": "failure"}}
        other = run(850, "Format", "success", "2026-10-05T13:00:00Z", head, suite=SUITE + 1)
        runs = self.op.runs_doc({SUITE: CI_YML, SUITE + 1: CI_YML})
        doc = self.ci(overrides=failed, extra=[other], runs=runs)
        self.assertRefused(doc, "ci_context_multiple_suites")
        # even two successful suites are ambiguous: exactly one suite per context
        ok_other = run(851, "Clippy", "success", "2026-10-05T13:00:00Z", head, suite=SUITE + 1)
        self.assertRefused(self.ci(extra=[ok_other], runs=runs), "ci_context_multiple_suites")

    def test_required_contexts_must_come_from_a_ci_yml_workflow_run(self):
        for label, suites in (("other workflow", {SUITE: ".github/workflows/evil.yml"}),
                              ("suite missing", {SUITE + 7: CI_YML}),
                              ("ambiguous suite", None)):
            with self.subTest(label):
                if suites is None:
                    runs = self.op.runs_doc({SUITE: CI_YML})
                    runs["workflow_runs"].append({**runs["workflow_runs"][0], "id": 999,
                                                  "path": ".github/workflows/agent-evidence.yml"})
                    runs["total_count"] = 2
                else:
                    runs = self.op.runs_doc(suites)
                self.assertRefused(self.ci(runs=runs), "ci_workflow_unbound")

    def test_other_apps_cannot_satisfy_a_required_context(self):
        forged = run(950, "Test (ubuntu-latest)", "success", "2026-10-05T13:00:00Z",
                     self.world.head, slug="writer-bot")
        doc = self.ci(overrides={"Test (ubuntu-latest)": {"conclusion": "failure"}}, extra=[forged])
        self.assertRefused(doc, "ci_blocked")

    def test_results_for_another_head_incomplete_or_malformed_are_refused(self):
        self.assertRefused(self.ci(head=self.world.base), "ci_results_head_mismatch")
        doc = self.op.ci_doc()
        doc["total_count"] += 3
        self.assertRefused(self.evaluate(self.op.inputs(ci=doc)), "ci_results_incomplete")
        self.assertRefused(self.evaluate(self.op.inputs(ci={"statuses": []})),
                           "ci_results_malformed")
        for over in ({"app": "github-actions"}, {"check_suite": None}):
            with self.subTest(over):
                broken = self.op.ci_doc(overrides={"Format": over})
                self.assertRefused(self.evaluate(self.op.inputs(ci=broken)), "ci_results_malformed")
        self.assertRefused(self.evaluate(self.op.inputs(drop=("ci_results",))),
                           "ci_results_unavailable")

    def test_workflow_runs_must_be_present_complete_and_for_this_head(self):
        self.assertRefused(self.evaluate(self.op.inputs(drop=("workflow_runs",))),
                           "workflow_runs_unavailable")
        runs = self.op.runs_doc()
        runs["total_count"] = 5
        self.assertRefused(self.evaluate(self.op.inputs(runs=runs)), "workflow_runs_incomplete")
        foreign = self.op.runs_doc(head=self.world.base)
        self.assertRefused(self.evaluate(self.op.inputs(runs=foreign)),
                           "workflow_runs_head_mismatch")
        broken = self.op.runs_doc()
        del broken["workflow_runs"][0]["check_suite_id"]
        self.assertRefused(self.evaluate(self.op.inputs(runs=broken)), "workflow_runs_malformed")

    def test_paginated_slurp_output_is_accepted(self):
        doc = self.op.ci_doc()
        half = len(doc["check_runs"]) // 2
        pages = [{"total_count": doc["total_count"], "check_runs": doc["check_runs"][:half]},
                 {"total_count": doc["total_count"], "check_runs": doc["check_runs"][half:]}]
        self.assertEligible(self.evaluate(self.op.inputs(ci=pages, runs=[self.op.runs_doc()])))


class CiPolicyPaths(Base):
    """A PR that touches what produces or judges its own CI results gets no CI trust at all."""

    def test_changes_to_ci_definitions_or_gate_scripts_are_refused(self):
        w = self.world
        cases = {"ci workflow": CI_YML, "new workflow": ".github/workflows/format.yml",
                 "composite action": ".github/actions/setup/action.yml",
                 "security gate": "scripts/check-security-gate.py",
                 "findings policy": "scripts/check-security-findings.py",
                 "supply chain": "scripts/check-workflow-supply-chain.py",
                 "quality budgets": "scripts/check-quality-budgets.py",
                 "offline lane": "docs/harness/bin/run-offline-lane.sh",
                 "lane component": "docs/harness/tests/test_runner.py",
                 "review gate": "docs/harness/bin/review-gate.sh",
                 "merge gate": "docs/harness/bin/merge-gate.py",
                 "security matrix": "tests/fixtures/security_gate_matrix.json"}
        for label, path in cases.items():
            with self.subTest(label):
                head = w.commit({path: b"changed\n", "src/new.txt": b"hello\n"})
                doc = self.evaluate(self.op.inputs(head_ref=w.head_ref(head)))
                got = self.assertRefused(doc, "ci_policy_paths_changed")
                if path == CI_YML:
                    self.assertIn("ci_workflow_changed", got)
                detail = next(r["detail"] for r in doc["reasons"]
                              if r["code"] == "ci_policy_paths_changed")
                self.assertIn(path, detail)

    def test_tool_configs_and_indirect_gate_inputs_are_protected(self):
        w = self.world
        for path in ("scripts/generate-mcp-reference.sh", "scripts/generate_mcp_reference.py",
                     "deny.toml", ".cargo/audit.toml", ".github/codeql/codeql-config.yml",
                     ".gitleaks.toml", ".semgrepignore", "docs/security/advisory-exceptions.toml",
                     "docs/security/finding-exceptions.toml", "scripts/ci-required-features.env",
                     "scripts/ci-features.env", "scripts/ci.sh", "rust-toolchain.toml",
                     "docs/quality/budgets.json", "tests/fixtures/retrieval_quality/baseline.json",
                     "benches/results/benchmark_results.txt", ".github/CODEOWNERS"):
            with self.subTest(path):
                head = w.commit({path: b"changed\n"})
                self.assertEqual(MG.CI.protected_changes(MG.HG, w.repo, w.base, head), [path])

    def test_every_file_a_required_job_references_is_protected(self):
        """Drift guard: derive the required-job closure of the real ci.yml (names -> needs) and
        every repository file its steps reference; each must be covered by the static policy."""
        text = (REPO_ROOT / CI_YML).read_text(encoding="utf-8")
        jobs = MG.CI.required_jobs(text)
        self.assertTrue({"fmt", "clippy", "test", "docs", "audit", "deny", "security-gate",
                         "codeql-security", "semgrep-security", "gitleaks-security"} <= jobs, jobs)
        refs = MG.CI.required_job_references(text, lambda p: (REPO_ROOT / p).is_file())
        self.assertIn("scripts/generate-mcp-reference.sh", refs)
        self.assertIn(".github/codeql/codeql-config.yml", refs)
        self.assertEqual(sorted(p for p in refs if not MG.CI.is_protected(p)), [])

    def test_files_referenced_by_the_base_ci_yml_are_protected_dynamically(self):
        w = self.world
        ci_text = ("jobs:\n  fmt:\n    name: Format\n    runs-on: ubuntu-latest\n    steps:\n"
                   "      - run: python3 tools/fmt_gate.py --config tools/fmt.json\n")
        base = w.commit({CI_YML: ci_text.encode(), "tools/fmt_gate.py": b"gate\n",
                         "tools/fmt.json": b"{}\n"})
        head = w.commit({CI_YML: ci_text.encode(), "tools/fmt_gate.py": b"gate\n",
                         "tools/fmt.json": b'{"skip": true}\n'}, parent=base)
        self.assertFalse(MG.CI.is_protected("tools/fmt.json"))
        self.assertEqual(MG.CI.protected_changes(MG.HG, w.repo, base, head), ["tools/fmt.json"])

    def test_ordinary_source_changes_are_not_ci_policy_changes(self):
        self.assertEqual(MG.CI.protected_changes(MG.HG, self.world.repo, self.world.base,
                                                 self.world.head), [])
        for path in ("src/lib.rs", "benches/search.rs", "docs/guide.md", "tests/a_test.rs"):
            self.assertFalse(MG.CI.is_protected(path), path)

    def test_ci_policy_constants_are_pinned_to_the_policy_version(self):
        pinned = {"harness-hardening-v1": {"id": "merge-policy-v1", "ci_policy_sha256": "0" * 64}}
        with mock.patch.dict(MG.MERGE_POLICIES, pinned):
            self.assertRefused(self.evaluate(self.op.inputs()), "policy_mismatch")
        self.assertEqual(MG.MERGE_POLICIES[POLICY_VERSION]["ci_policy_sha256"],
                         MG.CI.policy_fingerprint())


class IntegratedTree(Base):
    def test_merge_queue_commit_needs_its_own_evidence_unless_its_tree_is_the_head_tree(self):
        w = self.world
        env = {"GIT_AUTHOR_NAME": "q", "GIT_AUTHOR_EMAIL": "q@example.invalid",
               "GIT_COMMITTER_NAME": "q", "GIT_COMMITTER_EMAIL": "q@example.invalid"}
        same_tree = run_git(w.repo, "commit-tree", w.tree, "-p", w.base, "-p", w.head, "-m",
                            "merge group", env_extra=env).decode().strip()
        doc = self.evaluate(self.op.inputs(integrated_ref=same_tree))
        self.assertEligible(doc)
        self.assertEqual(doc["bound"]["integrated"], same_tree)
        other = w.commit({"src/new.txt": b"hello\n", "README.md": b"queued sibling\n"},
                         parent=w.head)
        self.assertRefused(self.evaluate(self.op.inputs(integrated_ref=other)),
                           "integrated_tree_unverified")
        self.assertRefused(self.evaluate(self.op.inputs(integrated_ref="refs/none")),
                           "integrated_unresolved")


class TamperedEvidence(unittest.TestCase):
    """Attacks on the trusted-runner bundle itself use a fresh world per test (destructive)."""

    def setUp(self):
        self.world = World(self)
        self.op = Operator(self, self.world)

    def codes(self, doc):
        self.assertEqual(doc["decision"], "refused")
        return {r["code"] for r in doc["reasons"]}

    def test_missing_log_is_refused(self):
        log = next((self.world.evidence_receipt.parent / "run" / "logs").glob("*.stdout.log"))
        log.chmod(0o600)
        log.unlink()
        self.assertIn("evidence_refused", self.codes(MG.evaluate(self.op.inputs())))

    def test_a_later_failing_gate_for_the_same_candidate_is_refused(self):
        self.assertEqual(MG.evaluate(self.op.inputs())["decision"], "eligible")
        gate = self.world.fx.runs_root / "h4-task" / "20991231T000000Z-deadbeef" / "a1" / "g1"
        gate.mkdir(parents=True, mode=0o700)
        receipt = {"candidate_sha": self.world.head, "verdict": "fail"}
        (gate / "receipt.json").write_text(json.dumps(receipt))
        self.assertIn("conflicting_gate_result", self.codes(MG.evaluate(self.op.inputs())))

    def test_missing_evidence_is_refused(self):
        codes = self.codes(MG.evaluate(self.op.inputs(drop=("evidence_receipt",))))
        expected = {"evidence_unavailable", "gate_history_unavailable", "lineage_unavailable"}
        self.assertTrue(expected <= codes, codes)


# --- agent-evidence.yml: read-only, base-revision evaluator, untrusted head ---------------------

class WorkflowContract(unittest.TestCase):
    def setUp(self):
        self.text = WORKFLOW.read_text(encoding="utf-8")

    def test_shipped_workflow_meets_the_read_only_contract(self):
        self.assertEqual(workflow_contract_errors(self.text), [])
        self.assertIn("ref: ${{ github.event.pull_request.base.sha }}\n          path: trusted",
                      self.text)
        self.assertIn("ref: ${{ github.event.pull_request.head.sha }}\n          path: candidate",
                      self.text)

    def test_untrusted_variants_are_flagged(self):
        t = self.text
        variants = {
            "pull_request_target": t.replace("  pull_request:\n", "  pull_request_target:\n", 1),
            "write permission": t.replace("  contents: read\n",
                                          "  contents: read\n  pull-requests: write\n", 1),
            "secret": t.replace("HEAD_SHA: ${{",
                                "TOKEN: ${{ secrets.MERGE_TOKEN }}\n          HEAD_SHA: ${{", 1),
            "candidate code": t.replace("python3 trusted/docs/harness/bin/merge-gate.py",
                                        "python3 candidate/docs/harness/bin/merge-gate.py", 1),
            "unpinned": t.replace("@93cb6efe18208431cddfb8368fd83d5badbf9bfd", "@v5", 1),
            "persisted credentials": t.replace("persist-credentials: false",
                                               "persist-credentials: true", 1),
            "approve": t.replace("python3 - \"$rc\"",
                                 "gh pr review --approve; python3 - \"$rc\"", 1),
            "inline expression": t.replace('--task "pr-${PR_NUMBER}"',
                                           '--task "pr-${{ github.event.number }}"', 1),
        }
        for label, text in variants.items():
            with self.subTest(label):
                self.assertNotEqual(text, t)
                self.assertNotEqual(workflow_contract_errors(text), [])

    def test_q5_supply_chain_checker_covers_the_workflow(self):
        path = REPO_ROOT / "scripts" / "check-workflow-supply-chain.py"
        spec = importlib.util.spec_from_file_location("check_workflow_supply_chain_h5", path)
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module  # dataclasses resolve their module through sys.modules
        self.addCleanup(sys.modules.pop, spec.name, None)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as tmp:
            shutil.copy(WORKFLOW, Path(tmp) / WORKFLOW.name)
            ledger = Path(tmp) / "ledger.toml"
            ledger.write_text('schema_version = 1\nas_of = 2026-10-05\nowner = "t"\n',
                              encoding="utf-8")
            today = datetime.date(2026, 10, 5)
            self.assertEqual(module.run_checks(Path(tmp), ledger, None, today), [])
            bad = WORKFLOW.read_text(encoding="utf-8").replace(
                "@93cb6efe18208431cddfb8368fd83d5badbf9bfd", "@v5", 1)
            (Path(tmp) / WORKFLOW.name).write_text(bad, encoding="utf-8")
            self.assertNotEqual(module.run_checks(Path(tmp), ledger, None, today), [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
