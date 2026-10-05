#!/usr/bin/env python3
"""Scope checker tests (task H4): docs/harness/bin/check-scope.py. Offline, stdlib only.

Every case builds a base and a candidate commit with plumbing in a throw-away repository and asserts
the finding codes. Covered: allowed/outside paths, protected modify/delete/rename (also case-folded),
TCB, CI and CODEOWNERS, lockfiles (refused unless allowed), symlink escape, submodules, mode and type
changes, newline / dash / non-UTF-8 filenames, existing-test weakening (deleted file, deleted lines,
added skip markers, assertion removal) and wrong repo / ref. Replace refs must not fool the checker.
"""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from runner_test_support import BIN, TempRepo, load_module, run_git  # noqa: E402

SCOPE = load_module("check-scope")

BASE_FILES = {
    "src/lib.rs": b"pub fn f() -> u32 { 1 }\n#[cfg(test)]\nmod t {\n    #[test]\n    fn one() {\n        assert_eq!(super::f(), 1);\n    }\n}\n",
    "src/a.txt": b"alpha\n",
    "tests/api_test.py": b"import unittest\nclass T(unittest.TestCase):\n    def test_a(self):\n        self.assertEqual(1, 1)\n        self.assertTrue(True)\n",
    "tests/-weird_test.py": b"def test_x():\n    assert 1 == 1\n    assert 2 == 2\n",
    "tests/it.rs": b"#[test]\nfn it_works() {\n    assert!(true);\n}\n",
    "docs/harness/bin/tool.sh": ("exec", b"#!/bin/sh\necho tool\n"),
    "docs/harness/checks/registry.json": b"{}\n",
    "docs/notes.md": b"notes\n",
    "Cargo.lock": b"# lock\n",
    "README.md": b"readme\n",
}
ALLOWED = ["src/", "tests/", "docs/", "Cargo.lock", "README.md", "CODEOWNERS", ".github/", ".gitattributes",
           ".gitleaksignore", ".pre-commit-config.yaml", "codecov.yml", ".config/", "scripts/"]


class ScopeCase(unittest.TestCase):
    def setUp(self):
        self.repo = TempRepo(self)
        self.base = self.repo.commit(BASE_FILES)

    def candidate(self, changes=None, removed=()):
        files = dict(BASE_FILES)
        for path in removed:
            del files[path]
        files.update(changes or {})
        return self.repo.commit(files, parent=self.base)

    def check(self, cand, allowed=None, **kw):
        return SCOPE.check_scope(self.repo.path, self.base, cand, allowed if allowed is not None else ALLOWED, **kw)

    def codes(self, report):
        return sorted({f.code for f in report.findings})

    def assert_refused(self, report, code):
        self.assertEqual(report.verdict, "refused", report.to_dict())
        self.assertIn(code, self.codes(report), report.to_dict())


