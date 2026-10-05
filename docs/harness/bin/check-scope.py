#!/usr/bin/env python3
"""check-scope.py - scope checker of the trusted runner (task H4, ADR agent-harness-hardening-v1).

Compares a base commit with a candidate (commit or bare tree) using NUL-safe
`git diff --raw -z --no-renames` under the neutralized git environment of harness_git.py (replace
refs, grafts, attributes, hooks and global/system config ignored). Only real objects are judged.

Refusal rules (each produces a finding; any finding refuses the candidate):

  path_not_allowed      changed path outside the allowed paths (entry == path, or entry is a directory
                        prefix ending at a path separator)
  protected_path        changed (added / modified / deleted) path under a protected path. Built-in:
                        the harness TCB (docs/harness/, which holds bin/, schemas/, checks/registry.json,
                        tests/), CI and agent config (.github/, .claude/, .githooks/, .cargo/, governance/,
                        scripts/ci.sh, scripts/ci-required-features.env, justfile, Makefile, deny.toml,
                        rust-toolchain.toml, .gitleaks.toml, Dockerfile, fly.toml) and, at any depth,
                        CODEOWNERS, .gitattributes, .gitmodules, AGENTS.md, CLAUDE.md. Callers may add
                        more. Matching is case-folded (case-insensitive filesystems). A rename is a
                        delete + add (--no-renames), so moving a protected file away is refused too.
  lockfile              a lockfile changed (Cargo.lock, package-lock.json, ...) unless allow_lockfiles
  symlink_escape        a symlink is absolute, leaves the repository, loops, or exceeds 40 hops. Targets are
                        RESOLVED against the candidate tree, following symlinks of the tree itself (not a
                        lexical normpath), for every new or changed symlink and for every unchanged symlink
                        whose resolution differs from the base (a new link can redirect an old one)
  symlink_to_protected  a symlink resolves into a protected path or into .git
  submodule             a gitlink (mode 160000) is added, changed or removed
  mode_change           the mode of a file changed (e.g. 100644 -> 100755)
  type_change           a path changed type (file <-> symlink)
  unsafe_filename       a path is not valid UTF-8 or contains a control character (newline, tab, ...);
                        such a path is never handed to another git command
  test_deleted          an existing test file was deleted
  test_lines_removed    an existing test file lost lines (any removed line counts: weakening is judged
                        by a human, not by this heuristic)
  test_skip_added       a skip / ignore marker was added to a test file (#[ignore], @unittest.skip,
                        pytest.mark.skip / xfail, it|test|describe.skip(, xit(, xdescribe()
  assertion_removed     a non-test source file lost more assertion lines than it gained (inline tests)
  rust_test_attr_removed  a modified or deleted .rs file lost a line carrying a test attribute: #[test],
                        #[tokio::test] / any path::test, #[rstest], #[test_case], or a #[cfg(...test...)] gate
                        (so #[cfg(test)] -> #[cfg(any())] and a dropped #[test] are refused; editing such a
                        line counts as removing it)
                        (outer #[...] and inner #![...] attributes)
  cfg_gate_added        a modified .rs file (test or not) gained a #[cfg(...)] / #![cfg(...)] / #[cfg_attr(...)]
                        line whose predicate is not plainly a gate: allowed atoms are test, unix, windows,
                        debug_assertions, doc, miri and key = "value" for feature / target_* / panic, combined
                        with non-empty all(...) / any(...). not(...), any(), bare identifiers (cfg(FALSE)),
                        every added cfg_attr (it can inject #[ignore]) and attributes that do not close on the
                        same line are refused, because each can silently switch off existing tests.
  Known limits: replacing an assertion with a weaker one on the same line count (assert_eq!(a, b) ->
  assert!(true)) is not detected; the assertion heuristic counts lines, it does not understand them. A
  multi-line attribute is judged by its first line only: continuation lines added under an unchanged
  `#[cfg(` opener are not parsed (an added unclosed opener is refused).

Test files: any path with a `tests`, `test`, `__tests__` or `spec` directory component, or a basename
matching test_*.py, *_test.py, *_test.rs, *_tests.rs, *.test.{js,jsx,ts,tsx}, *.spec.{js,jsx,ts,tsx}.

TCB note (H3 review): the registry check pr_title_policy runs docs/harness/bin/pr-title-policy.sh from the
candidate checkout. Because docs/harness/ is a built-in protected path, a candidate that changes any
harness script is refused here, so the gate always runs the base copy of the script.

CLI exit codes: 0 PASS, 4 REFUSED, 2 usage / input error (wrong repo or ref), 1 internal error.
Final line: `SCOPE: PASS changed=<n>` | `SCOPE: REFUSED findings=<n> codes=<a,b>` | `SCOPE: INPUT_ERROR ...`.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import posixpath
import re
import sys
import unicodedata
from collections import deque
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple


def _load_sibling(name: str) -> Any:
    path = Path(__file__).resolve().parent / f"{name}.py"
    cached = sys.modules.get(name)
    if cached is not None and getattr(cached, "__file__", None) == str(path):
        return cached
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


HG = _load_sibling("harness_git")

SCHEMA = "scope-report-v1"
EXIT_PASS, EXIT_ERROR, EXIT_USAGE, EXIT_REFUSED = 0, 1, 2, 4

BUILTIN_PROTECTED_PREFIXES = (
    "docs/harness/", ".github/", ".claude/", ".githooks/", ".cargo/", ".config/", "governance/", "docs/quality/",
    "scripts/quality_baseline/",
)
BUILTIN_PROTECTED_FILES = (
    "scripts/ci.sh", "scripts/ci-required-features.env", "scripts/check-quality-budgets.py",
    "scripts/check-quality-baseline.py", "justfile", "Makefile", "deny.toml", "rust-toolchain.toml", ".gitleaks.toml",
    "Dockerfile", "fly.toml",
)
PROTECTED_BASENAMES = (
    "CODEOWNERS", ".gitattributes", ".gitmodules", "AGENTS.md", "CLAUDE.md", ".gitleaksignore", ".pre-commit-config.yaml",
    "codecov.yml", ".codecov.yml", "tarpaulin.toml", ".tarpaulin.toml", ".coveragerc", "nextest.toml",
)
MAX_SYMLINK_HOPS = 40
LOCKFILE_BASENAMES = (
    "Cargo.lock", "package-lock.json", "npm-shrinkwrap.json", "yarn.lock", "pnpm-lock.yaml", "bun.lockb",
    "poetry.lock", "uv.lock", "Pipfile.lock", "Gemfile.lock", "go.sum", "composer.lock", "flake.lock",
)
TEST_DIR_NAMES = {"tests", "test", "__tests__", "spec"}
TEST_BASENAMES = {"tests.rs", "test.rs"}
TEST_BASENAME_RE = re.compile(
    r"(test_.*\.py|.*_test\.py|.*_tests?\.rs|.*\.(test|spec)\.(js|jsx|ts|tsx|mjs|cjs))"
)
SKIP_MARKER_RE = re.compile(
    r"#\[\s*ignore\b|@unittest\.skip|unittest\.skip(If|Unless)?\(|pytest\.mark\.(skip|xfail)|\bpytest\.skip\(|"
    r"\b(?:it|test|describe)\.skip\(|\bxit\(|\bxdescribe\(|\bxtest\("
)
ASSERT_RE = re.compile(r"\bassert(_eq|_ne|_matches)?!|\bassert\b|\bself\.assert[A-Za-z]*\(|(?<![.\w])expect\(|\bdebug_assert")
RUST_TEST_ATTR_RE = re.compile(
    r"#!?\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:test|rstest|test_case)\b|#!?\[\s*cfg\s*\([^\]]*\btest\b"
)
CFG_ATTR_RE = re.compile(r"#!?\[\s*cfg(?:_attr)?\s*\(")
CFG_ATOMS = {"test", "unix", "windows", "debug_assertions", "doc", "miri"}
CFG_KEYS = {"feature", "target_os", "target_arch", "target_family", "target_env", "target_pointer_width",
            "target_endian", "target_vendor", "target_has_atomic", "panic"}
CFG_TOKEN_RE = re.compile(r'\s*(?:(all|any)\s*\(|([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"[^"\\]*"|([A-Za-z_][A-Za-z0-9_]*)|(\))|(,))')
CONTROL_RE = re.compile(r"[\x00-\x1f\x7f]")
SOURCE_SUFFIXES = (".rs", ".py", ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs")


class ScopeInputError(Exception):
    """The repository, base or candidate cannot be resolved (wrong repo / ref)."""


@dataclass(frozen=True)
class Finding:
    code: str
    path: str
    detail: str = ""

    def to_dict(self) -> Dict[str, str]:
        return {"code": self.code, "path": self.path, "detail": self.detail}


@dataclass(frozen=True)
class Change:
    status: str
    old_mode: str
    new_mode: str
    old_sha: str
    new_sha: str
    path: str
    raw_path: bytes

    def to_dict(self) -> Dict[str, str]:
        return {"status": self.status, "old_mode": self.old_mode, "new_mode": self.new_mode,
                "old_sha": self.old_sha, "new_sha": self.new_sha, "path": self.path}


@dataclass
class ScopeReport:
    base_sha: str
    candidate: str
    candidate_tree: str
    allowed_paths: List[str]
    protected_paths: List[str]
    allow_lockfiles: bool
    changes: List[Change] = field(default_factory=list)
    findings: List[Finding] = field(default_factory=list)

    @property
    def verdict(self) -> str:
        return "pass" if not self.findings else "refused"

    def to_dict(self) -> Dict[str, Any]:
        return {
            "schema": SCHEMA, "verdict": self.verdict, "base_sha": self.base_sha, "candidate": self.candidate,
            "candidate_tree": self.candidate_tree, "allowed_paths": list(self.allowed_paths),
            "protected_paths": list(self.protected_paths), "allow_lockfiles": self.allow_lockfiles,
            "changes": [c.to_dict() for c in self.changes], "findings": [f.to_dict() for f in self.findings],
        }


# --- path classification -------------------------------------------------------------------------

def _fold(text: str) -> str:
    return unicodedata.normalize("NFC", text).casefold()


def _under(path: str, entry: str, fold: bool) -> bool:
    p, e = (_fold(path), _fold(entry)) if fold else (path, entry)
    e = e.rstrip("/")
    return bool(e) and (p == e or p.startswith(e + "/"))


def is_allowed(path: str, allowed: Sequence[str]) -> bool:
    return any(_under(path, entry, fold=False) for entry in allowed)


def protected_reason(path: str, extra: Sequence[str]) -> Optional[str]:
    for entry in (*BUILTIN_PROTECTED_PREFIXES, *BUILTIN_PROTECTED_FILES, *extra):
        if _under(path, entry, fold=True):
            return entry
    base = _fold(posixpath.basename(path))
    for name in PROTECTED_BASENAMES:
        if base == _fold(name):
            return name
    if any(_fold(part) == ".git" for part in path.split("/")):
        return ".git"
    return None


def is_lockfile(path: str) -> bool:
    base = _fold(posixpath.basename(path))
    return any(base == _fold(name) for name in LOCKFILE_BASENAMES)


def is_test_file(path: str) -> bool:
    parts = path.split("/")
    return (any(p in TEST_DIR_NAMES for p in parts[:-1]) or parts[-1] in TEST_BASENAMES
            or TEST_BASENAME_RE.fullmatch(parts[-1]) is not None)


# --- git access -----------------------------------------------------------------------------------

def _resolve_tree(repo: Path, rev: str) -> str:
    if not rev or rev.startswith("-") or "\0" in rev:
        raise ScopeInputError(f"invalid revision {rev!r}")
    try:
        kind = HG.git(repo, ["cat-file", "-t", "--end-of-options", rev]).decode().strip()
        target = rev if kind == "tree" else f"{rev}^{{tree}}"
        return HG.git(repo, ["rev-parse", "--verify", "--end-of-options", target]).decode().strip()
    except HG.GitError as exc:
        raise ScopeInputError(f"'{rev}' does not name a commit or tree in {repo}") from exc


def raw_diff(repo: Path, base_tree: str, cand_tree: str) -> List[Change]:
    out = HG.git(repo, ["diff", "--raw", "-z", "--no-renames", "--full-index", "--abbrev=40", "--no-ext-diff",
                        "--no-textconv", "--ignore-submodules=none", "--no-color", base_tree, cand_tree])
    fields = out.split(b"\0")
    changes: List[Change] = []
    i = 0
    while i < len(fields) and fields[i]:
        meta = fields[i]
        if not meta.startswith(b":"):
            raise HG.GitError(f"unexpected raw diff record {meta[:40]!r}")
        old_mode, new_mode, old_sha, new_sha, status = meta[1:].decode("ascii").split(" ")
        if status[0] not in "AMDT":  # copies / renames are impossible with --no-renames: fail closed
            raise HG.GitError(f"unexpected diff status {status!r}")
        raw_path = fields[i + 1]
        changes.append(Change(status[0], old_mode, new_mode, old_sha, new_sha,
                              raw_path.decode("utf-8", "backslashreplace"), raw_path))
        i += 2
    return changes


def _blob(repo: Path, sha: str) -> bytes:
    return HG.git(repo, ["cat-file", "blob", sha])


def _patch_lines(repo: Path, base_tree: str, cand_tree: str, path: str) -> List[str]:
    out = HG.git(repo, ["diff", "--unified=0", "--text", "--no-color", "--no-ext-diff", "--no-textconv",
                        "--no-renames", base_tree, cand_tree, "--", path])
    lines = out.decode("utf-8", "replace").split("\n")
    return [ln for ln in lines if (ln.startswith("+") or ln.startswith("-")) and not ln.startswith(("+++ ", "--- "))]


# --- rules ----------------------------------------------------------------------------------------

class _LinkResolver:
    """Resolve a symlink of a tree the way a checkout would: following the tree's own symlinks."""

    def __init__(self, repo: Path, tree: str):
        self.repo = repo
        self.links = {path.decode("utf-8", "surrogateescape"): sha
                      for mode, sha, path in HG.ls_tree(repo, tree) if mode == HG.MODE_LINK}
        self._targets: Dict[str, str] = {}

    def target(self, link: str) -> str:
        if link not in self._targets:
            self._targets[link] = _blob(self.repo, self.links[link]).decode("utf-8", "surrogateescape")
        return self._targets[link]

    def resolve(self, link: str) -> Tuple[Optional[str], str]:
        """(resolved repo path, "") or (None, problem)."""
        current = [p for p in posixpath.dirname(link).split("/") if p]
        pending = deque(self.target(link).split("/"))
        if self.target(link).startswith("/"):
            return None, f"absolute target {self.target(link)!r}"
        hops = 1
        while pending:
            part = pending.popleft()
            if part in ("", "."):
                continue
            if CONTROL_RE.search(part) or any("\udc80" <= ch <= "\udcff" for ch in part):  # control / non-UTF-8
                return None, "unsafe or non-UTF-8 target"
            if part == "..":
                if not current:
                    return None, "target leaves the repository"
                current.pop()
                continue
            current.append(part)
            here = "/".join(current)
            if here in self.links:
                hops += 1
                if hops > MAX_SYMLINK_HOPS:
                    return None, f"more than {MAX_SYMLINK_HOPS} symlink hops (loop?)"
                nested = self.target(here)
                if nested.startswith("/"):
                    return None, f"chain reaches absolute target {nested!r}"
                current.pop()
                pending = deque(nested.split("/") + list(pending))
        return "/".join(current), ""


