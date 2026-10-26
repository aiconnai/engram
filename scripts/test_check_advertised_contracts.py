#!/usr/bin/env python3
"""Tests for the advertised-contracts checker and its --write regeneration mode."""

from __future__ import annotations

import importlib.util
import shutil
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
ROOT = SCRIPTS.parent
sys.path.insert(0, str(SCRIPTS))

_spec = importlib.util.spec_from_file_location(
    "check_advertised_contracts", SCRIPTS / "check-advertised-contracts.py"
)
checker = importlib.util.module_from_spec(_spec)
sys.modules[_spec.name] = checker  # dataclasses/annotations resolve via sys.modules
_spec.loader.exec_module(checker)

INVENTORY_PATH = ROOT / "docs/contracts/advertised-surfaces.toml"

SAMPLE = """\
# header comment
schema_version = "advertised-surfaces-v1"
base_sha = "old"
captured_date = "2020-01-01"

[mcp]
total_tool_count = 1
tools = ["a"]

[cli]
binaries = ["x"]
pdf_worker_distribution = "sibling-binary"

[cargo]
# keep this comment
features = ["old"]
"""


class PythonSdkSurface(unittest.TestCase):
    def test_methods_include_those_inherited_from_resource_mixins(self) -> None:
        methods = set(checker.python_methods())

        # `create`, `search` and `memory_council` live in resources/*.py mixins,
        # not in the EngramClient class body.
        self.assertTrue({"create", "search", "memory_council", "close"} <= methods, methods)
        self.assertGreater(len(methods), 50)


class InventoryRendering(unittest.TestCase):
    def render(self, current: dict) -> dict:
        text = checker.render_inventory(SAMPLE, current, "abc123", "2026-10-26")
        return text, tomllib.loads(text)

    def test_tracked_keys_are_updated_and_new_keys_appended(self) -> None:
        current = {
            "mcp": {"total_tool_count": 2, "tools": ["a", "b"], "added": ["n"]},
            "cli": {"binaries": ["x", "y"]},
            "cargo": {"features": ["default"]},
        }

        _, data = self.render(current)

        self.assertEqual(data["mcp"]["total_tool_count"], 2)
        self.assertEqual(data["mcp"]["tools"], ["a", "b"])
        self.assertEqual(data["mcp"]["added"], ["n"])
        self.assertEqual(data["cli"]["binaries"], ["x", "y"])
        self.assertEqual(data["cargo"]["features"], ["default"])
        self.assertEqual(data["base_sha"], "abc123")
        self.assertEqual(data["captured_date"], "2026-10-26")

    def test_untracked_keys_and_comments_survive(self) -> None:
        text, data = self.render({"cli": {"binaries": ["z"]}})

        self.assertEqual(data["cli"]["pdf_worker_distribution"], "sibling-binary")
        self.assertIn("# header comment", text)
        self.assertIn("# keep this comment", text)
        self.assertEqual(data["cargo"]["features"], ["old"])

    def test_new_sections_are_appended(self) -> None:
        _, data = self.render({"extra": {"key": 3}})

        self.assertEqual(data["extra"], {"key": 3})

    def test_strings_with_quotes_round_trip(self) -> None:
        example = 'engram-cli create "Architecture decision"'
        _, data = self.render({"cli": {"binaries": [example, "back\\slash"]}})

        self.assertEqual(data["cli"]["binaries"], [example, "back\\slash"])


class CheckedInInventory(unittest.TestCase):
    def test_write_regenerates_an_inventory_that_passes_the_check(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            copy = Path(directory) / "advertised-surfaces.toml"
            shutil.copyfile(INVENTORY_PATH, copy)

            current = checker.current_inventory()
            copy.write_text(
                checker.render_inventory(copy.read_text(encoding="utf-8"), current, "deadbeef", "2026-10-26"),
                encoding="utf-8",
            )

            failures = checker.compare_inventory(checker.load_inventory(copy), current)
            data = tomllib.loads(copy.read_text(encoding="utf-8"))

        self.assertEqual(failures, [])
        # keys the checker does not generate are preserved for check-pdf-worker-packaging.py
        self.assertEqual(data["cli"]["pdf_worker_distribution"], "sibling-binary")

    def test_committed_inventory_matches_the_tree(self) -> None:
        failures = checker.compare_inventory(
            checker.load_inventory(INVENTORY_PATH), checker.current_inventory()
        )

        self.assertEqual(failures, [], "run: python3 scripts/check-advertised-contracts.py "
                         "--inventory docs/contracts/advertised-surfaces.toml --write")


if __name__ == "__main__":
    unittest.main()