class AllowedAndProtectedPaths(ScopeCase):
    def test_change_inside_allowed_paths_passes_and_lists_changed_paths(self):
        report = self.check(self.candidate({"src/a.txt": b"beta\n", "src/new.rs": b"fn g() {}\n"}))
        self.assertEqual(report.verdict, "pass", report.to_dict())
        self.assertEqual(report.findings, [])
        self.assertEqual(sorted(c["path"] for c in report.to_dict()["changes"]), ["src/a.txt", "src/new.rs"])

    def test_change_outside_allowed_paths_is_refused(self):
        self.assert_refused(self.check(self.candidate({"Makefile2": b"x\n"})), "path_not_allowed")
        report = self.check(self.candidate({"srcx/evil.rs": b"x\n"}))  # prefix must end at a path separator
        self.assert_refused(report, "path_not_allowed")

    def test_tcb_modification_is_refused_even_when_allowed(self):
        report = self.check(self.candidate({"docs/harness/bin/tool.sh": ("exec", b"#!/bin/sh\necho evil\n")}))
        self.assert_refused(report, "protected_path")
        self.assert_refused(self.check(self.candidate({"docs/harness/bin/new.py": b"x\n"})), "protected_path")

    def test_protected_delete_and_rename_are_refused(self):
        self.assert_refused(self.check(self.candidate(removed=["docs/harness/checks/registry.json"])), "protected_path")
        renamed = self.candidate({"src/registry.json": b"{}\n"}, removed=["docs/harness/checks/registry.json"])
        report = self.check(renamed)
        self.assert_refused(report, "protected_path")
        statuses = {c["path"]: c["status"] for c in report.to_dict()["changes"]}
        self.assertEqual(statuses["docs/harness/checks/registry.json"], "D")  # --no-renames: delete + add

    def test_case_variant_of_a_protected_path_is_refused(self):
        self.assert_refused(self.check(self.candidate({"Docs/Harness/bin/x.sh": b"x\n"}), allowed=["Docs/"]), "protected_path")

    def test_ci_codeowners_attributes_and_task_protected_paths_are_refused(self):
        for path in (".github/workflows/ci.yml", "CODEOWNERS", "src/CODEOWNERS", ".gitattributes", "src/.gitmodules",
                     ".gitleaksignore", ".pre-commit-config.yaml", "codecov.yml", ".config/nextest.toml",
                     "docs/quality/budgets.json", "scripts/check-quality-budgets.py"):
            with self.subTest(path):
                self.assert_refused(self.check(self.candidate({path: b"x\n"})), "protected_path")
        report = self.check(self.candidate({"docs/notes.md": b"changed\n"}), protected_paths=["docs/notes.md"])
        self.assert_refused(report, "protected_path")

    def test_lockfile_is_refused_unless_explicitly_allowed(self):
        cand = self.candidate({"Cargo.lock": b"# lock 2\n"})
        self.assert_refused(self.check(cand), "lockfile")
        self.assertEqual(self.check(cand, allow_lockfiles=True).verdict, "pass")
        self.assert_refused(self.check(self.candidate({"sdks/js/package-lock.json": b"{}\n"}), allowed=["sdks/"]), "lockfile")