def _symlink_findings(repo: Path, base_tree: str, cand_tree: str, changes: Sequence[Change],
                      extra: Sequence[str]) -> List[Finding]:
    cand = _LinkResolver(repo, cand_tree)
    if not cand.links:
        return []
    base: Optional[_LinkResolver] = None
    changed = {c.path for c in changes if c.status != "D" and c.new_mode == HG.MODE_LINK.decode()}
    findings: List[Finding] = []
    for link in sorted(cand.links):
        if CONTROL_RE.search(link):
            continue  # already refused as unsafe_filename
        resolved, problem = cand.resolve(link)
        if link not in changed:
            base = base or _LinkResolver(repo, base_tree)
            if link in base.links and base.resolve(link) == (resolved, problem):
                continue  # untouched link whose meaning did not change
        if resolved is None:
            findings.append(Finding("symlink_escape", link, f"target {cand.target(link)!r}: {problem}"))
            continue
        reason = protected_reason(resolved, extra) if resolved else None
        if reason is not None:
            findings.append(Finding("symlink_to_protected", link,
                                    f"target {cand.target(link)!r} resolves to '{resolved}' inside '{reason}'"))
    return findings


def _plain_cfg_predicate(text: str) -> bool:
    """True when `text` (inside cfg(...)) only combines plain test / feature / target atoms."""
    depth, pos, expect_item = 0, 0, True
    while pos < len(text):
        if text[pos:].strip() == "" and depth == 0:
            break
        match = CFG_TOKEN_RE.match(text, pos)
        if match is None:
            return False
        combinator, key, atom, close, comma = match.groups()
        if combinator:
            if not expect_item:
                return False
            depth += 1
            rest = text[match.end():].lstrip()
            if rest.startswith(")"):
                return False  # any() / all() with nothing inside
        elif key or atom:
            if not expect_item or (key and key not in CFG_KEYS) or (atom and atom not in CFG_ATOMS):
                return False
            expect_item = False
        elif close:
            depth -= 1
            if depth < 0 or expect_item:
                return False
        elif comma:
            if expect_item:
                return False
            expect_item = True
        pos = match.end()
    return depth == 0 and not expect_item


