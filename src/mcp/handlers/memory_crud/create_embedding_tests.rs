//! C7: `memory_create` / `memory_create_batch` keep the embedding row, the
//! `has_embedding` flag, the queue job and the vector index coherent, including
//! when the provider or a local write fails after the memory has committed.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

use super::{memory_create, memory_create_batch};
use crate::embedding::{drain_pending_embeddings, Embedder, EmbeddingCache, TfIdfEmbedder};
use crate::error::{EngramError, Result};
use crate::mcp::handlers::HandlerContext;
use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use crate::storage::{health_check_storage, DerivedIndexHealth, DerivedIndexStatus, Storage};

const DIMS: usize = 8;

/// Deterministic embedder whose provider can be switched off to simulate an outage.
struct SwitchableEmbedder {
    inner: TfIdfEmbedder,
    down: AtomicBool,
}

impl SwitchableEmbedder {
    fn new() -> Self {
        Self {
            inner: TfIdfEmbedder::new(DIMS),
            down: AtomicBool::new(false),
        }
    }

    fn set_down(&self, down: bool) {
        self.down.store(down, Ordering::SeqCst);
    }
}

impl Embedder for SwitchableEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.down.load(Ordering::SeqCst) {
            return Err(EngramError::Embedding("provider outage".to_string()));
        }
        self.inner.embed(text)
    }

    fn dimensions(&self) -> usize {
        DIMS
    }

    fn model_name(&self) -> &str {
        "switchable-test"
    }
}

