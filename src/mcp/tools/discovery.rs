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
        description: "List available Engram tools by tier and category. Use this to progressively discover capabilities beyond the essential tool set. Returns tool names, descriptions, and tiers.",
        schema: r#"{
            "type": "object",
            "properties": {
                "tier": {"type": "string", "enum": ["essential", "standard", "advanced", "all"], "default": "all", "description": "Filter by tier: essential (~20 core tools), standard (~57 common tools), advanced (~104 specialized tools), all (everything)"},
                "category": {"type": "string", "description": "Filter by category keyword (e.g., 'search', 'graph', 'session', 'identity', 'quality')"},
                "search": {"type": "string", "description": "Search tool names and descriptions"}
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