def _rust_cfg_findings(path: str, lines: Sequence[str]) -> List[Finding]:
    """Added cfg / cfg_attr lines in a modified .rs file whose predicate could switch tests off."""
    findings = []
    for line in lines:
        if not line.startswith("+"):
            continue
        body = line[1:].strip()
        match = CFG_ATTR_RE.match(body)
        if match is None:
            continue
        is_cfg_attr = body[match.start():match.end()].replace(" ", "").rstrip("(").endswith("cfg_attr")
        inner = body[match.end():]
        close = inner.rfind(")]")
        if close < 0 or inner[close:].strip() != ")]":
            findings.append(Finding("cfg_gate_added", path, f"added cfg attribute does not close on its line: {body[:80]!r}"))
            continue
        if is_cfg_attr or not _plain_cfg_predicate(inner[:close]):  # cfg_attr can inject #[ignore]: always refused
            findings.append(Finding("cfg_gate_added", path, f"added cfg gate is not a plain test/feature/target gate: {body[:80]!r}"))
    return findings


def _test_findings(repo: Path, base_tree: str, cand_tree: str, change: Change) -> List[Finding]:
    path = change.path
    if change.status == "D":
        return [Finding("test_deleted", path, "an existing test file was deleted")]
    lines = _patch_lines(repo, base_tree, cand_tree, path)
    findings: List[Finding] = []
    removed = [ln for ln in lines if ln.startswith("-")]
    if change.status != "A" and removed:
        findings.append(Finding("test_lines_removed", path, f"{len(removed)} line(s) removed from an existing test"))
    if any(SKIP_MARKER_RE.search(ln[1:]) for ln in lines if ln.startswith("+")):
        findings.append(Finding("test_skip_added", path, "a skip / ignore marker was added"))
    if path.endswith(".rs") and change.status == "M":
        attrs = [ln[1:].strip() for ln in lines if ln.startswith("-") and RUST_TEST_ATTR_RE.search(ln[1:])]
        if attrs:
            findings.append(Finding("rust_test_attr_removed", path, f"test attribute line(s) removed or edited: {attrs[:3]}"))
        findings.extend(_rust_cfg_findings(path, lines))
    return findings


