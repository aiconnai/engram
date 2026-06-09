#!/usr/bin/env python3
"""Tests for the MCP contract validator."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import validate_mcp_contract as validator


class McpContractValidatorTests(unittest.TestCase):
    def test_negative_fixture_catches_required_field_missing_from_properties(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "alpha",
        description: "Read alpha",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "alpha" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `alpha`
"""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            registry_path = root / "registry.rs"
            handlers_path = root / "handlers.rs"
            docs_path = root / "MCP_TOOLS.md"
            registry_path.write_text(registry)
            handlers_path.write_text(handlers)
            docs_path.write_text(docs)

            result = validator.validate_contract(registry_path, handlers_path, docs_path)

        self.assertEqual(result["status"], "fail")
        self.assertEqual(result["exit_code"], 1)
        self.assertTrue(
            any(
                check["id"] == "mcp_schema:required_properties:alpha"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )


if __name__ == "__main__":
    unittest.main()
