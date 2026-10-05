#!/usr/bin/env python3
"""Fixtures for test_merge_gate.py (task H5). Stdlib only, offline.

* World: one verified H4 run (stub docker, fake writer) over a base that carries ci.yml, plus a
  trusted review-gate.sh copy outside the repository and its `receipt-template` output.
* Operator: the operator-owned inputs of one evaluation (review receipt filled from the producer's
  template, review-v2 artifact, identity allowlist, check runs, workflow runs), written into a
  private directory outside the repository.
* workflow_contract_errors: static read-only contract of .github/workflows/agent-evidence.yml.
"""

from __future__ import annotations

import datetime
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from runner_test_support import (BASE_TREE, BIN, POLICY_VERSION, REPO_ROOT,  # noqa: E402
                                 RunnerFixture, load_module, run_git)

MG = load_module("merge-gate")
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "agent-evidence.yml"
REVIEWER, OPERATOR = "rev-claude", "ronaldo"
CI_YML = ".github/workflows/ci.yml"
SUITE = 5000



def private_dir(owner, prefix: str) -> Path:
    path = Path(tempfile.mkdtemp(prefix=prefix)).resolve()
    owner.addCleanup(shutil.rmtree, path, ignore_errors=True)
    return path


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def now_stamp(delta_minutes: int = -1) -> str:
    delta = datetime.timedelta(minutes=delta_minutes)
    return (datetime.datetime.now(datetime.timezone.utc) + delta).strftime("%Y-%m-%dT%H:%M:%SZ")


def receipt_dict(text: str) -> dict:
    lines = [line for line in text.splitlines() if line and not line.startswith("#")]
    return dict(line.split("=", 1) for line in lines)


class _ClassCleanup:
    """Lets the per-test fixtures (TempRepo, stub docker) live for a whole TestCase class."""

    def __init__(self, cls):
        self.cls = cls

    def addCleanup(self, fn, *args, **kwargs):  # noqa: N802 - unittest naming
        self.cls.addClassCleanup(fn, *args, **kwargs)


class World:
    """A verified H4 run (base carries ci.yml), a trusted review-gate copy and its template."""

    def __init__(self, owner, invocations=None):
        self.fx = RunnerFixture(owner)
        base_files = {**BASE_TREE, CI_YML: b"name: CI\n"}
        self.fx.base = self.fx.repo.commit(base_files, parent=self.fx.base, message="ci")
        self.fx.repo.checkout(self.fx.base)
        summary = self.fx.run(invocations or [["write_file", "src/new.txt", "hello\n"]])
        if summary["status"] != "passed":
            raise AssertionError(summary)
        self.base_files = base_files
        self.head = summary["result"]["candidate"]["sha"]
        self.evidence_receipt = Path(summary["result"]["receipt"])
        self.base = self.fx.base
        self.repo = self.fx.repo.path
        self.tree = run_git(self.repo, "rev-parse", f"{self.head}^{{tree}}").decode().strip()
        self.gate_dir = private_dir(owner, "h5-gate-")
        self.gate = self.gate_dir / "review-gate.sh"
        shutil.copy(BIN / "review-gate.sh", self.gate)
        self.gate_sha = sha256(self.gate.read_bytes())
        self.template = self.receipt_template()
        self._n = 0

    def receipt_template(self, review_file=None) -> str:
        """The H1 producer: every bound value recomputed by the trusted gate copy."""
        argv = ["bash", str(self.gate), "receipt-template", "h4-task", "--repo", str(self.repo),
                "--base", self.base, "--head", self.head]
        if review_file is not None:
            argv += ["--review-file", str(review_file)]
        return subprocess.run(argv, capture_output=True, text=True, timeout=120, check=True).stdout

    def commit(self, changes: dict, parent=None) -> str:
        return self.fx.repo.commit({**self.base_files, **changes}, parent=parent or self.base)

    def head_ref(self, sha=None) -> str:
        self._n += 1
        ref = f"refs/pr/{self._n}/head"
        run_git(self.repo, "update-ref", ref, sha or self.head)
        return ref

    def base_ref(self, sha=None) -> str:
        self._n += 1
        ref = f"refs/test/base-{self._n}"
        run_git(self.repo, "update-ref", ref, sha or self.base)
        return ref