def _assertion_findings(repo: Path, base_tree: str, cand_tree: str, change: Change) -> List[Finding]:
    if change.status not in ("M", "D") or not change.path.endswith(SOURCE_SUFFIXES):
        return []
    lines = _patch_lines(repo, base_tree, cand_tree, change.path)
    findings: List[Finding] = []
    if change.path.endswith(".rs"):
        attrs = [ln[1:].strip() for ln in lines if ln.startswith("-") and RUST_TEST_ATTR_RE.search(ln[1:])]
        if attrs:
            findings.append(Finding("rust_test_attr_removed", change.path,
                                    f"test attribute line(s) removed or edited: {attrs[:3]}"))
        if change.status == "M":
            findings.extend(_rust_cfg_findings(change.path, lines))
    removed = sum(1 for ln in lines if ln.startswith("-") and ASSERT_RE.search(ln[1:]))
    added = sum(1 for ln in lines if ln.startswith("+") and ASSERT_RE.search(ln[1:]))
    if removed > added:
        findings.append(Finding("assertion_removed", change.path, f"{removed} assertion line(s) removed, {added} added"))
    if any(SKIP_MARKER_RE.search(ln[1:]) for ln in lines if ln.startswith("+")):
        findings.append(Finding("test_skip_added", change.path, "a skip / ignore marker was added"))
    return findings


