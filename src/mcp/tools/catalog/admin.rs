//! MCP tool definitions: Administration & Utilities.
//! Workspaces, identities, maintenance, sync, sharing, markdown blocks, and consolidation.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
            name: "memory_stats",
            description: "Get storage statistics",
            schema: r#"{"type": "object", "properties": {}}"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "workspace_list",
            description: "List all workspaces with their statistics (memory count, tier breakdown, etc.)",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Essential,
        },
    ToolDef {
            name: "workspace_stats",
            description: "Get detailed statistics for a specific workspace",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace name"}
                },
                "required": ["workspace"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "workspace_move",
            description: "Move a memory to a different workspace",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID to move"},
                    "workspace": {"type": "string", "description": "Target workspace name"}
                },
                "required": ["id", "workspace"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "workspace_delete",
            description: "Delete a workspace. Can either move all memories to 'default' workspace or hard delete them.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace to delete"},
                    "move_to_default": {"type": "boolean", "default": true, "description": "If true, moves memories to 'default' workspace. If false, deletes all memories in the workspace."}
                },
                "required": ["workspace"]
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "embedding_cache_stats",
            description: "Get statistics about the embedding cache (hits, misses, entries, bytes used, hit rate)",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "embedding_cache_clear",
            description: "Clear all entries from the embedding cache",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "identity_create",
            description: "Create a new identity with canonical ID, display name, and optional aliases",
            schema: r#"{
                "type": "object",
                "properties": {
                    "canonical_id": {"type": "string", "description": "Unique canonical identifier (e.g., 'user:ronaldo', 'org:acme')"},
                    "display_name": {"type": "string", "description": "Human-readable display name"},
                    "entity_type": {"type": "string", "enum": ["person", "organization", "project", "tool", "concept", "other"], "default": "person"},
                    "description": {"type": "string", "description": "Optional description"},
                    "aliases": {"type": "array", "items": {"type": "string"}, "description": "Initial aliases for this identity"},
                    "metadata": {"type": "object", "description": "Additional metadata"}
                },
                "required": ["canonical_id", "display_name"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_get",
            description: "Get an identity by its canonical ID",
            schema: r#"{
                "type": "object",
                "properties": {
                    "canonical_id": {"type": "string", "description": "Canonical identifier"}
                },
                "required": ["canonical_id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_update",
            description: "Update an identity's display name, description, or type",
            schema: r#"{
                "type": "object",
                "properties": {
                    "canonical_id": {"type": "string", "description": "Canonical identifier"},
                    "display_name": {"type": "string", "description": "New display name"},
                    "description": {"type": "string", "description": "New description"},
                    "entity_type": {"type": "string", "enum": ["person", "organization", "project", "tool", "concept", "other"]}
                },
                "required": ["canonical_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_delete",
            description: "Delete an identity and all its aliases",
            schema: r#"{
                "type": "object",
                "properties": {
                    "canonical_id": {"type": "string", "description": "Canonical identifier to delete"}
                },
                "required": ["canonical_id"]
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_add_alias",
            description: "Add an alias to an identity. Aliases are normalized (lowercase, trimmed). Conflicts with existing aliases are rejected.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "canonical_id": {"type": "string", "description": "Canonical identifier"},
                    "alias": {"type": "string", "description": "Alias to add"},
                    "source": {"type": "string", "description": "Optional source of the alias (e.g., 'manual', 'extracted')"}
                },
                "required": ["canonical_id", "alias"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_remove_alias",
            description: "Remove an alias from any identity",
            schema: r#"{
                "type": "object",
                "properties": {
                    "alias": {"type": "string", "description": "Alias to remove"}
                },
                "required": ["alias"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "identity_resolve",
            description: "Resolve an alias to its canonical identity. Returns the identity if found, null otherwise.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "alias": {"type": "string", "description": "Alias to resolve"}
                },
                "required": ["alias"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_list",
            description: "List all identities with optional type filter",
            schema: r#"{
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["person", "organization", "project", "tool", "concept", "other"]},
                    "limit": {"type": "integer", "default": 50}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_search",
            description: "Search identities by alias or display name",
            schema: r#"{
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"},
                    "limit": {"type": "integer", "default": 20}
                },
                "required": ["query"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_link",
            description: "Link an identity to a memory (mark that the identity is mentioned in the memory)",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memory_id": {"type": "integer", "description": "Memory ID"},
                    "canonical_id": {"type": "string", "description": "Identity canonical ID"},
                    "mention_text": {"type": "string", "description": "The text that mentions this identity"}
                },
                "required": ["memory_id", "canonical_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "identity_unlink",
            description: "Remove the link between an identity and a memory",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memory_id": {"type": "integer", "description": "Memory ID"},
                    "canonical_id": {"type": "string", "description": "Identity canonical ID"}
                },
                "required": ["memory_id", "canonical_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_get_identities",
            description: "Get all identities (persons, organizations, projects, etc.) linked to a memory. Returns identity details including display name, type, aliases, and mention information.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_tags",
            description: "List all tags with usage counts and most recent usage timestamps.",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_tag_hierarchy",
            description: "Get tags organized in a hierarchical tree structure. Tags with slashes are treated as paths (e.g., 'project/engram/core').",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_validate_tags",
            description: "Validate tag consistency across memories. Reports orphaned tags, unused tags, and suggested normalizations.",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_rebuild_embeddings",
            description: "Rebuild embeddings for all memories that are missing them. Useful after model changes or data recovery.",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::idempotent(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_rebuild_crossrefs",
            description: "Rebuild cross-reference links between memories. Re-analyzes all memories to find and create links.",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::idempotent(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_archive_old",
            description: "Compress already-Archived memories by creating summary rows. Does not move originals to archived state; use lifecycle_run for lifecycle transitions.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "max_age_days": {"type": "integer", "default": 90, "description": "Compress archived memories older than this many days"},
                    "max_importance": {"type": "number", "default": 0.5, "description": "Only compress already-Archived memories with importance below this"},
                    "min_access_count": {"type": "integer", "default": 5, "description": "Skip already-Archived memories accessed more than this many times"},
                    "workspace": {"type": "string", "description": "Limit to specific workspace"},
                    "dry_run": {"type": "boolean", "default": true, "description": "If true, only report what would be compressed"}
                }
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_from_trace",
            description: "Create a memory from a specific Langfuse trace ID.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "trace_id": {"type": "string", "description": "Langfuse trace ID"},
                    "memory_type": {"type": "string", "enum": ["note", "episodic", "procedural", "learning"], "default": "episodic", "description": "Type of memory to create"},
                    "workspace": {"type": "string", "description": "Workspace for the memory"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Additional tags"}
                },
                "required": ["trace_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_export_markdown",
            description: "Export a workspace as human-readable Markdown files with YAML frontmatter and wiki-style [[links]]. Creates one .md file per memory, organized by type in subdirectories, with an index.md overview.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace to export"},
                    "output_dir": {"type": "string", "description": "Output directory path (default: ./engram-export/{workspace}/)"},
                    "group": {"type": "string", "enum": ["flat", "day", "workspace", "type", "entity"], "default": "flat", "description": "Grouping strategy for exported files ('flat', 'day', 'workspace', 'type', 'entity')"},
                    "include_links": {"type": "boolean", "default": true, "description": "Include [[wiki links]] to related memories in each file"}
                },
                "required": ["workspace"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_import_markdown",
            description: "Import memories from Markdown files with engram_ frontmatter (RFC 0004). Review mode by default (confirm: false) — returns a staged list without writing. Detects drift via content_hash and version conflicts via engram_version. Ignores non-engram_ frontmatter keys (Obsidian-safe).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "input_dir": {"type": "string", "description": "Directory to scan recursively for .md files"},
                    "workspace": {"type": "string", "description": "Override workspace (default: from each file's engram_workspace)"},
                    "confirm": {"type": "boolean", "default": false, "description": "Apply writes. When false (default), dry-run review only"},
                    "force_version": {"type": "boolean", "default": false, "description": "Bypass version conflict checks"}
                },
                "required": ["input_dir"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_block_create",
            description: "Create a named, token-bounded memory block (Letta/MemGPT-style self-editing context slot).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Unique name for the memory block"},
                    "content": {"type": "string", "description": "Initial content of the block (default: empty string)"},
                    "max_tokens": {"type": "integer", "description": "Maximum token capacity for the block (default: 4096)"}
                },
                "required": ["name"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_block_get",
            description: "Retrieve a memory block by name.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Name of the memory block to retrieve"}
                },
                "required": ["name"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_block_edit",
            description: "Update the content of an existing memory block, incrementing its version and recording the reason.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Name of the memory block to edit"},
                    "content": {"type": "string", "description": "New content for the block"},
                    "reason": {"type": "string", "description": "Human-readable reason for this edit (optional)"}
                },
                "required": ["name", "content"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_block_list",
            description: "List all memory blocks with their names, versions, and token usage.",
            schema: r#"{
                "type": "object",
                "properties": {},
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_block_archive",
            description: "Permanently delete a memory block and return its final content before deletion. Destructive and irreversible.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Name of the memory block to archive and delete"}
                },
                "required": ["name"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_block_history",
            description: "Return the edit history for a named memory block.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Name of the memory block"},
                    "limit": {"type": "integer", "description": "Maximum number of history entries to return (default: 20)"}
                },
                "required": ["name"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_cache_stats",
            description: "Return hit/miss statistics and entry count for the in-memory semantic search cache.",
            schema: r#"{
                "type": "object",
                "properties": {},
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_cache_clear",
            description: "Evict all entries from the semantic search cache. Mutates in-memory cache state.",
            schema: r#"{
                "type": "object",
                "properties": {},
                "required": []
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_compress_for_context",
            description: "Pack a set of memories into a token budget for LLM context, returning compressed entries and diagnostics about skipped memories.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "ids": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Memory IDs to compress and pack (alias: memory_ids)."
                    },
                    "memory_ids": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Alias for ids."
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for the packed context (default: 4096)."
                    }
                },
                "required": ["ids"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_embedding_migrate",
            description: "Re-embed all memories using the active embedding model; use dry_run to count affected memories without writing.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "dry_run": {
                        "type": "boolean",
                        "description": "If true, count memories to migrate without re-embedding them (default: false)."
                    },
                    "target_model": {
                        "type": "string",
                        "description": "Target embedding model name to record in embedding_model column. Defaults to the active embedder's model name."
                    }
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_embedding_providers",
            description: "List the active embedding provider including model name and vector dimensions.",
            schema: r#"{
                "type": "object",
                "properties": {},
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_enrichment_timeline",
            description: "List all enrichment events for a specific memory (lifecycle transitions, consolidation, compression, etc.). Shows what automated operations affected this memory and why.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memory_id": {
                        "type": "integer",
                        "description": "ID of the memory whose enrichment history to retrieve."
                    },
                    "event_type": {
                        "type": "string",
                        "description": "Filter to a specific event type (e.g. \"consolidation\", \"lifecycle_transition\")."
                    },
                    "include_dry_runs": {
                        "type": "boolean",
                        "description": "Include events that were executed in dry-run mode (default: true)."
                    },
                    "include_snapshots": {
                        "type": "boolean",
                        "description": "Include snapshot events (default: true)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of events to return (default: 20, max: 100)."
                    }
                },
                "required": ["memory_id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_enrichment_audit",
            description: "Query enrichment events globally with filters (status, event_type, agent_id, operation_id, workspace, time range). Use for compliance audit and batch tracing.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "description": "Filter by event type (e.g. \"consolidation\", \"lifecycle_transition\", \"compression\")."
                    },
                    "triggered_by": {
                        "type": "string",
                        "description": "Filter by the tool name that triggered the event."
                    },
                    "agent_id": {
                        "type": "string",
                        "description": "Filter by the agent ID that triggered the event."
                    },
                    "status": {
                        "type": "string",
                        "description": "Filter by event outcome status.",
                        "enum": ["completed", "failed", "skipped"]
                    },
                    "workspace": {
                        "type": "string",
                        "description": "Filter to a specific workspace."
                    },
                    "operation_id": {
                        "type": "string",
                        "description": "Filter by a specific operation ID (exact match)."
                    },
                    "memory_id": {
                        "type": "integer",
                        "description": "Filter to events that reference a specific memory."
                    },
                    "version_id": {
                        "type": "integer",
                        "description": "Filter to events that reference a specific memory version."
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "Filter by dry-run flag (true = only dry-run events, false = only real events)."
                    },
                    "since": {
                        "type": "string",
                        "description": "ISO-8601 timestamp: return events created at or after this time."
                    },
                    "until": {
                        "type": "string",
                        "description": "ISO-8601 timestamp: return events created at or before this time."
                    },
                    "order": {
                        "type": "string",
                        "description": "Sort order by creation time: \"desc\" (newest first, default) or \"asc\".",
                        "enum": ["desc", "asc"]
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of events to return (default: 50, max: 200)."
                    }
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_sentiment_analyze",
            description: "Analyze the sentiment of a single memory's content, returning a score, label (positive/neutral/negative), confidence, and keyword signals.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "ID of the memory to analyze (required)."}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_sentiment_timeline",
            description: "Compute a chronological sentiment timeline over memories in a workspace within an optional time range.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace to scan (default: \"default\")."},
                    "from": {"type": "string", "description": "ISO-8601 start timestamp (default: epoch)."},
                    "to": {"type": "string", "description": "ISO-8601 end timestamp (default: far future)."},
                    "limit": {"type": "integer", "description": "Maximum number of timeline entries to return (default: 50)."}
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_sync_status",
            description: "Get cloud sync status",
            schema: r#"{"type": "object", "properties": {}}"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_sync_media",
            description: "Sync local media assets (images, audio, video) to cloud S3/R2 storage. Uploads files from media_assets table that have not yet been synced. Returns a report of synced files. Requires both multimodal and cloud features.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "dry_run": {"type": "boolean", "default": false, "description": "If true, report what would be synced without actually uploading"}
                }
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_events_poll",
            description: "Poll for memory events (create, update, delete, etc.) since a given point. Useful for syncing and monitoring.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "since_id": {"type": "integer", "description": "Return events after this event ID"},
                    "since_time": {"type": "string", "format": "date-time", "description": "Return events after this timestamp (RFC3339)"},
                    "agent_id": {"type": "string", "description": "Filter events for specific agent"},
                    "limit": {"type": "integer", "default": 100, "description": "Maximum events to return"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_events_clear",
            description: "Clear old events from the event log. Helps manage storage for long-running systems.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "before_id": {"type": "integer", "description": "Delete events before this ID"},
                    "before_time": {"type": "string", "format": "date-time", "description": "Delete events before this timestamp"},
                    "keep_recent": {"type": "integer", "description": "Keep only the N most recent events"}
                }
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "sync_version",
            description: "Get the current sync version and metadata. Used to check if local data is up-to-date.",
            schema: r#"{
                "type": "object",
                "properties": {}
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "sync_delta",
            description: "Get changes (delta) since a specific version. Returns created, updated, and deleted memories.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "since_version": {"type": "integer", "description": "Version to get changes from"}
                },
                "required": ["since_version"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "sync_state",
            description: "Get or update sync state for a specific agent. Tracks what each agent has synced.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Agent identifier"},
                    "update_version": {"type": "integer", "description": "If provided, updates the agent's last synced version"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "sync_cleanup",
            description: "Clean up old sync data (events, etc.) older than specified days.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "older_than_days": {"type": "integer", "default": 30, "description": "Delete sync data older than this many days"}
                }
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_share",
            description: "Share a memory with another agent. The target agent can poll for shared memories.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memory_id": {"type": "integer", "description": "ID of memory to share"},
                    "from_agent": {"type": "string", "description": "Sender agent identifier"},
                    "to_agent": {"type": "string", "description": "Recipient agent identifier"},
                    "message": {"type": "string", "description": "Optional message to include with share"}
                },
                "required": ["memory_id", "from_agent", "to_agent"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_shared_poll",
            description: "Poll for memories shared with this agent.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Agent identifier to check shares for"},
                    "include_acknowledged": {"type": "boolean", "default": false, "description": "Include already acknowledged shares"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_share_ack",
            description: "Acknowledge receipt of a shared memory.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "share_id": {"type": "integer", "description": "Share ID to acknowledge"},
                    "agent_id": {"type": "string", "description": "Agent acknowledging the share"}
                },
                "required": ["share_id", "agent_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_grant_access",
            description: "Grant an agent access to a scope path. Supports read, write, and admin permissions. Access also applies to all descendant scopes.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Agent ID to grant access to"},
                    "scope_path": {"type": "string", "description": "Scope path to grant access to (e.g. 'global/org:acme')"},
                    "permissions": {"type": "string", "enum": ["read", "write", "admin"], "default": "read", "description": "Permission level"},
                    "granted_by": {"type": "string", "description": "Optional: ID of the granting agent"}
                },
                "required": ["agent_id", "scope_path"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_revoke_access",
            description: "Revoke an agent's access to a specific scope path.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Agent ID to revoke access from"},
                    "scope_path": {"type": "string", "description": "Scope path to revoke access from"}
                },
                "required": ["agent_id", "scope_path"]
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_list_grants",
            description: "List all scope access grants for a given agent.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Agent ID to list grants for"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_check_access",
            description: "Check whether an agent has a required permission level on a scope path (including ancestor grants).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Agent ID to check"},
                    "scope_path": {"type": "string", "description": "Scope path to check access for"},
                    "permissions": {"type": "string", "enum": ["read", "write", "admin"], "default": "read", "description": "Required permission level"}
                },
                "required": ["agent_id", "scope_path"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_suggest_tags",
            description: "Suggest tags for a memory based on AI content analysis. Uses pattern matching, keyword extraction, and structure detection to suggest relevant tags with confidence scores.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID to analyze (alternative to content)"},
                    "memory_id": {"type": "integer", "description": "Memory ID to analyze (alias for id)"},
                    "content": {"type": "string", "description": "Content to analyze (alternative to id/memory_id)"},
                    "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential"], "description": "Memory type (used when providing content directly)"},
                    "existing_tags": {"type": "array", "items": {"type": "string"}, "description": "Tags already on the memory (excluded from suggestions)"},
                    "min_confidence": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.5, "description": "Minimum confidence threshold for suggestions"},
                    "max_tags": {"type": "integer", "default": 5, "description": "Maximum number of tags to suggest"},
                    "enable_patterns": {"type": "boolean", "default": true, "description": "Use pattern-based tagging"},
                    "enable_keywords": {"type": "boolean", "default": true, "description": "Use keyword-based tagging"},
                    "enable_entities": {"type": "boolean", "default": true, "description": "Use entity-based tagging"},
                    "enable_type_tags": {"type": "boolean", "default": true, "description": "Add tags based on memory type"},
                    "keyword_mappings": {"type": "object", "description": "Custom keyword-to-tag mappings (e.g., {\"ibvi\": \"project/ibvi\"})"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_auto_tag",
            description: "Automatically suggest and optionally apply tags to a memory. Analyzes content using AI heuristics and can merge suggested tags with existing ones.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID to auto-tag"},
                    "memory_id": {"type": "integer", "description": "Memory ID (alias for id)"},
                    "apply": {"type": "boolean", "default": false, "description": "If true, apply the suggested tags to the memory. If false, only return suggestions."},
                    "merge": {"type": "boolean", "default": true, "description": "If true and apply=true, merge with existing tags. If false, replace existing tags."},
                    "min_confidence": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.5, "description": "Minimum confidence threshold"},
                    "max_tags": {"type": "integer", "default": 5, "description": "Maximum tags to suggest/apply"},
                    "keyword_mappings": {"type": "object", "description": "Custom keyword-to-tag mappings"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_soft_trim",
            description: "Intelligently trim memory content while preserving context. Keeps the beginning (head) and end (tail) of content with an ellipsis in the middle. Useful for displaying long content in limited space while keeping important context from both ends.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID to trim"},
                    "max_chars": {"type": "integer", "default": 500, "description": "Maximum characters for trimmed output"},
                    "head_percent": {"type": "integer", "default": 60, "description": "Percentage of space for the head (0-100)"},
                    "tail_percent": {"type": "integer", "default": 30, "description": "Percentage of space for the tail (0-100)"},
                    "ellipsis": {"type": "string", "default": "\n...\n", "description": "Text to insert between head and tail"},
                    "preserve_words": {"type": "boolean", "default": true, "description": "Avoid breaking in the middle of words"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_content_stats",
            description: "Get content statistics for a memory (character count, word count, line count, sentence count, paragraph count)",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_export",
            description: "Export all memories to a JSON-serializable format for backup or migration.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Export only memories from this workspace (omit for all)"},
                    "include_embeddings": {"type": "boolean", "default": false, "description": "Reserved feature: currently not supported (reserved for future embedding-inclusive exports)"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_import",
            description: "Import memories from a previously exported JSON format.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "data": {"type": "object", "description": "The exported data object"},
                    "skip_duplicates": {"type": "boolean", "default": true, "description": "Skip memories with matching content hash"}
                },
                "required": ["data"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_summarize",
            description: "Create a summary of one or more memories. Returns a new Summary-type memory with summary_of_id set.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memory_ids": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "IDs of memories to summarize"
                    },
                    "summary": {"type": "string", "description": "The summary text (provide this or let the system generate one)"},
                    "max_length": {"type": "integer", "default": 500, "description": "Maximum length for auto-generated summary"},
                    "workspace": {"type": "string", "description": "Workspace for the summary memory"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for the summary memory"}
                },
                "required": ["memory_ids"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_auto_consolidate",
            description: "Enable, disable, configure, or inspect the automatic consolidation scheduler. Use action='enable'/'disable' to toggle it, 'set_interval' with interval_seconds to change the period (60–86400), or 'get_status' to inspect current settings.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["enable", "disable", "set_interval", "get_status"]
                    },
                    "interval_seconds": {"type": "integer", "minimum": 60, "maximum": 86400}
                },
                "required": ["action"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_consolidate_batch",
            description: "Run one auto-consolidation pass over a workspace: detect duplicates, conflicts, and archive-eligible memories. Defaults to dry-run; returns a structured report of actions taken (or that would be taken).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "default": "default"},
                    "dry_run": {"type": "boolean", "default": true},
                    "policy": {
                        "type": "object",
                        "properties": {
                            "duplicate_threshold": {"type": "number", "default": 0.92},
                            "conflict_auto_resolve": {"type": "boolean", "default": false},
                            "summarize_age_days": {"type": "integer", "default": 90},
                            "max_actions_per_run": {"type": "integer", "default": 50},
                            "dry_run": {"type": "boolean", "default": true},
                            "utility_threshold": {"type": "number", "default": 0.3, "minimum": 0, "maximum": 1},
                            "min_feedback_events": {"type": "integer", "default": 3, "minimum": 0},
                            "max_access_count_for_archival": {"type": "integer", "default": 10, "minimum": 0},
                            "utility_weight": {"type": "number", "default": 0.5, "minimum": 0, "maximum": 1},
                            "age_weight": {"type": "number", "default": 0.3, "minimum": 0, "maximum": 1},
                            "feedback_weight": {"type": "number", "default": 0.2, "minimum": 0, "maximum": 1},
                            "composite_cutoff": {"type": "number", "default": 0.5, "minimum": 0, "maximum": 1},
                            "max_importance_for_archival": {"type": "number", "default": 0.5, "minimum": 0, "maximum": 1}
                        }
                    }
                }
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_consolidation_history",
            description: "List recent auto-consolidation runs for a workspace (or all workspaces). Newest-first.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string"},
                    "limit": {"type": "integer", "default": 20, "minimum": 1, "maximum": 1000}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_compress",
            description: "Apply rule-based semantic compression to a single memory and return the structured result with key entities and facts.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "ID of the memory to compress."
                    },
                    "target_ratio": {
                        "type": "number",
                        "description": "Target compression ratio as a fraction of original tokens (default: 0.1)."
                    }
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_consolidate",
            description: "Run offline consolidation over a workspace, merging and archiving similar memories; use dry_run to preview without writing.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {
                        "type": "string",
                        "description": "Workspace to consolidate (default: \"default\")."
                    },
                    "strategy": {
                        "type": "string",
                        "description": "Grouping strategy: \"content_overlap\" (default), \"tag_similarity\", or \"temporal_proximity\"."
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "If true, report what would be merged/archived without writing changes (default: false)."
                    }
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_decompress",
            description: "Retrieve the original (uncompressed) content of a memory by ID.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "ID of the memory whose content to retrieve."
                    }
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_synthesis",
            description: "Check semantic overlap between two content strings and produce a merged synthesis using the chosen strategy.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content_a": {
                        "type": "string",
                        "description": "First content string to synthesise."
                    },
                    "content_b": {
                        "type": "string",
                        "description": "Second content string to synthesise."
                    },
                    "id_a": {
                        "type": "integer",
                        "description": "Optional memory ID associated with content_a (default: 0)."
                    },
                    "strategy": {
                        "type": "string",
                        "description": "Synthesis strategy: \"merge\" (default), \"replace\", or \"append\".",
                        "enum": ["merge", "replace", "append"]
                    }
                },
                "required": ["content_a", "content_b"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_replay_at_time",
            description: "Replay one memory as it existed at a given RFC3339 timestamp and optionally return enrichment events affecting it up to that time.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memory_id": {
                        "type": "integer",
                        "description": "ID of the memory to replay."
                    },
                    "timestamp": {
                        "type": "string",
                        "description": "RFC3339 timestamp to replay state at."
                    },
                    "event_type": {
                        "type": "string",
                        "description": "Optional event type filter for replayed event list (e.g. \"consolidation\")."
                    },
                    "include_events": {
                        "type": "boolean",
                        "default": true,
                        "description": "Whether to include enrichment events in the response."
                    },
                    "include_failed": {
                        "type": "boolean",
                        "default": false,
                        "description": "Whether to include failed enrichment events."
                    },
                    "include_dry_runs": {
                        "type": "boolean",
                        "default": false,
                        "description": "Whether to include dry-run events."
                    },
                    "event_limit": {
                        "type": "integer",
                        "description": "Max number of events to include in replay trail (default 50, max 200)."
                    }
                },
                "required": ["memory_id", "timestamp"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
];
