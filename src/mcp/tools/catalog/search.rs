//! MCP tool definitions: Search & Retrieval.
//! Hybrid search, semantic retrieval, caching, Meilisearch, and Langfuse integrations.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

pub const TOOLS: &[ToolDef] = &[
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
];