def _change_findings(repo: Path, base_tree: str, cand_tree: str, change: Change, allowed: Sequence[str],
                     extra: Sequence[str], allow_lockfiles: bool) -> List[Finding]:
    path = change.path
    try:
        change.raw_path.decode("utf-8")
    except UnicodeDecodeError:
        return [Finding("unsafe_filename", path, "path is not valid UTF-8")]
    if CONTROL_RE.search(path):
        return [Finding("unsafe_filename", path, "path contains a control character (newline, tab, ...)")]
    findings: List[Finding] = []
    if not is_allowed(path, allowed):
        findings.append(Finding("path_not_allowed", path, "outside the task's allowed paths"))
    reason = protected_reason(path, extra)
    if reason is not None:
        findings.append(Finding("protected_path", path, f"protected by '{reason}' ({change.status})"))
    if not allow_lockfiles and is_lockfile(path):
        findings.append(Finding("lockfile", path, "lockfile changes need allow_lockfiles"))
    if HG.MODE_GITLINK.decode() in (change.old_mode, change.new_mode):
        findings.append(Finding("submodule", path, "gitlinks (submodules) are not allowed"))
        return findings
    if change.status == "T":
        findings.append(Finding("type_change", path, f"{change.old_mode} -> {change.new_mode}"))
    elif change.status == "M" and change.old_mode != change.new_mode:
        findings.append(Finding("mode_change", path, f"{change.old_mode} -> {change.new_mode}"))
    if is_test_file(path):
        findings.extend(_test_findings(repo, base_tree, cand_tree, change))
    else:
        findings.extend(_assertion_findings(repo, base_tree, cand_tree, change))
    return findings


