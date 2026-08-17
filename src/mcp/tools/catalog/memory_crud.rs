//! MCP tool definitions: Memory CRUD.
//! Core CRUD, creation convenience tools, fact ingestion, and basic memory operations.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
            name: "memory_create",
            description: "Store an explicit durable memory with inspectable provenance. Use for stable preferences, decisions, insights, and project context when the fact is intentional and worth preserving.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content": {"type": "string", "description": "The content to remember"},
                    "memory_type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "default": "note", "description": "Memory type (preferred field; alias: type)"},
                    "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "default": "note", "description": "Deprecated alias for memory_type"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for categorization"},
                    "metadata": {"type": "object", "description": "Additional metadata as key-value pairs"},
                    "importance": {"type": "number", "minimum": 0, "maximum": 1, "description": "Importance score (0-1)"},
                    "workspace": {"type": "string", "description": "Workspace to store the memory in (default: 'default')"},
                    "tier": {"type": "string", "enum": ["permanent", "daily"], "default": "permanent", "description": "Memory tier: permanent (never expires) or daily (auto-expires)"},
                    "defer_embedding": {"type": "boolean", "default": false, "description": "Defer embedding to background queue"},
                    "ttl_seconds": {"type": "integer", "description": "Time-to-live in seconds. Memory will auto-expire after this duration. Omit for permanent storage. Setting this implies tier='daily'."},
                    "dedup_mode": {"type": "string", "enum": ["reject", "merge", "skip", "allow"], "default": "allow", "description": "How to handle duplicate content: reject (error if exact match), merge (combine tags/metadata with existing), skip (return existing unchanged), allow (create duplicate)"},
                    "dedup_threshold": {"type": "number", "minimum": 0, "maximum": 1, "description": "Similarity threshold for semantic deduplication (0.0-1.0). When set with dedup_mode != 'allow', memories with cosine similarity >= threshold are treated as duplicates. Requires embeddings. If not set, only exact content hash matching is used."},
                    "event_time": {"type": "string", "format": "date-time", "description": "ISO8601 timestamp for episodic memories (when the event occurred)"},
                    "event_duration_seconds": {"type": "integer", "description": "Duration of the event in seconds (for episodic memories)"},
                    "trigger_pattern": {"type": "string", "description": "Pattern that triggers this procedure (for procedural memories)"},
                    "summary_of_id": {"type": "integer", "description": "ID of the memory this summarizes (for summary memories)"},
                    "media_url": {"type": "string", "description": "URL or local path to the primary media asset (for Image/Audio/Video memory types). Format: local:///path, https://..., or s3://..."}
                },
                "required": ["content"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Essential,
        },
    ToolDef {
            name: "context_seed",
            description: "Injects initial context (premises, persona assumptions, or structured facts) about an entity to avoid cold start. Seeded memories are tagged as origin:seed and status:unverified, and should be treated as revisable assumptions.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "entity_context": {
                        "type": "string",
                        "maxLength": 200,
                        "description": "Name or ID of the entity (e.g., 'Client: Roberto', 'Account: ACME', 'Project: Alpha')"
                    },
                    "workspace": {"type": "string", "description": "Workspace to store the memories in (default: 'default')"},
                    "base_tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags applied to all facts (e.g., ['vip', 'prospect'])"
                    },
                    "ttl_seconds": {
                        "type": "integer",
                        "description": "Override TTL for all facts in seconds (0 = disable TTL). If omitted, TTL is derived from confidence."
                    },
                    "disable_ttl": {
                        "type": "boolean",
                        "default": false,
                        "description": "Disable TTL and keep seeded memories permanent regardless of confidence."
                    },
                    "facts": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": {"type": "string", "minLength": 1},
                                "category": {
                                    "type": "string",
                                    "enum": ["fact", "behavior_instruction", "interest", "persona", "preference"],
                                    "description": "Structured category for filtering and ranking"
                                },
                                "confidence": {
                                    "type": "number",
                                    "minimum": 0.0,
                                    "maximum": 1.0,
                                    "description": "0.0 to 1.0 (defaults to 0.7 for seeds). TTL derived by confidence if ttl_seconds not provided."
                                }
                            },
                            "required": ["content"]
                        }
                    }
                },
                "required": ["facts"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Essential,
        },
    ToolDef {
            name: "memory_seed",
            description: "Deprecated alias for context_seed. Use context_seed instead.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "entity_context": {
                        "type": "string",
                        "maxLength": 200,
                        "description": "Name or ID of the entity (e.g., 'Client: Roberto', 'Account: ACME', 'Project: Alpha')"
                    },
                    "workspace": {"type": "string", "description": "Workspace to store the memories in (default: 'default')"},
                    "base_tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags applied to all facts (e.g., ['vip', 'prospect'])"
                    },
                    "ttl_seconds": {
                        "type": "integer",
                        "description": "Override TTL for all facts in seconds (0 = disable TTL). If omitted, TTL is derived from confidence."
                    },
                    "disable_ttl": {
                        "type": "boolean",
                        "default": false,
                        "description": "Disable TTL and keep seeded memories permanent regardless of confidence."
                    },
                    "facts": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": {"type": "string", "minLength": 1},
                                "category": {
                                    "type": "string",
                                    "enum": ["fact", "behavior_instruction", "interest", "persona", "preference"],
                                    "description": "Structured category for filtering and ranking"
                                },
                                "confidence": {
                                    "type": "number",
                                    "minimum": 0.0,
                                    "maximum": 1.0,
                                    "description": "0.0 to 1.0 (defaults to 0.7 for seeds). TTL derived by confidence if ttl_seconds not provided."
                                }
                            },
                            "required": ["content"]
                        }
                    }
                },
                "required": ["facts"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_get",
            description: "Retrieve a memory by its ID",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID"},
                    "strip_private": {"type": "boolean", "description": "When true, removes all <private>...</private> tagged sections from the content before returning (default: false)"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Essential,
        },
    ToolDef {
            name: "memory_update",
            description: "Update an existing memory",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID"},
                    "content": {"type": "string", "description": "New content"},
                    "memory_type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "description": "Memory type (preferred field; alias: type)"},
                    "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "description": "Deprecated alias for memory_type"},
                    "tags": {"type": "array", "items": {"type": "string"}},
                    "metadata": {"type": "object"},
                    "importance": {"type": "number", "minimum": 0, "maximum": 1},
                    "ttl_seconds": {"type": "integer", "description": "Time-to-live in seconds (0 = remove expiration, positive = set new expiration)"},
                    "event_time": {"type": ["string", "null"], "format": "date-time", "description": "ISO8601 timestamp for episodic memories (null to clear)"},
                    "trigger_pattern": {"type": ["string", "null"], "description": "Pattern that triggers this procedure (null to clear)"},
                    "media_url": {"type": ["string", "null"], "description": "URL or local path to the primary media asset (null to clear)"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Essential,
        },
    ToolDef {
            name: "memory_delete",
            description: "Delete a memory (soft delete)",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID"},
                    "cascade_chain": {
                        "type": "boolean",
                        "default": false,
                        "description": "When true, also delete all memories in the supersedes chain (ancestors this memory replaced)."
                    }
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_list",
            description: "List memories with filtering and pagination. Supports workspace isolation, tier filtering, and advanced filter syntax with AND/OR and comparison operators.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "default": 20},
                    "offset": {"type": "integer", "default": 0},
                    "tags": {"type": "array", "items": {"type": "string"}},
                    "memory_type": {"type": "string", "description": "Filter by memory type (preferred field; alias: type)"},
                    "type": {"type": "string", "description": "Deprecated alias for memory_type"},
                    "workspace": {"type": "string", "description": "Filter by single workspace"},
                    "workspaces": {"type": "array", "items": {"type": "string"}, "description": "Filter by multiple workspaces"},
                    "tier": {"type": "string", "enum": ["permanent", "daily"], "description": "Filter by memory tier"},
                    "sort_by": {"type": "string", "enum": ["created_at", "updated_at", "last_accessed_at", "importance", "access_count"]},
                    "sort_order": {"type": "string", "enum": ["asc", "desc"], "default": "desc"},
                    "filter": {
                        "type": "object",
                        "description": "Advanced filter with AND/OR logic and comparison operators. Supports workspace, tier, and metadata fields. Example: {\"AND\": [{\"metadata.project\": {\"eq\": \"engram\"}}, {\"importance\": {\"gte\": 0.5}}]}. Supported operators: eq, neq, gt, gte, lt, lte, contains, not_contains, exists. Fields: content, memory_type, importance, tags, workspace, tier, created_at, updated_at, metadata.*"
                    },
                    "metadata_filter": {
                        "type": "object",
                        "description": "Legacy simple key-value filter (deprecated, use 'filter' instead)"
                    }
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_create_todo",
            description: "Create a TODO memory with priority",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content": {"type": "string"},
                    "priority": {"type": "string", "enum": ["low", "medium", "high", "critical"], "default": "medium"},
                    "due_date": {"type": "string", "format": "date"},
                    "tags": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["content"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_create_issue",
            description: "Create an ISSUE memory for tracking problems",
            schema: r#"{
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "description": {"type": "string"},
                    "severity": {"type": "string", "enum": ["low", "medium", "high", "critical"], "default": "medium"},
                    "tags": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["title"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_versions",
            description: "Get version history for a memory",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_create_batch",
            description: "Create multiple memories in a single operation. More efficient than individual creates for bulk imports.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "memories": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": {"type": "string"},
                                "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential"]},
                                "tags": {"type": "array", "items": {"type": "string"}},
                                "metadata": {"type": "object"},
                                "importance": {"type": "number", "minimum": 0, "maximum": 1},
                                "workspace": {"type": "string"}
                            },
                            "required": ["content"]
                        },
                        "description": "Array of memories to create"
                    }
                },
                "required": ["memories"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_delete_batch",
            description: "Delete multiple memories in a single operation.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "ids": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Array of memory IDs to delete"
                    },
                    "cascade_chain": {
                        "type": "boolean",
                        "default": false,
                        "description": "When true, also delete all memories in the supersedes chain (ancestors this memory replaced)."
                    }
                },
                "required": ["ids"]
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_ingest_fact",
            description: "Append-only fact ingest for high-frequency sources (sessions, file watchers). Always inserts a new memory with memory_type='fact'. No dedup or upsert.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "fact": {"type": "string", "description": "The fact text to store"},
                    "source": {"type": "string", "description": "Origin identifier, e.g. 'session:abc' or 'watcher:/path/to/file'"},
                    "session_id": {"type": "string", "description": "Session ID stored in metadata.session_id"},
                    "workspace": {"type": "string", "description": "Workspace name (default: 'default')"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Optional tags"},
                    "importance": {"type": "number", "minimum": 0, "maximum": 1, "description": "Importance score (default: 0.8)"},
                    "scope": {"type": "string", "description": "Memory scope (default: 'global')"}
                },
                "required": ["fact"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_ingest_fact_batch",
            description: "Batch append-only fact ingest. Inserts all facts in a single transaction. Returns count and ids.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "facts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "fact": {"type": "string", "description": "The fact text"},
                                "source": {"type": "string"},
                                "session_id": {"type": "string"},
                                "workspace": {"type": "string", "description": "Overrides top-level workspace for this item"},
                                "tags": {"type": "array", "items": {"type": "string"}},
                                "importance": {"type": "number", "minimum": 0, "maximum": 1}
                            },
                            "required": ["fact"]
                        },
                        "description": "Array of fact objects to insert"
                    },
                    "workspace": {"type": "string", "description": "Default workspace applied to all facts (default: 'default')"},
                    "scope": {"type": "string", "description": "Memory scope applied to all facts (default: 'global')"}
                },
                "required": ["facts"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_get_public",
            description: "Get a memory with all <private>...</private> tagged sections removed. Safe for sharing in multi-agent contexts.",
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
            name: "memory_get_full",
            description: "Get the full/original content of a memory. If the memory is a Summary, returns the original content from summary_of_id.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Memory ID to get full content for"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_create_section",
            description: "Create a section memory for organizing content hierarchically. Sections can have parent sections for nested organization.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "Section title"},
                    "content": {"type": "string", "description": "Section description or content"},
                    "parent_id": {"type": "integer", "description": "Optional parent section ID for nesting"},
                    "level": {"type": "integer", "default": 1, "description": "Heading level (1-6)"},
                    "workspace": {"type": "string", "description": "Workspace for the section"}
                },
                "required": ["title"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_checkpoint",
            description: "Create a checkpoint memory marking a significant point in a session. Useful for session resumption and context restoration.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session identifier"},
                    "summary": {"type": "string", "description": "Summary of session state at checkpoint"},
                    "context": {"type": "object", "description": "Additional context data to preserve"},
                    "workspace": {"type": "string", "description": "Workspace for the checkpoint"}
                },
                "required": ["session_id", "summary"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_create_episodic",
            description: "Create an episodic memory representing an event with temporal context. Use for tracking when things happened and their duration.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content": {"type": "string", "description": "Description of the event"},
                    "event_time": {"type": "string", "format": "date-time", "description": "ISO8601 timestamp when the event occurred"},
                    "event_duration_seconds": {"type": "integer", "description": "Duration of the event in seconds"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for categorization"},
                    "metadata": {"type": "object", "description": "Additional metadata"},
                    "importance": {"type": "number", "minimum": 0, "maximum": 1, "description": "Importance score (0-1)"},
                    "workspace": {"type": "string", "description": "Workspace (default: 'default')"}
                },
                "required": ["content", "event_time"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_create_procedural",
            description: "Create a procedural memory representing a learned pattern or workflow. Tracks success/failure to measure effectiveness.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content": {"type": "string", "description": "Description of the procedure/workflow"},
                    "trigger_pattern": {"type": "string", "description": "Pattern that triggers this procedure (e.g., 'When user asks about auth')"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for categorization"},
                    "metadata": {"type": "object", "description": "Additional metadata"},
                    "importance": {"type": "number", "minimum": 0, "maximum": 1, "description": "Importance score (0-1)"},
                    "workspace": {"type": "string", "description": "Workspace (default: 'default')"}
                },
                "required": ["content", "trigger_pattern"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_get_timeline",
            description: "Query episodic memories by time range. Returns events ordered by event_time.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "start_time": {"type": "string", "format": "date-time", "description": "Start of time range (ISO8601)"},
                    "end_time": {"type": "string", "format": "date-time", "description": "End of time range (ISO8601)"},
                    "workspace": {"type": "string", "description": "Filter by workspace"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Filter by tags"},
                    "limit": {"type": "integer", "default": 50, "description": "Maximum results to return"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_get_procedures",
            description: "List procedural memories (learned patterns/workflows). Optionally filter by trigger pattern.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "trigger_pattern": {"type": "string", "description": "Filter by trigger pattern (partial match)"},
                    "workspace": {"type": "string", "description": "Filter by workspace"},
                    "min_success_rate": {"type": "number", "minimum": 0, "maximum": 1, "description": "Minimum success rate (successes / (successes + failures))"},
                    "limit": {"type": "integer", "default": 50, "description": "Maximum results to return"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_record_procedure_outcome",
            description: "Record a success or failure for a procedural memory. Increments the corresponding counter.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "description": "Procedural memory ID"},
                    "success": {"type": "boolean", "description": "true = success, false = failure"}
                },
                "required": ["id", "success"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_list_compact",
            description: "List memories with compact preview instead of full content. More efficient for browsing/listing UIs. Returns only essential fields and a truncated content preview with metadata about original content length.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "default": 20, "description": "Maximum memories to return"},
                    "offset": {"type": "integer", "default": 0, "description": "Pagination offset"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Filter by tags"},
                    "memory_type": {"type": "string", "description": "Filter by memory type (preferred field; alias: type)"},
                    "type": {"type": "string", "description": "Deprecated alias for memory_type"},
                    "workspace": {"type": "string", "description": "Filter by workspace"},
                    "tier": {"type": "string", "enum": ["permanent", "daily"], "description": "Filter by tier"},
                    "sort_by": {"type": "string", "enum": ["created_at", "updated_at", "last_accessed_at", "importance", "access_count"], "default": "created_at"},
                    "sort_order": {"type": "string", "enum": ["asc", "desc"], "default": "desc"},
                    "preview_chars": {"type": "integer", "default": 100, "description": "Maximum characters for content preview"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Essential,
        },
];
