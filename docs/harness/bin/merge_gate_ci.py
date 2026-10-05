#!/usr/bin/env python3
"""CI provenance rules of the read-only merge-policy evaluator (task H5). Stdlib only.

Imported by merge-gate.py. A check run is trusted CI evidence for the PR head only when:
  * the PR touches nothing that defines, configures or implements a required job: CI_PROTECTED_*
    (all of .github/, tool configs, scripts/, gate inputs) plus every file the required-job closure
    of the BASE ci.yml references (derived at evaluation time). A PR that edits them could make its
    own jobs report success, so its CI results are not policy input (a human judges it);
  * the candidate's .github/workflows/ci.yml blob is the base blob;
  * it was produced by the `github-actions` app for exactly this head;
  * every required context resolves to exactly ONE check suite, and that suite belongs to a workflow
    run whose path is .github/workflows/ci.yml (operator-saved `actions/runs?head_sha=` metadata).
    Re-runs inside that suite are allowed (latest wins, an in-progress re-run blocks); a run of the
    same context in any other suite refuses, so a same-named job elsewhere cannot launder a failure.
    Consequence: a PR reopen or a workflow_dispatch that creates a second ci.yml suite for the same
    head makes that head permanently `ci_context_multiple_suites`; recovery is a new commit (new
    head, new evidence, review and receipt), never deleting runs from the saved file.

Operator commands (read-only token, never the writer's), saved outside every worktree:
  gh api --paginate --slurp "repos/<o>/<r>/commits/<head>/check-runs?per_page=100" > ci.json
  gh api --paginate --slurp "repos/<o>/<r>/actions/runs?head_sha=<head>&per_page=100" > runs.json
"""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any, Callable, Dict, List, Mapping, Sequence, Set, Tuple

REQUIRED_CI_CONTEXTS = ("Format", "Clippy", "Test (ubuntu-latest)", "Documentation",
                        "Security Audit", "Cargo Deny", "Security Gate")
CI_APP_SLUG = "github-actions"  # commit statuses / other apps can be posted by any token: ignored
CI_WORKFLOW_PATH = ".github/workflows/ci.yml"
# Everything that defines, configures or implements a required job (directly, through `needs:`, or
# implicitly as a tool config) is protected. test_merge_gate.py derives every file the required-job
# closure of ci.yml references and fails if one is not covered here, so additions cannot drift.
CI_PROTECTED_PREFIXES = (".github/", ".cargo/", ".config/", "scripts/", "docs/harness/bin/",
                         "docs/harness/tests/", "docs/harness/checks/", "docs/harness/schemas/",
                         "docs/security/", "docs/quality/", "benches/results/",
                         "tests/fixtures/retrieval_quality/")
CI_PROTECTED_FILES = ("deny.toml", ".gitleaks.toml", ".gitleaksignore", ".semgrepignore",
                      "rust-toolchain.toml", "rust-toolchain", "rustfmt.toml", ".rustfmt.toml",
                      "clippy.toml", ".clippy.toml", "Makefile", "justfile",
                      "tests/fixtures/security_gate_matrix.json")


class CiRefusal(Exception):
    def __init__(self, code: str, detail: str):
        super().__init__(f"{code}: {detail}")
        self.code, self.detail = code, detail


def policy_fingerprint() -> str:
    """sha256 of the CI policy constants, reported in `bound` with the merge policy id."""
    policy = {"contexts": REQUIRED_CI_CONTEXTS, "app": CI_APP_SLUG, "workflow": CI_WORKFLOW_PATH,
              "protected_prefixes": CI_PROTECTED_PREFIXES, "protected_files": CI_PROTECTED_FILES}
    return hashlib.sha256(json.dumps(policy, sort_keys=True).encode("utf-8")).hexdigest()


def is_protected(path: str) -> bool:
    return path.startswith(CI_PROTECTED_PREFIXES) or path in CI_PROTECTED_FILES