class Operator:
    """Operator-owned inputs for one evaluation, in a private directory outside the repository."""

    def __init__(self, owner, world: World):
        self.w = world
        self.dir = private_dir(owner, "h5-operator-")

    def write(self, name: str, data, mode: int = 0o600) -> Path:
        path = self.dir / name
        if path.exists():
            path.unlink()
        path.write_bytes(data if isinstance(data, bytes) else data.encode("utf-8"))
        path.chmod(mode)
        return path

    def review(self, **over) -> dict:
        doc = {"schema_version": "review-v2", "policy_version": POLICY_VERSION,
               "review_id": "h5-review-1", "base_sha": self.w.base, "head_sha": self.w.head,
               "intensity": "high", "lanes": ["correctness", "security"], "findings": [],
               "verdict": "pass", "reviewer": REVIEWER, "timestamp": now_stamp(),
               "summary": "fresh-context review of task, diff, files and evidence"}
        doc.update(over)
        return doc

    def receipt_values(self, review_path: Path, **over) -> dict:
        """Fill the producer's template: the operator adds identities, policy and the review."""
        values = receipt_dict(self.w.template)
        values.update({"REVIEW_ARTIFACT": str(review_path),
                       "REVIEW_SHA256": sha256(review_path.read_bytes()), "OPERATOR": OPERATOR,
                       "REVIEWER": REVIEWER, "POLICY_VERSION": POLICY_VERSION,
                       "REVIEW_CONTEXT": "fresh-readonly"})
        values.update(over)
        return {k: v for k, v in values.items() if v is not None}

    @staticmethod
    def identities_doc(**over) -> dict:
        doc = {"identities_version": "merge-gate-identities-v1", "operators": [OPERATOR],
               "reviewers": {REVIEWER: {"lineage": "anthropic-claude"},
                             "rev-codex": {"lineage": "openai-gpt"}},
               "writers": {"fake_writer": {"lineage": "engram-fake-writer"}}}
        doc.update(over)
        return doc

    def ci_doc(self, head=None, overrides=None, extra=None) -> dict:
        runs = []
        for i, name in enumerate(MG.REQUIRED_CI_CONTEXTS):
            run = {"id": 100 + i, "name": name, "head_sha": head or self.w.head,
                   "status": "completed", "conclusion": "success",
                   "started_at": "2026-10-05T10:00:00Z", "completed_at": f"2026-10-05T10:0{i}:00Z",
                   "app": {"slug": "github-actions"}, "check_suite": {"id": SUITE}}
            run.update((overrides or {}).get(name, {}))
            runs.append(run)
        runs.extend(extra or [])
        return {"total_count": len(runs), "check_runs": runs}

    def runs_doc(self, suites=None, head=None) -> dict:
        suites = suites if suites is not None else {SUITE: CI_YML}
        runs = [{"id": 900 + i, "check_suite_id": suite, "path": path, "name": "CI",
                 "head_sha": head or self.w.head, "event": "pull_request", "run_attempt": 1}
                for i, (suite, path) in enumerate(suites.items())]
        return {"total_count": len(runs), "workflow_runs": runs}

    def inputs(self, review=None, review_raw=None, receipt=None, receipt_raw=None, ids=None,
               ci=None, runs=None, drop=(), **over) -> "MG.Inputs":
        review_text = review_raw if review_raw is not None else json.dumps(review or self.review())
        review_path = self.write("review.json", review_text)
        values = self.receipt_values(review_path, **(receipt or {}))
        receipt_text = receipt_raw if receipt_raw is not None else \
            "".join(f"{k}={v}\n" for k, v in values.items())
        ci_doc = ci if ci is not None else self.ci_doc()
        args = {"repo": self.w.repo, "task": "h4-task", "head_ref": self.w.head_ref(),
                "base_ref": self.w.base_ref(), "policy_version": POLICY_VERSION,
                "runs_root": self.w.fx.runs_root, "evidence_receipt": self.w.evidence_receipt,
                "review_receipt": self.write("receipt.env", receipt_text),
                "review_gate": self.w.gate, "expect_gate_sha256": self.w.gate_sha,
                "identities": self.write("identities.json",
                                         json.dumps(ids or self.identities_doc())),
                "ci_results": self.write("ci.json", json.dumps(ci_doc)),
                "workflow_runs": self.write("runs.json", json.dumps(runs or self.runs_doc())),
                "registry": self.w.fx.registry, "catalog": self.w.fx.catalog}
        args.update(over)
        for key in drop:
            args[key] = None
        return MG.Inputs(**args)


