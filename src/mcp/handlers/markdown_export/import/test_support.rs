use crate::storage::queries::create_memory;
use crate::storage::Storage;
use crate::types::{CreateMemoryInput, MemoryType};
use std::fs;
use std::sync::Arc;

pub(super) fn ctx() -> crate::mcp::handlers::HandlerContext {
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

pub(super) fn make_memory(
    c: &crate::mcp::handlers::HandlerContext,
    content: &str,
    tags: &[&str],
) -> i64 {
    let input = CreateMemoryInput {
        content: content.to_string(),
        memory_type: MemoryType::Note,
        tags: tags.iter().map(|t| t.to_string()).collect(),
        importance: Some(0.5),
        workspace: Some("default".to_string()),
        ..Default::default()
    };
    c.storage
        .with_transaction(|conn| create_memory(conn, &input))
        .expect("create memory")
        .id
}

/// Write a markdown file with engram_ frontmatter into `dir`.
#[allow(clippy::too_many_arguments)]
pub(super) fn write_md(
    dir: &std::path::Path,
    fname: &str,
    id: Option<i64>,
    version: i64,
    tags: &[&str],
    importance: f64,
    body: &str,
    extra_keys: &[(&str, &str)],
) {
    let mut fm = String::from("---\n");
    if let Some(id) = id {
        fm.push_str(&format!("engram_id: {}\n", id));
    }
    fm.push_str("engram_workspace: default\n");
    fm.push_str("engram_scope: user\n");
    fm.push_str("engram_type: note\n");
    fm.push_str(&format!("engram_version: {}\n", version));
    fm.push_str(&format!("engram_importance: {}\n", importance));
    for (k, v) in extra_keys {
        fm.push_str(&format!("{}: {}\n", k, v));
    }
    if !tags.is_empty() {
        fm.push_str("engram_tags:\n");
        for t in tags {
            fm.push_str(&format!("  - {}\n", t));
        }
    }
    fm.push_str("---\n");
    fm.push_str(body);
    fm.push('\n');
    fs::write(dir.join(fname), fm).expect("write md");
}

pub(super) fn db_row(c: &crate::mcp::handlers::HandlerContext, id: i64) -> (String, i64, f64) {
    c.storage
        .with_connection(|conn| {
            conn.query_row(
                "SELECT content, version, importance FROM memories WHERE id = ?1",
                rusqlite::params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(crate::error::EngramError::Database)
        })
        .expect("db row")
}

pub(super) fn db_tags(c: &crate::mcp::handlers::HandlerContext, id: i64) -> Vec<String> {
    c.storage
        .with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.name FROM memory_tags mt JOIN tags t ON mt.tag_id = t.id \
                         WHERE mt.memory_id = ?1 ORDER BY t.name",
            )?;
            let v = stmt
                .query_map(rusqlite::params![id], |r| r.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(v)
        })
        .expect("db tags")
}

pub(super) fn status_of(result: &serde_json::Value, id: i64) -> Option<&str> {
    result["files"].as_array()?.iter().find_map(|f| {
        if f["engram_id"].as_i64() == Some(id) {
            f["status"].as_str()
        } else {
            None
        }
    })
}
