#!/usr/bin/env python3
"""Unit tests for scripts/bump-version.py."""

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Import functions under test
import importlib.util
spec = importlib.util.spec_from_file_location("bump_version", Path(__file__).parent / "bump-version.py")
bump_version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bump_version)


# Guard: no test may write to the real repository files the bump functions manage.
GUARDED_REAL_FILES = (
    bump_version.CARGO_TOML,
    bump_version.CARGO_LOCK,
    bump_version.WASM_CARGO_TOML,
    bump_version.PYPROJECT_TOML,
    bump_version.PYTHON_INIT_PY,
    bump_version.PACKAGE_JSON,
    bump_version.PACKAGE_LOCK_JSON,
    bump_version.CHANNEL_MATRIX,
    bump_version.README_MD,
    bump_version.DOCS_INDEX_HTML,
)
_REAL_FILE_SNAPSHOT = {path: path.read_bytes() for path in GUARDED_REAL_FILES if path.exists()}


def tearDownModule() -> None:
    changed = [str(p) for p, before in _REAL_FILE_SNAPSHOT.items() if p.read_bytes() != before]
    if changed:
        for path in changed:  # restore so a leaking test cannot corrupt the checkout
            Path(path).write_bytes(_REAL_FILE_SNAPSHOT[Path(path)])
        raise AssertionError(f"tests modified real repository files (restored): {changed}")


CARGO_LOCK_TEMPLATE = """version = 4

[[package]]
name = "engram-core"
version = "{core}"
dependencies = ["serde"]

[[package]]
name = "engram-types"
version = "{types}"

[[package]]
name = "engram-wasm"
version = "{wasm}"

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"""

PACKAGE_LOCK_TEMPLATE = """{{
  "name": "engram-client",
  "version": "{root}",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {{
    "": {{
      "name": "engram-client",
      "version": "{package}",
      "license": "MIT",
      "devDependencies": {{
        "typescript": "^5.0.0"
      }}
    }},
    "node_modules/typescript": {{
      "version": "5.0.0"
    }}
  }}
}}
"""

MATRIX_TEMPLATE = """[matrix]
core_version = "{core}"
core_tag = "{core_tag}"
observed_utc = "{observed}"

[[channels]]
id = "git_tag"
target = "{git_tag}"

[[channels]]
id = "github_release"
target = "{release_tag}"

[[channels]]
id = "pypi"
local_version = "{python}"

[[channels]]
id = "npm"
local_version = "{typescript}"

[[sdk_compatibility]]
sdk = "python"
local_version = "{python}"
compatible_core_min = "0.20.0"
compatible_core_max = "{python_max}"

[[sdk_compatibility]]
sdk = "typescript"
local_version = "{typescript}"
compatible_core_min = "0.20.0"
compatible_core_max = "0.24.x"
"""

PATCHED_PATHS = {
    "CARGO_TOML": "Cargo.toml",
    "CARGO_LOCK": "Cargo.lock",
    "TYPES_CARGO_TOML": "types/Cargo.toml",
    "WASM_CARGO_TOML": "wasm/Cargo.toml",
    "PYPROJECT_TOML": "pyproject.toml",
    "PYTHON_INIT_PY": "__init__.py",
    "PACKAGE_JSON": "package.json",
    "PACKAGE_LOCK_JSON": "package-lock.json",
    "CHANNEL_MATRIX": "matrix.toml",
    "CHANGELOG_MD": "CHANGELOG.md",
    "README_MD": "README.md",
    "DOCS_INDEX_HTML": "index.html",
}


