&[
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
        tier: ToolTier::Standard,
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
    ToolDef {
        name: "dream_create",
        description: "Create a reviewable dream snapshot job and optionally run deterministic candidate generation. Generated candidates are proposals, not canonical memories.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "default": "default"},
                "job_id": {"type": "string", "description": "Optional stable job id. Omit to generate one."},
                "instructions": {"type": "string"},
                "run": {"type": "boolean", "default": true, "description": "When true, run deterministic generation immediately."},
                "max_memories": {"type": "integer", "default": 50, "minimum": 1},
                "max_candidates": {"type": "integer", "default": 25, "minimum": 1},
                "summary_min_memories": {"type": "integer", "default": 2, "minimum": 1}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_get",
        description: "Inspect one dream snapshot job.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Dream job id"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_list",
        description: "List dream snapshot jobs by workspace and status.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string"},
                "status": {"type": "string", "enum": ["pending", "running", "completed", "failed", "canceled", "archived"]},
                "limit": {"type": "integer", "default": 100, "minimum": 1, "maximum": 1000}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_cancel",
        description: "Cancel a pending or running dream snapshot job idempotently.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Dream job id"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_archive",
        description: "Archive a terminal dream snapshot job.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Dream job id"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_candidates_list",
        description: "List review candidates emitted by dream snapshot jobs. Results are proposals and are not canonical memory facts.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string"},
                "job_id": {"type": "string"},
                "review_state": {"type": "string", "enum": ["pending", "accepted", "edited", "rejected", "applied", "archived"]},
                "limit": {"type": "integer", "default": 100, "minimum": 1, "maximum": 1000}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_candidate_get",
        description: "Inspect one dream candidate and its evidence sources.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Dream candidate id"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_candidate_review",
        description: "Review a dream candidate by accepting, editing, rejecting, or archiving it. This does not mutate canonical memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Dream candidate id"},
                "review_state": {"type": "string", "enum": ["accepted", "edited", "rejected", "archived"]},
                "edited_content": {"type": "string", "description": "Reviewed replacement content when review_state is edited."},
                "metadata_patch": {"type": "object", "description": "Optional review metadata merged into candidate metadata."}
            },
            "required": ["id", "review_state"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_candidate_apply",
        description: "Apply an accepted or edited dream candidate to canonical memory. Requires confirm=true unless dry_run=true; repeated apply is idempotent.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Dream candidate id"},
                "confirm": {"type": "boolean", "default": false, "description": "Must be true for canonical mutation."},
                "dry_run": {"type": "boolean", "default": false, "description": "Preview planned canonical mutation without applying."}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_agent_writeback",
        description: "Create a pending agent-generated memory proposal as an agent_writeback dream candidate. Defaults to dry_run=true and never mutates canonical memory; review/apply still happens through dream_candidate_get/review/apply.",
        schema: r#"{
            "type": "object",
            "properties": {
                "proposed_content": {"type": "string", "minLength": 1, "description": "Agent-generated memory content to propose for human/agent review."},
                "workspace": {"type": "string", "default": "default", "description": "Workspace for the pending candidate."},
                "job_id": {"type": "string", "description": "Optional dream job id to group writeback candidates. Omit to generate one."},
                "candidate_id": {"type": "string", "description": "Optional stable candidate id. Omit to generate one."},
                "confidence": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.5, "description": "Confidence in the proposed writeback."},
                "reason_codes": {"type": "array", "items": {"type": "string"}, "description": "Reason codes for the pending candidate. Defaults to agent_writeback."},
                "metadata": {"type": "object", "description": "Additional candidate metadata. Governance markers are added by Engram."},
                "source_memory_ids": {
                    "type": "array",
                    "items": {"type": "integer", "minimum": 1},
                    "description": "Canonical memory ids that support this proposal."
                },
                "evidence": {
                    "type": "array",
                    "description": "Additional non-memory evidence sources. At least one source_memory_ids entry or evidence item is required.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "source_type": {"type": "string", "minLength": 1},
                            "source_id": {"type": "string", "minLength": 1},
                            "source_ref": {"type": "string"},
                            "evidence": {"type": "object"}
                        },
                        "required": ["source_type", "source_id"]
                    }
                },
                "dry_run": {"type": "boolean", "default": true, "description": "Preview the pending candidate without writing dream_candidates."},
                "confirm": {"type": "boolean", "default": false, "description": "Required with dry_run=false to create the pending candidate."}
            },
            "required": ["proposed_content"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "dream_eval_run",
        description: "Run deterministic local dream snapshot evaluation fixtures and return parseable CI-safe metrics. Does not require network, credentials, or model access.",
        schema: r#"{
            "type": "object",
            "properties": {
                "fixtures": {
                    "type": "array",
                    "items": {"type": "string", "enum": ["carry_forward_context", "preferences_constraints", "freshness_temporal", "provenance_correctness", "unsafe_raw_log_rejection", "no_canonical_mutation_before_apply"]},
                    "description": "Optional subset of fixed fixture names. Omit to run all fixtures."
                },
                "include_details": {"type": "boolean", "default": true, "description": "Include per-fixture candidate details."}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
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
        description: "Compute or apply conservative memory policy decay for a workspace. Dry-run is the default; apply updates memory_policy scores only. Use lifecycle_run for lifecycle_state transitions.",
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
        tier: ToolTier::Standard,
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
        tier: ToolTier::Standard,
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
        description: "Run the canonical lifecycle predicate to mark stale and archive idle memories. Dry run by default; this is the only decay-derived lifecycle writer.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": true, "description": "Preview changes without applying"},
                "workspace": {"type": "string", "description": "Limit to specific workspace"},
                "stale_days": {"type": "integer", "default": 30, "description": "Base idle days before marking as stale"},
                "archive_days": {"type": "integer", "default": 90, "description": "Base idle days before archiving"},
                "hard_idle_cap_days": {"type": "integer", "default": 365, "description": "Absolute idle-day cap before archiving"},
                "max_importance_mult": {"type": "number", "default": 4.0, "description": "Maximum multiplier by which importance extends stale/archive windows"}
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
        description: "Get lifecycle predicate configuration defaults and optional overrides.",
        schema: r#"{
            "type": "object",
            "properties": {
                "stale_days": {"type": "integer", "description": "Base idle days before marking as stale"},
                "archive_days": {"type": "integer", "description": "Base idle days before archiving"},
                "hard_idle_cap_days": {"type": "integer", "description": "Absolute idle-day cap before archiving"},
                "max_importance_mult": {"type": "number", "description": "Maximum multiplier by which importance extends stale/archive windows"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Retention Policies
    ToolDef {
        name: "retention_policy_set",
        description: "Set a retention policy for a workspace. Controls compression of already-Archived memories, max memory count, and auto-deletion.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace name"},
                "max_age_days": {"type": "integer", "description": "Hard age limit — auto-delete after this many days"},
                "max_memories": {"type": "integer", "description": "Maximum active memories in this workspace"},
                "compress_after_days": {"type": "integer", "description": "Compress already-Archived memories older than this"},
                "compress_max_importance": {"type": "number", "description": "Only compress already-Archived memories with importance <= this (default 0.3)"},
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
        description: "Run salience score decay and optionally record salience history. Does not update lifecycle_state; use lifecycle_run for lifecycle transitions.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": false, "description": "If true, compute score/history changes without persisting updates"},
                "record_history": {"type": "boolean", "default": true, "description": "Record salience history entries while updating"},
                "workspace": {"type": "string", "description": "Limit to specific workspace"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
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
// MCP tool definitions by domain.

    // Search
    ToolDef {
        name: "memory_search",
        description: "Search memories using hybrid search (keyword + semantic). Automatically selects optimal strategy with optional reranking. Supports workspace isolation, tier filtering, and advanced filters.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "limit": {"type": "integer", "default": 10},
                "min_score": {"type": "number", "default": 0.1},
                "tags": {"type": "array", "items": {"type": "string"}},
                "memory_type": {"type": "string", "description": "Filter by memory type (preferred field; alias: type)"},
                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
                "workspace": {"type": "string", "description": "Filter by single workspace"},
                "workspaces": {"type": "array", "items": {"type": "string"}, "description": "Filter by multiple workspaces"},
                "tier": {"type": "string", "enum": ["permanent", "daily"], "description": "Filter by memory tier"},
                "include_transcripts": {"type": "boolean", "default": false, "description": "Include transcript chunk memories (excluded by default)"},
                "strategy": {"type": "string", "enum": ["auto", "keyword", "keyword_only", "semantic", "semantic_only", "hybrid"], "description": "Force specific strategy (auto selects based on query; keyword/semantic are aliases for keyword_only/semantic_only)"},
                "explain": {"type": "boolean", "default": false, "description": "Include match explanations"},
                "rerank": {"type": "boolean", "default": true, "description": "Apply reranking to improve result quality"},
                "rerank_strategy": {"type": "string", "enum": ["none", "heuristic", "multi_signal"], "default": "heuristic", "description": "Reranking strategy to use"},
                "policy_rerank": {
                    "type": "boolean",
                    "default": false,
                    "description": "Apply memory policy retrieval_priority as an opt-in rerank layer after hybrid search."
                },
                "policy_explain": {
                    "type": "boolean",
                    "default": false,
                    "description": "Include policy score and reason for each reranked result when policy_rerank is true."
                },
                "filter": {
                    "type": "object",
                    "description": "Advanced filter with AND/OR logic. Supports workspace, tier, and metadata fields. Example: {\"AND\": [{\"workspace\": {\"eq\": \"my-project\"}}, {\"importance\": {\"gte\": 0.5}}]}"
                },
                "global": {"type": "boolean", "default": false, "description": "Search across all workspaces (default: false). When true, ignores any workspace filter and returns results from all workspaces with a workspace field in each result."}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_smart_retrieve",
        description: "Intent-aware unified retrieval. Classifies the query (lookup, exploration, context, path) and dispatches to the right combination of internal retrievers, then merges and dedupes results. Returns audit fields `intents_used` and `strategies_called`.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Natural-language query"},
                "limit": {"type": "integer", "default": 10, "minimum": 1, "maximum": 100},
                "workspace": {"type": "string", "description": "Optional workspace filter"},
                "force_intents": {
                    "type": "array",
                    "items": {"type": "string", "enum": ["lookup", "exploration", "context", "path"]},
                    "description": "Override the classifier (for testing/debugging)"
                }
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_digest",
        description: "Build an actionable, source-linked digest for a topic by orchestrating existing read-only retrieval, graph, and operational-context tools. Returns summaries, source memory IDs, relationships, next actions, provenance, and warnings without mutating memory or invoking an LLM.",
        schema: r#"{
            "type": "object",
            "properties": {
                "topic": {"type": "string", "description": "Topic, task, decision, or question to summarize from memory"},
                "workspace": {"type": "string", "description": "Optional workspace scope"},
                "mode": {"type": "string", "enum": ["brief", "standard", "deep"], "default": "standard", "description": "Digest depth and section size"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50, "default": 12, "description": "Maximum source memories to inspect"},
                "related_depth": {"type": "integer", "minimum": 0, "maximum": 2, "default": 1, "description": "How many relationship hops to include from selected source memories"},
                "total_budget": {"type": "integer", "minimum": 512, "maximum": 12000, "default": 4096, "description": "Token budget passed to context assembly for accounting only; raw prompt content is not returned"},
                "include_types": {"type": "array", "items": {"type": "string"}, "description": "Optional memory_type allowlist for digest source memories"},
                "timeframe": {"type": "string", "enum": ["1h", "24h", "7d", "30d", "all"], "default": "all", "description": "Time window for context assembly"},
                "include_graph": {"type": "boolean", "default": true, "description": "Include cross-reference relationships for source memories"},
                "include_operational_context": {"type": "boolean", "default": true, "description": "Include compact Operational Context bundle sections"},
                "include_next_actions": {"type": "boolean", "default": true, "description": "Include extractive next-action suggestions grounded in source IDs"},
                "current_git_branch": {"type": "string", "description": "Current branch used for Operational Context staleness warnings"},
                "current_commit_hash": {"type": "string", "description": "Current commit used for Operational Context staleness warnings"}
            },
            "required": ["topic"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_agent_contract",
        description: "Return the read-only Agent Memory Contract: governed recall paths, safe writeback rules, provenance requirements, tool tier defaults, and future pending-review boundaries for agent-generated memory.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_council",
        description: "Run a question through an llm-council instance (Karpathy council orchestration) and return consolidated stage outputs and final answer. Optionally persist a checkpoint memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "prompt": {"type": "string", "description": "Prompt to send to the council"},
                "conversation_id": {"type": "string", "description": "Optional existing conversation ID to continue"},
                "council_url": {"type": "string", "default": "http://127.0.0.1:8001", "description": "Council HTTP base URL"},
                "timeout_seconds": {"type": "integer", "minimum": 1, "maximum": 300, "default": 90, "description": "Request timeout in seconds (1-300)"},
                "include_raw_stages": {"type": "boolean", "default": true, "description": "Whether to include raw stage payloads"},
                "persist": {"type": "boolean", "default": false, "description": "Persist final answer as checkpoint memory"},
                "workspace": {"type": "string", "default": "default", "description": "Target workspace when persist=true"},
                "memory_tags": {
                    "type": "array",
                    "description": "Extra tags to include when persist=true (default tags: llm-council, consensus)",
                    "items": {"type": "string"}
                }
            },
            "required": ["prompt"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_search_suggest",
        description: "Get search suggestions and typo corrections",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // Clustering and duplicates
    ToolDef {
        name: "memory_find_duplicates",
        description: "Find potential duplicate memories",
        schema: r#"{
            "type": "object",
            "properties": {
                "threshold": {"type": "number", "default": 0.9}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_find_semantic_duplicates",
        description: "Find semantically similar memories using embedding cosine similarity (LLM-powered dedup). Goes beyond hash/n-gram to detect paraphrased content.",
        schema: r#"{
            "type": "object",
            "properties": {
                "threshold": {"type": "number", "default": 0.92, "description": "Cosine similarity threshold (0.92 = very similar)"},
                "workspace": {"type": "string", "description": "Filter by workspace (optional)"},
                "limit": {"type": "integer", "default": 50, "description": "Maximum duplicate pairs to return"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // Phase 3: Langfuse Integration (ENG-35) - feature-gated
    ToolDef {
        name: "langfuse_connect",
        description: "Configure Langfuse connection for observability integration. Stores config in metadata.",
        schema: r#"{
            "type": "object",
            "properties": {
                "public_key": {"type": "string", "description": "Langfuse public key (or use LANGFUSE_PUBLIC_KEY env var)"},
                "secret_key": {"type": "string", "description": "Langfuse secret key (or use LANGFUSE_SECRET_KEY env var)"},
                "base_url": {"type": "string", "default": "https://cloud.langfuse.com", "description": "Langfuse API base URL"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "langfuse_sync",
        description: "Start background sync from Langfuse traces to memories. Returns task_id for status checking.",
        schema: r#"{
            "type": "object",
            "properties": {
                "since": {"type": "string", "format": "date-time", "description": "Sync traces since this timestamp (default: 24h ago)"},
                "limit": {"type": "integer", "default": 100, "description": "Maximum traces to sync"},
                "workspace": {"type": "string", "description": "Workspace to create memories in"},
                "dry_run": {"type": "boolean", "default": false, "description": "Preview what would be synced without creating memories"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "langfuse_sync_status",
        description: "Check the status of a Langfuse sync task.",
        schema: r#"{
            "type": "object",
            "properties": {
                "task_id": {"type": "string", "description": "Task ID returned from langfuse_sync"}
            },
            "required": ["task_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "langfuse_extract_patterns",
        description: "Extract patterns from Langfuse traces without saving. Preview mode for pattern discovery.",
        schema: r#"{
            "type": "object",
            "properties": {
                "since": {"type": "string", "format": "date-time", "description": "Analyze traces since this timestamp"},
                "limit": {"type": "integer", "default": 50, "description": "Maximum traces to analyze"},
                "min_confidence": {"type": "number", "default": 0.7, "description": "Minimum confidence for patterns"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Phase 4: Search Result Caching (ENG-36)
    ToolDef {
        name: "search_cache_feedback",
        description: "Report feedback on search results quality. Helps tune the adaptive cache threshold.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "The search query"},
                "positive": {"type": "boolean", "description": "True if results were helpful, false otherwise"},
                "workspace": {"type": "string", "description": "Workspace filter used (if any)"}
            },
            "required": ["query", "positive"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "search_cache_stats",
        description: "Get search result cache statistics including hit rate, entry count, and current threshold.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "search_cache_clear",
        description: "Clear the search result cache. Useful after bulk operations.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Only clear cache for this workspace (optional)"}
            }
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    // Search Variants
    ToolDef {
        name: "memory_search_by_identity",
        description: "Search memories by identity (person, entity, or alias). Finds all mentions of a specific identity across memories.",
        schema: r#"{
            "type": "object",
            "properties": {
                "identity": {"type": "string", "description": "Identity name or alias to search for"},
                "workspace": {"type": "string", "description": "Optional: limit search to specific workspace"},
                "limit": {"type": "integer", "default": 50, "description": "Maximum results to return"}
            },
            "required": ["identity"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_session_search",
        description: "Search within session transcript chunks. Useful for finding content from past conversations.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "session_id": {"type": "string", "description": "Optional: limit to specific session"},
                "workspace": {"type": "string", "description": "Optional: limit to specific workspace"},
                "limit": {"type": "integer", "default": 20, "description": "Maximum results to return"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // Phase 7: Meilisearch Integration (ENG-58) - feature-gated
    ToolDef {
        name: "meilisearch_search",
        description: "Search memories using Meilisearch (typo-tolerant, fast full-text). Requires Meilisearch to be configured. Falls back to hybrid search if unavailable.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query text"},
                "limit": {"type": "integer", "default": 20, "description": "Maximum results to return"},
                "offset": {"type": "integer", "default": 0, "description": "Number of results to skip"},
                "workspace": {"type": "string", "description": "Filter by workspace"},
                "tags": {"type": "array", "items": {"type": "string"}, "description": "Filter by tags (AND logic)"},
                "memory_type": {"type": "string", "description": "Filter by memory type"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "meilisearch_reindex",
        description: "Trigger a full re-sync from SQLite to Meilisearch. Use after bulk imports or if the index is out of sync.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "meilisearch_status",
        description: "Get Meilisearch index status including document count, indexing state, and health.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "meilisearch_config",
        description: "Show current Meilisearch configuration (URL, sync interval, enabled status).",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_search_compact",
        description: "Token-efficient search returning only id, title (first line, max 80 chars), created_at, and tags. Use memory_expand to get full content for specific IDs.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "limit": {"type": "integer", "description": "Max results (default: 10)"},
                "workspace": {"type": "string", "description": "Filter to workspace"},
                "global": {"type": "boolean", "default": false, "description": "Search across all workspaces (default: false). When true, ignores any workspace filter and includes a workspace field in each result."}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_expand",
        description: "Fetch full memory content for specific IDs. Used after memory_search_compact to get full content only for memories you need.",
        schema: r#"{
            "type": "object",
            "properties": {
                "ids": {"type": "array", "items": {"type": "integer"}, "description": "Memory IDs to expand"}
            },
            "required": ["ids"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_detect_updates",
        description: "Given new content, identify existing memories in a workspace that may be stale or in need of an update.",
        schema: r#"{
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "New content to compare against stored memories."
                },
                "workspace": {
                    "type": "string",
                    "description": "Workspace to search for update candidates (default: \"default\")."
                }
            },
            "required": ["content"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_explain_search",
        description: "Explain how each result in a scored search batch was ranked, breaking down bm25, vector, fuzzy, recency, importance, and optional rerank contributions.",
        schema: r#"{
            "type": "object",
            "properties": {
                "results": {
                    "type": "array",
                    "description": "Array of scored search result objects to explain.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "memory_id":   {"type": "integer", "description": "ID of the memory."},
                            "bm25":        {"type": "number",  "description": "BM25 full-text score."},
                            "vector":      {"type": "number",  "description": "Cosine similarity / vector score."},
                            "fuzzy":       {"type": "number",  "description": "Fuzzy string match score."},
                            "recency":     {"type": "number",  "description": "Recency decay score."},
                            "importance":  {"type": "number",  "description": "Stored importance weight."},
                            "final_score": {"type": "number",  "description": "Final blended score used for ranking."},
                            "rerank_score":{"type": "number",  "description": "Optional cross-encoder rerank score."}
                        },
                        "required": ["memory_id", "final_score"]
                    }
                },
                "reranking_active": {
                    "type": "boolean",
                    "description": "Whether cross-encoder reranking was active for this result set (default: false)."
                },
                "rrf_k": {
                    "type": "integer",
                    "description": "RRF k constant used during retrieval (default: 60)."
                }
            },
            "required": ["results"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // -- reconciliation batch D --
    ToolDef {
        name: "memory_suggest_acquisitions",
        description: "Analyse knowledge gaps in a workspace and suggest new memories to create.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {
                    "type": "string",
                    "description": "Workspace to analyse (default: \"default\")."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of suggestions to return (default: 10)."
                }
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
// MCP tool definitions by domain.

    // Cross-references
    ToolDef {
        name: "memory_link",
        description: "Create a cross-reference between two memories",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {"type": "integer"},
                "to_id": {"type": "integer"},
                "edge_type": {"type": "string", "enum": ["related_to", "supersedes", "contradicts", "implements", "extends", "references", "derived_from", "depends_on", "blocks", "follows_up"], "default": "related_to"},
                "strength": {"type": "number", "minimum": 0, "maximum": 1, "description": "Relationship strength"},
                "source_context": {"type": "string", "description": "Why this link exists"},
                "pinned": {"type": "boolean", "default": false, "description": "Exempt from confidence decay"}
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_unlink",
        description: "Remove a cross-reference",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {"type": "integer"},
                "to_id": {"type": "integer"},
                "edge_type": {"type": "string", "default": "related_to"}
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_related",
        description: "Get memories related to a given memory (depth>1 or include_entities returns traversal result)",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Starting memory ID"},
                "depth": {"type": "integer", "default": 1, "description": "Traversal depth (1 = direct relations only)"},
                "include_entities": {"type": "boolean", "default": false, "description": "Include connections through shared entities"},
                "edge_type": {"type": "string", "description": "Filter by edge type"},
                "include_decayed": {"type": "boolean", "default": false}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // Entity Extraction (RML-925)
    ToolDef {
        name: "memory_extract_entities",
        description: "Extract named entities (people, organizations, projects, concepts) from a memory and store them",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to extract entities from"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_get_entities",
        description: "Get all entities linked to a memory",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_search_entities",
        description: "Search for entities by name prefix",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query (prefix match)"},
                "entity_type": {"type": "string", "description": "Filter by entity type (person, organization, project, concept, etc.)"},
                "limit": {"type": "integer", "default": 20}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_entity_stats",
        description: "Get statistics about extracted entities",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Graph Traversal (RML-926)
    ToolDef {
        name: "memory_traverse",
        description: "Traverse the knowledge graph from a starting memory with full control over traversal options",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Starting memory ID"},
                "depth": {"type": "integer", "default": 2, "description": "Maximum traversal depth"},
                "direction": {"type": "string", "enum": ["outgoing", "incoming", "both"], "default": "both"},
                "edge_types": {"type": "array", "items": {"type": "string"}, "description": "Filter by edge types (related_to, depends_on, etc.)"},
                "min_score": {"type": "number", "default": 0, "description": "Minimum edge score threshold"},
                "min_confidence": {"type": "number", "default": 0, "description": "Minimum confidence threshold"},
                "limit_per_hop": {"type": "integer", "default": 50, "description": "Max results per hop"},
                "include_entities": {"type": "boolean", "default": true, "description": "Include entity-based connections"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_find_path",
        description: "Find the shortest path between two memories in the knowledge graph",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {"type": "integer", "description": "Starting memory ID"},
                "to_id": {"type": "integer", "description": "Target memory ID"},
                "max_depth": {"type": "integer", "default": 5, "description": "Maximum path length to search"}
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // ── DuckDB Graph Tools (duckdb-graph) ──────────────────────────────────────
    ToolDef {
        name: "memory_graph_path",
        description: "Finds how two entities are connected in the knowledge graph via DuckDB OLAP engine. Discovers hidden relationships across multiple hops using recursive path-finding.",
        schema: r#"{
            "type": "object",
            "properties": {
                "scope": {"type": "string", "description": "Tenant scope prefix, e.g., 'global/org/user'"},
                "source_id": {"type": "integer", "description": "Starting node ID"},
                "target_id": {"type": "integer", "description": "Target node ID"},
                "max_depth": {"type": "integer", "description": "Maximum hops to traverse (default: 4, max: 10)", "default": 4}
            },
            "required": ["scope", "source_id", "target_id"]
        }"#,
        annotations: ToolAnnotations {
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        },
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_temporal_snapshot",
        description: "Retrieves the exact facts and relationships that were true at a specific historical point in time. Uses DuckDB OLAP engine for fast columnar scans over temporal edges.",
        schema: r#"{
            "type": "object",
            "properties": {
                "scope": {"type": "string", "description": "Tenant scope prefix"},
                "timestamp": {"type": "string", "description": "ISO-8601 timestamp for the point-in-time query"}
            },
            "required": ["scope", "timestamp"]
        }"#,
        annotations: ToolAnnotations {
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        },
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_scope_snapshot",
        description: "Compares the knowledge graph between two timestamps, showing what relationships were added, removed, or changed. Uses DuckDB OLAP engine for efficient temporal diff.",
        schema: r#"{
            "type": "object",
            "properties": {
                "scope": {"type": "string", "description": "Tenant scope prefix"},
                "from_timestamp": {"type": "string", "description": "Start of comparison window (ISO-8601)"},
                "to_timestamp": {"type": "string", "description": "End of comparison window (ISO-8601)"}
            },
            "required": ["scope", "from_timestamp", "to_timestamp"]
        }"#,
        annotations: ToolAnnotations {
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        },
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_auto_link",
        description: "Run semantic and temporal auto-linker on a workspace, creating crossref edges in the database. Mutates the database.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to auto-link (default: all workspaces)"},
                "similarity_threshold": {"type": "number", "description": "Minimum cosine similarity to create a semantic link (default: 0.75)"},
                "time_window_minutes": {"type": "integer", "description": "Time window in minutes for temporal linking (default: 30)"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_auto_link_stats",
        description: "Return aggregate statistics about auto-generated semantic and temporal links.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // -- reconciliation batch B --
    ToolDef {
        name: "memory_cluster",
        description: "Run Louvain community detection on the memory graph and return detected clusters with modularity score.",
        schema: r#"{
            "type": "object",
            "properties": {
                "min_cluster_size": {
                    "type": "integer",
                    "description": "Minimum number of members for a cluster to be reported (default: 2)."
                },
                "resolution": {
                    "type": "number",
                    "description": "Louvain resolution parameter controlling cluster granularity (default: 1.0)."
                },
                "link_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Restrict clustering to these edge/link types. Omit to use all link types."
                }
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_coactivation_report",
        description: "Return coactivation graph statistics including edge count, average strength, and strongest co-occurring memory pairs.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_fact_graph",
        description: "Return all stored subject-predicate-object facts for a given subject entity.",
        schema: r#"{
            "type": "object",
            "properties": {
                "subject": {
                    "type": "string",
                    "description": "Entity name to look up in the facts table."
                }
            },
            "required": ["subject"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 2. memory_garden
    ToolDef {
        name: "memory_garden",
        description: "Run full autonomous garden maintenance on a workspace: prunes stale memories, merges duplicates, archives cold entries, and compresses verbose content.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to garden (default: \"default\")."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // 3. memory_garden_preview
    ToolDef {
        name: "memory_garden_preview",
        description: "Dry-run garden maintenance: reports what would be pruned, merged, archived, or compressed without making any changes.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to preview gardening for (default: \"default\")."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 4. memory_get_cluster
    ToolDef {
        name: "memory_get_cluster",
        description: "Return the Louvain community cluster that contains a specific memory, including its cluster ID, size, and member IDs.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "ID of the memory whose cluster to look up."}
            },
            "required": ["memory_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 5. memory_knowledge_stats
    ToolDef {
        name: "memory_knowledge_stats",
        description: "Return aggregate statistics over the knowledge-graph facts table: total facts, unique subjects/predicates/objects, and top entities.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 6. memory_list_auto_links
    ToolDef {
        name: "memory_list_auto_links",
        description: "List auto-generated graph links (semantic or temporal) between memories, optionally filtered by link type.",
        schema: r#"{
            "type": "object",
            "properties": {
                "link_type": {"type": "string", "description": "Filter by link type: \"semantic\" or \"temporal\". Omit for all types."},
                "limit": {"type": "integer", "description": "Maximum number of links to return (default: 50)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 7. memory_list_clusters
    ToolDef {
        name: "memory_list_clusters",
        description: "List all detected memory clusters from the persistent cluster table, optionally selecting the detection algorithm.",
        schema: r#"{
            "type": "object",
            "properties": {
                "algorithm": {"type": "string", "description": "Clustering algorithm to filter by (default: \"louvain\")."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 8. memory_list_facts
    ToolDef {
        name: "memory_list_facts",
        description: "List extracted subject-predicate-object facts, optionally scoped to a single source memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "Source memory ID to filter facts; omit to list facts from all memories."},
                "limit": {"type": "integer", "description": "Maximum number of facts to return (default: 100)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 11. memory_query_triplets
    ToolDef {
        name: "memory_query_triplets",
        description: "SPARQL-like pattern query over the knowledge-graph facts table: match any combination of subject, predicate, and object (all optional, acts as wildcard when omitted).",
        schema: r#"{
            "type": "object",
            "properties": {
                "subject": {"type": "string", "description": "Subject entity to match (wildcard if omitted)."},
                "predicate": {"type": "string", "description": "Predicate/relation to match (wildcard if omitted)."},
                "object": {"type": "string", "description": "Object value to match (wildcard if omitted)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 12. memory_reflect
    ToolDef {
        name: "memory_reflect",
        description: "Generate a reflective synthesis over a set of memories at a configurable analytical depth (surface, analytical, or meta).",
        schema: r#"{
            "type": "object",
            "properties": {
                "ids": {
                    "type": "array",
                    "items": {"type": "integer"},
                    "description": "Array of memory IDs to reflect on (required, must be non-empty)."
                },
                "depth": {"type": "string", "description": "Reflection depth: \"surface\" (default), \"analytical\", or \"meta\"."}
            },
            "required": ["ids"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 13. memory_resolve_conflict
    ToolDef {
        name: "memory_resolve_conflict",
        description: "Resolve a saved knowledge-graph conflict by ID using a chosen strategy, removing or retaining the conflicting edges accordingly.",
        schema: r#"{
            "type": "object",
            "properties": {
                "conflict_id": {"type": "integer", "description": "ID of the conflict record to resolve (required)."},
                "strategy": {"type": "string", "description": "Resolution strategy: \"keep_newer\" (default), \"keep_higher_confidence\", \"merge\", or \"manual\"."}
            },
            "required": ["conflict_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // 14. memory_sentiment_analyze
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
    // 15. memory_sentiment_timeline
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
// MCP tool definitions by domain.

    // Sync
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
    // Event System
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
    // Advanced Sync
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
    // Multi-Agent Sharing
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
    // Scope-based access grants for multi-agent memory sharing
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
// MCP tool definitions by domain.

    ToolDef {
        name: "memory_search_by_image",
        description: "Search memories using an image as the query. Uses multimodal embeddings (CLIP-style) or falls back to describing the image via vision model and searching by description. Returns semantically similar memories — text or media — ranked by relevance.",
        schema: r#"{
            "type": "object",
            "properties": {
                "image_path": {"type": "string", "description": "Path to the local image file to use as the search query"},
                "limit": {"type": "integer", "default": 10, "description": "Maximum number of results to return"},
                "min_score": {"type": "number", "minimum": 0, "maximum": 1, "description": "Minimum similarity score (0.0-1.0)"},
                "workspace": {"type": "string", "description": "Restrict search to a specific workspace"},
                "strategy": {"type": "string", "enum": ["clip", "description", "auto"], "default": "auto", "description": "Embedding strategy: clip (requires CLIP embedder), description (vision model + text embedding), auto (use CLIP if available, else description)"}
            },
            "required": ["image_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Image Handling
    ToolDef {
        name: "memory_upload_image",
        description: "Upload an image file and attach it to a memory. The image will be stored locally and linked to the memory's metadata.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "ID of the memory to attach the image to"},
                "file_path": {"type": "string", "description": "Path to the image file to upload"},
                "image_index": {"type": "integer", "default": 0, "description": "Index for ordering multiple images (0-based)"},
                "caption": {"type": "string", "description": "Optional caption for the image"}
            },
            "required": ["memory_id", "file_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_migrate_images",
        description: "Migrate existing base64-encoded images in memories to file storage. Scans all memories and uploads any embedded data URIs to storage, replacing them with file references.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": false, "description": "If true, only report what would be migrated without making changes"}
            }
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_capture_screenshot",
        description: "Capture a screenshot of the full screen or a specific application window and save it to a local file.",
        schema: r#"{
            "type": "object",
            "properties": {
                "app_name": {"type": "string", "description": "Name of the application window to capture; omit to capture the full screen"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_describe_image",
        description: "Describe the contents of an image file using the configured vision provider (requires VISION_PROVIDER env).",
        schema: r#"{
            "type": "object",
            "properties": {
                "image_path": {
                    "type": "string",
                    "description": "Absolute filesystem path to the image file (JPEG, PNG, WebP, etc.)."
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional custom prompt to guide the image description."
                }
            },
            "required": ["image_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // 9. memory_list_media
    ToolDef {
        name: "memory_list_media",
        description: "List media assets stored in the media_assets table, optionally filtered by type (image, audio, video).",
        schema: r#"{
            "type": "object",
            "properties": {
                "media_type": {"type": "string", "description": "Filter by media type: \"image\", \"audio\", or \"video\". Omit for all types."},
                "limit": {"type": "integer", "description": "Maximum number of assets to return (default: 50)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 10. memory_process_video
    ToolDef {
        name: "memory_process_video",
        description: "Process a video file: extract metadata and keyframe descriptions via the configured vision provider, and create a memory record for the result.",
        schema: r#"{
            "type": "object",
            "properties": {
                "video_path": {"type": "string", "description": "Absolute path to the video file to process."}
            },
            "required": ["video_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // ── multimodal.rs ────────────────────────────────────────────────────────
    ToolDef {
        name: "memory_transcribe_audio",
        description: "Transcribe an audio file to text using the configured audio transcription provider.",
        schema: r#"{
            "type": "object",
            "properties": {
                "audio_path": {
                    "type": "string",
                    "description": "Absolute or relative path to the audio file to transcribe."
                }
            },
            "required": ["audio_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
// MCP tool definitions by domain.

    // ── Agent Registry ────────────────────────────────────────────────────
    ToolDef {
        name: "agent_register",
        description: "Register an AI agent with capabilities and namespace isolation. Upserts if agent_id already exists.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Unique identifier for the agent"},
                "display_name": {"type": "string", "description": "Human-readable name (defaults to agent_id)"},
                "capabilities": {"type": "array", "items": {"type": "string"}, "description": "List of capabilities (e.g., 'search', 'create', 'analyze')"},
                "namespaces": {"type": "array", "items": {"type": "string"}, "description": "Namespaces the agent operates in (default: ['default'])"},
                "metadata": {"type": "object", "description": "Additional metadata as key-value pairs"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "agent_deregister",
        description: "Deregister an AI agent (soft delete — sets status to 'inactive').",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "ID of the agent to deregister"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "agent_heartbeat",
        description: "Update an agent's heartbeat timestamp to indicate it is still alive.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "ID of the agent sending heartbeat"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "agent_list",
        description: "List registered agents, optionally filtered by status or namespace.",
        schema: r#"{
            "type": "object",
            "properties": {
                "status": {"type": "string", "enum": ["active", "inactive"], "description": "Filter by agent status"},
                "namespace": {"type": "string", "description": "Filter by namespace (returns agents that include this namespace)"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "agent_get",
        description: "Get details of a specific registered agent by ID.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "ID of the agent to retrieve"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "agent_capabilities",
        description: "Update the capabilities list of a registered agent.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "ID of the agent to update"},
                "capabilities": {"type": "array", "items": {"type": "string"}, "description": "New capabilities list (replaces existing)"}
            },
            "required": ["agent_id", "capabilities"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // ── Snapshot Tools (snapshot) ────────────────────────────────────
    ToolDef {
        name: "snapshot_create",
        description: "Create a portable .egm snapshot of memories filtered by workspace, tags, date range, or importance. Optionally encrypt with AES-256-GCM or sign with Ed25519.",
        schema: r#"{
            "type": "object",
            "properties": {
                "output_path": {"type": "string", "description": "File path for the .egm snapshot"},
                "workspace": {"type": "string", "description": "Filter by workspace"},
                "tags": {"type": "array", "items": {"type": "string"}, "description": "Filter by tags"},
                "importance_min": {"type": "number", "description": "Minimum importance score"},
                "memory_types": {"type": "array", "items": {"type": "string"}, "description": "Filter by memory types"},
                "description": {"type": "string", "description": "Human-readable description"},
                "creator": {"type": "string", "description": "Creator name"},
                "encrypt_key": {"type": "string", "description": "Hex-encoded 32-byte AES key"},
                "sign_key": {"type": "string", "description": "Hex-encoded 32-byte Ed25519 secret key"}
            },
            "required": ["output_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "snapshot_load",
        description: "Load a .egm snapshot into the memory store. Strategies: merge (skip duplicates), replace (clear workspace first), isolate (new workspace), dry_run (preview only).",
        schema: r#"{
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to .egm file"},
                "strategy": {"type": "string", "enum": ["merge", "replace", "isolate", "dry_run"], "description": "Load strategy"},
                "target_workspace": {"type": "string", "description": "Target workspace (defaults to snapshot's workspace)"},
                "decrypt_key": {"type": "string", "description": "Hex-encoded 32-byte AES key for encrypted snapshots"}
            },
            "required": ["path", "strategy"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "snapshot_inspect",
        description: "Inspect a .egm snapshot without loading it. Returns manifest, file list, and size.",
        schema: r#"{
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to .egm file"}
            },
            "required": ["path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // ── Attestation Tools (attestation) ──────────────────────────────────
    ToolDef {
        name: "attestation_log",
        description: "Log a document ingestion with cryptographic attestation. Creates a chained record proving the document was processed.",
        schema: r#"{
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "Document content to attest"},
                "document_name": {"type": "string", "description": "Name of the document"},
                "agent_id": {"type": "string", "description": "ID of the attesting agent"},
                "memory_ids": {"type": "array", "items": {"type": "integer"}, "description": "IDs of memories created from this document"},
                "sign_key": {"type": "string", "description": "Hex-encoded 32-byte Ed25519 secret key"}
            },
            "required": ["content", "document_name"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "attestation_verify",
        description: "Verify whether a document has been attested (ingested and recorded).",
        schema: r#"{
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "Document content to verify"}
            },
            "required": ["content"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "attestation_chain_verify",
        description: "Verify the integrity of the entire attestation chain. Returns valid, broken (with location), or empty.",
        schema: r#"{
            "type": "object",
            "properties": {
                "verifying_key": {
                    "type": "string",
                    "description": "Hex-encoded 32-byte Ed25519 verifying key. When provided, every record must carry a valid signature; missing or invalid signatures cause the chain to report as Broken. When omitted, only hash-chain integrity is verified."
                }
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "attestation_list",
        description: "List attestation records with optional filters. Supports JSON, CSV, and Merkle proof export formats.",
        schema: r#"{
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "description": "Maximum records to return", "default": 50},
                "offset": {"type": "integer", "description": "Number of records to skip", "default": 0},
                "agent_id": {"type": "string", "description": "Filter by agent ID"},
                "document_name": {"type": "string", "description": "Filter by document name"},
                "export_format": {"type": "string", "enum": ["json", "csv", "merkle_proof"], "description": "Export format"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // === Reconciliation: previously-unadvertised tools (dispatch/registry parity) ===
    // -- reconciliation batch A --
    ToolDef {
        name: "memory_agent_start",
        description: "Configure a tick-based memory agent for a workspace and return its initial configuration.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace the agent will operate on (default: \"default\")"},
                "interval_secs": {"type": "integer", "description": "Desired check interval in seconds (default: 300)"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_agent_stop",
        description: "Stop a tick-based memory agent (no-op for stateless agents; resets client-side tracking).",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace whose agent should be stopped (default: \"default\")"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_agent_status",
        description: "Return current status and memory statistics for a workspace agent.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to report status for (default: \"default\")"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_agent_metrics",
        description: "Run one full agent cycle (prune/merge/archive) and return the actions taken and aggregate metrics. Mutates the database.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to run the agent cycle on (default: \"default\")"},
                "max_actions": {"type": "integer", "description": "Maximum number of actions to take in this cycle (default: 10)"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
// MCP tool definitions by domain.

    // ── Harness Memory (cross-session continuity) ────────────────────────────
    ToolDef {
        name: "harness_record",
        description: "Record a durable harness event (decision, handoff, failed_attempt, verification_result, risk, assumption, bug_reproduction, issue_update) with structured metadata for cross-session continuity. Use instead of memory_create when capturing work-state evidence rather than facts.",
        schema: r#"{
            "type": "object",
            "properties": {
                "kind": {"type": "string", "enum": ["decision", "handoff", "failed_attempt", "bug_reproduction", "verification_result", "risk", "assumption", "issue_update"], "description": "The harness event kind"},
                "summary": {"type": "string", "maxLength": 500, "description": "Concise summary of the event (1-500 chars)"},
                "details": {"type": "string", "maxLength": 8000, "description": "Optional additional context appended to the summary"},
                "source_paths": {"type": "array", "items": {"type": "string"}, "description": "File paths relevant to this event"},
                "command": {"type": "string", "description": "CLI/shell command that produced this evidence"},
                "issue_number": {"type": "integer", "description": "Related GitHub issue number"},
                "commit_sha": {"type": "string", "description": "Related git commit SHA"},
                "evidence_refs": {"type": "array", "items": {"type": "string"}, "description": "Free-form references (URLs, paths, IDs)"},
                "importance": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.7, "description": "Importance score (0-1)"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"}
            },
            "required": ["kind", "summary"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "harness_status",
        description: "Assemble current project state from harness memory records and optional git state. Returns current objective, active issues, recent decisions, known blockers, last verification, last handoff, and a suggested next action. Token-budget aware; degrades gracefully when git is unavailable.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"},
                "max_records": {"type": "integer", "minimum": 1, "maximum": 50, "default": 10, "description": "Max recent harness records to include"},
                "token_budget": {"type": "integer", "default": 2000, "description": "Approximate max tokens for the output (chars/4 heuristic)"},
                "include_git": {"type": "boolean", "default": true, "description": "Attempt to collect git branch/status/log state"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "harness_handoff",
        description: "Generate a structured handoff packet for next-agent continuity: current goal, files touched, decisions, tests run/not run, risks, blockers, and next steps. Optionally persists as a harness record. Does NOT claim completion unless verification_evidence is provided.",
        schema: r#"{
            "type": "object",
            "properties": {
                "current_goal": {"type": "string", "maxLength": 300, "description": "What the agent was working toward"},
                "files_touched": {"type": "array", "items": {"type": "string"}, "description": "Paths modified this session"},
                "decisions_made": {"type": "array", "items": {"type": "string"}, "description": "Short decision summaries"},
                "tests_run": {"type": "array", "items": {"type": "string"}, "description": "Test commands/names that were run"},
                "tests_not_run": {"type": "array", "items": {"type": "string"}, "description": "Tests known to be missing or skipped"},
                "known_risks": {"type": "array", "items": {"type": "string"}, "description": "Open risks"},
                "blockers": {"type": "array", "items": {"type": "string"}, "description": "Things blocking progress"},
                "next_steps": {"type": "array", "items": {"type": "string"}, "minItems": 1, "description": "Recommended actions for the next agent"},
                "issue_numbers": {"type": "array", "items": {"type": "integer"}, "description": "Related GitHub issue numbers"},
                "plan_doc_paths": {"type": "array", "items": {"type": "string"}, "description": "Paths to relevant plan docs"},
                "verification_evidence": {"type": "string", "description": "Evidence that work is complete (test count, command output summary)"},
                "persist": {"type": "boolean", "default": true, "description": "Persist the handoff as a harness record"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"}
            },
            "required": ["current_goal", "next_steps"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "harness_verify",
        description: "Record a verification command outcome with exit code, output summary, and optional evidence path/hash. Supports negative evidence (failures, skips with reason). Surfaces in harness_status as last_verification and feeds harness_handoff completion gating.",
        schema: r#"{
            "type": "object",
            "properties": {
                "command": {"type": "string", "maxLength": 200, "description": "The command that was run (e.g. 'cargo test --lib')"},
                "exit_code": {"type": "integer", "description": "Process exit code (0 = success)"},
                "passed": {"type": "boolean", "description": "Explicit pass/fail; derived from exit_code == 0 if omitted"},
                "output_summary": {"type": "string", "maxLength": 500, "description": "Concise summary (e.g. '873 tests passed, 0 failed')"},
                "evidence_path": {"type": "string", "description": "Path to the full output file or log"},
                "evidence_hash": {"type": "string", "description": "SHA-256 of the full output for integrity"},
                "skipped_reason": {"type": "string", "description": "If skipped, why (negative evidence)"},
                "issue_numbers": {"type": "array", "items": {"type": "integer"}, "description": "Linked GitHub issues"},
                "memory_ids": {"type": "array", "items": {"type": "integer"}, "description": "Linked memory IDs"},
                "importance": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.8, "description": "Importance score (0-1)"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"}
            },
            "required": ["command", "exit_code", "output_summary"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
// MCP tool definitions by domain.

    // Auto-Tagging
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
    // Session Context Tools (ENG-70, ENG-71)
    ToolDef {
        name: "session_context_create",
        description: "Create a new session context for tracking related memories during a conversation or task.",
        schema: r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Session name"},
                "description": {"type": "string", "description": "Session description"},
                "workspace": {"type": "string", "description": "Workspace for the session"},
                "metadata": {"type": "object", "description": "Additional session metadata"}
            },
            "required": ["name"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "session_context_add_memory",
        description: "Add a memory to a session context with relevance score and role.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID"},
                "memory_id": {"type": "integer", "description": "Memory ID to add"},
                "relevance_score": {"type": "number", "minimum": 0, "maximum": 1, "default": 1.0, "description": "How relevant this memory is to the session"},
                "context_role": {"type": "string", "enum": ["referenced", "created", "updated", "pinned"], "default": "referenced", "description": "Role of the memory in the session"}
            },
            "required": ["session_id", "memory_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "session_context_remove_memory",
        description: "Remove a memory from a session context.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID"},
                "memory_id": {"type": "integer", "description": "Memory ID to remove"}
            },
            "required": ["session_id", "memory_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "session_context_get",
        description: "Get a session context with its linked memories.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "session_context_list",
        description: "List all session contexts with optional filtering.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Filter by workspace"},
                "active_only": {"type": "boolean", "default": false, "description": "Only return active sessions"},
                "limit": {"type": "integer", "default": 50, "description": "Maximum sessions to return"},
                "offset": {"type": "integer", "default": 0, "description": "Offset for pagination"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "session_context_search",
        description: "Search memories within a specific session context.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID to search within"},
                "query": {"type": "string", "description": "Search query"},
                "limit": {"type": "integer", "default": 20, "description": "Maximum results"}
            },
            "required": ["session_id", "query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "session_context_update_summary",
        description: "Update the summary of a session context.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID"},
                "summary": {"type": "string", "description": "New session summary"}
            },
            "required": ["session_id", "summary"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "session_context_end",
        description: "End a session context, marking it as inactive.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID to end"},
                "summary": {"type": "string", "description": "Optional final summary"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "session_context_export",
        description: "Export a session context with all its memories for archival or sharing.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session ID to export"},
                "include_content": {"type": "boolean", "default": true, "description": "Include full memory content"},
                "format": {"type": "string", "enum": ["json", "markdown"], "default": "json", "description": "Export format"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // ── Claude-Mem Parity (v0.14.0) ──────────────────────────────────────────
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
        name: "memory_get_injection_prompt",
        description: "Assembles the most relevant memories into a ready-to-inject system prompt block. Uses hybrid search to find relevant memories and formats them as markdown, respecting a token budget.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query to find relevant memories"},
                "token_budget": {"type": "integer", "description": "Max tokens for output (default: 2000)"},
                "workspace": {"type": "string", "description": "Filter to specific workspace"},
                "include_types": {"type": "array", "items": {"type": "string"}, "description": "Filter by memory types"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_observe_tool_use",
        description: "Store a tool observation as an episodic memory for session continuity. Automatically compresses large inputs/outputs.",
        schema: r#"{
            "type": "object",
            "properties": {
                "tool_name": {"type": "string", "description": "Name of the tool that was used"},
                "tool_input": {"type": "object", "description": "Tool input parameters"},
                "tool_output": {"type": "string", "description": "Tool output/result"},
                "session_id": {"type": "string", "description": "Session identifier for grouping observations"},
                "compress": {"type": "boolean", "description": "Compress to 200-char previews (default: true)"}
            },
            "required": ["tool_name", "tool_input", "tool_output"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // ── Endless Mode (O(N) context management) ───────────────────────────────
    ToolDef {
        name: "memory_archive_tool_output",
        description: "Archives a tool's full raw output to memory and returns a compressed summary (~500 tokens) for use in the active context. Transforms O(N²) context growth to O(N) by keeping only summaries in the working context while preserving full outputs for on-demand retrieval.",
        schema: r#"{
            "type": "object",
            "properties": {
                "tool_name": {"type": "string", "description": "Name of the tool whose output is being archived"},
                "raw_output": {"type": "string", "description": "Full raw output to archive"},
                "session_id": {"type": "string", "description": "Session identifier for grouping archived outputs (default: 'unknown')"},
                "compress_summary": {"type": "boolean", "description": "Whether to generate a compressed summary (default: true)"},
                "summary_tokens": {"type": "integer", "description": "Max tokens for the compressed summary (default: 500)"}
            },
            "required": ["tool_name", "raw_output"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_get_archived_output",
        description: "Retrieves the full raw output for an archived tool observation by its archive ID. Use when you need the complete output that was previously compressed for context efficiency.",
        schema: r#"{
            "type": "object",
            "properties": {
                "archive_id": {"type": "integer", "description": "Archive ID returned by memory_archive_tool_output"}
            },
            "required": ["archive_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_get_working_memory",
        description: "Assembles all compressed tool observations for a session into a token-budgeted working memory block. Includes archive references for retrieving full outputs on demand. This is the core of the Endless Mode context management system.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session identifier to retrieve observations for"},
                "token_budget": {"type": "integer", "description": "Max tokens for the working memory block (default: 4000)"},
                "include_tool_names": {"type": "array", "items": {"type": "string"}, "description": "Whitelist of tool names to include (default: all)"},
                "since_minutes": {"type": "integer", "description": "Only include observations from the last N minutes (default: all time)"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // ── Session Handoff ────────────────────────────────────────────────
    ToolDef {
        name: "session_land",
        description: "Generate a structured session handoff ('land the plane'). Creates a checkpoint memory with session summary, open items, recent decisions, and a bootstrap prompt for the next session. Call this at the end of every work session for seamless continuity.",
        schema: r#"{
            "type": "object",
            "properties": {
                "session_id": {"type": "string", "description": "Session identifier to hand off"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"},
                "summary": {"type": "string", "description": "Summary of what was accomplished this session"},
                "next_session_hints": {"type": "array", "items": {"type": "string"}, "description": "Hints for what should be done next session"}
            },
            "required": ["session_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_build_context",
        description: "Build a structured prompt context from relevant memories using hybrid search, with optional graph traversal depth, timeframe filtering, type filtering, and relationship graph inclusion. Inspired by Basic Memory's build_context.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query to retrieve relevant memories"},
                "total_budget": {"type": "integer", "description": "Max tokens for the entire prompt (default: 4096)"},
                "strategy": {"type": "string", "enum": ["greedy", "balanced", "recency"], "default": "greedy", "description": "Context assembly strategy"},
                "workspace": {"type": "string", "description": "Workspace to search in"},
                "limit": {"type": "integer", "description": "Max memories to retrieve (default: 20)"},
                "depth": {"type": "integer", "minimum": 1, "maximum": 3, "default": 1, "description": "Graph traversal depth: 1=search only, 2=search+1 hop of related memories, 3=search+2 hops"},
                "timeframe": {"type": "string", "enum": ["1h", "24h", "7d", "30d", "all"], "default": "all", "description": "Time window for memory filtering"},
                "include_types": {"type": "array", "items": {"type": "string"}, "description": "Only include these memory types (e.g., ['note', 'decision'])"},
                "include_graph": {"type": "boolean", "default": false, "description": "Include entity relationship graph in response"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "context_record",
        description: "Record a scoped Operational Context event and optional derived summary. Redacts text before storage, requires provenance scope, keeps raw payload storage off, and supports RTK-compatible external summary metadata without dereferencing external pointers.",
        schema: r#"{
            "type": "object",
            "properties": {
                "source": {"type": "string", "description": "Source system or adapter, for example codex, harness, rtk, github_actions"},
                "source_version": {"type": "string", "description": "Optional source or adapter version"},
                "repo_id": {"type": "string", "description": "Repository scope identifier, for example github:aiconnai/engram"},
                "workspace_path_hash": {"type": "string", "description": "Workspace path hash scope"},
                "workspace": {"type": "string", "description": "Alias for workspace_path_hash"},
                "git_branch": {"type": "string", "description": "Branch observed when the event occurred"},
                "worktree_name": {"type": "string", "description": "Worktree name observed when the event occurred"},
                "commit_hash": {"type": "string", "description": "Commit observed when the event occurred"},
                "session_id": {"type": "string", "description": "Agent or user session scope"},
                "task_id": {"type": "string", "description": "Task, issue, or ticket scope"},
                "agent_id": {"type": "string", "description": "Agent identity"},
                "event_type": {"type": "string", "description": "Event family such as command, tool, decision_made, verification_run, verification_skipped, blocker_found, review_result, handoff_created"},
                "command": {"type": "string", "description": "Command line or command name for command events"},
                "command_name": {"type": "string", "description": "Alias/preferred command field"},
                "tool": {"type": "string", "description": "Tool name for tool events"},
                "tool_name": {"type": "string", "description": "Alias/preferred tool field"},
                "cwd": {"type": "string", "description": "Working directory context"},
                "exit_code": {"type": "integer", "description": "Command exit code; required for event_type=command"},
                "summary": {"type": "string", "description": "Optional derived/lossy summary to store with provenance"},
                "key_errors": {"type": "array", "items": {"type": "string"}, "description": "Important errors to index after redaction"},
                "touched_files": {"type": "array", "items": {"type": "string"}, "description": "Files inspected or changed by the event"},
                "reducer": {"type": "object", "description": "Optional reducer metadata: name, version, lossy, confidence, structured_facts, warnings, labels, tokens_raw_est, tokens_compact_est"},
                "external_reducer": {"type": "string", "description": "External reducer name for RTK-compatible summaries"},
                "raw_pointer": {"type": "string", "description": "External raw pointer recorded as metadata only; Engram does not dereference it"},
                "external_unverified": {"type": "boolean", "description": "Mark external summary as unverified"},
                "labels": {"type": "array", "items": {"type": "string"}, "description": "Additional labels; external records add derived/lossy/external_unverified conservatively"},
                "retention_policy": {"type": "string", "description": "Retention label; sensitive commands may be forced to ephemeral_sensitive by policy"},
                "raw_artifact_id": {"type": "string", "description": "Optional existing artifact id pointer; raw payload content is not accepted by this tool"},
                "metadata": {"type": "object", "description": "Additional metadata redacted recursively before storage"},
                "started_at": {"type": "string", "description": "RFC3339 event start time; defaults to now"},
                "finished_at": {"type": "string", "description": "RFC3339 event finish time"}
            },
            "required": ["source", "session_id", "event_type"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "context_record_artifact",
        description: "Record an Operational Context artifact pointer or explicitly retained redacted raw artifact. Pointer-only is the default; raw_content requires retain_raw=true and policy approval.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Optional artifact id; generated when omitted"},
                "source_event_id": {"type": "integer", "description": "Related context event id"},
                "source": {"type": "string", "description": "Source system or adapter"},
                "source_version": {"type": "string", "description": "Optional source or adapter version"},
                "repo_id": {"type": "string", "description": "Repository scope identifier"},
                "workspace_path_hash": {"type": "string", "description": "Workspace path hash scope"},
                "workspace": {"type": "string", "description": "Alias for workspace_path_hash"},
                "session_id": {"type": "string", "description": "Session scope for access policy"},
                "task_id": {"type": "string", "description": "Task scope for access policy"},
                "agent_id": {"type": "string", "description": "Agent scope for access policy"},
                "kind": {"type": "string", "description": "Artifact type, for example command_output_summary, raw_command_output, review_artifact, diff_reference, test_report, external_url"},
                "label": {"type": "string", "description": "Human label"},
                "uri": {"type": "string", "description": "External/source-of-truth pointer"},
                "raw_pointer": {"type": "string", "description": "Alias pointer stored as uri/metadata only; not dereferenced"},
                "media_type": {"type": "string", "description": "Media type for pointer or raw content"},
                "raw_content": {"type": "string", "description": "Raw content to retain only when retain_raw=true and policy allows it"},
                "content_sha256": {"type": "string", "description": "Optional digest for pointer-only artifacts; raw digests are recomputed after redaction"},
                "byte_len": {"type": "integer", "description": "Optional source byte length for pointer-only artifacts"},
                "retention_policy": {"type": "string", "description": "Retention label, default pointer_only or raw_retained"},
                "access_policy": {"type": "string", "enum": ["same_session", "same_task", "same_agent", "repo", "public"], "default": "same_session"},
                "retain_raw": {"type": "boolean", "default": false, "description": "Must be true to persist raw_content"},
                "ttl_seconds": {"type": "integer", "description": "Optional raw retention TTL from now"},
                "stale_after_seconds": {"type": "integer", "description": "Optional stale threshold from now"},
                "metadata": {"type": "object", "description": "Additional metadata redacted recursively before storage"}
            },
            "required": ["kind"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "context_get_artifact",
        description: "Explicitly retrieve retained Operational Context artifact content after access, retention, staleness, and redaction checks. Search and bundle tools return artifact pointers only; this tool requires an artifact_id and reason.",
        schema: r#"{
            "type": "object",
            "properties": {
                "artifact_id": {"type": "string", "description": "Artifact ID to retrieve; broad search queries are not accepted"},
                "reason": {"type": "string", "description": "Why raw or retained artifact content is needed"},
                "requester_agent_id": {"type": "string", "description": "Agent identity used for same_agent access checks"},
                "session_id": {"type": "string", "description": "Session scope used for same_session access checks"},
                "task_id": {"type": "string", "description": "Task scope used for same_task access checks"},
                "repo_id": {"type": "string", "description": "Repository scope used for repo access checks"},
                "workspace_path_hash": {"type": "string", "description": "Workspace path hash scope used for repo/workspace access checks"},
                "workspace": {"type": "string", "description": "Alias for workspace_path_hash"},
                "max_bytes": {"type": "integer", "minimum": 1, "description": "Maximum raw bytes to return; response marks truncation explicitly"},
                "allow_stale": {"type": "boolean", "default": false, "description": "Allow retrieval after stale_at has passed"},
                "require_redacted": {"type": "boolean", "default": true, "description": "Require a redaction status that permits raw retrieval"}
            },
            "required": ["artifact_id", "reason"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "context_search",
        description: "Search scoped Operational Context events and derived summaries. Searches event metadata, command/tool names, summaries, structured facts, failure signals, decisions, inspected/touched file metadata, and artifact pointers without returning raw artifact content.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query for operational events, summaries, decisions, failures, files, or artifact metadata"},
                "repo_id": {"type": "string", "description": "Repository scope identifier, for example github:aiconnai/engram"},
                "workspace_path_hash": {"type": "string", "description": "Workspace path hash scope"},
                "workspace": {"type": "string", "description": "Alias for workspace_path_hash when clients only have a workspace scope value"},
                "session_id": {"type": "string", "description": "Session scope filter"},
                "task_id": {"type": "string", "description": "Task scope filter"},
                "event_type": {"type": "string", "description": "Restrict results to one event type"},
                "event_types": {"type": "array", "items": {"type": "string"}, "description": "Restrict results to these event types"},
                "event_type_filters": {"type": "array", "items": {"type": "string"}, "description": "Alias for event_types"},
                "failure_only": {"type": "boolean", "default": false, "description": "Only include failures/errors inferred from exit_code or event_type"},
                "max_results": {"type": "integer", "minimum": 1, "maximum": 200, "default": 25, "description": "Maximum results to return"},
                "include_artifact_pointers": {"type": "boolean", "default": false, "description": "Include artifact IDs/pointers only; raw artifact content is never returned"},
                "current_git_branch": {"type": "string", "description": "Current branch used to mark branch mismatch staleness"},
                "current_commit_hash": {"type": "string", "description": "Current commit used to mark commit mismatch staleness"},
                "stale_after_days": {"type": "integer", "default": 7, "description": "Age threshold for stale warnings"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "context_build_bundle",
        description: "Build a compact agent-ready Operational Context bundle for resuming work. Includes relevant failures, inferred unresolved blockers, recent decisions, commands already run, inspected/touched files, staleness warnings, and optional artifact pointers with provenance for every item. Does not include raw artifact content.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Optional query describing the work to resume"},
                "repo_id": {"type": "string", "description": "Repository scope identifier, for example github:aiconnai/engram"},
                "workspace_path_hash": {"type": "string", "description": "Workspace path hash scope"},
                "workspace": {"type": "string", "description": "Alias for workspace_path_hash when clients only have a workspace scope value"},
                "session_id": {"type": "string", "description": "Session scope filter"},
                "task_id": {"type": "string", "description": "Task scope filter"},
                "max_results": {"type": "integer", "minimum": 1, "maximum": 200, "default": 80, "description": "Maximum operational context rows to inspect"},
                "section_limit": {"type": "integer", "minimum": 1, "maximum": 50, "default": 12, "description": "Maximum entries per bundle section"},
                "include_artifact_pointers": {"type": "boolean", "default": false, "description": "Include artifact IDs/pointers only; raw artifact content is never returned"},
                "current_git_branch": {"type": "string", "description": "Current branch used to mark branch mismatch staleness"},
                "current_commit_hash": {"type": "string", "description": "Current commit used to mark commit mismatch staleness"},
                "stale_after_days": {"type": "integer", "default": 7, "description": "Age threshold for stale warnings"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "recent_activity",
        description: "Discover recently created or updated memories. Returns compact previews sorted by most recent activity. Useful for understanding what has changed recently.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Filter by workspace (omit for all workspaces)"},
                "timeframe": {"type": "string", "enum": ["1h", "24h", "7d", "30d"], "default": "24h", "description": "Time window for activity"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 100, "default": 20, "description": "Max results to return"},
                "include_types": {"type": "array", "items": {"type": "string"}, "description": "Only include these memory types"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    // ── Meta / Discovery ─────────────────────────────────────────────────────
    ToolDef {
        name: "discover_tools",
        description: "List Engram tools by tier, group, or search query. Includes feature-disabled tools with enablement hints so agents can progressively discover capabilities beyond the small default tools/list surface.",
        schema: r#"{
            "type": "object",
            "properties": {
                "tier": {"type": "string", "enum": ["essential", "standard", "advanced", "all"], "default": "all", "description": "Filter by tier: essential (small first-connect surface), standard (common workflows), advanced (specialized tools), all (everything)"},
                "group": {"type": "string", "description": "Filter by structured group (e.g., 'memory.search', 'context', 'identity', 'feature.attestation')"},
                "category": {"type": "string", "description": "Deprecated alias for group/search-style filtering."},
                "search": {"type": "string", "description": "Search tool names and descriptions"},
                "detail": {"type": "string", "enum": ["names", "summary", "schema"], "default": "summary", "description": "Per-tool detail level: 'names' (name only, cheapest), 'summary' (name + description + tier + group + availability + feature hints, the default), or 'schema' (summary plus the full input schema as a JSON object, so the tool can be called without a separate tools/list round-trip)."}
            }
        }"#,
        tier: ToolTier::Essential,
        annotations: ToolAnnotations::read_only(),
    },
    // RTK-Inspired Context Preparation
    ToolDef {
        name: "memory_prepare_context",
        description: "Prepare optimized context for LLM using RTK-inspired pipeline (filter, group, truncate). Reduces token usage by 70-95% through intelligent context preparation.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Query to prepare context for"},
                "budget": {"type": "integer", "default": 4000, "description": "Token budget for prepared context"},
                "workspace": {"type": "string", "description": "Optional workspace filter"}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_extract_facts",
        description: "Extract subject-predicate-object facts from a memory's content using rule-based NLP and persist them to the facts table.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "integer",
                    "description": "ID of the memory from which to extract and store facts."
                }
            },
            "required": ["memory_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // ── temporal.rs — scope handlers ─────────────────────────────────────────
    ToolDef {
        name: "scope_get",
        description: "Return the current scope path and level for a given memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "integer",
                    "description": "ID of the memory whose scope to retrieve."
                }
            },
            "required": ["memory_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "scope_list",
        description: "List all distinct scope paths currently present in the database.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "scope_search",
        description: "Search for memories whose content matches a query within a given scope, including ancestor scopes.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Substring to search for within scoped memories."
                },
                "scope_path": {
                    "type": "string",
                    "description": "Hierarchical scope path to search within (e.g. \"global/org:acme/user:alice\")."
                }
            },
            "required": ["query", "scope_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "scope_set",
        description: "Assign or update the hierarchical scope of a memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "integer",
                    "description": "ID of the memory to re-scope."
                },
                "scope_path": {
                    "type": "string",
                    "description": "Target scope path (e.g. \"global/org:acme/user:alice\")."
                }
            },
            "required": ["memory_id", "scope_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "scope_tree",
        description: "Return a hierarchical tree of all scopes in the database.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
// MCP tool definitions by domain.

    // Content Utilities
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
    // Import/Export
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
    // Special Memory Types
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
    // Phase 1: Cognitive Memory Types (ENG-33)
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
        name: "memory_boost",
        description: "Temporarily boost a memory's importance score. The boost can optionally decay over time.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to boost"},
                "boost_amount": {"type": "number", "default": 0.2, "description": "Amount to increase importance (0-1)"},
                "duration_seconds": {"type": "integer", "description": "Optional: duration before boost decays (omit for permanent boost)"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_explain_utility",
        description: "Explain why a memory has its current utility score. Returns the full feedback history summary (useful vs. not-useful retrievals), how much temporal decay has been applied, and a plain-English narrative. Useful for debugging or auditing memory quality.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "ID of the memory to explain"}
            },
            "required": ["memory_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // Phase 2: Context Compression Engine
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
        name: "memory_detect_conflicts",
        description: "Detect contradictory or conflicting facts in the knowledge graph; optionally persist detected conflicts for later resolution.",
        schema: r#"{
            "type": "object",
            "properties": {
                "save": {
                    "type": "boolean",
                    "description": "If true, persist detected conflicts to the conflicts table for later resolution (default: false)."
                }
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_feedback",
        description: "Record relevance feedback for a search result and update the memory's utility score; schedules low-utility memories for consolidation.",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query that produced the result."
                },
                "memory_id": {
                    "type": "integer",
                    "description": "ID of the memory being rated."
                },
                "signal": {
                    "type": "string",
                    "description": "Feedback signal: \"useful\" (alias \"helpful\"), \"irrelevant\" (alias \"not_helpful\"), \"outdated\", or \"conflict\"."
                },
                "rank_position": {
                    "type": "integer",
                    "description": "0-based rank position of the result in the original result list (optional)."
                },
                "original_score": {
                    "type": "number",
                    "description": "The final_score from the original search result (optional)."
                },
                "workspace": {
                    "type": "string",
                    "description": "Workspace context for the feedback (default: \"default\")."
                }
            },
            "required": ["query", "memory_id", "signal"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // -- reconciliation batch C --
    ToolDef {
        name: "memory_feedback_stats",
        description: "Return aggregated search-feedback statistics (thumbs-up/down counts, top-rated queries) for a workspace.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace name to filter stats; omit for all workspaces."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // ── compression.rs ───────────────────────────────────────────────────────
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
    // ── evolution.rs ─────────────────────────────────────────────────────────
    ToolDef {
        name: "memory_utility_score",
        description: "Compute the Q-value utility score for a memory based on its retrieval feedback history.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {
                    "type": "integer",
                    "description": "Memory ID to score."
                }
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
// MCP tool definitions by domain.

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
]