fn make_ctx(embedder: Arc<SwitchableEmbedder>) -> HandlerContext {
    HandlerContext {
        storage: Storage::open_in_memory().expect("in-memory storage"),
        embedder: embedder.clone(),
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

/// (embedding rows, flag, job status) for one memory.
fn coherence(ctx: &HandlerContext, id: i64) -> (i64, i64, Option<String>) {
    ctx.storage
        .with_connection(|conn| {
            let rows = conn.query_row(
                "SELECT COUNT(*) FROM embeddings WHERE memory_id = ?",
                params![id],
                |r| r.get(0),
            )?;
            let flag = conn.query_row(
                "SELECT has_embedding FROM memories WHERE id = ?",
                params![id],
                |r| r.get(0),
            )?;
            let job = conn
                .query_row(
                    "SELECT status FROM embedding_queue WHERE memory_id = ?",
                    params![id],
                    |r| r.get(0),
                )
                .optional()?;
            Ok((rows, flag, job))
        })
        .expect("coherence readback")
}

fn count(ctx: &HandlerContext, sql: &str) -> i64 {
    ctx.storage
        .with_connection(|conn| Ok(conn.query_row(sql, [], |r| r.get(0))?))
        .expect("count")
}

fn embeddings_health(ctx: &HandlerContext) -> DerivedIndexHealth {
    health_check_storage(&ctx.storage)
        .expect("health")
        .derived_indexes
        .into_iter()
        .find(|index| index.name == "embeddings")
        .expect("embeddings health")
}

fn create(ctx: &HandlerContext, content: &str, extra: Value) -> Value {
    let mut params = json!({"content": content, "memory_type": "note"});
    if let (Some(base), Some(extra)) = (params.as_object_mut(), extra.as_object()) {
        base.extend(extra.clone());
    }
    memory_create(ctx, params)
}

#[test]
fn immediate_create_with_provider_outage_leaves_explicit_pending_job_then_converges() {
    let embedder = Arc::new(SwitchableEmbedder::new());
    let ctx = make_ctx(embedder.clone());
    embedder.set_down(true);

    let created = create(&ctx, "provider is down at create time", json!({}));
    let id = created["id"].as_i64().expect("memory is created durably");

    // Not a silent success: no embedding, no flag, an explicit pending job, a
    // backlog in health, and nothing in the vector index.
    assert_eq!(coherence(&ctx, id), (0, 0, Some("pending".to_string())));
    assert_eq!(
        embeddings_health(&ctx).status,
        DerivedIndexStatus::Backlogged
    );
    assert!(ctx.hnsw_index.read().is_empty());

    // Provider recovers; the worker drains the pending job.
    embedder.set_down(false);
    assert_eq!(
        drain_pending_embeddings(&ctx.storage, ctx.embedder.as_ref(), 10).unwrap(),
        1
    );
    assert_eq!(coherence(&ctx, id), (1, 1, Some("complete".to_string())));
    assert_eq!(embeddings_health(&ctx).status, DerivedIndexStatus::Healthy);
}

#[test]
fn immediate_create_with_failed_persistence_does_not_report_or_index_an_embedding() {
    let embedder = Arc::new(SwitchableEmbedder::new());
    let ctx = make_ctx(embedder);
    ctx.storage
        .with_connection(|conn| {
            conn.execute_batch(
                "CREATE TRIGGER inject_flag_failure BEFORE UPDATE OF has_embedding ON memories
                 WHEN NEW.has_embedding = 1
                 BEGIN SELECT RAISE(ABORT, 'injected flag failure'); END;",
            )?;
            Ok(())
        })
        .unwrap();

    let created = create(&ctx, "flag write fails after the row write", json!({}));
    let id = created["id"].as_i64().expect("memory itself is durable");

    // The row write is rolled back together with the failed flag write, the job
    // stays pending (explicit, retryable) and the vector index is not touched.
    assert_eq!(coherence(&ctx, id), (0, 0, Some("pending".to_string())));
    assert!(
        ctx.hnsw_index.read().is_empty(),
        "an embedding that was not persisted must not enter the vector index"
    );
    assert_eq!(
        embeddings_health(&ctx).status,
        DerivedIndexStatus::Backlogged
    );
}

#[test]
fn deferred_create_enqueue_is_atomic_with_the_memory_insert() {
    let embedder = Arc::new(SwitchableEmbedder::new());
    let ctx = make_ctx(embedder);
    ctx.storage
        .with_connection(|conn| {
            conn.execute_batch(
                "CREATE TRIGGER inject_enqueue_failure BEFORE INSERT ON embedding_queue
                 BEGIN SELECT RAISE(ABORT, 'injected enqueue failure'); END;",
            )?;
            Ok(())
        })
        .unwrap();

    let result = create(&ctx, "enqueue fails", json!({"defer_embedding": true}));

    assert!(result.get("error").is_some(), "create must fail: {result}");
    assert_eq!(
        count(&ctx, "SELECT COUNT(*) FROM memories"),
        0,
        "a failed enqueue must not leave a memory that no job will ever embed"
    );
}

#[test]
fn skip_dedup_of_an_embedded_memory_creates_no_second_row_or_job() {
    let embedder = Arc::new(SwitchableEmbedder::new());
    let ctx = make_ctx(embedder);

    let first = create(&ctx, "idempotent content", json!({}));
    let id = first["id"].as_i64().unwrap();
    let again = create(
        &ctx,
        "idempotent content",
        json!({"dedup_mode": "skip", "defer_embedding": true}),
    );

    assert_eq!(again["id"].as_i64(), Some(id));
    assert_eq!(coherence(&ctx, id), (1, 1, Some("complete".to_string())));
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM embedding_queue"), 1);
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM embeddings"), 1);
}

#[test]
fn batch_create_completes_jobs_and_survives_provider_outage() {
    let embedder = Arc::new(SwitchableEmbedder::new());
    let ctx = make_ctx(embedder.clone());

    let ok = memory_create_batch(
        &ctx,
        json!({"memories": [
            {"content": "batch one", "memory_type": "note"},
            {"content": "batch two", "memory_type": "note", "defer_embedding": true}
        ]}),
    );
    let ids: Vec<i64> = ok["created"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids.len(), 2);
    // Immediate item is embedded and completed; deferred item waits in the queue.
    assert_eq!(
        coherence(&ctx, ids[0]),
        (1, 1, Some("complete".to_string()))
    );
    assert_eq!(coherence(&ctx, ids[1]), (0, 0, Some("pending".to_string())));

    embedder.set_down(true);
    let down = memory_create_batch(
        &ctx,
        json!({"memories": [{"content": "batch three", "memory_type": "note"}]}),
    );
    let down_id = down["created"][0]["id"].as_i64().unwrap();
    assert_eq!(
        coherence(&ctx, down_id),
        (0, 0, Some("pending".to_string()))
    );

    embedder.set_down(false);
    assert_eq!(
        drain_pending_embeddings(&ctx.storage, ctx.embedder.as_ref(), 10).unwrap(),
        2
    );
    assert_eq!(count(&ctx, "SELECT COUNT(*) FROM embeddings"), 3);
    assert_eq!(
        count(
            &ctx,
            "SELECT COUNT(*) FROM memories WHERE has_embedding = 1"
        ),
        3
    );
    assert_eq!(embeddings_health(&ctx).status, DerivedIndexStatus::Healthy);
}
