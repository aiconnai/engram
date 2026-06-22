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
        tier: ToolTier::Standard,
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