class TreeShapes(ScopeCase):
    def test_symlink_escaping_the_repository_is_refused(self):
        for target in ("../../etc/passwd", "/etc/passwd", "../../../../tmp"):
            with self.subTest(target):
                self.assert_refused(self.check(self.candidate({"src/l": ("link", target)})), "symlink_escape")

    def test_symlink_into_protected_or_git_paths_is_refused_and_benign_link_passes(self):
        self.assert_refused(self.check(self.candidate({"src/l": ("link", "../docs/harness/bin")})), "symlink_to_protected")
        self.assert_refused(self.check(self.candidate({"src/l": ("link", "../.git/config")})), "symlink_to_protected")
        self.assertEqual(self.check(self.candidate({"src/l": ("link", "a.txt")})).verdict, "pass")

    def test_symlink_chained_through_a_tree_symlink_is_resolved_not_lexical(self):
        escape = self.candidate({"src/a/d": ("link", "../.."), "src/a/x": ("link", "d/../../outside")})
        self.assert_refused(self.check(escape), "symlink_escape")
        into_tcb = self.candidate({"src/a/d": ("link", "../.."), "src/a/x": ("link", "d/docs/harness/bin")})
        self.assert_refused(self.check(into_tcb), "symlink_to_protected")
        benign = self.candidate({"src/a/d": ("link", "../.."), "src/a/x": ("link", "d/src/a.txt")})
        self.assertEqual(self.check(benign).verdict, "pass", self.check(benign).to_dict())

    def test_unchanged_symlink_that_starts_escaping_through_a_new_link_is_refused(self):
        base = self.repo.commit({**BASE_FILES, "src/a/x": ("link", "d/../../outside")})
        cand = self.repo.commit({**BASE_FILES, "src/a/x": ("link", "d/../../outside"), "src/a/d": ("link", "../..")}, parent=base)
        report = SCOPE.check_scope(self.repo.path, base, cand, ALLOWED)
        self.assert_refused(report, "symlink_escape")
        self.assertIn("src/a/x", {f.path for f in report.findings if f.code == "symlink_escape"})

    def test_symlink_loops_are_refused(self):
        report = self.check(self.candidate({"src/l1": ("link", "l2"), "src/l2": ("link", "l1")}))
        self.assert_refused(report, "symlink_escape")

    def test_submodule_is_refused(self):
        report = self.check(self.candidate({"src/vendored": ("gitlink", "1" * 40)}))
        self.assert_refused(report, "submodule")

    def test_mode_change_and_type_change_are_refused(self):
        self.assert_refused(self.check(self.candidate({"src/a.txt": ("exec", b"alpha\n")})), "mode_change")
        self.assert_refused(self.check(self.candidate({"src/a.txt": ("link", "lib.rs")})), "type_change")

    def test_newline_and_non_utf8_filenames_are_refused_without_confusing_the_parser(self):
        report = self.check(self.candidate({b"src/evil\nREADME.md": b"x\n", "src/ok.txt": b"ok\n"}))
        self.assert_refused(report, "unsafe_filename")
        paths = [c["path"] for c in report.to_dict()["changes"]]
        self.assertIn("src/ok.txt", paths)
        self.assertNotIn("README.md", paths)  # the newline did not split one record into two
        self.assert_refused(self.check(self.candidate({b"src/\xff\xfe.txt": b"x\n"})), "unsafe_filename")

    def test_dash_leading_filenames_are_handled_as_paths_not_options(self):
        report = self.check(self.candidate({"src/-rf": b"x\n", "src/--output=pwned": b"y\n"}))
        self.assertEqual(report.verdict, "pass", report.to_dict())
        self.assertEqual(sorted(c["path"] for c in report.to_dict()["changes"]), ["src/--output=pwned", "src/-rf"])
        weakened = self.candidate({"tests/-weird_test.py": b"def test_x():\n    assert 1 == 1\n"})
        self.assert_refused(self.check(weakened), "test_lines_removed")

    def test_candidate_may_be_a_bare_tree(self):
        cand = self.candidate({"src/a.txt": b"gamma\n"})
        tree = run_git(self.repo.path, "rev-parse", cand + "^{tree}").decode().strip()
        report = self.check(tree)
        self.assertEqual(report.verdict, "pass", report.to_dict())
        self.assertEqual(report.candidate_tree, tree)

    def test_replace_refs_do_not_hide_a_protected_change(self):
        evil = self.candidate({"docs/harness/bin/tool.sh": ("exec", b"#!/bin/sh\necho evil\n")})
        innocent = self.candidate({"src/a.txt": b"innocent\n"})
        run_git(self.repo.path, "replace", evil, innocent)
        self.assert_refused(self.check(evil), "protected_path")


