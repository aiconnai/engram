//! MCP tool definitions: Context & Sessions.
//! Project context, session management, context artifacts, handoff, and working memory.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

pub const TOOLS: &[ToolDef] = &[
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
    ToolDef {
            name: "session_land",
            description: "Generate a structured session handoff ('land the plane'). Creates a checkpoint memory with session summary, open items, recent decisions, warnings, and a copy-ready block for the next session. If session_id is omitted, Engram falls back to the most recent session in the workspace, or returns a workspace-level packet with a warning.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Optional session identifier to hand off. Omit to use the most recent session in the workspace when available."},
                    "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"},
                    "summary": {"type": "string", "description": "Summary of what was accomplished this session"},
                    "next_session_hints": {"type": "array", "items": {"type": "string"}, "description": "Hints for what should be done next session"}
                }
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
];