class Base(unittest.TestCase):
    world: World

    @classmethod
    def setUpClass(cls):
        cls.world = World(_ClassCleanup(cls))

    def setUp(self):
        self.op = Operator(self, self.world)

    def evaluate(self, inputs, **kw):
        return MG.evaluate(inputs, **kw)

    def assertEligible(self, doc):  # noqa: N802
        self.assertEqual(doc["decision"], "eligible", json.dumps(doc["reasons"], indent=1))
        self.assertEqual(doc["reasons"], [])

    def assertRefused(self, doc, *codes):  # noqa: N802
        self.assertEqual(doc["decision"], "refused")
        got = {r["code"] for r in doc["reasons"]}
        for code in codes:
            self.assertIn(code, got, json.dumps(doc["reasons"], indent=1))
        return got


def run(run_id, name, conclusion, completed, head, suite=SUITE, status="completed",
        slug=None):
    return {"id": run_id, "name": name, "head_sha": head, "status": status,
            "conclusion": conclusion, "completed_at": completed,
            "app": {"slug": slug or "github-actions"}, "check_suite": {"id": suite}}


def run_bodies(code_lines: list) -> list:
    """Lines of every `run:` script (inline value and indented block)."""
    bodies, indent = [], None
    for line in code_lines:
        stripped = line.lstrip()
        width = len(line) - len(stripped)
        if indent is not None and stripped and width > indent:
            bodies.append(line)
            continue
        indent = None
        key = stripped[2:] if stripped.startswith("- ") else stripped
        if key.startswith("run:"):
            indent = width
            bodies.append(key[4:])
    return bodies


def workflow_contract_errors(text: str) -> list:
    """Static contract of the agent-evidence workflow (no YAML dependency)."""
    errors = []
    code = [line.split(" #", 1)[0] for line in text.splitlines()
            if not line.lstrip().startswith("#")]
    body = "\n".join(code)
    if "pull_request_target" in body:
        errors.append("pull_request_target is forbidden")
    if "\non:\n  pull_request:\n" not in "\n" + body:
        errors.append("must trigger on pull_request only")
    perms = [i for i, line in enumerate(code) if line.strip() == "permissions:"]
    if len(perms) != 1 or code[perms[0] + 1].strip() != "contents: read" \
            or code[perms[0] + 2].startswith("  "):
        errors.append("top-level permissions must be exactly `contents: read`")
    for needle, why in (("secrets.", "no secrets"), ("write", "no write permission"),
                        ("gh pr", "no gh pr actions"), ("--approve", "never approves"),
                        ("merge-queue", "no merge queue action"), ("git push", "never pushes"),
                        ("/merges", "no merge API call")):
        if needle in body:
            errors.append(f"{why}: found {needle!r}")
    uses = [line.strip() for line in code if line.strip().startswith(("- uses:", "uses:"))]
    if len(uses) != 2 or any(u.count("@") != 1 or len(u.split("@", 1)[1].strip()) != 40
                             for u in uses):
        errors.append("exactly two checkouts, pinned by full commit SHA")
    if body.count("persist-credentials: false") != 2:
        errors.append("checkouts must not persist credentials")
    scripts = run_bodies(code)
    if any("candidate/" in line for line in scripts if "python3" in line or "bash " in line):
        errors.append("a command executes something from the untrusted candidate checkout")
    if "python3 trusted/docs/harness/bin/merge-gate.py" not in body \
            or "--repo candidate" not in body:
        errors.append("the evaluator must run from the trusted base checkout against the candidate")
    if any("${{" in line for line in scripts):
        errors.append("expressions must reach run scripts through env, never inline")
    return errors