def _jobs(ci_text: str) -> Dict[str, List[str]]:
    jobs: Dict[str, List[str]] = {}
    current, inside = None, False
    for line in ci_text.splitlines():
        if line == "jobs:":
            inside = True
            continue
        match = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line) if inside else None
        if match:
            current = match.group(1)
            jobs[current] = []
        elif current is not None:
            jobs[current].append(line)
    return jobs


def _needs(body: Sequence[str]) -> List[str]:
    text = "\n".join(body) + "\n"
    inline = re.search(r"^    needs:\s*\[(.*?)\]", text, re.M)
    if inline:
        return [item.strip().strip("'\"") for item in inline.group(1).split(",") if item.strip()]
    block = re.search(r"^    needs:\s*\n((?:      - .*\n)+)", text, re.M)
    return re.findall(r"- ([A-Za-z0-9_-]+)", block.group(1)) if block else []


def required_jobs(ci_text: str) -> Set[str]:
    """Jobs named after a required context plus their transitive `needs:`."""
    jobs = _jobs(ci_text)
    pending = [job for job, body in jobs.items()
               if any(re.fullmatch(rf"    name:\s*['\"]?{re.escape(c)}['\"]?\s*", line)
                      for line in body for c in REQUIRED_CI_CONTEXTS)]
    seen: Set[str] = set()
    while pending:
        job = pending.pop()
        if job in seen or job not in jobs:
            continue
        seen.add(job)
        pending.extend(_needs(jobs[job]))
    return seen


def required_job_references(ci_text: str, exists: Callable[[str], bool]) -> Set[str]:
    """Repository files named in the steps of the required-job closure (run:, with:, ...)."""
    jobs, refs = _jobs(ci_text), set()
    for job in required_jobs(ci_text):
        for line in jobs[job]:
            if line.lstrip().startswith("#"):
                continue
            for token in re.findall(r"[A-Za-z0-9_.@/-]+", line):
                token = token[2:] if token.startswith("./") else token
                if ("/" in token or "." in token) and exists(token):
                    refs.add(token)
    return refs


def base_references(hg: Any, repo: Path, base: str) -> Set[str]:
    """required_job_references() of the BASE ci.yml against the base tree (dynamic protection)."""
    try:
        text = hg.git(repo, ["show", f"{base}:{CI_WORKFLOW_PATH}"]).decode("utf-8", "replace")
        listing = hg.git(repo, ["ls-tree", "-r", "-z", "--name-only", "--full-tree",
                                "--end-of-options", base])
    except hg.GitError:
        return set()
    files = {p.decode("utf-8", "replace") for p in listing.split(b"\0") if p}
    return required_job_references(text, files.__contains__)


def protected_changes(hg: Any, repo: Path, base: str, head: str) -> List[str]:
    out = hg.git(repo, ["diff", "--name-only", "-z", "--no-renames", "--no-ext-diff",
                        "--end-of-options", base, head])
    paths = [p.decode("utf-8", "replace") for p in out.split(b"\0") if p]
    referenced = base_references(hg, repo, base)
    return sorted(p for p in paths if is_protected(p) or p in referenced)


def workflow_blob(hg: Any, repo: Path, commit: str) -> str:
    try:
        return hg.git(repo, ["rev-parse", "--verify", "--end-of-options",
                             f"{commit}:{CI_WORKFLOW_PATH}"]).decode().strip()
    except hg.GitError:
        return ""


def _is_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def pages(data: Any, key: str, what: str) -> List[Mapping[str, Any]]:
    """Items of a `gh api` listing (object or --slurp page list); total_count must match."""
    items: List[Any] = []
    totals: Set[int] = set()
    for page in data if isinstance(data, list) else [data]:
        if not isinstance(page, dict) or not isinstance(page.get(key), list) \
                or not _is_int(page.get("total_count")):
            raise CiRefusal(f"{what}_malformed",
                            f"expected `gh api` output with total_count and {key}")
        totals.add(page["total_count"])
        items.extend(page[key])
    if len(totals) != 1 or totals.pop() != len(items):
        raise CiRefusal(f"{what}_incomplete",
                        f"{len(items)} {key} saved; total_count differs (paginate)")
    if not all(isinstance(item, dict) for item in items):
        raise CiRefusal(f"{what}_malformed", f"{key} entries must be objects")
    return items


