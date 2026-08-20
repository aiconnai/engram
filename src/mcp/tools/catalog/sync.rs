//! MCP tool definitions: Cloudflare R2 / S3 Continuous SQLite WAL Delta Replication & PITR.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "replication_status",
        description: "Get SQLite WAL continuous replication status, lag metrics, and frame sequence numbers for R2/S3 cloud backup and PITR.",
        schema: r#"{
            "type": "object",
            "properties": {
                "db_path": {"type": "string", "description": "Optional SQLite database path (defaults to active storage database)"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "replication_sync_now",
        description: "Force an immediate WAL delta extraction and replication flush, generating a compressed and checksummed delta package.",
        schema: r#"{
            "type": "object",
            "properties": {
                "db_path": {"type": "string", "description": "Optional SQLite database path (defaults to active storage database)"},
                "compress": {"type": "boolean", "default": true, "description": "Whether to gzip compress delta frame payloads"},
                "identifier": {"type": "string", "description": "Optional logical database or workspace identifier"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "replication_recover",
        description: "Perform Point-In-Time Recovery (PITR) by replaying SQLite WAL delta frames into a target database.",
        schema: r#"{
            "type": "object",
            "properties": {
                "target_db_path": {"type": "string", "description": "Destination file path for the recovered SQLite database"},
                "source_db_path": {"type": "string", "description": "Source SQLite database path (defaults to active storage database)"},
                "source_wal_path": {"type": "string", "description": "Source .db-wal path (defaults to source_db_path + '-wal')"},
                "target_frame": {"type": "integer", "description": "Target frame sequence number to stop recovery at (inclusive)"},
                "target_time": {"type": "string", "format": "date-time", "description": "Target timestamp (ISO-8601 / RFC3339) to stop recovery at"},
                "commit_boundary_only": {"type": "boolean", "default": true, "description": "Only apply frames up to the last transaction commit boundary"},
                "verify_integrity": {"type": "boolean", "default": true, "description": "Verify database integrity after recovery using PRAGMA integrity_check"}
            },
            "required": ["target_db_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
];