class TestWeakening(ScopeCase):
    def test_deleting_an_existing_test_file_is_refused(self):
        self.assert_refused(self.check(self.candidate(removed=["tests/api_test.py"])), "test_deleted")

    def test_removing_lines_from_an_existing_test_is_refused(self):
        weakened = {"tests/api_test.py": b"import unittest\nclass T(unittest.TestCase):\n    def test_a(self):\n        self.assertEqual(1, 1)\n"}
        self.assert_refused(self.check(self.candidate(weakened)), "test_lines_removed")

    def test_adding_lines_to_an_existing_test_passes(self):
        stronger = {"tests/it.rs": BASE_FILES["tests/it.rs"] + b"\n#[test]\nfn more() {\n    assert_eq!(2, 2);\n}\n"}
        self.assertEqual(self.check(self.candidate(stronger)).verdict, "pass")

    def test_added_skip_markers_are_refused(self):
        cases = {
            "tests/it.rs": b"#[test]\n#[ignore]\nfn it_works() {\n    assert!(true);\n}\n",
            "tests/new_test.py": b"import unittest\n@unittest.skip('later')\nclass T(unittest.TestCase):\n    pass\n",
            "tests/other_test.py": b"import pytest\n@pytest.mark.skip\ndef test_z():\n    assert True\n",
        }
        for path, content in cases.items():
            with self.subTest(path):
                self.assert_refused(self.check(self.candidate({path: content})), "test_skip_added")

    def test_removing_assertions_from_source_inline_tests_is_refused(self):
        weakened = {"src/lib.rs": BASE_FILES["src/lib.rs"].replace(b"        assert_eq!(super::f(), 1);\n", b"")}
        self.assert_refused(self.check(self.candidate(weakened)), "assertion_removed")

    def test_rust_inline_test_cfg_or_attribute_weakening_is_refused(self):
        lib = BASE_FILES["src/lib.rs"]
        disabled = lib.replace(b"#[cfg(test)]", b"#[cfg(any())]")
        self.assert_refused(self.check(self.candidate({"src/lib.rs": disabled})), "rust_test_attr_removed")
        no_attr = lib.replace(b"    #[test]\n", b"")
        self.assert_refused(self.check(self.candidate({"src/lib.rs": no_attr})), "rust_test_attr_removed")
        tokio_base = {**BASE_FILES, "src/svc.rs": b"#[cfg(test)]\nmod t {\n    #[tokio::test]\n    async fn a() {}\n}\n"}
        base = self.repo.commit(tokio_base)
        cand = self.repo.commit({**tokio_base, "src/svc.rs": b"#[cfg(test)]\nmod t {\n    async fn a() {}\n}\n"}, parent=base)
        self.assert_refused(SCOPE.check_scope(self.repo.path, base, cand, ALLOWED), "rust_test_attr_removed")
        ignored = lib.replace(b"    #[test]\n", b"    #[test]\n    #[ignore]\n")
        self.assert_refused(self.check(self.candidate({"src/lib.rs": ignored})), "test_skip_added")

    def test_additive_cfg_that_disables_existing_rust_tests_is_refused(self):
        lib = BASE_FILES["src/lib.rs"]
        probes = {
            "cfg(any()) above cfg(test) mod": lib.replace(b"#[cfg(test)]\n", b"#[cfg(any())]\n#[cfg(test)]\n"),
            "bare false feature above #[test]": lib.replace(b"    #[test]\n", b"    #[cfg(FALSE_FEATURE)]\n    #[test]\n"),
            "not(test)": lib.replace(b"#[cfg(test)]\n", b"#[cfg(not(test))]\n#[cfg(test)]\n"),
            "multi-line cfg": lib.replace(b"#[cfg(test)]\n", b"#[cfg(\n    any()\n)]\n#[cfg(test)]\n"),
        }
        for label, content in probes.items():
            with self.subTest(label):
                self.assert_refused(self.check(self.candidate({"src/lib.rs": content})), "cfg_gate_added")
        additive_inner = {"tests/it.rs": b"#![cfg(any())]\n" + BASE_FILES["tests/it.rs"]}
        self.assert_refused(self.check(self.candidate(additive_inner)), "cfg_gate_added")

    def test_inner_cfg_test_gate_edited_in_a_tests_module_is_refused(self):
        files = {**BASE_FILES, "src/tests.rs": b"#![cfg(test)]\nuse super::*;\n#[test]\nfn a() {\n    assert!(true);\n}\n"}
        base = self.repo.commit(files)
        edited = {**files, "src/tests.rs": files["src/tests.rs"].replace(b"#![cfg(test)]", b"#![cfg(any())]")}
        report = SCOPE.check_scope(self.repo.path, base, self.repo.commit(edited, parent=base), ALLOWED)
        self.assert_refused(report, "rust_test_attr_removed")
        self.assertIn("test_lines_removed", self.codes(report))  # src/tests.rs is a test module
        self.assertIn("cfg_gate_added", self.codes(report))

    def test_plain_test_feature_and_target_gates_may_be_added(self):
        lib = BASE_FILES["src/lib.rs"] + (b'#[cfg(feature = "extra")]\npub fn extra() {}\n'
                                           b'#[cfg(target_os = "linux")]\npub fn linux() {}\n'
                                           b"#[cfg(all(test, unix))]\nmod unix_tests {}\n")
        report = self.check(self.candidate({"src/lib.rs": lib}))
        self.assertEqual(report.verdict, "pass", report.to_dict())

    def test_adding_rust_tests_to_a_source_file_passes(self):
        more = BASE_FILES["src/lib.rs"] + b"#[cfg(test)]\nmod more {\n    #[test]\n    fn two() {\n        assert_eq!(2, 2);\n    }\n}\n"
        self.assertEqual(self.check(self.candidate({"src/lib.rs": more})).verdict, "pass")

    def test_new_test_file_with_assertions_passes(self):
        report = self.check(self.candidate({"tests/new_test.py": b"def test_n():\n    assert 3 == 3\n"}))
        self.assertEqual(report.verdict, "pass", report.to_dict())