def check_scope(repo: Path, base_sha: str, candidate: str, allowed_paths: Sequence[str],
                protected_paths: Sequence[str] = (), allow_lockfiles: bool = False) -> ScopeReport:
    """Judge base..candidate. Raises ScopeInputError for an unresolvable repo, base or candidate."""
    repo = Path(repo)
    if not repo.is_dir() or HG.toplevel(repo) is None:
        raise ScopeInputError(f"{repo} is not a git repository")
    base_commit = HG.resolve_commit(repo, base_sha)
    if base_commit is None:
        raise ScopeInputError(f"base '{base_sha}' is not a commit in {repo} (wrong repository or ref)")
    base_tree = _resolve_tree(repo, base_commit)
    cand_tree = _resolve_tree(repo, candidate)
    report = ScopeReport(base_commit, candidate, cand_tree, list(allowed_paths), list(protected_paths), allow_lockfiles)
    report.changes = raw_diff(repo, base_tree, cand_tree)
    for change in report.changes:
        report.findings.extend(_change_findings(repo, base_tree, cand_tree, change, allowed_paths, protected_paths,
                                                allow_lockfiles))
    if report.changes:
        report.findings.extend(_symlink_findings(repo, base_tree, cand_tree, report.changes, protected_paths))
    return report


# --- CLI ------------------------------------------------------------------------------------------

def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Scope checker of the trusted runner (task H4)")
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--base", required=True, help="base commit")
    parser.add_argument("--candidate", required=True, help="candidate commit or tree")
    parser.add_argument("--allowed", action="append", required=True, help="allowed path (repeatable)")
    parser.add_argument("--protected", action="append", default=[], help="extra protected path (repeatable)")
    parser.add_argument("--allow-lockfiles", action="store_true")
    parser.add_argument("--json", action="store_true", help="print the full report as JSON")
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    try:
        args = _parser().parse_args(argv)
    except SystemExit as exc:
        return EXIT_USAGE if exc.code else EXIT_PASS
    try:
        report = check_scope(args.repo, args.base, args.candidate, args.allowed, args.protected, args.allow_lockfiles)
    except ScopeInputError as exc:
        print(f"SCOPE: INPUT_ERROR reason={exc}")
        return EXIT_USAGE
    except (HG.GitError, OSError) as exc:
        print(f"SCOPE: ERROR reason={exc}")
        return EXIT_ERROR
    if args.json:
        print(json.dumps(report.to_dict(), indent=2, sort_keys=True))
        return EXIT_PASS if report.verdict == "pass" else EXIT_REFUSED
    for finding in report.findings:
        print(f"{finding.code}\t{finding.path!r}\t{finding.detail}")
    if report.verdict == "pass":
        print(f"SCOPE: PASS changed={len(report.changes)}")
        return EXIT_PASS
    codes = ",".join(sorted({f.code for f in report.findings}))
    print(f"SCOPE: REFUSED findings={len(report.findings)} codes={codes}")
    return EXIT_REFUSED


if __name__ == "__main__":
    sys.exit(main())
