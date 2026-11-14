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
    crate::text_util::truncate_bytes(s, max_bytes)
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
            principal: None,
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

    #[test]
    fn build_context_depth_expansion_stays_in_the_searched_workspace() {
        use crate::types::{CreateCrossRefInput, EdgeType};

        let ctx = test_ctx();
        let public = seed_memory(&ctx, "Quarterly roadmap for the orchard project", "note");
        let foreign = super::super::dispatch(
            &ctx,
            "memory_create",
            serde_json::json!({
                "content": "zebracrossing payroll figures",
                "memory_type": "note",
                "workspace": "private"
            }),
        )["id"]
            .as_i64()
            .expect("foreign memory must be created");
        ctx.storage
            .with_connection(|conn| {
                crate::storage::queries::create_crossref(
                    conn,
                    &CreateCrossRefInput {
                        from_id: public,
                        to_id: foreign,
                        edge_type: EdgeType::default(),
                        strength: None,
                        source_context: None,
                        pinned: false,
                    },
                )
            })
            .expect("cross-workspace crossref");

        let scoped = super::memory_build_context(
            &ctx,
            serde_json::json!({"query": "orchard roadmap", "workspace": "default", "depth": 3}),
        );
        assert!(
            !scoped.to_string().contains("zebracrossing"),
            "expansion leaked a foreign workspace: {scoped}"
        );

        // An unrestricted caller that searches every workspace still expands.
        let global = super::memory_build_context(
            &ctx,
            serde_json::json!({"query": "orchard roadmap", "depth": 3}),
        );
        assert!(global.to_string().contains("zebracrossing"), "{global}");

        // A restricted principal never receives the foreign memory.
        let mut restricted = test_ctx();
        restricted.storage = ctx.storage.clone();
        restricted.principal = Some(crate::auth::TransportPrincipal::anonymous_loopback());
        let anonymous = super::memory_build_context(
            &restricted,
            serde_json::json!({"query": "orchard roadmap", "depth": 3}),
        );
        assert!(
            !anonymous.to_string().contains("zebracrossing"),
            "{anonymous}"
        );
    }

    #[test]
    fn context_assembly_enforces_workspaces_filters_and_normalization_for_restricted_principals() {
        let ctx = test_ctx();
        seed_memory(&ctx, "Orchard launch checklist public note", "note");
        super::super::dispatch(
            &ctx,
            "memory_create",
            serde_json::json!({
                "content": "Orchard launch checklist secretcapybara",
                "memory_type": "note",
                "workspace": "private"
            }),
        );

        // Uppercase workspace claim normalizes to canonical "default" and matches seeded rows.
        let normalized_upper = super::memory_build_context(
            &ctx,
            serde_json::json!({"query": "Orchard launch checklist", "workspace": "DEFAULT"}),
        );
        assert!(
            normalized_upper["prompt"]
                .as_str()
                .unwrap_or_default()
                .contains("public note"),
            "uppercase workspace must normalize to 'default': {normalized_upper}"
        );
        assert!(
            !normalized_upper.to_string().contains("secretcapybara"),
            "normalized default workspace must exclude private workspace: {normalized_upper}"
        );

        // AnonymousLoopback using `workspaces: ["default"]` or `filters: {"workspace": "default"}`
        // (omitting top-level `workspace`) must stay confined across all 3 context assembly tools.
        let mut anon_ctx = test_ctx();
        anon_ctx.storage = ctx.storage.clone();
        anon_ctx.principal = Some(crate::auth::TransportPrincipal::anonymous_loopback());

        for claim in [
            serde_json::json!({"query": "Orchard launch checklist", "workspaces": ["default"]}),
            serde_json::json!({"query": "Orchard launch checklist", "filters": {"workspace": "default"}}),
        ] {
            let built = super::memory_build_context(&anon_ctx, claim.clone());
            assert!(
                built["prompt"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("public note"),
                "expected default memory in build_context for {claim}: {built}"
            );
            assert!(
                !built.to_string().contains("secretcapybara"),
                "build_context leaked foreign memory for {claim}: {built}"
            );

            let injected = super::memory_get_injection_prompt(&anon_ctx, claim.clone());
            assert!(
                !injected.to_string().contains("secretcapybara"),
                "get_injection_prompt leaked foreign memory for {claim}: {injected}"
            );

            let prepared = super::memory_prepare_context(&anon_ctx, claim.clone());
            assert!(
                !prepared.to_string().contains("secretcapybara"),
                "prepare_context leaked foreign memory for {claim}: {prepared}"
            );
        }

        // Namespaced StoredToken without explicit workspace defaults to its namespace ("alpha"),
        // never leaking "default" or "private".
        super::super::dispatch(
            &ctx,
            "memory_create",
            serde_json::json!({
                "content": "Orchard launch checklist alphatokenonly",
                "memory_type": "note",
                "workspace": "alpha"
            }),
        );
        let mut token_ctx = test_ctx();
        token_ctx.storage = ctx.storage.clone();
        token_ctx.principal = Some(
            crate::auth::TransportPrincipal::from_token_claims(crate::auth::TokenClaims {
                user_id: crate::auth::UserId::from_string("token-alpha"),
                key_id: "key-alpha".to_string(),
                permissions: crate::auth::PermissionSet::read_only(),
                namespace: Some("alpha".to_string()),
                issued_at: chrono::Utc::now(),
                expires_at: None,
            })
            .expect("valid token claims"),
        );

        for tool_fn in [
            super::memory_build_context
                as fn(&super::super::HandlerContext, serde_json::Value) -> serde_json::Value,
            super::memory_get_injection_prompt,
            super::memory_prepare_context,
        ] {
            let unstated = tool_fn(
                &token_ctx,
                serde_json::json!({"query": "Orchard launch checklist"}),
            );
            assert!(
                !unstated.to_string().contains("secretcapybara")
                    && !unstated.to_string().contains("public note"),
                "namespaced token without workspace leaked out-of-namespace memory: {unstated}"
            );

            for alpha_claim in [
                serde_json::json!({"query": "Orchard launch checklist", "workspaces": ["alpha"]}),
                serde_json::json!({"query": "Orchard launch checklist", "filters": {"workspace": "alpha"}}),
            ] {
                let scoped_alpha = tool_fn(&token_ctx, alpha_claim.clone());
                assert!(
                    !scoped_alpha.to_string().contains("secretcapybara")
                        && !scoped_alpha.to_string().contains("public note"),
                    "namespaced token with {alpha_claim} leaked out-of-namespace memory: {scoped_alpha}"
                );
                assert!(
                    scoped_alpha.to_string().contains("alphatokenonly"),
                    "namespaced token with {alpha_claim} must read its own namespace: {scoped_alpha}"
                );
            }
        }
    }

    #[test]
    fn build_context_depth3_blocks_foreign_bridge_and_include_graph_omits_foreign_edges() {
        use crate::types::{CreateCrossRefInput, EdgeType};

        let ctx = test_ctx();
        let mem_a = seed_memory(&ctx, "Anchor mercurybridge node", "note");
        let mem_b_foreign = super::super::dispatch(
            &ctx,
            "memory_create",
            serde_json::json!({
                "content": "Intermediate foreign secretbridge node",
                "memory_type": "note",
                "workspace": "private"
            }),
        )["id"]
            .as_i64()
            .expect("foreign memory B");
        let mem_c = seed_memory(
            &ctx,
            "Unmatched twohopdest payload reachable only via foreign B",
            "note",
        );

        // Graph: A (default) -> B (private) -> C (default)
        ctx.storage
            .with_connection(|conn| {
                crate::storage::queries::create_crossref(
                    conn,
                    &CreateCrossRefInput {
                        from_id: mem_a,
                        to_id: mem_b_foreign,
                        edge_type: EdgeType::default(),
                        strength: None,
                        source_context: None,
                        pinned: false,
                    },
                )?;
                crate::storage::queries::create_crossref(
                    conn,
                    &CreateCrossRefInput {
                        from_id: mem_b_foreign,
                        to_id: mem_c,
                        edge_type: EdgeType::default(),
                        strength: None,
                        source_context: None,
                        pinned: false,
                    },
                )
            })
            .expect("create A->B->C crossrefs");

        let scoped = super::memory_build_context(
            &ctx,
            serde_json::json!({
                "query": "mercurybridge",
                "workspace": "default",
                "depth": 3,
                "include_graph": true
            }),
        );
        let serialized = scoped.to_string();
        assert!(
            !serialized.contains("secretbridge"),
            "foreign hop-1 memory leaked: {scoped}"
        );
        assert!(
            !serialized.contains("twohopdest"),
            "depth=3 traversed rejected foreign node B to pull 2-hop C: {scoped}"
        );

        let edges = scoped["graph"]["edges"]
            .as_array()
            .expect("graph.edges array");
        for edge in edges {
            assert_ne!(
                edge["source"].as_i64(),
                Some(mem_b_foreign),
                "include_graph leaked foreign source ID: {edge}"
            );
            assert_ne!(
                edge["target"].as_i64(),
                Some(mem_b_foreign),
                "include_graph leaked foreign target ID: {edge}"
            );
        }
    }
}
