//! MCP tool definitions: Spatial Navigation ("Method of Loci").
//! Mnemonic spatial abstractions (Palace, Wings, Rooms, Drawers) over Engram workspaces and scopes.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

#[allow(dead_code)]
pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "palace_navigate",
        description: "Navigate the Memory Palace: inspect active wings (top-level domains), rooms (sub-topics), and memory counts for spatial orientation.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {
                    "type": "string",
                    "description": "Target workspace (Palace). Defaults to 'default'."
                },
                "wing": {
                    "type": "string",
                    "description": "Optional wing filter to inspect rooms inside a specific wing."
                }
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "room_search",
        description: "Search memories scoped within a specific spatial room and wing using hybrid retrieval.",
        schema: r#"{
            "type": "object",
            "properties": {
                "wing": {
                    "type": "string",
                    "description": "Target wing (domain/project)."
                },
                "room": {
                    "type": "string",
                    "description": "Optional room (subtopic/component)."
                },
                "query": {
                    "type": "string",
                    "description": "Search query."
                },
                "limit": {
                    "type": "integer",
                    "default": 10,
                    "description": "Maximum number of drawer memories to return."
                },
                "workspace": {
                    "type": "string",
                    "description": "Optional workspace name."
                }
            },
            "required": ["wing", "query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "drawer_open",
        description: "Open a specific memory drawer by ID to read its full verbatim content and metadata.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {
                    "type": "integer",
                    "description": "Drawer (memory) ID to open."
                }
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "palace_visualize",
        description: "Generate or export a topological visualization of the memory palace in interactive HTML, ASCII floorplan, vector SVG, Mermaid mindmap, or Cytoscape JSON.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {
                    "type": "string",
                    "description": "Target workspace (Palace). Defaults to 'default'."
                },
                "wing": {
                    "type": "string",
                    "description": "Optional wing filter to isolate a specific palace wing."
                },
                "format": {
                    "type": "string",
                    "enum": ["html", "ascii", "svg", "mermaid", "json"],
                    "default": "html",
                    "description": "Visualization export format ('html', 'ascii', 'svg', 'mermaid', 'json'). Defaults to 'html'."
                },
                "output_path": {
                    "type": "string",
                    "description": "Optional local file path to save the rendered output."
                }
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
];