def write_fixture(root: Path, **overrides: str) -> None:
    """Write a fully consistent fake repo, then apply per-field overrides."""
    v = {
        "core": "0.24.0", "types": "0.24.0", "wasm": "0.1.0",
        "python": "0.7.0", "typescript": "0.7.0", "dunder": "0.7.0",
        "lock_core": "0.24.0", "lock_types": "0.24.0", "lock_wasm": "0.1.0",
        "lock_root": "0.7.0", "lock_package": "0.7.0",
        "core_tag": "v0.24.0", "git_tag": "v0.24.0", "release_tag": "v0.24.0",
        "matrix_core": "0.24.0", "observed": "2020-01-01T00:00:00Z", "python_max": "0.24.x",
    }
    v.update(overrides)
    files = {
        "Cargo.toml": f'[package]\nname = "engram-core"\nversion = "{v["core"]}"\n',
        "types/Cargo.toml": f'[package]\nname = "engram-types"\nversion = "{v["types"]}"\n',
        "wasm/Cargo.toml": f'[package]\nname = "engram-wasm"\nversion = "{v["wasm"]}"\n',
        "Cargo.lock": CARGO_LOCK_TEMPLATE.format(core=v["lock_core"], types=v["lock_types"], wasm=v["lock_wasm"]),
        "pyproject.toml": f'[project]\nname = "engram-client"\nversion = "{v["python"]}"\n',
        "__init__.py": f'"""Client."""\n\n__version__ = "{v["dunder"]}"\n',
        "package.json": json.dumps({"name": "engram-client", "version": v["typescript"]}, indent=2) + "\n",
        "package-lock.json": PACKAGE_LOCK_TEMPLATE.format(root=v["lock_root"], package=v["lock_package"]),
        "matrix.toml": MATRIX_TEMPLATE.format(
            core=v["matrix_core"], core_tag=v["core_tag"], observed=v["observed"],
            git_tag=v["git_tag"], release_tag=v["release_tag"], python=v["python"],
            typescript=v["typescript"], python_max=v["python_max"],
        ),
        "CHANGELOG.md": f"## [{v['core']}] - 2026-01-01\n",
        "README.md": "![Release](https://img.shields.io/badge/Release-v0.1.0%20GA-success.svg)\n",
        "index.html": "<p>v0.1.0 GA</p>\n",
    }
    for relative, content in files.items():
        target = root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")


@contextlib.contextmanager
def fixture_repo(**overrides: str):
    with tempfile.TemporaryDirectory() as tmpdir:
        root = Path(tmpdir)
        write_fixture(root, **overrides)
        with contextlib.ExitStack() as stack:
            for attr, relative in PATCHED_PATHS.items():
                stack.enter_context(patch.object(bump_version, attr, root / relative))
            yield root


class TestBumpVersion(unittest.TestCase):
    def test_validate_semver_valid(self):
        self.assertEqual(bump_version.validate_semver("0.22.0"), "0.22.0")
        self.assertEqual(bump_version.validate_semver("v0.22.0"), "0.22.0")
        self.assertEqual(bump_version.validate_semver("1.0.0-rc1"), "1.0.0-rc1")
        self.assertEqual(bump_version.validate_semver("v2.1.3-beta.1"), "2.1.3-beta.1")

    def test_validate_semver_invalid(self):
        invalid_versions = ["", "invalid", "1.0", "v1", "0.22.0.1", "1.0.0.0"]
        for ver in invalid_versions:
            with self.subTest(ver=ver):
                with self.assertRaises(ValueError):
                    bump_version.validate_semver(ver)

    def test_get_current_versions_live(self):
        report = bump_version.get_current_versions()
        self.assertRegex(report.core, r"^[0-9]+\.[0-9]+\.[0-9]+")
        self.assertRegex(report.wasm, r"^[0-9]+\.[0-9]+\.[0-9]+")
        self.assertRegex(report.python, r"^[0-9]+\.[0-9]+\.[0-9]+")
        self.assertRegex(report.typescript, r"^[0-9]+\.[0-9]+\.[0-9]+")

    def test_bump_cargo_simulation(self):
        with fixture_repo(core="0.22.0", matrix_core="0.22.0", lock_core="0.22.0") as root, \
             contextlib.redirect_stdout(io.StringIO()):
            bump_version.bump_core_version("0.23.0")
            self.assertIn('version = "0.23.0"', (root / "Cargo.toml").read_text(encoding="utf-8"))
            matrix = (root / "matrix.toml").read_text(encoding="utf-8")
            self.assertIn('core_version = "0.23.0"', matrix)
            self.assertIn('core_tag = "v0.23.0"', matrix)

    def test_bump_python_simulation(self):
        with fixture_repo(python="0.5.0", dunder="0.5.0") as root, contextlib.redirect_stdout(io.StringIO()):
            bump_version.bump_python_version("0.6.0")
            self.assertIn('version = "0.6.0"', (root / "pyproject.toml").read_text(encoding="utf-8"))
            self.assertIn('local_version = "0.6.0"', (root / "matrix.toml").read_text(encoding="utf-8"))

    def test_bump_typescript_simulation(self):
        with fixture_repo(typescript="0.5.0", lock_root="0.5.0", lock_package="0.5.0") as root, \
             contextlib.redirect_stdout(io.StringIO()):
            bump_version.bump_typescript_version("0.6.0")
            updated = json.loads((root / "package.json").read_text(encoding="utf-8"))
            self.assertEqual(updated["version"], "0.6.0")
            self.assertIn('local_version = "0.6.0"', (root / "matrix.toml").read_text(encoding="utf-8"))


