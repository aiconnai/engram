use super::handler::memory_export_markdown;
use crate::storage::queries::create_memory;
use crate::storage::Storage;
use crate::types::{CreateMemoryInput, MemoryType};
use serde_json::json;
use std::sync::Arc;

fn ctx() -> crate::mcp::handlers::HandlerContext {
    crate::mcp::handlers::HandlerContext {
        storage: Storage::open_in_memory().expect("in-memory storage"),
        embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(128)),
        fuzzy_engine: Arc::new(parking_lot::Mutex::new(crate::search::FuzzyEngine::new())),
        search_config: crate::search::SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
        search_cache: Arc::new(crate::search::SearchResultCache::new(
            crate::search::AdaptiveCacheConfig::default(),
        )),
        hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
            crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
        ))),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 300,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        ),
        progress_reporter: None,
    }
}

fn make_memory(c: &crate::mcp::handlers::HandlerContext) {
    let input = CreateMemoryInput {
        content: "Markdown export schema smoke".to_string(),
        memory_type: MemoryType::Note,
        workspace: Some("markdown_export_schema".to_string()),
        ..Default::default()
    };
    c.storage
        .with_transaction(|conn| create_memory(conn, &input))
        .expect("create memory");
}

#[test]
fn export_markdown_uses_current_scope_schema() {
    let c = ctx();
    make_memory(&c);
    let dir = tempfile::tempdir().expect("tempdir");

    let result = memory_export_markdown(
        &c,
        json!({
            "workspace": "markdown_export_schema",
            "output_dir": dir.path().to_str().unwrap(),
            "include_links": false
        }),
    );

    assert!(result.get("error").is_none(), "result={result}");
    assert_eq!(result["memories_exported"].as_u64(), Some(1));
    assert!(dir.path().join("index.md").exists());
}
