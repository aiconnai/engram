#!/usr/bin/env python3
"""Routing, reading and retention tests for the short, resumable harness context (task H6).

Covers, offline and without a model or network:
  * the Markdown link/anchor checker (doc_links.py / check-doc-links.py) on synthetic fixtures
    and on the real live-state documents;
  * the live summary budget (docs/harness/progress.md <= 150 lines) and the explicit
    retention of the history it replaced (docs/harness/progress-history.md, byte-exact);
  * the bootstrap contract (<= 50 lines, < 500 ms) and that the mandatory read ORDER is the
    same in bootstrap, AGENTS.md, CLAUDE.md and the harness INVARIANTS (it must never drift);
  * the context measurement tool (identified tokenizer, labelled fallback, whole-section reads);
  * the retomada scenario: scope, limits and last evidence are reachable from the live summary
    without chat memory.

The real-repository assertions only read tracked/untracked-not-ignored files, so the suite is
independent of the machine it runs on and passes in a clean clone (git worktree of HEAD).
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
BIN = REPO_ROOT / "docs" / "harness" / "bin"
sys.path.insert(0, str(BIN))

import doc_links  # noqa: E402  (sibling module under test)

CHECK_LINKS = BIN / "check-doc-links.py"
MEASURE = BIN / "measure-context.py"
BOOTSTRAP = BIN / "bootstrap.sh"
PROGRESS = REPO_ROOT / "docs" / "harness" / "progress.md"
HISTORY = REPO_ROOT / "docs" / "harness" / "progress-history.md"
BUDGET_DOC = REPO_ROOT / "docs" / "harness" / "context-budget.md"
SPEC = REPO_ROOT / "docs" / "harness" / "SPEC.md"

# Documented budgets (docs/harness/context-budget.md). Raising one needs review, like the lane floors.
PROGRESS_MAX_LINES = 150
PROGRESS_MAX_BYTES = 24_000
BOOTSTRAP_MAX_LINES = 50
BOOTSTRAP_MAX_SECONDS = 0.5  # local default; see bootstrap_time_limit()
BOOTSTRAP_MAX_SECONDS_CI = 2.0  # shared CI runners are slow and noisy; the line budget stays hard everywhere

# SHA-256 and size of the pre-H6 progress.md (docs/harness/progress.md at commit 4d341a0), pinned here and in
# context-budget.md so the archive cannot be silently regenerated from an edited file.
HISTORY_SOURCE_COMMIT = "4d341a0"
HISTORY_SHA256 = "6c3fc88f85a183a8bd5821c4046905f840c0d46945da82f750c57a442b2d8dd6"
HISTORY_BYTES = 87559

# The lane-R log is created in the other lane's worktree; links to it are knowingly pending.
PENDING_TARGETS = ["docs/harness/progress/2026-10-05-improvement-lane-r.md"]

# Mandatory read order (AGENTS.md / CLAUDE.md / bootstrap / harness INVARIANTS). Changing it
# needs an approved policy change, never a context-budget edit.
CANONICAL_ORDER = [
    "spec", "harness-invariants", "what-we-dont-do", "gates", "code-review-policy",
    "security-boundary", "harness-readme", "progress", "active-plan", "agents", "claude",
    "root-invariants", "standards", "errors-and-lessons",
]
TOKEN_RE = re.compile(
    r"(?P<spec>docs/harness/SPEC\.md)|(?P<harness_invariants>docs/harness/INVARIANTS\.md)"
    r"|(?P<what_we_dont_do>WHAT_WE_DONT_DO\.md)|(?P<gates>GATES\.md)"
    r"|(?P<code_review_policy>CODE_REVIEW_POLICY\.md)"
    r"|(?P<security_boundary>anthropic-reference-harness\.md)"
    r"|(?P<harness_readme>docs/harness/README\.md)|(?P<progress>docs/harness/progress\.md)"
    r"|(?P<active_plan>active plan)|(?P<agents>AGENTS\.md)|(?P<claude>(?i:claude\.md))"
    r"|(?P<root_invariants>(?<![/\w])INVARIANTS\.md)|(?P<standards>STANDARDS\.md)"
    r"|(?P<errors_and_lessons>ERRORS_AND_LESSONS\.md)",
)


def bootstrap_time_limit() -> float:
    """HARNESS_BOOTSTRAP_MAX_SECONDS wins; otherwise 0.5 s locally and a looser 2.0 s when CI=true."""
    explicit = os.environ.get("HARNESS_BOOTSTRAP_MAX_SECONDS")
    if explicit:
        return float(explicit)
    return BOOTSTRAP_MAX_SECONDS_CI if os.environ.get("CI", "").lower() == "true" else BOOTSTRAP_MAX_SECONDS


def run(cmd, **kw):
    kw.setdefault("capture_output", True)
    kw.setdefault("text", True)
    kw.setdefault("cwd", REPO_ROOT)
    kw.setdefault("timeout", 120)
    return subprocess.run(cmd, check=False, **kw)


def active_plan() -> str:
    """The Active plan row of the live summary (repo-relative path)."""
    m = re.search(r"(?m)^\| Active plan \| `([^`]+)` \|$", PROGRESS.read_text(encoding="utf-8"))
    assert m, "progress.md has no Active plan row"
    return m.group(1)


def read_order(block: str) -> list[str]:
    seen: list[str] = []
    for m in TOKEN_RE.finditer(block):
        name = m.lastgroup.replace("_", "-")
        if name not in seen:
            seen.append(name)
    return seen


def between(text: str, start_pat: str, end_pat: str) -> str:
    start = re.search(start_pat, text)
    assert start, f"start marker not found: {start_pat}"
    end = re.search(end_pat, text[start.end():])
    assert end, f"end marker not found: {end_pat}"
    return text[start.end(): start.end() + end.start()]


class LinkCheckerFixtures(unittest.TestCase):
    """The checker itself, on a synthetic repository (so failures are provoked on purpose)."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="h6-links-"))
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        (self.root / ".gitignore").write_text("ignored.md\n")

    def write(self, rel: str, text: str) -> None:
        p = self.root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")

    def check(self, *files: str, allow=()):
        cmd = [sys.executable, str(CHECK_LINKS), "--root", str(self.root)]
        for a in allow:
            cmd += ["--allow-missing", a]
        proc = run(cmd + list(files), cwd=self.root)
        return proc.returncode, proc.stdout + proc.stderr

    def test_valid_links_and_anchors_pass(self):
        self.write("a.md", "# Title\n\n## Second `code` Section\n\n[x](./b.md) [y](b.md#hello-world) [z](#second-code-section)\n")
        self.write("b.md", "# Hello World\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 0, out)
        self.assertIn("LINK_CHECK: PASS", out)

    def test_nonexistent_target_fails(self):
        self.write("a.md", "[gone](./missing.md)\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 1, out)
        self.assertIn("BROKEN a.md:1: target does not exist", out)

    def test_nonexistent_anchor_fails_even_when_the_file_exists(self):
        self.write("a.md", "[x](b.md#nope)\n")
        self.write("b.md", "# Real\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 1, out)
        self.assertIn("anchor #nope not found", out)

    def test_duplicate_headings_get_numeric_suffixes(self):
        self.write("a.md", "[1](b.md#same) [2](b.md#same-1) [3](b.md#same-2)\n")
        self.write("b.md", "# Same\n\n# Same\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 1, out)
        self.assertEqual(out.count("BROKEN"), 1, out)
        self.assertIn("#same-2", out)

    def test_links_in_code_fences_and_inline_code_are_not_checked(self):
        self.write("a.md", "```\n[x](./missing.md)\n```\n\nText `[y](./missing2.md)` end.\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 0, out)

    def test_external_links_are_ignored_and_reference_definitions_are_checked(self):
        self.write("a.md", "[e](https://example.invalid/x#y) [m](mailto:a@b.c)\n\n[ref]: ./missing.md\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 1, out)
        self.assertIn("missing.md", out)
        self.assertNotIn("example.invalid", out)

    def test_git_ignored_target_is_rejected_because_a_clean_clone_lacks_it(self):
        self.write("a.md", "[x](./ignored.md)\n")
        self.write("ignored.md", "# local only\n")
        rc, out = self.check("a.md")
        self.assertEqual(rc, 1, out)
        self.assertIn("git-ignored", out)

    def test_pending_targets_are_allowed_only_when_named_and_stay_visible(self):
        self.write("a.md", "[x](./later.md)\n")
        rc, out = self.check("a.md", allow=["later.md"])
        self.assertEqual(rc, 0, out)
        self.assertIn("ALLOWED a.md:1", out)
        rc, out = self.check("a.md", allow=["other.md"])
        self.assertEqual(rc, 1, out)

    def test_missing_file_to_check_fails_instead_of_passing_vacuously(self):
        rc, out = self.check("nope.md")
        self.assertEqual(rc, 1, out)

    def test_section_text_returns_whole_sections_only(self):
        self.write("h.md", "# T\n\n## A\n\npara one\n\npara two\n\n### A1\n\nsub\n\n## B\n\n```\n## not a heading\n```\n\ntail\n")
        sec = doc_links.section_text(self.root / "h.md", "a")
        self.assertEqual(sec, "## A\n\npara one\n\npara two\n\n### A1\n\nsub\n\n")
        sec_b = doc_links.section_text(self.root / "h.md", "b")
        self.assertTrue(sec_b.startswith("## B\n") and sec_b.endswith("tail\n"), sec_b)
        self.assertIsNone(doc_links.section_text(self.root / "h.md", "zzz"))


class LiveSummaryAndHistory(unittest.TestCase):
    def test_live_summary_fits_the_documented_budget(self):
        text = PROGRESS.read_text(encoding="utf-8")
        lines = len(text.splitlines())
        self.assertLessEqual(lines, PROGRESS_MAX_LINES, f"progress.md has {lines} lines (budget {PROGRESS_MAX_LINES})")
        self.assertLessEqual(len(text.encode()), PROGRESS_MAX_BYTES, "progress.md exceeds its byte budget")

    def test_doctor_compatible_live_state_fields_survive(self):
        text = PROGRESS.read_text(encoding="utf-8")
        for field in ("Project", "Active sprint", "Active task", "Active plan", "Last review", "Last sensors",
                      "Last commit", "Last live-state check"):
            self.assertRegex(text, rf"(?m)^\| {re.escape(field)} \| `[^`]+` \|$", field)
        spec = SPEC.read_text(encoding="utf-8")
        for field in ("Active sprint", "Active task", "Active plan"):
            pat = rf"(?m)^\| {field} \| `([^`]+)` \|$"
            self.assertEqual(re.search(pat, text).group(1), re.search(pat, spec).group(1), f"SPEC/progress drift: {field}")

    def test_history_is_preserved_byte_for_byte(self):
        raw = HISTORY.read_bytes()
        m = re.search(rb"<!-- BEGIN-VERBATIM source=(\S+) bytes=(\d+) sha256=([0-9a-f]{64}) -->\n", raw)
        self.assertIsNotNone(m, "history file lost its BEGIN-VERBATIM marker")
        body = raw[m.end():]
        self.assertEqual(len(body), int(m.group(2)), "verbatim body length changed")
        self.assertEqual(hashlib.sha256(body).hexdigest(), m.group(3).decode(), "verbatim body was edited")
        # The marker itself is not trusted: the constants pinned in this test (and in context-budget.md) are.
        self.assertEqual(m.group(3).decode(), HISTORY_SHA256, "marker hash differs from the pinned SHA-256")
        self.assertEqual(int(m.group(2)), HISTORY_BYTES)
        self.assertEqual(hashlib.sha256(body).hexdigest(), HISTORY_SHA256, "history body differs from the pinned SHA-256")
        self.assertIn(HISTORY_SHA256, BUDGET_DOC.read_text(encoding="utf-8"), "context-budget.md must state the pinned SHA-256")
        self.assertGreater(len(body.decode().splitlines()), 1000, "history was truncated")

    def test_history_matches_the_source_commit_when_that_object_exists(self):
        have = run(["git", "cat-file", "-e", f"{HISTORY_SOURCE_COMMIT}:docs/harness/progress.md"])
        if have.returncode != 0:  # shallow clone or rewritten history: the pinned constant above still binds
            return
        shown = subprocess.run(["git", "show", f"{HISTORY_SOURCE_COMMIT}:docs/harness/progress.md"],
                               capture_output=True, cwd=REPO_ROOT, check=True).stdout
        self.assertEqual(hashlib.sha256(shown).hexdigest(), HISTORY_SHA256)

    def test_every_historical_section_has_a_resolving_index_entry(self):
        raw = HISTORY.read_text(encoding="utf-8")
        head, _, body = raw.partition("<!-- BEGIN-VERBATIM")
        h2_in_body = [ln for ln in doc_links.iter_unfenced_lines(body) if re.match(r"^## ", ln)]
        index = re.findall(r"(?m)^\d{2}\. \[[^\]]+\]\(#([^)]+)\)$", head)
        self.assertEqual(len(index), len(h2_in_body), "index and body disagree on the number of sections")
        anchors = doc_links.anchors_of(HISTORY, {})
        self.assertEqual([a for a in index if a not in anchors], [])

    def test_historical_section_is_read_whole_through_the_tool(self):
        proc = run([sys.executable, str(MEASURE), "--section", "docs/harness/progress-history.md#pr-title-guard--2026-06-20"])
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertTrue(proc.stdout.startswith("## PR title guard — 2026-06-20\n"), proc.stdout[:80])
        self.assertIn("check-pr-title.sh", proc.stdout)
        self.assertNotRegex(proc.stdout, r"(?m)^## (?!PR title guard)")
        self.assertTrue(proc.stdout.endswith("\n"))
        # The slice equals the same section extracted from the verbatim original.
        expected = doc_links.section_text(HISTORY, "pr-title-guard--2026-06-20")
        self.assertEqual(proc.stdout, expected)

    def test_unknown_section_fails_loudly(self):
        proc = run([sys.executable, str(MEASURE), "--section", "docs/harness/progress-history.md#no-such-section"])
        self.assertEqual(proc.returncode, 1)
        self.assertIn("section not found", proc.stderr)

    def test_live_state_documents_have_no_broken_links_or_anchors(self):
        files = ["docs/harness/progress.md", "docs/harness/progress-history.md", "docs/harness/context-budget.md"]
        cmd = [sys.executable, str(CHECK_LINKS)]
        for p in PENDING_TARGETS:
            cmd += ["--allow-missing", p]
        proc = run(cmd + files)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("LINK_CHECK: PASS files=3", proc.stdout)

    def test_live_summary_itself_links_to_nothing_pending(self):
        proc = run([sys.executable, str(CHECK_LINKS), "docs/harness/progress.md"])
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertNotIn("ALLOWED", proc.stdout)


class BootstrapAndReadOrder(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.proc = run(["bash", str(BOOTSTRAP)])
        cls.out = cls.proc.stdout

    def test_bootstrap_exits_zero_with_the_contract_markers(self):
        self.assertEqual(self.proc.returncode, 0, self.proc.stderr)
        for marker in ("engram harness state", "Branch:", "Sprint:", "Read next"):
            self.assertIn(marker, self.out)

    def test_bootstrap_output_fits_the_line_budget(self):
        n = len(self.out.splitlines())
        self.assertLessEqual(n, BOOTSTRAP_MAX_LINES, f"bootstrap printed {n} lines")

    def test_bootstrap_median_time_is_under_the_budget(self):
        samples = []
        for _ in range(5):
            t0 = time.perf_counter()
            proc = run(["bash", str(BOOTSTRAP)])
            samples.append(time.perf_counter() - t0)
            self.assertEqual(proc.returncode, 0)
        limit = bootstrap_time_limit()
        self.assertLess(statistics.median(samples), limit, f"samples={samples} limit={limit}s (HARNESS_BOOTSTRAP_MAX_SECONDS, CI=true -> {BOOTSTRAP_MAX_SECONDS_CI}s)")

    def test_bootstrap_points_at_the_active_plan_and_it_exists(self):
        m = re.search(r"(?m)^Active plan: (\S+)$", self.out)
        self.assertIsNotNone(m, self.out)
        self.assertTrue((REPO_ROOT / m.group(1)).is_file(), m.group(1))

    def test_bootstrap_is_read_only(self):
        before = run(["git", "status", "--porcelain"]).stdout
        run(["bash", str(BOOTSTRAP)])
        self.assertEqual(run(["git", "status", "--porcelain"]).stdout, before)

    def test_mandatory_read_order_is_identical_everywhere(self):
        agents = read_order(between((REPO_ROOT / "AGENTS.md").read_text(), r"Em seguida leia \(em ordem\):", r"\n\n"))
        claude = read_order(between((REPO_ROOT / "CLAUDE.md").read_text(), r"Then read in order:", r"\n\nThis operational"))
        boot = read_order(between(self.out, r"--- Read next \(in order\) ---", r"Then run:"))
        self.assertEqual(agents, CANONICAL_ORDER)
        # CLAUDE.md is the file being read, so it does not list itself.
        self.assertEqual(claude, [x for x in CANONICAL_ORDER if x != "claude"])
        # bootstrap lists the static files; the active plan is resolved from progress.md instead.
        self.assertEqual(boot, [x for x in CANONICAL_ORDER if x != "active-plan"])

    def test_harness_invariant_order_is_a_consistent_prefix(self):
        text = (REPO_ROOT / "docs" / "harness" / "INVARIANTS.md").read_text(encoding="utf-8")
        line = next(ln for ln in text.splitlines() if "Ordem de leitura obrigatória" in ln)
        names = ["SPEC.md", "INVARIANTS.md", "WHAT_WE_DONT_DO.md", "GATES.md", "CODE_REVIEW_POLICY.md", "progress.md", "active plan"]
        pos = [line.find(n) for n in names]
        self.assertTrue(all(p >= 0 for p in pos), line)
        self.assertEqual(pos, sorted(pos), line)

    def test_every_mandatory_file_exists(self):
        for rel in ("docs/harness/SPEC.md", "docs/harness/INVARIANTS.md", "docs/harness/WHAT_WE_DONT_DO.md",
                    "docs/harness/GATES.md", "docs/harness/CODE_REVIEW_POLICY.md",
                    "docs/harness/security/anthropic-reference-harness.md", "docs/harness/README.md",
                    "docs/harness/progress.md", "AGENTS.md", "CLAUDE.md", "INVARIANTS.md", "STANDARDS.md",
                    "ERRORS_AND_LESSONS.md"):
            self.assertTrue((REPO_ROOT / rel).is_file(), rel)


class DoctorLiveStateAncestry(unittest.TestCase):
    """doctor.sh must surface what the structural live-state check could not prove, never pass silently.

    Runs the real doctor.sh from clones of the committed HEAD (shallow and full history), with the
    Last commit row rewritten in the clone's working tree only.
    """

    ABSENT = "0123456789abcdef0123456789abcdef01234567"

    def clone(self, shallow: bool) -> Path:
        dest = Path(tempfile.mkdtemp(prefix="h6-doctor-"))
        self.addCleanup(shutil.rmtree, dest, ignore_errors=True)
        cmd = ["git", "clone", "-q"]
        cmd += ["--no-local", "--depth", "1", f"file://{REPO_ROOT}"] if shallow else ["--local", str(REPO_ROOT)]
        proc = subprocess.run(cmd + [str(dest / "repo")], capture_output=True, text=True, check=False)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return dest / "repo"

    def set_last_commit(self, repo: Path, value: str) -> None:
        prog = repo / "docs/harness/progress.md"
        text = prog.read_text(encoding="utf-8")
        new, n = re.subn(r"(?m)^(\| Last commit \| `)[^`]+(` \|)$", rf"\g<1>{value}\2", text)
        self.assertEqual(n, 1)
        prog.write_text(new, encoding="utf-8")

    def doctor(self, repo: Path):
        return run(["bash", "docs/harness/bin/doctor.sh"], cwd=repo)

    def test_shallow_clone_warns_that_ancestry_was_not_verified(self):
        repo = self.clone(shallow=True)
        self.set_last_commit(repo, self.ABSENT)
        proc = self.doctor(repo)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("WARN: ancestry not verified (shallow clone)", proc.stdout + proc.stderr)

    def test_unreachable_last_commit_on_full_history_warns_but_does_not_fail(self):
        repo = self.clone(shallow=False)
        self.set_last_commit(repo, self.ABSENT)
        proc = self.doctor(repo)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("not reachable from HEAD (squash/rebase merges rewrite SHAs", proc.stdout + proc.stderr)

    def test_malformed_last_commit_is_a_hard_failure_even_in_a_shallow_clone(self):
        repo = self.clone(shallow=True)
        self.set_last_commit(repo, "not-a-sha")
        proc = self.doctor(repo)
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        self.assertIn("is not a commit id", proc.stdout + proc.stderr)

    def test_reachable_last_commit_passes_without_an_ancestry_warning(self):
        repo = self.clone(shallow=False)
        parent = run(["git", "rev-parse", "--short", "HEAD~1"], cwd=repo).stdout.strip()
        self.set_last_commit(repo, parent)
        proc = self.doctor(repo)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertNotIn("ancestry not verified", proc.stdout + proc.stderr)
        self.assertNotIn("not reachable from HEAD", proc.stdout + proc.stderr)


class MeasureTool(unittest.TestCase):
    def measure(self, *args, env=None):
        full_env = {**os.environ, **(env or {})}
        return run([sys.executable, str(MEASURE), *args], env=full_env)

    def test_mandatory_set_is_measured_in_bytes_lines_and_tokens_with_a_named_tokenizer(self):
        proc = self.measure("--json")
        self.assertEqual(proc.returncode, 0, proc.stderr)
        data = json.loads(proc.stdout)
        paths = [f["path"] for f in data["files"]]
        self.assertEqual(paths[0], "docs/harness/SPEC.md")
        self.assertIn("docs/harness/progress.md", paths)
        self.assertEqual(data["missing"], [])
        self.assertIn(active_plan(), paths, "active plan not in the set")
        self.assertEqual(paths.index(active_plan()), paths.index("docs/harness/progress.md") + 1, "plan must follow progress.md")
        self.assertGreater(data["totals"]["bytes"], 100_000)
        self.assertGreater(data["totals"]["tokens"], 10_000)
        self.assertTrue(data["tokenizer"]["name"])
        self.assertIn("exact", data["tokenizer"])
        for f in data["files"]:
            self.assertGreater(f["tokens"], 0, f["path"])

    def test_without_tiktoken_the_fallback_is_labelled_as_an_approximation(self):
        shim = Path(tempfile.mkdtemp(prefix="h6-shim-"))
        self.addCleanup(shutil.rmtree, shim, ignore_errors=True)
        (shim / "tiktoken.py").write_text("raise ImportError('blocked for the test')\n")
        proc = self.measure("--json", env={"PYTHONPATH": str(shim)})
        self.assertEqual(proc.returncode, 0, proc.stderr)
        tok = json.loads(proc.stdout)["tokenizer"]
        self.assertEqual(tok["name"], "approx:utf8-bytes/4")
        self.assertFalse(tok["exact"])
        self.assertIn("tiktoken", tok["fallback_reason"])

    def test_explicit_approx_and_unknown_tokenizer_choices(self):
        proc = self.measure("--json", "--tokenizer", "approx")
        self.assertEqual(json.loads(proc.stdout)["tokenizer"]["name"], "approx:utf8-bytes/4")
        proc = self.measure("--tokenizer", "made-up")
        self.assertEqual(proc.returncode, 2)

    def test_forcing_an_unavailable_exact_tokenizer_fails_instead_of_silently_approximating(self):
        shim = Path(tempfile.mkdtemp(prefix="h6-shim-"))
        self.addCleanup(shutil.rmtree, shim, ignore_errors=True)
        (shim / "tiktoken.py").write_text("raise ImportError('blocked for the test')\n")
        proc = self.measure("--tokenizer", "tiktoken", env={"PYTHONPATH": str(shim)})
        self.assertEqual(proc.returncode, 1)
        self.assertIn("tiktoken", proc.stderr)

    def test_a_missing_mandatory_file_is_reported_and_fails(self):
        proc = self.measure("--json", "--files", "docs/harness/SPEC.md", "docs/harness/does-not-exist.md")
        self.assertEqual(proc.returncode, 1)
        data = json.loads(proc.stdout)
        self.assertEqual(data["missing"], ["docs/harness/does-not-exist.md"])

    def test_repeated_measurement_is_deterministic(self):
        a = self.measure("--json", "--tokenizer", "approx").stdout
        b = self.measure("--json", "--tokenizer", "approx").stdout
        self.assertEqual(a, b)

    def test_live_summary_is_a_small_share_of_the_mandatory_read_set(self):
        data = json.loads(self.measure("--json", "--tokenizer", "approx").stdout)
        by_path = {f["path"]: f for f in data["files"]}
        share = by_path["docs/harness/progress.md"]["bytes"] / data["totals"]["bytes"]
        self.assertLess(share, 0.15, f"progress.md is {share:.0%} of the mandatory read set")


class RetomadaScenario(unittest.TestCase):
    """A fresh agent, no chat memory: bootstrap + progress.md must reach scope, limits, last evidence."""

    @classmethod
    def setUpClass(cls):
        cls.text = PROGRESS.read_text(encoding="utf-8")
        cls.section = doc_links.section_text(PROGRESS, "retomada-rápida-escopo-limites-última-evidência")

    def bullet(self, label: str) -> str:
        m = re.search(rf"(?m)^- \*\*{re.escape(label)}\*\*:.*(?:\n  .*)*", self.section or "")
        self.assertIsNotNone(m, f"retomada section lacks '{label}'")
        return m.group(0)

    def test_retomada_section_exists(self):
        self.assertIsNotNone(self.section, "progress.md lacks the retomada section")

    def test_scope_limits_and_last_evidence_each_carry_resolving_links(self):
        for label in ("Escopo", "Limites", "Última evidência"):
            links = re.findall(r"\]\((\.[^)#]*)(?:#[^)]*)?\)", self.bullet(label))
            self.assertGreaterEqual(len(links), 1, f"{label} has no relative link")
            for rel in links:
                self.assertTrue((PROGRESS.parent / rel).exists(), f"{label}: {rel}")

    def test_scope_leads_to_the_active_plan_and_to_the_spec(self):
        scope = self.bullet("Escopo")
        self.assertIn(active_plan().removeprefix("docs/harness/"), scope)
        self.assertIn("SPEC.md", scope)

    def test_limits_name_the_negative_scope_and_the_not_granted_list(self):
        limits = self.bullet("Limites")
        self.assertIn("WHAT_WE_DONT_DO.md", limits)
        self.assertRegex(limits, r"(?i)push|merge")

    def test_last_evidence_points_to_an_existing_entry_of_the_active_plan(self):
        ev = self.bullet("Última evidência")
        m = re.search(r"\]\((\./progress/[^)#]+)#([^)]+)\)", ev)
        self.assertIsNotNone(m, ev)
        target = (PROGRESS.parent / m.group(1)).resolve()
        self.assertEqual(target, (REPO_ROOT / active_plan()).resolve(), "last evidence must live in the active plan")
        self.assertIn(m.group(2), doc_links.anchors_of(target, {}))

    def test_history_is_reachable_from_the_live_summary(self):
        self.assertIn("(./progress-history.md", self.text)
        self.assertIn("(./context-budget.md", self.text)

    def test_active_plan_field_resolves_and_is_a_whole_document(self):
        body = (REPO_ROOT / active_plan()).read_text(encoding="utf-8")
        self.assertTrue(body.startswith("# "))
        self.assertTrue(body.endswith("\n"))


class ContextBudgetDocument(unittest.TestCase):
    def setUp(self):
        self.doc = BUDGET_DOC.read_text(encoding="utf-8")

    def test_records_budgets_tokenizer_and_measured_before_after(self):
        for needle in ("150", "50 linhas", "500 ms", "cl100k_base", "tiktoken-rs", "approx:utf8-bytes/4", "Antes", "Depois"):
            self.assertIn(needle, self.doc, needle)

    def test_records_the_owner_ruling_on_issue_152(self):
        section = doc_links.section_text(BUDGET_DOC, "6-decisão-152--chunks-por-caracteres-não-por-tokens")
        self.assertIsNotNone(section, "decision #152 section missing or renamed")
        section = " ".join(section.split())  # line wrapping must not matter
        self.assertRegex(section, r"(?i)ruling do controller")
        self.assertRegex(section, r"(?i)pendente de confirma[cç][aã]o do owner")
        self.assertNotRegex(section, r"(?i)ruling do owner")
        self.assertRegex(section, r"(?i)caracteres")
        self.assertIn("Nenhum tokenizer", section)
        self.assertIn("Q7", section)
        for fact in ("DEFAULT_CHUNK_SIZE", "TokenChunker", "chamador de produção"):
            self.assertIn(fact, section, fact)

    def test_states_the_retention_and_live_state_enforcement_rules(self):
        for needle in ("progress-history.md", "check-live-state.sh", "--structural", "--require-ancestor",
                       "unreachable", "skipped-shallow", "squash", "doctor", "HARNESS_BOOTSTRAP_MAX_SECONDS"):
            self.assertIn(needle, self.doc, needle)

    def test_states_that_the_read_order_and_mandatory_authority_are_unchanged(self):
        self.assertRegex(self.doc, r"(?i)ordem de leitura")
        self.assertRegex(self.doc, r"(?i)autoridade obrigat")


if __name__ == "__main__":
    unittest.main(verbosity=2)
