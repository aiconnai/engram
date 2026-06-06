// MCP tool definitions by domain.

    // Memory CRUD
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
        tier: ToolTier::Essential,
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
        tier: ToolTier::Essential,
    },
    // Convenience creators
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
    // Versioning
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
    // Memory TTL / Expiration (RML-930)
    ToolDef {
        name: "memory_set_expiration",
        description: "Set or update the expiration time for a memory",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"},
                "ttl_seconds": {"type": "integer", "description": "Time-to-live in seconds from now. Use 0 to remove expiration (make permanent)."}
            },
            "required": ["id", "ttl_seconds"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_cleanup_expired",
        description: "Delete all expired memories. Typically called by a background job, but can be invoked manually.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Standard,
    },
    // Stats and aggregation
    ToolDef {
        name: "memory_stats",
        description: "Get storage statistics",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    // Graph
    ToolDef {
        name: "memory_export_graph",
        description: "Export knowledge graph visualization",
        schema: r#"{
            "type": "object",
            "properties": {
                "format": {"type": "string", "enum": ["html", "json"], "default": "html"},
                "max_nodes": {"type": "integer", "default": 500},
                "focus_id": {"type": "integer", "description": "Center graph on this memory"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // Project Context Discovery
    ToolDef {
        name: "memory_scan_project",
        description: "Scan current directory for AI instruction files (CLAUDE.md, AGENTS.md, .cursorrules, etc.) and ingest them as memories. Creates parent memory for each file and child memories for sections.",
        schema: r#"{
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Directory to scan (defaults to current working directory)"},
                "scan_parents": {"type": "boolean", "default": false, "description": "Also scan parent directories (security: disabled by default)"},
                "extract_sections": {"type": "boolean", "default": true, "description": "Create separate memories for each section"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_get_project_context",
        description: "Get all project context memories for the current working directory. Returns instruction files and their sections.",
        schema: r#"{
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Project path (defaults to current working directory)"},
                "include_sections": {"type": "boolean", "default": true, "description": "Include section memories"},
                "file_types": {"type": "array", "items": {"type": "string"}, "description": "Filter by file type (claude-md, cursorrules, etc.)"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_list_instruction_files",
        description: "List AI instruction files (CLAUDE.md, AGENTS.md, .cursorrules, etc.) in a directory without ingesting them. Returns file paths, types, and sizes for discovery purposes.",
        schema: r#"{
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Directory to scan (defaults to current working directory)"},
                "scan_parents": {"type": "boolean", "default": false, "description": "Also scan parent directories for instruction files"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Document Ingestion (RML-928)
    ToolDef {
        name: "memory_ingest_document",
        description: "Ingest a document (PDF or Markdown) into memory. Extracts text, splits into chunks with overlap, and creates memories with deduplication.",
        schema: r#"{
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Local file path to the document"},
                "format": {"type": "string", "enum": ["auto", "md", "pdf"], "default": "auto", "description": "Document format (auto-detect from extension if not specified)"},
                "chunk_size": {"type": "integer", "default": 1200, "description": "Maximum characters per chunk"},
                "overlap": {"type": "integer", "default": 200, "description": "Overlap between chunks in characters"},
                "max_file_size": {"type": "integer", "default": 10485760, "description": "Maximum file size in bytes (default 10MB)"},
                "tags": {"type": "array", "items": {"type": "string"}, "description": "Additional tags to add to all chunks"}
            },
            "required": ["path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // Dream Phase
    ToolDef {
        name: "dream_run_now",
        description: "Manually trigger the Dream Phase (background consolidation) across all workspaces. This process compresses old memories and identifies patterns while the agent is 'sleeping'.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    // Workspace Management
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
    // Memory Tiering
    ToolDef {
        name: "memory_create_daily",
        description: "Create a daily (ephemeral) memory that auto-expires after the specified TTL. Useful for session context and scratch notes.",
        schema: r#"{
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "The content to remember"},
                "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential"], "default": "note"},
                "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags for categorization"},
                "metadata": {"type": "object", "description": "Additional metadata as key-value pairs"},
                "importance": {"type": "number", "minimum": 0, "maximum": 1, "description": "Importance score (0-1)"},
                "ttl_seconds": {"type": "integer", "default": 86400, "description": "Time-to-live in seconds (default: 24 hours)"},
                "workspace": {"type": "string", "description": "Workspace to store the memory in (default: 'default')"}
            },
            "required": ["content"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_score",
        description: "Compute deterministic memory policy scores for a memory. When persist=true, upserts the memory_policy row and emits a best-effort policy audit event.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to score"},
                "persist": {"type": "boolean", "default": false, "description": "Persist the computed policy score to memory_policy"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_promote",
        description: "Reinforce a memory's policy record, optionally promoting a Daily-tier memory to the canonical Permanent tier.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to promote or reinforce"},
                "canonical_tier": {"type": "boolean", "default": false, "description": "When true, also call promote_to_permanent for canonical tier promotion"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_decay",
        description: "Compute or apply conservative memory policy decay for a workspace. Dry-run is the default; apply only updates memory_policy scores and active lifecycle transitions.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "default": "default", "description": "Workspace to decay"},
                "dry_run": {"type": "boolean", "default": true, "description": "When true, compute candidate changes without mutation"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_explain",
        description: "Explain a memory's current policy score with feature components, reason text, and policy audit count.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to explain"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_reconcile_conflict",
        description: "Record a conflict reconciliation signal for a memory policy without deleting or mutating memory content.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID with the conflict signal"},
                "reason": {"type": "string", "description": "Audit reason for the conflict reconciliation"}
            },
            "required": ["id", "reason"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_promote_to_permanent",
        description: "Promote a daily memory to permanent tier. Clears the expiration and makes the memory permanent.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to promote"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // Embedding Cache
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
    // Session Transcript Indexing
    ToolDef {
        name: "session_index",
        description: "Index a conversation into searchable memory chunks. Uses dual-limiter chunking (messages + characters) with overlap.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Unique session identifier"},
                "messages": {
                    "type": "array",
                    "description": "Array of conversation messages",
                    "items": {
                        "type": "object",
                        "properties": {
                            "role": {"type": "string", "description": "Message role (user, assistant, system)"},
                            "content": {"type": "string", "description": "Message content"},
                            "timestamp": {"type": "string", "description": "ISO 8601 timestamp"},
                            "id": {"type": "string", "description": "Optional message ID"}
                        },
                        "required": ["role", "content"]
                    }
                },
                "title": {"type": "string", "description": "Optional session title"},
                "workspace": {"type": "string", "description": "Workspace to store chunks in (default: 'default')"},
                "agent_id": {"type": "string", "description": "Optional agent identifier"},
                "max_messages": {"type": "integer", "default": 10, "description": "Max messages per chunk"},
                "max_chars": {"type": "integer", "default": 8000, "description": "Max characters per chunk"},
                "overlap": {"type": "integer", "default": 2, "description": "Overlap messages between chunks"},
                "ttl_days": {"type": "integer", "default": 7, "description": "TTL for transcript chunks in days"}
            },
            "required": ["session_id", "messages"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "session_index_delta",
        description: "Incrementally index new messages to an existing session. More efficient than full reindex.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session to update"},
                "messages": {
                    "type": "array",
                    "description": "New messages to add",
                    "items": {
                        "type": "object",
                        "properties": {
                            "role": {"type": "string"},
                            "content": {"type": "string"},
                            "timestamp": {"type": "string"},
                            "id": {"type": "string"}
                        },
                        "required": ["role", "content"]
                    }
                }
            },
            "required": ["session_id", "messages"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "session_get",
        description: "Get information about an indexed session",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID to retrieve"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "session_list",
        description: "List indexed sessions with optional workspace filter",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Filter by workspace"},
                "limit": {"type": "integer", "default": 20, "description": "Maximum sessions to return"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "session_delete",
        description: "Delete a session and all its indexed chunks",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session to delete"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Standard,
    },
    // Identity Management
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
        tier: ToolTier::Essential,
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
        tier: ToolTier::Essential,
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
    // Batch Operations
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
    // Fact Ingest (append-only, high-frequency)
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
    // Tag Utilities
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
    // Maintenance
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
        name: "context_budget_check",
        description: "Check token usage of memories against a budget. Returns token counts and suggestions if over budget.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_ids": {
                    "type": "array",
                    "items": {"type": "integer"},
                    "description": "IDs of memories to check"
                },
                "model": {
                    "type": "string",
                    "description": "Model name for tokenization (gpt-4, gpt-4o, gpt-4o-mini, claude-3-opus, etc.)"
                },
                "encoding": {
                    "type": "string",
                    "description": "Override encoding (cl100k_base, o200k_base). Optional if model is known."
                },
                "budget": {"type": "integer", "description": "Token budget to check against"}
            },
            "required": ["memory_ids", "model", "budget"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "pending_injections_count",
        description: "Count of non-expired payloads queued in pending_injections for a workspace, waiting to be consumed by the next SessionStart.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "default": "default"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "pending_injections_cleanup",
        description: "Drop every pending_injections row whose expires_at has passed. Idempotent. Returns the count removed.",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_archive_old",
        description: "Archive old, low-importance memories by creating summaries. Moves originals to archived state.",
        schema: r#"{
            "type": "object",
            "properties": {
                "max_age_days": {"type": "integer", "default": 90, "description": "Archive memories older than this many days"},
                "max_importance": {"type": "number", "default": 0.5, "description": "Only archive memories with importance below this"},
                "min_access_count": {"type": "integer", "default": 5, "description": "Skip memories accessed more than this many times"},
                "workspace": {"type": "string", "description": "Limit to specific workspace"},
                "dry_run": {"type": "boolean", "default": true, "description": "If true, only report what would be archived"}
            }
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "langfuse")]
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
    // Phase 5: Memory Lifecycle Management (ENG-37)
    ToolDef {
        name: "lifecycle_status",
        description: "Get lifecycle statistics (active/stale/archived counts by workspace).",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Filter by workspace (optional)"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "lifecycle_run",
        description: "Manually trigger a lifecycle cycle (mark stale, archive old). Dry run by default.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": true, "description": "Preview changes without applying"},
                "workspace": {"type": "string", "description": "Limit to specific workspace"},
                "stale_days": {"type": "integer", "default": 30, "description": "Mark memories older than this as stale"},
                "archive_days": {"type": "integer", "default": 90, "description": "Archive memories older than this"},
                "min_importance": {"type": "number", "default": 0.5, "description": "Only process memories below this importance"}
            }
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_set_lifecycle",
        description: "Manually set the lifecycle state of a memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"},
                "state": {"type": "string", "enum": ["active", "stale", "archived"], "description": "New lifecycle state"}
            },
            "required": ["id", "state"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "lifecycle_config",
        description: "Get or set lifecycle configuration (intervals, thresholds).",
        schema: r#"{
            "type": "object",
            "properties": {
                "stale_days": {"type": "integer", "description": "Days before marking as stale"},
                "archive_days": {"type": "integer", "description": "Days before auto-archiving"},
                "min_importance": {"type": "number", "description": "Importance threshold for lifecycle"},
                "min_access_count": {"type": "integer", "description": "Access count threshold"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Retention Policies
    ToolDef {
        name: "retention_policy_set",
        description: "Set a retention policy for a workspace. Controls auto-compression, max memory count, and auto-deletion.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace name"},
                "max_age_days": {"type": "integer", "description": "Hard age limit — auto-delete after this many days"},
                "max_memories": {"type": "integer", "description": "Maximum active memories in this workspace"},
                "compress_after_days": {"type": "integer", "description": "Auto-compress memories older than this"},
                "compress_max_importance": {"type": "number", "description": "Only compress memories with importance <= this (default 0.3)"},
                "compress_min_access": {"type": "integer", "description": "Skip compression if access_count >= this (default 3)"},
                "auto_delete_after_days": {"type": "integer", "description": "Auto-delete archived memories older than this"},
                "exclude_types": {"type": "array", "items": {"type": "string"}, "description": "Memory types exempt from policy (e.g. [\"decision\", \"checkpoint\"])"}
            },
            "required": ["workspace"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "retention_policy_get",
        description: "Get the retention policy for a workspace.",
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
        name: "retention_policy_list",
        description: "List all retention policies across all workspaces.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "retention_policy_delete",
        description: "Delete a retention policy for a workspace.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace name"}
            },
            "required": ["workspace"]
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "retention_policy_apply",
        description: "Apply all retention policies now. Compresses, caps, and deletes per workspace rules.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": false, "description": "Preview what would happen without making changes"}
            }
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    // Phase 8: Salience & Sessions (ENG-66 to ENG-77)
    ToolDef {
        name: "salience_get",
        description: "Get the salience score for a memory. Returns recency, frequency, importance, and feedback components with the combined score and lifecycle state.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to get salience for"},
                "feedback_signal": {"type": "number", "minimum": -1, "maximum": 1, "default": 0, "description": "Optional feedback signal (-1 to 1) to include in calculation"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_set_importance",
        description: "Set the importance score for a memory. This is the static importance component of salience.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"},
                "importance": {"type": "number", "minimum": 0, "maximum": 1, "description": "Importance score (0-1)"}
            },
            "required": ["id", "importance"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_boost",
        description: "Boost a memory's salience score temporarily or permanently. Useful for marking memories as contextually relevant.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to boost"},
                "boost_amount": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.2, "description": "Amount to boost (0-1)"},
                "reason": {"type": "string", "description": "Optional reason for boosting"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_demote",
        description: "Demote a memory's salience score. Useful for marking memories as less relevant.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to demote"},
                "demote_amount": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.2, "description": "Amount to demote (0-1)"},
                "reason": {"type": "string", "description": "Optional reason for demoting"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_decay_run",
        description: "Run temporal decay on all memories. Updates lifecycle states (Active → Stale → Archived) based on salience scores.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": false, "description": "If true, compute changes without persisting updates"},
                "record_history": {"type": "boolean", "default": true, "description": "Record salience history entries while updating"},
                "workspace": {"type": "string", "description": "Limit to specific workspace"},
                "stale_threshold_days": {"type": "integer", "minimum": 1, "description": "Days of inactivity before marking stale"},
                "archive_threshold_days": {"type": "integer", "minimum": 1, "description": "Days of inactivity before suggesting archive"}
            }
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_stats",
        description: "Get salience statistics across all memories. Returns distribution, percentiles, and state counts.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Limit to specific workspace"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_history",
        description: "Get salience score history for a memory. Shows how salience has changed over time.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"},
                "limit": {"type": "integer", "default": 50, "description": "Maximum history entries to return"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "salience_top",
        description: "Get top memories by salience score. Useful for context injection.",
        schema: r#"{
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "default": 20, "description": "Maximum memories to return"},
                "workspace": {"type": "string", "description": "Limit to specific workspace"},
                "min_score": {"type": "number", "minimum": 0, "maximum": 1, "description": "Minimum salience score"},
                "memory_type": {"type": "string", "description": "Filter by memory type"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Phase 9: Context Quality (ENG-48 to ENG-66)
    ToolDef {
        name: "quality_score",
        description: "Get the quality score for a memory with detailed breakdown of clarity, completeness, freshness, consistency, and source trust components.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to score"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "quality_report",
        description: "Generate a comprehensive quality report for a workspace. Includes quality distribution, top issues, conflict and duplicate counts.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to analyze (default: 'default')"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "quality_find_duplicates",
        description: "Find near-duplicate memories using text similarity. Returns pairs of similar memories above the threshold.",
        schema: r#"{
            "type": "object",
            "properties": {
                "threshold": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.85, "description": "Similarity threshold (0-1)"},
                "limit": {"type": "integer", "default": 100, "description": "Maximum memories to compare"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "quality_get_duplicates",
        description: "Get pending duplicate candidates that need review.",
        schema: r#"{
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "default": 50, "description": "Maximum duplicates to return"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "quality_find_conflicts",
        description: "Detect conflicts for a memory against existing memories. Finds contradictions, staleness, and semantic overlaps.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to check for conflicts"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "quality_get_conflicts",
        description: "Get unresolved conflicts that need attention.",
        schema: r#"{
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "default": 50, "description": "Maximum conflicts to return"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "quality_resolve_conflict",
        description: "Resolve a conflict between memories. Options: keep_a, keep_b, merge, keep_both, delete_both, false_positive.",
        schema: r#"{
            "type": "object",
            "properties": {
                "conflict_id": {"type": "integer", "description": "Conflict ID to resolve"},
                "resolution": {"type": "string", "enum": ["keep_a", "keep_b", "merge", "keep_both", "delete_both", "false_positive"], "description": "How to resolve the conflict"},
                "notes": {"type": "string", "description": "Optional notes about the resolution"}
            },
            "required": ["conflict_id", "resolution"]
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "quality_source_trust",
        description: "Get or update trust score for a source type. Higher trust means memories from this source are weighted more in quality calculations.",
        schema: r#"{
            "type": "object",
            "properties": {
                "source_type": {"type": "string", "description": "Source type (user, seed, extraction, inference, external)"},
                "source_identifier": {"type": "string", "description": "Optional specific source identifier"},
                "trust_score": {"type": "number", "minimum": 0, "maximum": 1, "description": "New trust score (omit to just get current score)"},
                "notes": {"type": "string", "description": "Notes about this source"}
            },
            "required": ["source_type"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "quality_improve",
        description: "Get suggestions for improving a memory's quality. Returns actionable recommendations.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to analyze"}
            },
            "required": ["id"]
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
    // ── temporal.rs — temporal graph handlers ────────────────────────────────
    ToolDef {
        name: "temporal_add_edge",
        description: "Add a bi-temporal validity edge between two memories in the knowledge graph.",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {
                    "type": "integer",
                    "description": "Source memory ID."
                },
                "to_id": {
                    "type": "integer",
                    "description": "Target memory ID."
                },
                "relation": {
                    "type": "string",
                    "description": "Semantic label for the edge (e.g. \"works_at\")."
                },
                "valid_from": {
                    "type": "string",
                    "description": "RFC3339 timestamp marking the start of edge validity."
                },
                "properties": {
                    "type": "object",
                    "description": "Arbitrary JSON metadata to attach to the edge."
                },
                "confidence": {
                    "type": "number",
                    "description": "Edge confidence score between 0.0 and 1.0 (default: 1.0)."
                },
                "source": {
                    "type": "string",
                    "description": "Provenance string identifying where this edge originates."
                },
                "scope_path": {
                    "type": "string",
                    "description": "Optional scope path to associate with this edge."
                }
            },
            "required": ["from_id", "to_id", "relation", "valid_from"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "temporal_contradictions",
        description: "Detect overlapping or contradictory edge pairs in the temporal knowledge graph.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "temporal_diff",
        description: "Compute the set of added, removed, and changed edges between two RFC3339 timestamps in the temporal graph.",
        schema: r#"{
            "type": "object",
            "properties": {
                "t1": {
                    "type": "string",
                    "description": "Earlier RFC3339 timestamp (snapshot baseline)."
                },
                "t2": {
                    "type": "string",
                    "description": "Later RFC3339 timestamp (snapshot target)."
                },
                "scope_path": {
                    "type": "string",
                    "description": "Optional scope path to restrict the diff."
                }
            },
            "required": ["t1", "t2"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "temporal_snapshot",
        description: "Return all currently-valid temporal graph edges as of a given RFC3339 timestamp.",
        schema: r#"{
            "type": "object",
            "properties": {
                "timestamp": {
                    "type": "string",
                    "description": "RFC3339 point-in-time for the snapshot."
                },
                "scope_path": {
                    "type": "string",
                    "description": "Optional scope path to restrict the snapshot."
                }
            },
            "required": ["timestamp"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "temporal_timeline",
        description: "Return the full edge history between two memory IDs, ordered chronologically.",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {
                    "type": "integer",
                    "description": "Source memory ID."
                },
                "to_id": {
                    "type": "integer",
                    "description": "Target memory ID."
                },
                "scope_path": {
                    "type": "string",
                    "description": "Optional scope path to restrict the timeline."
                }
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // ── enrichment_audit.rs (ENG-1240) ───────────────────────────────────────
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
