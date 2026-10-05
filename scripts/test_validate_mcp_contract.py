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
    def test_live_repository_contract_validation(self) -> None:
        result = validator.validate_contract()
        self.assertEqual(result["schema_version"], "v1")
        self.assertEqual(result["tool"], "mcp-contract-validator")
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(result["failures"], [])
        self.assertEqual(result["degraded_mode"], "ok")
        self.assertGreaterEqual(result["counts"]["tools"], 300)
        self.assertEqual(result["counts"]["failures"], 0)

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

    def test_negative_fixture_catches_empty_description(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "beta",
        description: "",
        schema: r#"{
            "type": "object",
            "properties": {"id": {"type": "string"}},
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "beta" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `beta`
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
                check["id"] == "mcp_schema:description:beta"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )

    def test_negative_fixture_catches_invalid_tier(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "gamma",
        description: "Gamma tool",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::SuperCustom,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "gamma" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `gamma`
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
                check["id"] == "mcp_schema:tier:gamma"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )

    def test_negative_fixture_catches_duplicate_tool_names(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "delta",
        description: "Delta first",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "delta",
        description: "Delta second",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "delta" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `delta`
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
                check["id"] == "mcp_registry:unique_names"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )

    def test_negative_fixture_catches_missing_dispatch(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "orphaned_tool",
        description: "Tool with no dispatch arm",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "other_tool" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `orphaned_tool`
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
                check["id"] == "mcp_dispatch:tool:orphaned_tool"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )

    def test_negative_fixture_catches_missing_docs(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "undocumented_tool",
        description: "Tool missing docs entry",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "undocumented_tool" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `different_tool`
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
                check["id"] == "mcp_docs:tool:undocumented_tool"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )

    def test_negative_fixture_catches_mutating_tool_with_read_only_hint(self) -> None:
        registry = """
pub const TOOL_DEFINITIONS: &[ToolDef] = &[
    ToolDef {
        name: "memory_create_custom",
        description: "Create memory but wrongly annotated read only",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
"""
        handlers = """
pub fn dispatch(tool_name: &str) {
    match tool_name {
        "memory_create_custom" => {}
        _ => {}
    }
}
"""
        docs = """
# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

### `memory_create_custom`
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
                check["id"] == "mcp_annotation:consistency:memory_create_custom"
                and check["status"] == "fail"
                for check in result["checks"]
            )
        )


if __name__ == "__main__":
    unittest.main()