class InputsAndCli(ScopeCase):
    def test_wrong_repo_or_ref_is_an_input_error(self):
        other = TempRepo(self)
        foreign = other.commit({"x": b"1\n"})
        with self.assertRaises(SCOPE.ScopeInputError):
            SCOPE.check_scope(self.repo.path, foreign, self.base, ALLOWED)
        with self.assertRaises(SCOPE.ScopeInputError):
            SCOPE.check_scope(self.repo.path, self.base, "refs/heads/does-not-exist", ALLOWED)
        with self.assertRaises(SCOPE.ScopeInputError):
            SCOPE.check_scope(self.repo.root / "not-a-repo", self.base, self.base, ALLOWED)

    def run_cli(self, *args):
        proc = subprocess.run([sys.executable, str(BIN / "check-scope.py"), *args], capture_output=True, text=True, timeout=60)
        return proc.returncode, proc.stdout, proc.stderr

    def test_cli_exit_codes_and_final_line(self):
        good = self.candidate({"src/a.txt": b"b\n"})
        bad = self.candidate({"docs/harness/bin/tool.sh": b"x\n"})
        rc, out, _ = self.run_cli("--repo", str(self.repo.path), "--base", self.base, "--candidate", good, "--allowed", "src/")
        self.assertEqual(rc, 0, out)
        self.assertTrue(out.strip().splitlines()[-1].startswith("SCOPE: PASS changed=1"), out)
        rc, out, _ = self.run_cli("--repo", str(self.repo.path), "--base", self.base, "--candidate", bad, "--allowed", "docs/", "--json")
        self.assertEqual(rc, 4, out)
        payload = json.loads(out)
        self.assertEqual(payload["verdict"], "refused")
        self.assertIn("protected_path", {f["code"] for f in payload["findings"]})
        rc, out, err = self.run_cli("--repo", str(self.repo.path), "--base", "0" * 40, "--candidate", good, "--allowed", "src/")
        self.assertEqual(rc, 2, out + err)
        self.assertIn("SCOPE: INPUT_ERROR", out + err)
        rc, _, _ = self.run_cli("--repo", str(self.repo.path))
        self.assertEqual(rc, 2)

    def test_report_binds_base_and_candidate_tree(self):
        cand = self.candidate({"src/a.txt": b"c\n"})
        payload = self.check(cand).to_dict()
        self.assertEqual(payload["schema"], "scope-report-v1")
        self.assertEqual(payload["base_sha"], self.base)
        self.assertEqual(payload["candidate_tree"], run_git(self.repo.path, "rev-parse", cand + "^{tree}").decode().strip())


if __name__ == "__main__":
    unittest.main(verbosity=2)