class TestArtifactConsistency(unittest.TestCase):
    def issues(self, **overrides: str) -> list[str]:
        with fixture_repo(**overrides):
            return bump_version.collect_issues(bump_version.get_current_versions(), check_staleness=False)

    def test_consistent_fixture_has_no_issues(self):
        self.assertEqual(self.issues(), [])

    def test_package_lock_root_version_drift_is_reported(self):
        issues = self.issues(lock_root="0.5.0")
        self.assertEqual(len(issues), 1)
        self.assertIn("package-lock.json version (0.5.0)", issues[0])

    def test_package_lock_package_entry_drift_is_reported(self):
        issues = self.issues(lock_package="0.5.0")
        self.assertEqual(len(issues), 1)
        self.assertIn('packages[""].version (0.5.0)', issues[0])

    def test_python_dunder_version_drift_is_reported(self):
        issues = self.issues(dunder="0.6.0")
        self.assertEqual(len(issues), 1)
        self.assertIn("__version__ (0.6.0)", issues[0])

    def test_cargo_lock_workspace_drift_is_reported(self):
        for field, name in (("lock_core", "engram-core"), ("lock_types", "engram-types"), ("lock_wasm", "engram-wasm")):
            with self.subTest(package=name):
                issues = self.issues(**{field: "9.9.9"})
                self.assertEqual(len(issues), 1)
                self.assertIn(f"Cargo.lock entry for {name} (9.9.9)", issues[0])

    def test_registry_package_with_same_name_is_ignored(self):
        with fixture_repo() as root:
            lock = root / "Cargo.lock"
            lock.write_text(
                lock.read_text(encoding="utf-8")
                + '\n[[package]]\nname = "engram-core"\nversion = "0.1.0"\n'
                + 'source = "registry+https://github.com/rust-lang/crates.io-index"\n',
                encoding="utf-8",
            )
            self.assertEqual(bump_version.collect_cargo_lock_issues(), [])

    def test_missing_lockfiles_are_reported(self):
        with fixture_repo() as root:
            (root / "Cargo.lock").unlink()
            (root / "package-lock.json").unlink()
            report = bump_version.get_current_versions()
            issues = bump_version.collect_artifact_issues(report)
        self.assertTrue(any("Cargo.lock is missing" in issue for issue in issues))
        self.assertTrue(any("package-lock.json is missing" in issue for issue in issues))

    def test_stale_matrix_is_reported_only_when_requested(self):
        with fixture_repo():
            report = bump_version.get_current_versions()
            self.assertTrue(any("stale" in i for i in bump_version.collect_issues(report)))
            self.assertEqual(bump_version.collect_issues(report, check_staleness=False), [])

    def test_live_repo_artifact_versions_are_consistent(self):
        report = bump_version.get_current_versions()
        self.assertEqual(bump_version.collect_artifact_issues(report), [])


