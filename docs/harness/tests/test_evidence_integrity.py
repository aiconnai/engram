#!/usr/bin/env python3
"""Evidence integrity tests (task H4): docs/harness/bin/record-evidence.py. OFFLINE (stub docker).

A passing run is produced by the runner, then attacked after the gate. Every attack must be refused:
one byte changed in a log, one byte changed in the candidate (new commit / moved ref), a deleted log,
a tampered hash (with and without re-sealing the receipt), a writer-forged PASS bundle (inside the
writer workspace or a gate checkout, or with the runs root pointed at the workspace), policy drift,
symlinked / group-writable locations, and a recorder fed log bytes that differ from what was streamed.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from runner_test_support import BIN, RunnerFixture, run_git  # noqa: E402


class PassedRun(unittest.TestCase):
    def setUp(self):
        self.fx = RunnerFixture(self)
        self.summary = self.fx.run([["write_file", "src/new.txt", "hello\n"]])
        self.assertEqual(self.summary["status"], "passed", self.summary)
        self.receipt = Path(self.summary["result"]["receipt"])
        self.gate_root = self.receipt.parent
        self.candidate = self.summary["result"]["candidate"]["sha"]
        self.ref = self.summary["result"]["candidate"]["ref"]
        self.rec = self.fx.rec

    def refused(self, code=None, receipt=None, candidate=None, **kw):
        with self.assertRaises(self.rec.VerifyRefused) as ctx:
            self.fx.verify(receipt or self.receipt, candidate or self.candidate, **kw)
        if code is not None:
            self.assertEqual(ctx.exception.code, code, ctx.exception)
        return ctx.exception

    def log(self):
        return next((self.gate_root / "run" / "logs").glob("*.stdout.log"))

    def make_writable(self, path):
        path.chmod(0o600)


class IntegrityAttacks(PassedRun):
    def test_untouched_evidence_verifies_and_cli_reports_verified(self):
        self.assertEqual(self.fx.verify(self.receipt, self.candidate)["status"], "verified")
        proc = subprocess.run([sys.executable, str(BIN / "record-evidence.py"), "verify", "--repo", str(self.fx.repo.path),
                               "--runs-root", str(self.fx.runs_root), "--receipt", str(self.receipt),
                               "--expect-candidate", self.candidate, "--registry", str(self.fx.registry),
                               "--catalog", str(self.fx.catalog)], capture_output=True, text=True, timeout=60)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual(proc.stdout.strip().splitlines()[-1], f"EVIDENCE: VERIFIED candidate={self.candidate}")

    def test_one_byte_changed_in_a_log_after_the_gate_is_refused(self):
        log = self.log()
        data = bytearray(log.read_bytes())
        data[0] ^= 0x01
        log.write_bytes(bytes(data))
        exc = self.refused("evidence_invalid")
        self.assertIn("log_hash_mismatch", exc.detail)

    def test_one_byte_changed_in_the_candidate_is_a_new_sha_and_is_refused(self):
        files = run_git(self.fx.repo.path, "show", self.candidate + ":src/new.txt")
        tree_files = {"src/a.txt": b"alpha\n", "src/new.txt": files[:-1] + b"!", "README.md": b"readme\n",
                      "docs/harness/bin/tool.sh": ("exec", b"#!/bin/sh\necho tool\n"),
                      "tests/api_test.py": b"def test_a():\n    assert 1 == 1\n"}
        changed = self.fx.repo.commit(tree_files, parent=self.fx.base)
        self.assertNotEqual(changed, self.candidate)
        self.refused("candidate_mismatch", candidate=changed)
        run_git(self.fx.repo.path, "update-ref", self.ref, changed)  # move the runner ref onto the new bytes
        self.refused("candidate_ref_mismatch")

    def test_deleted_log_is_refused(self):
        self.log().unlink()
        exc = self.refused("evidence_invalid")
        self.assertIn("log_file_missing", exc.detail)
        stderr_log = next((self.gate_root / "run" / "logs").glob("*.stderr.log"))
        stderr_log.unlink()
        self.refused()

    def test_tampered_hash_is_refused_even_after_resealing_the_receipt(self):
        evidence_path = self.gate_root / "evidence.json"
        self.make_writable(evidence_path)
        evidence = json.loads(evidence_path.read_text())
        evidence["checks"][0]["log_hash"] = "0" * 64
        evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
        self.refused("evidence_hash_mismatch")
        receipt = json.loads(self.receipt.read_text())
        receipt["evidence_sha256"] = hashlib.sha256(evidence_path.read_bytes()).hexdigest()
        self.make_writable(self.receipt)
        self.receipt.write_text(json.dumps(receipt))
        exc = self.refused("evidence_invalid")
        self.assertIn("log_hash_mismatch", exc.detail)

    def test_tampered_manifest_entry_is_refused(self):
        record = self.gate_root / "run-record.json"
        self.make_writable(record)
        record.write_text(record.read_text().replace('"attempt": 1', '"attempt": 9'))
        self.refused("manifest_mismatch")

    def test_manifest_keys_that_leave_the_gate_root_are_refused_locally(self):
        outside = self.fx.repo.root / "outside.txt"
        outside.write_text("x\n")
        digest = hashlib.sha256(b"x\n").hexdigest()
        rel_up = os.path.relpath(outside, self.gate_root)
        self.assertTrue(rel_up.startswith("../") and (self.gate_root / rel_up).is_file())
        for key in (rel_up, str(outside), "run//logs", "run/./x", "run\\x"):
            with self.subTest(key), self.assertRaises(self.rec.VerifyRefused) as ctx:
                self.rec._verify_manifest(self.gate_root, {"sha256_manifest": {key: digest}})
            self.assertEqual(ctx.exception.code, "manifest_mismatch")

    def test_policy_drift_after_the_gate_is_refused(self):
        drifted = self.fx.repo.root / "registry-drifted.json"
        data = json.loads(self.fx.registry.read_text())
        data["checks"]["pr_title_policy"]["timeout_seconds"] = 31
        drifted.write_text(json.dumps(data, indent=2))
        self.refused("policy_drift", registry_path=drifted)

    def test_wrong_or_malformed_expected_candidate_is_refused(self):
        self.refused("candidate_mismatch", candidate="e" * 40)
        self.refused("usage", candidate="HEAD")


class ForgedEvidence(PassedRun):
    def forge_bundle(self, root):
        """Copy the real bundle (consistent hashes, real candidate) to a writer-reachable place."""
        target = root / "h4-task" / self.receipt.parent.parent.parent.name / "a1" / "g0"
        shutil.copytree(self.gate_root, target)
        return target / "receipt.json"

    def test_writer_claimed_pass_in_its_workspace_is_refused(self):
        ws = self.gate_root.parent / "ws"
        (ws / "receipt.json").write_text(json.dumps({"verdict": "pass"}))
        self.refused("untrusted_location", receipt=ws / "receipt.json")
        forged = self.forge_bundle(ws)
        self.refused("untrusted_location", receipt=forged)

    def test_runs_root_pointed_at_a_writer_workspace_is_refused(self):
        ws = self.gate_root.parent / "ws"
        forged = self.forge_bundle(ws)
        (ws / ".engram-runs-root").write_text("forged marker\n")
        self.refused("nested_runs_root", receipt=forged, runs_root=ws)

    def test_bundle_in_the_gate_checkout_is_refused(self):
        checkout = self.gate_root.parent / "c0"
        forged = self.forge_bundle(checkout)
        self.refused("untrusted_location", receipt=forged)

    def test_symlinked_receipt_and_group_writable_directory_are_refused(self):
        link = self.gate_root.parent / "g9"
        link.symlink_to(self.gate_root)
        self.refused("untrusted_location", receipt=link / "receipt.json")
        alias = self.gate_root.parent / "receipt-alias.json"
        alias.symlink_to(self.receipt)
        self.refused("untrusted_location", receipt=alias)
        os.chmod(self.gate_root, 0o770)
        self.addCleanup(os.chmod, self.gate_root, 0o700)
        self.refused("untrusted_location")

    def test_runs_root_without_the_runner_marker_is_refused(self):
        (self.fx.runs_root / ".engram-runs-root").unlink()
        self.refused("untrusted_location")


class RecorderInputs(PassedRun):
    def test_recorder_refuses_log_bytes_that_differ_from_what_the_adapter_streamed(self):
        copy = self.fx.repo.root / "copy-gate"
        copy.mkdir(mode=0o700)
        for name in ("inputs", "run"):
            shutil.copytree(self.gate_root / name, copy / name)
        shutil.copy(self.gate_root / "scope.json", copy / "scope.json")
        log = next((copy / "run" / "logs").glob("*.stdout.log"))
        log.write_bytes(log.read_bytes() + b"appended after the run\n")
        ctx = json.loads((self.gate_root / "run-record.json").read_text())
        ctx = {k: v for k, v in ctx.items() if k not in ("run_record_version", "recorder", "sandbox_outcome",
                                                         "target_sha_bound_by", "log_capture")}
        with self.assertRaises(self.rec.RecordError):
            self.rec.record(copy, ctx)
        self.assertFalse((copy / "evidence.json").exists())

    def test_recorder_refuses_a_gate_task_that_does_not_bind_the_candidate(self):
        copy = self.fx.repo.root / "copy-gate2"
        copy.mkdir(mode=0o700)
        for name in ("inputs", "run"):
            shutil.copytree(self.gate_root / name, copy / name)
        shutil.copy(self.gate_root / "scope.json", copy / "scope.json")
        ctx = json.loads((self.gate_root / "run-record.json").read_text())
        ctx["candidate_sha"] = "d" * 40
        with self.assertRaises(self.rec.RecordError):
            self.rec.record(copy, ctx)


if __name__ == "__main__":
    unittest.main(verbosity=2)
