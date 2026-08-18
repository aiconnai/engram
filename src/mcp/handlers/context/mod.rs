//! Context-engineering and memory-block tool handlers (Round 3 — T8/T9/T10).
//!
//! Covers:
//! - Fact extraction from memory content (SPO triples)
//! - Fact retrieval and subject graphs
//! - Prompt-context assembly via ContextBuilder
//! - Self-editing memory blocks (Letta/MemGPT-style)

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Truncate `s` to at most `max_bytes` bytes, always landing on a valid UTF-8
/// char boundary. Avoids panics on multibyte (emoji, CJK, accented) input.
fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &s[..boundary]
}

// ── Operational Context retrieval ────────────────────────────────────────────

/// Record a command/tool/action event into Operational Context.
mod assembly;
mod blocks;
mod events;
mod facts;
mod tool_output;

pub use assembly::{memory_build_context, memory_get_injection_prompt, memory_prepare_context};
pub use blocks::{
    memory_block_archive, memory_block_create, memory_block_edit, memory_block_get,
    memory_block_history, memory_block_list,
};
pub use events::{
    context_build_bundle, context_get_artifact, context_record, context_record_artifact,
    context_search,
};
pub use facts::{memory_extract_facts, memory_fact_graph, memory_list_facts};
pub use tool_output::{
    memory_archive_tool_output, memory_get_archived_output, memory_get_working_memory,
    memory_observe_tool_use,
};

#[cfg(test)]
mod context_tests {
    use super::safe_truncate;

    // ── Helper: build a HandlerContext with in-memory storage ──────────────
    fn test_ctx() -> super::super::HandlerContext {
        use crate::embedding::{create_embedder, EmbeddingCache};
        use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
        use crate::storage::Storage;
        use crate::types::EmbeddingConfig;
        use parking_lot::Mutex;
        use std::sync::Arc;

        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        super::super::HandlerContext {
            storage,
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(
                    embedder.dimensions(),
                    crate::search::VectorMetric::Cosine,
                ),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
        }
    }

    /// Seed a memory via dispatch and return its id.
    fn seed_memory(ctx: &super::super::HandlerContext, content: &str, mem_type: &str) -> i64 {
        let result = super::super::dispatch(
            ctx,
            "memory_create",
            serde_json::json!({
                "content": content,
                "memory_type": mem_type,
                "workspace": "default"
            }),
        );
        result["id"].as_i64().expect("memory must be created")
    }

    /// Link two memories via dispatch.
    fn link_memories(ctx: &super::super::HandlerContext, from: i64, to: i64) {
        super::super::dispatch(
            ctx,
            "memory_link",
            serde_json::json!({
                "from_id": from,
                "to_id": to,
                "edge_type": "related_to",
                "score": 0.9
            }),
        );
    }

    #[test]
    fn test_build_context_backward_compat() {
        let ctx = test_ctx();
        seed_memory(&ctx, "Rust is a systems programming language", "note");

        let result =
            super::memory_build_context(&ctx, serde_json::json!({"query": "Rust programming"}));

        // Must contain the original fields
        assert!(result.get("prompt").is_some(), "must have prompt");
        assert!(
            result.get("token_estimate").is_some(),
            "must have token_estimate"
        );
        assert!(
            result.get("memories_used").is_some(),
            "must have memories_used"
        );
        assert!(
            result.get("total_budget").is_some(),
            "must have total_budget"
        );

        // Must contain the new default fields
        assert_eq!(result["depth"], 1, "default depth must be 1");
        assert_eq!(
            result["timeframe"], "all",
            "default timeframe must be 'all'"
        );

        // Graph should NOT be present when include_graph is false (default)
        assert!(
            result.get("graph").is_none(),
            "graph should not be present by default"
        );
    }