class TestReleasePlan(unittest.TestCase):
    def test_core_in_range(self):
        self.assertTrue(bump_version.core_in_range("0.24.0", "0.20.0", "0.24.x"))
        self.assertTrue(bump_version.core_in_range("0.24.9", "0.20.0", "0.24.x"))
        self.assertTrue(bump_version.core_in_range("0.22.0", "0.20.0", "0.22.0"))
        self.assertFalse(bump_version.core_in_range("0.25.0", "0.20.0", "0.24.x"))
        self.assertFalse(bump_version.core_in_range("0.19.9", "0.20.0", "0.24.x"))
        self.assertFalse(bump_version.core_in_range("0.24.0", "0.20.0", "bogus"))
        self.assertFalse(bump_version.core_in_range("junk", "0.20.0", "0.24.x"))

    def test_plan_derives_tag_and_sdk_versions_from_manifests(self):
        with fixture_repo():
            plan = bump_version.resolve_release_plan("v0.24.0")
        self.assertEqual(plan, bump_version.ReleasePlan("0.24.0", "v0.24.0", "0.7.0", "0.7.0"))

    def test_plan_without_tag_ignores_matrix_staleness(self):
        with fixture_repo(observed="2000-01-01T00:00:00Z"):
            self.assertEqual(bump_version.resolve_release_plan(None).core_tag, "v0.24.0")

    def test_plan_follows_manifest_not_a_hardcoded_release(self):
        with fixture_repo(
            core="0.25.1", matrix_core="0.25.1", core_tag="v0.25.1", git_tag="v0.25.1",
            release_tag="v0.25.1", lock_core="0.25.1", python="0.8.0", dunder="0.8.0",
            typescript="0.9.0", lock_root="0.9.0", lock_package="0.9.0", python_max="0.25.x",
        ) as root:
            matrix = root / "matrix.toml"
            matrix.write_text(matrix.read_text(encoding="utf-8").replace('"0.24.x"', '"0.25.x"'), encoding="utf-8")
            plan = bump_version.resolve_release_plan("v0.25.1")
        self.assertEqual(plan, bump_version.ReleasePlan("0.25.1", "v0.25.1", "0.8.0", "0.9.0"))

    def assert_rejected(self, tag, expected_fragment, **overrides):
        with fixture_repo(**overrides):
            with self.assertRaises(bump_version.ReleaseMismatchError) as caught:
                bump_version.resolve_release_plan(tag)
        self.assertTrue(
            any(expected_fragment in issue for issue in caught.exception.issues),
            f"{expected_fragment!r} not in {caught.exception.issues}",
        )

    def test_tag_must_equal_cargo_version(self):
        self.assert_rejected("v0.22.0", "!= v<Cargo.toml version> (v0.24.0)")
        self.assert_rejected("v0.24.1", "!= v<Cargo.toml version> (v0.24.0)")

    def test_tag_must_be_exact_stable(self):
        for tag in ("0.24.0", "v0.24", "v0.24.0-rc1", "v0.24.0 ", "refs/tags/v0.24.0", ""):
            with self.subTest(tag=tag):
                self.assert_rejected(tag, "exact stable vX.Y.Z tag")

    def test_prerelease_core_version_cannot_be_released(self):
        self.assert_rejected(
            None, "not a stable X.Y.Z", core="0.24.0-rc1", matrix_core="0.24.0-rc1", lock_core="0.24.0-rc1",
        )

    def test_matrix_must_declare_the_tag(self):
        self.assert_rejected("v0.24.0", "core_tag (v0.23.0)", core_tag="v0.23.0")
        self.assert_rejected("v0.24.0", "git_tag.target (v0.23.0)", git_tag="v0.23.0")
        self.assert_rejected("v0.24.0", "github_release.target (v0.23.0)", release_tag="v0.23.0")

    def test_core_must_be_inside_sdk_tested_range(self):
        self.assert_rejected("v0.24.0", "outside the python SDK tested range", python_max="0.23.x")

    def test_sdk_manifest_must_match_matrix_and_constants(self):
        self.assert_rejected(None, "package-lock.json version (0.5.0)", lock_root="0.5.0")
        self.assert_rejected(None, "__version__ (0.6.0)", dunder="0.6.0")
        self.assert_rejected(None, "Cargo.lock entry for engram-core", lock_core="0.23.0")

    def test_matrix_sdk_version_drift_is_rejected(self):
        with fixture_repo() as root:
            matrix = root / "matrix.toml"
            matrix.write_text(
                matrix.read_text(encoding="utf-8").replace('id = "pypi"\nlocal_version = "0.7.0"', 'id = "pypi"\nlocal_version = "0.5.0"'),
                encoding="utf-8",
            )
            with self.assertRaises(bump_version.ReleaseMismatchError) as caught:
                bump_version.resolve_release_plan(None)
        self.assertTrue(any("pypi local_version (0.5.0)" in i for i in caught.exception.issues))

    def test_cli_prints_only_key_value_lines_on_stdout(self):
        stdout, stderr = io.StringIO(), io.StringIO()
        with fixture_repo(), contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            self.assertEqual(bump_version.run_release_plan("v0.24.0"), 0)
        self.assertEqual(
            stdout.getvalue().splitlines(),
            ["core_version=0.24.0", "core_tag=v0.24.0", "python_version=0.7.0", "npm_version=0.7.0"],
        )
        self.assertEqual(stderr.getvalue(), "")

    def test_cli_failure_prints_nothing_on_stdout(self):
        stdout, stderr = io.StringIO(), io.StringIO()
        with fixture_repo(), contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            self.assertEqual(bump_version.run_release_plan("v0.22.0"), 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("FAILED", stderr.getvalue())

    def test_release_tag_requires_release_plan(self):
        with patch("sys.argv", ["bump-version.py", "--release-tag", "v0.24.0"]), \
             contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as caught:
            bump_version.main()
        self.assertEqual(caught.exception.code, 2)

    def test_release_plan_cannot_be_combined_with_mutations(self):
        with patch("sys.argv", ["bump-version.py", "--release-plan", "--core", "0.25.0"]), \
             contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as caught:
            bump_version.main()
        self.assertEqual(caught.exception.code, 2)


class TestBumpKeepsLockfilesInSync(unittest.TestCase):
    def quiet(self):
        return contextlib.redirect_stdout(io.StringIO())

    def test_typescript_bump_updates_package_lock_version_fields_only(self):
        with fixture_repo(), self.quiet():
            before = bump_version.PACKAGE_LOCK_JSON.read_text(encoding="utf-8")
            bump_version.bump_typescript_version("0.8.0")
            after = bump_version.PACKAGE_LOCK_JSON.read_text(encoding="utf-8")
            self.assertEqual(bump_version.read_package_lock_versions(bump_version.PACKAGE_LOCK_JSON), ("0.8.0", "0.8.0"))
        self.assertEqual(after, before.replace('"version": "0.7.0"', '"version": "0.8.0"'))
        self.assertIn('"node_modules/typescript": {\n      "version": "5.0.0"', after)

    def test_python_bump_updates_dunder_version(self):
        with fixture_repo(), self.quiet():
            bump_version.bump_python_version("0.8.0")
            self.assertEqual(bump_version.read_dunder_version(bump_version.PYTHON_INIT_PY), "0.8.0")
            self.assertIn('"""Client."""', bump_version.PYTHON_INIT_PY.read_text(encoding="utf-8"))

    def test_core_bump_updates_only_the_core_cargo_lock_entry(self):
        with fixture_repo(), self.quiet():
            bump_version.bump_core_version("0.25.0")
            versions = bump_version.read_cargo_lock_versions(bump_version.CARGO_LOCK)
            self.assertEqual(versions["engram-core"], ["0.25.0"])
            self.assertEqual(versions["engram-types"], ["0.24.0"])
            self.assertEqual(versions["engram-wasm"], ["0.1.0"])

    def test_wasm_bump_updates_the_wasm_cargo_lock_entry(self):
        with fixture_repo(), self.quiet():
            bump_version.bump_wasm_version("0.2.0")
            versions = bump_version.read_cargo_lock_versions(bump_version.CARGO_LOCK)
            self.assertEqual(versions["engram-wasm"], ["0.2.0"])
            self.assertEqual(versions["engram-core"], ["0.24.0"])


if __name__ == "__main__":
    unittest.main()
