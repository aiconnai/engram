//! O1: logs must not carry proprietary content (memory text, queries,
//! credentials, filesystem paths, provider error bodies) by default, while the
//! MCP protocol payload stays exactly as before.

use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use crate::embedding::{Embedder, EmbeddingCache};
use crate::error::{EngramError, Result};
use crate::mcp::handlers::{dispatch, HandlerContext};
use crate::observability::test_capture::{assert_logs_exclude, install};
use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use crate::storage::Storage;

const DIMS: usize = 8;

/// Provider that always fails and echoes the request text in its error body,
/// like a cloud API reflecting the prompt.
pub(super) struct LeakyProvider {
    pub(super) body: &'static str,
}

impl Embedder for LeakyProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Err(EngramError::Embedding(format!(
            "Embedding API error 500: {} input={text}",
            self.body
        )))
    }

    fn dimensions(&self) -> usize {
        DIMS
    }

    fn model_name(&self) -> &str {
        "leaky-provider-test"
    }
}

pub(super) fn ctx_with(embedder: Arc<dyn Embedder>) -> HandlerContext {
    HandlerContext {
        storage: Storage::open_in_memory().expect("in-memory storage"),
        embedder,
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
            crate::search::HnswConfig::new(DIMS, crate::search::VectorMetric::Cosine),
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

#[test]
fn create_with_failing_provider_logs_neither_provider_body_nor_memory_text() {
    install();
    const PROVIDER_BODY: &str = "provider-body-sentinel-8e61";
    const PRIVATE_TEXT: &str = "private-memory-sentinel-4b7d";
    let ctx = ctx_with(Arc::new(LeakyProvider {
        body: PROVIDER_BODY,
    }));

    let created = dispatch(
        &ctx,
        "memory_create",
        json!({"content": PRIVATE_TEXT, "memory_type": "note"}),
    );

    // Protocol payload is unchanged: the memory is durable and returned.
    assert!(created["id"].as_i64().is_some(), "created: {created}");
    assert_eq!(created["content"], PRIVATE_TEXT);
    // The failure is still diagnosable from logs, without the payloads.
    assert!(
        crate::observability::test_capture::captured().contains("Immediate embedding failed"),
        "the structured failure message must stay in the logs"
    );
    assert_logs_exclude(&[PROVIDER_BODY, PRIVATE_TEXT]);
}

#[test]
fn search_logs_do_not_contain_the_query() {
    install();
    const QUERY: &str = "query-sentinel-2f8a";
    let ctx = ctx_with(Arc::new(LeakyProvider {
        body: "provider-body-sentinel-search",
    }));

    let searched = dispatch(&ctx, "memory_search", json!({"query": QUERY}));

    assert!(
        searched.is_array() || searched.get("results").is_some(),
        "search keeps its result shape: {searched}"
    );
    assert_logs_exclude(&[QUERY, "provider-body-sentinel-search"]);
}

/// Real recovery over a file-backed database whose path contains a sentinel.
/// The in-memory storage used elsewhere in this file returns before any recovery
/// code runs, so it cannot prove anything about recovery logging.
#[test]
fn file_backed_recovery_succeeds_and_logs_no_filesystem_path() {
    install();
    let dir = tempfile::tempdir().expect("tempdir");
    let sentinel_dir = dir.path().join("fs-path-sentinel-5d2c");
    std::fs::create_dir_all(&sentinel_dir).expect("mkdir");
    let source = sentinel_dir.join("source.db");
    let target = sentinel_dir.join("recovered-sentinel.db");
    let storage = Storage::open(crate::types::StorageConfig {
        db_path: source.to_string_lossy().to_string(),
        storage_mode: crate::types::StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .expect("file-backed storage");
    let mut ctx = ctx_with(Arc::new(LeakyProvider { body: "unused" }));
    ctx.storage = storage;
    dispatch(
        &ctx,
        "memory_create",
        json!({"content": "recovery-private-text-sentinel-1b2c", "defer_embedding": true}),
    );
    let before = crate::observability::counters().snapshot().recovery_total["succeeded"];

    let recovered = dispatch(
        &ctx,
        "replication_recover",
        json!({"target_db_path": target.to_string_lossy()}),
    );

    // The real engine ran and the protocol payload is intact.
    assert_eq!(
        recovered["success"], true,
        "recovery of a file-backed database must succeed: {recovered}"
    );
    assert!(recovered["error"].is_null(), "{recovered}");
    assert!(target.exists(), "recovered database was written");
    assert!(
        crate::observability::counters().snapshot().recovery_total["succeeded"] > before,
        "a successful recovery is counted as an attempt"
    );
    assert_logs_exclude(&[
        "fs-path-sentinel-5d2c",
        "recovered-sentinel.db",
        "recovery-private-text-sentinel-1b2c",
    ]);
}

#[test]
fn rejected_recovery_input_is_not_counted_as_a_failed_attempt() {
    install();
    let ctx = ctx_with(Arc::new(LeakyProvider { body: "unused" }));
    let before = crate::observability::counters().snapshot().recovery_total;

    let result = dispatch(&ctx, "replication_recover", json!({}));

    assert_eq!(result["error"], "target_db_path is required");
    let after = crate::observability::counters().snapshot().recovery_total;
    assert!(after["rejected"] > before["rejected"]);
}
