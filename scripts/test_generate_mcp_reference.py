#!/usr/bin/env python3
"""Tests for the MCP reference generator."""

from __future__ import annotations

import tempfile
import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import generate_mcp_reference as generator


class McpReferenceGeneratorTests(unittest.TestCase):
    def test_parse_tools_handles_field_order_and_annotations(self) -> None:
        source = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "alpha",
        description: "Read alpha",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "beta",
        description: "Delete beta",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        tier: ToolTier::Advanced,
        annotations: ToolAnnotations {
            read_only_hint: None,
            destructive_hint: Some(true),
            idempotent_hint: None,
            open_world_hint: Some(true),
        },
    },
];
"""
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tools.rs"
            path.write_text(source)

            tools = generator.parse_tools(path)

        self.assertEqual([tool.name for tool in tools], ["alpha", "beta"])
        self.assertEqual(tools[0].annotations, "readOnlyHint")
        self.assertEqual(tools[1].annotations, "destructiveHint, openWorldHint")
        self.assertEqual(tools[1].tier, "advanced")

    def test_render_reference_includes_schema_summary(self) -> None:
        tool = generator.Tool(
            name="sample",
            description="Sample tool",
            schema={
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "description": "Mode",
                        "default": "fast",
                        "enum": ["fast", "safe"],
                    }
                },
                "required": ["mode"],
            },
            annotations="readOnlyHint",
            tier="essential",
        )

        markdown = generator.render_reference([tool], generator.DEFAULT_SOURCE)

        self.assertIn("### `sample`", markdown)
        self.assertIn("| `mode` | `string` | yes |", markdown)
        self.assertIn("Allowed: `fast`, `safe`.", markdown)


if __name__ == "__main__":
    unittest.main()