def _check_run_ok(run: Mapping[str, Any]) -> bool:
    suite, app = run.get("check_suite"), run.get("app")
    return (_is_int(run.get("id")) and isinstance(run.get("name"), str)
            and isinstance(run.get("head_sha"), str) and isinstance(run.get("status"), str)
            and (run.get("conclusion") is None or isinstance(run.get("conclusion"), str))
            and (run.get("completed_at") is None or isinstance(run.get("completed_at"), str))
            and isinstance(app, dict) and isinstance(app.get("slug"), str)
            and isinstance(suite, dict) and _is_int(suite.get("id")))


def _workflow_run_ok(run: Mapping[str, Any]) -> bool:
    return (_is_int(run.get("id")) and _is_int(run.get("check_suite_id"))
            and isinstance(run.get("path"), str) and isinstance(run.get("head_sha"), str))


def _suite_paths(workflow_runs: Sequence[Mapping[str, Any]], head: str) -> Dict[int, Set[str]]:
    if not all(_workflow_run_ok(r) for r in workflow_runs):
        raise CiRefusal("workflow_runs_malformed",
                        "workflow run without id/check_suite_id/path/head_sha")
    foreign = sorted({r["head_sha"] for r in workflow_runs if r["head_sha"] != head})
    if foreign:
        raise CiRefusal("workflow_runs_head_mismatch", f"other commits: {', '.join(foreign)[:200]}")
    paths: Dict[int, Set[str]] = {}
    for run in workflow_runs:
        paths.setdefault(run["check_suite_id"], set()).add(run["path"])
    return paths


def results_by_context(check_runs: Sequence[Mapping[str, Any]],
                       workflow_runs: Sequence[Mapping[str, Any]],
                       head: str) -> Tuple[Dict[str, str], List[Tuple[str, str]]]:
    """(Q5-style result per required context, refusals). A context with no trusted run is absent."""
    if not all(_check_run_ok(r) for r in check_runs):
        raise CiRefusal("ci_results_malformed",
                        "check run without id/name/head_sha/status/app/check_suite")
    foreign = sorted({r["head_sha"] for r in check_runs if r["head_sha"] != head})
    if foreign:
        raise CiRefusal("ci_results_head_mismatch", f"other commits: {', '.join(foreign)[:200]}")
    suite_paths = _suite_paths(workflow_runs, head)
    results: Dict[str, str] = {}
    refusals: List[Tuple[str, str]] = []
    for context in REQUIRED_CI_CONTEXTS:
        mine = [r for r in check_runs if r["name"] == context and r["app"]["slug"] == CI_APP_SLUG]
        suites = sorted({r["check_suite"]["id"] for r in mine})
        if len(suites) > 1:
            refusals.append(("ci_context_multiple_suites",
                             f"{context!r} ran in check suites {suites}; "
                             "only re-runs inside one ci.yml suite are accepted"))
            continue
        if not suites:
            continue
        if suite_paths.get(suites[0]) != {CI_WORKFLOW_PATH}:
            saved = sorted(suite_paths.get(suites[0], []))
            refusals.append(("ci_workflow_unbound", f"{context!r} suite {suites[0]} is not a "
                             f"{CI_WORKFLOW_PATH} run (saved paths: {saved})"))
            continue
        # in-progress re-run (no completed_at) sorts as the newest result
        latest = max(mine, key=lambda r: (r.get("completed_at") or "~", r["id"]))
        done = latest["status"] == "completed"
        results[context] = (latest.get("conclusion") or "missing") if done else "pending"
    return results, refusals
