//! MCP tool definitions: AAAK Ultra-Dense Compression.
//! Mnemonic abbreviation and 20x-30x token context reduction.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

#[allow(dead_code)]
pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "memory_compress_aaak",
        description: "Compress text or a memory drawer into AAAK (Agent Abbreviation Knowledge) ultra-dense format, achieving up to 20x-30x token savings for LLM prompts.",
        schema: r#"{
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Raw text or dialogue transcript to compress."
                },
                "memory_id": {
                    "type": "integer",
                    "description": "Optional memory ID to load content directly from storage."
                },
                "mode": {
                    "type": "string",
                    "enum": ["lossless", "ultradense", "transcript"],
                    "default": "ultradense",
                    "description": "Compression mode: 'lossless' (reversible table), 'ultradense' (max token reduction), or 'transcript' (dialogue turn compaction)."
                }
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_decompress_aaak",
        description: "Decompress an AAAK-encoded text back into standard natural language prose.",
        schema: r#"{
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "AAAK-encoded string (with [AAAK:v1] prefix or raw shorthands) to expand."
                }
            },
            "required": ["text"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
