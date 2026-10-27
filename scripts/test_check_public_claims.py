#!/usr/bin/env python3
"""Tests for the public-claims checker."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS))

_spec = importlib.util.spec_from_file_location(
    "check_public_claims", SCRIPTS / "check-public-claims.py"
)
claims = importlib.util.module_from_spec(_spec)
sys.modules[_spec.name] = claims  # dataclasses resolve annotations via sys.modules
_spec.loader.exec_module(claims)

COUNTS = claims.CatalogCounts(total=301, default=254, essential=17)


def tool(name: str, tier: str) -> claims.mcp_reference.Tool:
    return claims.mcp_reference.Tool(
        name=name,
        description="d",
        schema={},
        annotations="readOnlyHint",
        tier=tier,
        group="core",
        required_features=(),
    )


class ToolCountClaims(unittest.TestCase):
    def test_catalog_numbers_pass(self) -> None:
        text = (
            "301 MCP tools (254 in the default build), only the 17 essential tools; "
            "[254 tools registered] MCP%20Server-301%20Tools Catalog (301 Tools) "
            'stat-number">301</div><div class="stat-label">Active MCP Tools'
        )
        self.assertEqual(claims.check_text("x.md", text, COUNTS), [])

    def test_stale_counts_fail_in_every_phrasing(self) -> None:
        for text in (
            "243 tools",
            "243+ MCP tools",
            "MCP Tools (243)",
            "[243 tools registered]",
            "MCP%20Server-243%20Tools-emerald",
            "Browse MCP Catalog (243 Tools)",
            '<div class="stat-number">243+</div>\n<div class="stat-label">Active MCP Tools</div>',
            "243 in the default build",
            "only the 99 essential tools",
        ):
            with self.subTest(text=text):
                failures = claims.check_text("x.md", text, COUNTS)
                self.assertEqual(len(failures), 1, failures)

    def test_competitor_approximations_are_not_ours_to_verify(self) -> None:
        self.assertEqual(claims.check_text("x.md", "~10 tools | ~6 tools", COUNTS), [])

    def test_banned_phrases_fail_case_insensitively(self) -> None:
        failures = claims.check_text("x.md", "Sub-Millisecond retrieval", COUNTS)
        self.assertEqual(len(failures), 1)
        failures = claims.check_text("x.md", "Native adapters cover X", COUNTS)
        self.assertEqual(len(failures), 1)


class FeaturedToolsPage(unittest.TestCase):
    PAGE = (
        '<div class="tool-card" data-tier="{tier}" data-group="g"><div><div class="tool-header">'
        '<span class="tool-name">{name}</span>'
    )

    def test_unknown_tool_and_wrong_tier_fail(self) -> None:
        tools = [tool("memory_create", "essential")]
        ok = self.PAGE.format(tier="essential", name="memory_create")
        wrong_tier = self.PAGE.format(tier="standard", name="memory_create")
        unknown = self.PAGE.format(tier="standard", name="memory_made_up")

        self.assertEqual(claims.check_featured_tools("p", ok, tools), [])
        self.assertEqual(len(claims.check_featured_tools("p", wrong_tier, tools)), 1)
        self.assertEqual(len(claims.check_featured_tools("p", unknown, tools)), 1)


class CatalogDerivedCounts(unittest.TestCase):
    def test_default_feature_closure_follows_feature_edges(self) -> None:
        cargo = {
            "features": {
                "default": ["a"],
                "a": ["b", "dep:x", "y/z"],
                "b": [],
                "c": [],
            }
        }
        self.assertEqual(claims.default_feature_closure(cargo), {"a", "b"})

    def test_gated_tools_are_excluded_from_default_count(self) -> None:
        tools = [tool("a", "essential"), tool("discover_tools", "standard"), tool("b", "advanced")]
        gated = claims.mcp_reference.Tool(
            name="g",
            description="d",
            schema={},
            annotations="x",
            tier="advanced",
            group="feature.dream",
            required_features=("dream-phase",),
        )
        counts = claims.catalog_counts([*tools, gated], {"openai"})
        self.assertEqual((counts.total, counts.default, counts.essential), (4, 3, 2))


class RepositoryState(unittest.TestCase):
    def test_public_docs_match_the_catalog(self) -> None:
        self.assertEqual(claims.check_repo(), [])


if __name__ == "__main__":
    unittest.main()
