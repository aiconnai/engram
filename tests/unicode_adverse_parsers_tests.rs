//! Regression tests for task C3 (Unicode and adverse parsers).
//!
//! Each case here is a reproduced panic from `docs/quality/rust-risk-inventory.md`
//! (Q2-B01..B08). Release builds use `panic = "abort"` and the stdio loop has no
//! `catch_unwind`, so one stored memory (or one adverse argument) used to be able
//! to kill the whole server. The tests drive the real MCP `dispatch` entry point.
//!
//! Run with: cargo test --test unicode_adverse_parsers_tests

use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, EmbeddingConfig};

fn test_ctx() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    HandlerContext {
        storage,
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(
                embedder.dimensions(),
                engram::search::VectorMetric::Cosine,
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

fn add_memory(ctx: &HandlerContext, content: &str) -> i64 {
    ctx.storage
        .with_transaction(|conn| {
            let input = CreateMemoryInput {
                content: content.to_string(),
                importance: Some(0.9),
                ..Default::default()
            };
            create_memory(conn, &input).map(|m| m.id)
        })
        .expect("create memory")
}

/// `prefix` followed by filler so that the 2-byte char `é` occupies bytes
/// `[boundary - 1, boundary + 1)`, i.e. byte index `boundary` is NOT a char boundary.
fn content_with_split_char_at(prefix: &str, boundary: usize, tail: usize) -> String {
    assert!(prefix.len() < boundary);
    let mut s = String::from(prefix);
    s.push_str(&"a".repeat(boundary - 1 - prefix.len()));
    s.push('é');
    s.push_str(&"b".repeat(tail));
    assert!(!s.is_char_boundary(boundary));
    s
}

fn assert_no_error(result: &Value) {
    assert!(result.get("error").is_none(), "unexpected error: {result}");
}

// ── Q2-B01 memory_search_compact ─────────────────────────────────────────────

#[test]
fn compact_search_title_survives_multibyte_char_at_byte_80() {
    let ctx = test_ctx();
    let content = content_with_split_char_at("needle ", 80, 40);
    add_memory(&ctx, &content);

    let result = dispatch(&ctx, "memory_search_compact", json!({"query": "needle"}));

    assert_no_error(&result);
    let results = result["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1, "{result}");
    let title = results[0]["title"].as_str().expect("title");
    // Largest char boundary at or below byte 80 is 79 (the `é` starts there).
    assert_eq!(title, format!("{}...", &content[..79]));
}

#[test]
fn compact_search_title_exactly_80_bytes_is_not_truncated() {
    let ctx = test_ctx();
    let content = format!("needle {}", "a".repeat(73));
    assert_eq!(content.len(), 80);
    add_memory(&ctx, &content);

    let result = dispatch(&ctx, "memory_search_compact", json!({"query": "needle"}));

    assert_no_error(&result);
    assert_eq!(
        result["results"][0]["title"].as_str(),
        Some(content.as_str())
    );
}

// ── Q2-B02 memory_prepare_context ────────────────────────────────────────────

#[test]
fn prepare_context_survives_multibyte_char_at_byte_500() {
    let ctx = test_ctx();
    let content = content_with_split_char_at("needle ", 500, 100);
    add_memory(&ctx, &content);

    let result = dispatch(
        &ctx,
        "memory_prepare_context",
        json!({"query": "needle", "budget": 4000}),
    );

    assert_no_error(&result);
    let text = result["context"].as_str().expect("context");
    assert!(
        text.contains("(truncated)"),
        "expected truncation marker: {text}"
    );
    assert!(text.contains(&content[..499]), "kept prefix must be intact");
    assert!(!text.contains('é'), "split char must be dropped, not cut");
}

// ── Q2-B03 memory_garden ─────────────────────────────────────────────────────

#[test]
fn garden_preview_and_apply_survive_multibyte_char_at_byte_4096() {
    let ctx = test_ctx();
    let content = content_with_split_char_at("needle ", 4096, 200);
    let id = add_memory(&ctx, &content);

    // Dry run must report a compress action without panicking.
    let preview = dispatch(&ctx, "memory_garden_preview", json!({}));
    assert_no_error(&preview);

    // Applying must store a char-boundary-safe truncated copy.
    let applied = dispatch(&ctx, "memory_garden", json!({}));
    assert_no_error(&applied);

    let stored: String = ctx
        .storage
        .with_connection(|conn| {
            conn.query_row("SELECT content FROM memories WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .map_err(Into::into)
        })
        .expect("read back");
    assert_eq!(stored, format!("{} [compressed]", &content[..4095]));
}

// ── Q2-B04 context_budget_check ──────────────────────────────────────────────

#[test]
fn budget_check_preview_survives_multibyte_char_at_byte_50() {
    let ctx = test_ctx();
    let content = content_with_split_char_at("needle ", 50, 40);
    let id = add_memory(&ctx, &content);

    let result = dispatch(
        &ctx,
        "context_budget_check",
        json!({"memory_ids": [id], "model": "gpt-4", "budget": 1000}),
    );

    assert_no_error(&result);
    let preview = result["memory_tokens"][0]["content_preview"]
        .as_str()
        .unwrap_or_else(|| panic!("content_preview missing: {result}"));
    assert_eq!(preview, format!("{}...", &content[..49]));
}

// ── Q2-B05 attestation hex keys ──────────────────────────────────────────────

#[test]
fn attestation_chain_verify_rejects_malformed_keys_without_panic() {
    let ctx = test_ctx();
    let adverse = [
        "abc".to_string(),                // odd length
        "é".to_string(),                  // non-ASCII, even byte length
        format!("0é{}", "0".repeat(61)),  // 64 bytes, multibyte char splits a pair
        "+f".repeat(32),                  // sign chars accepted by from_str_radix
        "zz".repeat(32),                  // not hex
        "00".repeat(31),                  // valid hex, wrong size
        "\u{202e}\u{200d}ab".to_string(), // bidi override + ZWJ
    ];
    for key in adverse {
        let result = dispatch(
            &ctx,
            "attestation_chain_verify",
            json!({"verifying_key": key}),
        );
        assert!(
            result.get("error").is_some(),
            "key {key:?} must be rejected with a typed error, got {result}"
        );
    }
}

#[test]
fn attestation_log_rejects_malformed_sign_key_without_panic() {
    let ctx = test_ctx();
    for key in ["abc", "é", "+f+f"] {
        let result = dispatch(
            &ctx,
            "attestation_log",
            json!({"content": "doc", "document_name": "d", "sign_key": key}),
        );
        assert!(result.get("error").is_some(), "sign_key {key:?}: {result}");
    }
}

#[test]
fn attestation_chain_verify_accepts_valid_64_hex_key() {
    let ctx = test_ctx();
    let key = "ab".repeat(32);
    let result = dispatch(
        &ctx,
        "attestation_chain_verify",
        json!({"verifying_key": key}),
    );
    assert_no_error(&result);
    assert_eq!(result["signature_check"], json!(true));
}

// ── Q2-B06 snapshot hex keys ─────────────────────────────────────────────────

#[test]
fn snapshot_load_rejects_malformed_decrypt_key_without_panic() {
    let ctx = test_ctx();
    let dir = std::env::temp_dir().join(format!("engram-c3-snap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmp dir");
    let path = dir.join("missing.egm");
    for key in [
        format!("0é{}", "0".repeat(61)), // 64 bytes, splits a pair
        "+f".repeat(32),
        "é".repeat(32),
    ] {
        let result = dispatch(
            &ctx,
            "snapshot_load",
            json!({"path": path.to_string_lossy(), "strategy": "merge", "decrypt_key": key}),
        );
        let err = result["error"].as_str().unwrap_or_default();
        assert!(err.contains("Invalid decrypt_key"), "key {key:?}: {result}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// ── Q2-B08 session_index ─────────────────────────────────────────────────────

#[test]
fn session_index_tiny_max_chars_does_not_underflow() {
    let ctx = test_ctx();
    let result = dispatch(
        &ctx,
        "session_index",
        json!({
            "session_id": "c3-tiny",
            "max_chars": 5,
            "messages": [{"role": "user", "content": "hello world, this is long"}],
        }),
    );
    assert_no_error(&result);
}

#[test]
fn session_index_extreme_ttl_days_does_not_overflow() {
    let ctx = test_ctx();
    let result = dispatch(
        &ctx,
        "session_index",
        json!({
            "session_id": "c3-ttl",
            "ttl_days": i64::MAX,
            "messages": [{"role": "user", "content": "hello"}],
        }),
    );
    let err = result["error"].as_str().unwrap_or_default();
    assert!(
        err.contains("ttl_days"),
        "expected a typed ttl_days error: {result}"
    );

    for bad in [json!(i64::MIN), json!(-1)] {
        let result = dispatch(
            &ctx,
            "session_index",
            json!({"session_id": "c3-ttl", "ttl_days": bad, "messages": [{"role": "user", "content": "hello"}]}),
        );
        assert!(result.get("error").is_some(), "{result}");
    }

    // The documented range still works, including its upper bound.
    let ok = dispatch(
        &ctx,
        "session_index",
        json!({"session_id": "c3-ttl-ok", "ttl_days": 36500, "messages": [{"role": "user", "content": "hello"}]}),
    );
    assert_no_error(&ok);
}

// ── BM25 highlights (lowercased offsets applied to the original text) ────────

/// `ẞ` (3 bytes) lowercases to `ß` (2 bytes), so every offset found in the
/// lowercased text is shifted by one byte in the original. FTS5 matches whole
/// tokens, so the term is separated by a space here (the brief's exact
/// no-space string is pinned at function level in `search::bm25::tests`);
/// 29 filler bytes put `match_start - 30` inside the 3-byte `ẞ`.
fn bm25_reproducer_content() -> String {
    format!("{}ẞ{} target", "x".repeat(100), "a".repeat(29))
}

#[test]
fn bm25_explain_highlight_survives_case_folding_length_change() {
    use engram::search::bm25_search;

    let ctx = test_ctx();
    let content = bm25_reproducer_content();
    add_memory(&ctx, &content);

    let results = ctx
        .storage
        .with_connection(|conn| bm25_search(conn, "target", 10, true))
        .expect("bm25 search must not panic");

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].highlights,
        vec![format!("...{} target", "a".repeat(29))],
        "snippet must be cut from the ORIGINAL text at the original match position"
    );
}
