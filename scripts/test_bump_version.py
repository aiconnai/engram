#!/usr/bin/env python3
"""Unit tests for scripts/bump-version.py."""

from __future__ import annotations

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
        sample_cargo = """[package]
name = "engram-core"
version = "0.22.0"
edition = "2021"
"""
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir) / "Cargo.toml"
            tmp_path.write_text(sample_cargo, encoding="utf-8")
            with patch.object(bump_version, "CARGO_TOML", tmp_path), \
                 patch.object(bump_version, "CHANNEL_MATRIX", Path(tmpdir) / "matrix.toml"), \
                 patch.object(bump_version, "README_MD", Path(tmpdir) / "README.md"), \
                 patch.object(bump_version, "DOCS_INDEX_HTML", Path(tmpdir) / "index.html"):
                
                Path(tmpdir, "matrix.toml").write_text('core_version = "0.22.0"\ncore_tag = "v0.22.0"\nobserved_utc = "2026-01-01T00:00:00Z"\n', encoding="utf-8")
                bump_version.bump_core_version("0.23.0")
                updated = tmp_path.read_text(encoding="utf-8")
                self.assertIn('version = "0.23.0"', updated)

    def test_bump_python_simulation(self):
        sample_pyproject = """[project]
name = "engram-client"
version = "0.5.0"
description = "Python client"
"""
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir) / "pyproject.toml"
            tmp_path.write_text(sample_pyproject, encoding="utf-8")
            matrix_path = Path(tmpdir) / "matrix.toml"
            matrix_path.write_text('[[channels]]\nid = "pypi"\nlocal_version = "0.5.0"\n', encoding="utf-8")

            with patch.object(bump_version, "PYPROJECT_TOML", tmp_path), \
                 patch.object(bump_version, "CHANNEL_MATRIX", matrix_path):
                bump_version.bump_python_version("0.6.0")
                updated = tmp_path.read_text(encoding="utf-8")
                self.assertIn('version = "0.6.0"', updated)

    def test_bump_typescript_simulation(self):
        sample_package = {
            "name": "engram-client",
            "version": "0.5.0"
        }
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir) / "package.json"
            tmp_path.write_text(json.dumps(sample_package), encoding="utf-8")
            matrix_path = Path(tmpdir) / "matrix.toml"
            matrix_path.write_text('[[channels]]\nid = "npm"\nlocal_version = "0.5.0"\n', encoding="utf-8")

            with patch.object(bump_version, "PACKAGE_JSON", tmp_path), \
                 patch.object(bump_version, "CHANNEL_MATRIX", matrix_path):
                bump_version.bump_typescript_version("0.6.0")
                with tmp_path.open("r", encoding="utf-8") as f:
                    updated = json.load(f)
                self.assertEqual(updated["version"], "0.6.0")


if __name__ == "__main__":
    unittest.main()