    #[test]
    fn test_build_context_include_types_filters() {
        let ctx = test_ctx();
        seed_memory(&ctx, "Important decision about architecture", "decision");
        seed_memory(&ctx, "Remember to buy groceries", "note");

        let result = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "architecture decision groceries",
                "include_types": ["decision"]
            }),
        );

        // Only decision-type memories should be included
        let used = result["memories_used"].as_u64().unwrap_or(0);
        // At most 1 memory (the decision one) if search found both
        // Note: search may return 0 results in minimal test FTS setup
        assert!(
            used <= 1,
            "should only include decision-type memories, got {}",
            used
        );
    }

    #[test]
    fn test_build_context_timeframe_filters() {
        let ctx = test_ctx();
        // Create a memory (it will have a current timestamp)
        seed_memory(&ctx, "Recent memory about testing", "note");

        // With timeframe "1h", the memory should be included (just created)
        let result = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "testing",
                "timeframe": "1h"
            }),
        );
        assert_eq!(result["timeframe"], "1h");
        // Should still find the memory since it was just created
        let used = result["memories_used"].as_u64().unwrap_or(0);
        assert!(
            used >= 1,
            "recently created memory should be found with 1h timeframe"
        );
    }

    #[test]
    fn test_build_context_depth_expansion() {
        let ctx = test_ctx();
        let id1 = seed_memory(&ctx, "Core concept about neural networks", "note");
        let id2 = seed_memory(&ctx, "Backpropagation algorithm details", "note");
        let _id3 = seed_memory(&ctx, "Gradient descent optimization", "note");

        // Link id1 -> id2, id2 -> id3
        link_memories(&ctx, id1, id2);
        link_memories(&ctx, id2, _id3);

        // With depth=2, should find search results + 1 hop of related
        let result = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "neural networks",
                "depth": 2
            }),
        );
        assert_eq!(result["depth"], 2);
        // Should have more memories than depth=1
        let deep_used = result["memories_used"].as_u64().unwrap_or(0);

        let result_shallow = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "neural networks",
                "depth": 1
            }),
        );
        let shallow_used = result_shallow["memories_used"].as_u64().unwrap_or(0);

        // Deep should find >= shallow (may find related memories)
        assert!(
            deep_used >= shallow_used,
            "depth=2 ({}) should find >= depth=1 ({})",
            deep_used,
            shallow_used
        );
    }

    #[test]
    fn test_build_context_include_graph() {
        let ctx = test_ctx();
        let id1 = seed_memory(&ctx, "Graph data structures", "note");
        let id2 = seed_memory(&ctx, "Adjacency list representation", "note");
        link_memories(&ctx, id1, id2);

        let result = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "graph data structures",
                "include_graph": true
            }),
        );

        // Graph key must be present
        assert!(
            result.get("graph").is_some(),
            "graph must be present when include_graph=true"
        );
        let graph = &result["graph"];
        assert!(graph.get("edges").is_some(), "graph must have edges");
        assert!(
            graph.get("node_count").is_some(),
            "graph must have node_count"
        );
    }

    #[test]
    fn test_build_context_depth_clamped_to_max() {
        let ctx = test_ctx();
        seed_memory(&ctx, "Testing depth clamping", "note");

        let result = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "depth clamping",
                "depth": 10
            }),
        );
        // depth should be clamped to 3
        assert_eq!(result["depth"], 3, "depth should be clamped to max 3");
    }

    // safe_truncate tests
    #[test]
    fn test_safe_truncate_ascii() {
        assert_eq!(safe_truncate("hello world", 5), "hello");
    }

    #[test]
    fn test_safe_truncate_within_limit() {
        assert_eq!(safe_truncate("hi", 100), "hi");
    }

    #[test]
    fn test_safe_truncate_empty() {
        assert_eq!(safe_truncate("", 10), "");
    }

    #[test]
    fn test_safe_truncate_multibyte_emoji() {
        // "😀" is 4 bytes (U+1F600). Truncating at byte 5 should back up to byte 4
        // (the char boundary), not panic.
        let s = "😀hello";
        // 😀 = 4 bytes, 'h' starts at byte 4
        // max_bytes=5 should land at the char boundary at byte 4 (before 'h')
        let result = safe_truncate(s, 5);
        assert!(
            s.is_char_boundary(result.len()),
            "result must end on char boundary"
        );
        assert!(
            !result.contains('\u{FFFD}'),
            "must not produce replacement chars"
        );
    }

    #[test]
    fn test_safe_truncate_multibyte_cjk() {
        // "日" is 3 bytes. Truncating at byte 4 should back up to byte 3.
        let s = "日本語";
        let result = safe_truncate(s, 4);
        assert!(s.is_char_boundary(result.len()));
        // should contain exactly one CJK char ("日") or be empty
        assert!(result == "日" || result.is_empty());
    }

    #[test]
    fn test_safe_truncate_exact_boundary() {
        // Exactly at a char boundary should not back up
        let s = "abcdef";
        assert_eq!(safe_truncate(s, 3), "abc");
    }

    #[test]
    fn test_safe_truncate_zero() {
        assert_eq!(safe_truncate("hello", 0), "");
    }
}
