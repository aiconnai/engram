use super::*;
use crate::error::{EngramError, Result};
use crate::storage::queries::create_memory;
use crate::storage::Storage;
use crate::types::{CreateMemoryInput, MemoryId, MemoryType};
use chrono::{Duration as ChronoDuration, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_embedding_queue() {
    let queue = EmbeddingQueue::new(10);

    queue.queue(1, "Hello world".to_string()).await.unwrap();
    queue.queue(2, "Test content".to_string()).await.unwrap();

    assert_eq!(queue.len(), 2);
}

#[tokio::test]
async fn test_embedding_worker_process_batch_surfaces_db_write_errors() {
    // Given: an embedding worker connected to a database that lacks the queue table.
    let worker = EmbeddingWorker {
        embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(8)),
        queue: EmbeddingQueue::new(1),
        conn: Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
        batch_size: 1,
        batch_timeout: Duration::from_secs(1),
    };
    let mut batch = vec![EmbeddingRequest {
        memory_id: 1,
        content: "db write failure should be visible".to_string(),
    }];

    // When: processing attempts the first durable queue write.
    let result = worker.process_batch(&mut batch).await;

    // Then: the database error is surfaced and the attempted batch is cleared.
    assert!(
        matches!(
            result,
            Err(EngramError::Embedding(ref message))
                if message.contains("mark embedding queue row as processing")
                    && message.contains("memory_id=1")
        ),
        "expected contextual processing-mark database error, got {result:?}"
    );
    assert!(batch.is_empty());
}

#[tokio::test]
async fn test_embedding_worker_process_batch_surfaces_embedder_failures() {
    // Given: a valid queue row and an embedder that fails before persistence.
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE embedding_queue (
                memory_id INTEGER PRIMARY KEY,
                status TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                error TEXT,
                retry_count INTEGER NOT NULL DEFAULT 0
            )",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO embedding_queue (memory_id, status, retry_count)
             VALUES (1, 'pending', 0)",
        [],
    )
    .unwrap();
    let worker = EmbeddingWorker {
        embedder: Arc::new(FailingEmbedder),
        queue: EmbeddingQueue::new(1),
        conn: Arc::new(Mutex::new(conn)),
        batch_size: 1,
        batch_timeout: Duration::from_secs(1),
    };
    let mut batch = vec![EmbeddingRequest {
        memory_id: 1,
        content: "embedder failure should mark failed".to_string(),
    }];

    // When: the embedder fails after the processing mark succeeds.
    let result = worker.process_batch(&mut batch).await;

    // Then: the embedder error is surfaced and the queue row records failure.
    assert!(
        matches!(
            result,
            Err(EngramError::Embedding(ref message))
                if message.contains("forced embed failure")
        ),
        "expected embedder failure, got {result:?}"
    );
    assert!(batch.is_empty());

    let conn = worker.conn.lock();
    let state = queue_state(&conn, 1).unwrap();
    assert_eq!(state, ("failed".to_string(), 1));
}

#[test]
fn test_get_embedding_length_mismatch() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Test embedding".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: std::collections::HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: Default::default(),
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Insert embedding with incorrect byte length (dimensions=2 => expected 8 bytes)
            conn.execute(
                "INSERT INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                     VALUES (?, ?, ?, ?, datetime('now'))",
                params![memory.id, vec![0u8; 4], "test", 2],
            )?;

            match get_embedding(conn, memory.id) {
                Err(EngramError::InvalidInput(_)) => Ok(()),
                Err(e) => Err(e),
                Ok(_) => Err(EngramError::Internal(
                    "Expected embedding length mismatch error".to_string(),
                )),
            }
        })
        .unwrap();
}

#[test]
fn test_embedding_queue_health_counts_stale_and_retries() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let pending = create_memory(conn, &test_memory_input("pending"))?;
            let processing = create_memory(conn, &test_memory_input("processing"))?;
            let failed_retryable =
                create_memory(conn, &test_memory_input("failed retryable"))?;
            let failed_exhausted =
                create_memory(conn, &test_memory_input("failed exhausted"))?;
            let failed_zero = create_memory(conn, &test_memory_input("failed zero retry"))?;

            let old_started_at = (Utc::now() - ChronoDuration::minutes(30)).to_rfc3339();
            let old_started_or_completed = (Utc::now() - ChronoDuration::minutes(90)).to_rfc3339();
            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ? WHERE memory_id = ?",
                params![old_started_at, processing.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                params![failed_retryable.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 3 WHERE memory_id = ?",
                params![failed_exhausted.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue
                     SET status = 'failed', retry_count = 0, completed_at = ?
                     WHERE memory_id = ?",
                params![old_started_or_completed, failed_zero.id],
            )?;

            let health =
                get_embedding_queue_health(conn, Duration::from_secs(15 * 60), 3)?;

            assert_eq!(health.pending, 1);
            assert_eq!(health.processing, 1);
            assert_eq!(health.stale_processing, 1);
            assert_eq!(health.failed, 3);
            assert_eq!(health.retryable_failed, 2);
            assert_eq!(health.exhausted_failed, 1);
            assert_eq!(health.zero_retry_failed, 1);
            assert_eq!(health.max_retry_count, 3);
            assert_eq!(health.retry_count_0, 1);
            assert_eq!(health.retry_count_1, 1);
            assert_eq!(health.retry_count_2, 0);
            assert_eq!(health.retry_count_3_plus, 1);
            assert!(health.oldest_pending_seconds.is_some());
            assert!(health.oldest_processing_age_seconds.is_some());
            assert!(health.oldest_failed_age_seconds.is_some());

            let _ = pending;
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_embedding_queue_health_retry_buckets_are_stable_vs_config() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let retry_zero = create_memory(conn, &test_memory_input("retry zero"))?;
            let retry_one = create_memory(conn, &test_memory_input("retry one"))?;
            let retry_two = create_memory(conn, &test_memory_input("retry two"))?;
            let retry_three = create_memory(conn, &test_memory_input("retry three"))?;
            let retry_many = create_memory(conn, &test_memory_input("retry many"))?;

            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 0 WHERE memory_id = ?",
                params![retry_zero.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                params![retry_one.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 2 WHERE memory_id = ?",
                params![retry_two.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 3 WHERE memory_id = ?",
                params![retry_three.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 5 WHERE memory_id = ?",
                params![retry_many.id],
            )?;

            let config = EmbeddingQueueHygieneConfig {
                max_retries: 1,
                ..Default::default()
            };
            let health = get_embedding_queue_health_with_config(conn, &config)?;

            assert_eq!(health.retry_count_0, 1);
            assert_eq!(health.retry_count_1, 1);
            assert_eq!(health.retry_count_2, 1);
            assert_eq!(health.retry_count_3_plus, 2);
            assert_eq!(health.max_retry_count, 5);
            assert_eq!(health.retryable_failed, 1);
            assert_eq!(health.exhausted_failed, 4);

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_requeue_stale_processing_respects_retry_budget() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let retryable = create_memory(conn, &test_memory_input("retryable"))?;
            let exhausted = create_memory(conn, &test_memory_input("exhausted"))?;
            let fresh = create_memory(conn, &test_memory_input("fresh"))?;

            let old_started_at = (Utc::now() - ChronoDuration::minutes(30)).to_rfc3339();
            let fresh_started_at = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE embedding_queue
                     SET status = 'processing', started_at = ?, retry_count = 1
                     WHERE memory_id = ?",
                params![old_started_at, retryable.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue
                     SET status = 'processing', started_at = ?, retry_count = 3
                     WHERE memory_id = ?",
                params![old_started_at, exhausted.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue
                     SET status = 'processing', started_at = ?, retry_count = 0
                     WHERE memory_id = ?",
                params![fresh_started_at, fresh.id],
            )?;

            let report =
                requeue_stale_processing_embeddings(conn, Duration::from_secs(15 * 60), 3)?;
            assert_eq!(report.requeued_stale, 1);
            assert_eq!(report.failed_exhausted, 1);

            let retryable_state = queue_state(conn, retryable.id)?;
            let exhausted_state = queue_state(conn, exhausted.id)?;
            let fresh_state = queue_state(conn, fresh.id)?;

            assert_eq!(retryable_state, ("pending".to_string(), 2));
            assert_eq!(exhausted_state, ("failed".to_string(), 3));
            assert_eq!(fresh_state, ("processing".to_string(), 0));

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_embedding_queue_hygiene_dry_run_does_not_mutate_and_apply_can_repair() {
    let storage = Storage::open_in_memory().unwrap();
    let (stale_retryable, stale_exhausted, stale_fresh, failed_retryable, complete_recent, complete_old) =
        storage.with_connection(|conn| {
            let stale_retryable = create_memory(conn, &test_memory_input("stale retryable"))?;
            let stale_exhausted = create_memory(conn, &test_memory_input("stale exhausted"))?;
            let stale_fresh = create_memory(conn, &test_memory_input("processing fresh"))?;
            let failed_retryable = create_memory(conn, &test_memory_input("failed retryable"))?;
            let complete_recent = create_memory(conn, &test_memory_input("complete new"))?;
            let complete_old = create_memory(conn, &test_memory_input("complete old"))?;

            let old_started_at = (Utc::now() - ChronoDuration::minutes(30)).to_rfc3339();
            let fresh_started_at = Utc::now().to_rfc3339();
            let old_completed = (Utc::now() - ChronoDuration::days(30)).to_rfc3339();
            let new_completed = (Utc::now() - ChronoDuration::minutes(10)).to_rfc3339();

            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 1 WHERE memory_id = ?",
                params![old_started_at, stale_retryable.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 3 WHERE memory_id = ?",
                params![old_started_at, stale_exhausted.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 0 WHERE memory_id = ?",
                params![fresh_started_at, stale_fresh.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                params![failed_retryable.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'complete', queued_at = ?, completed_at = ? WHERE memory_id = ?",
                params![old_completed, old_completed, complete_old.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'complete', queued_at = ?, completed_at = ? WHERE memory_id = ?",
                params![new_completed, new_completed, complete_recent.id],
            )?;

            Ok((
                stale_retryable.id,
                stale_exhausted.id,
                stale_fresh.id,
                failed_retryable.id,
                complete_recent.id,
                complete_old.id,
            ))
        })
        .unwrap();

    let config = EmbeddingQueueHygieneConfig {
        complete_retention: Duration::from_secs(24 * 60 * 60),
        ..Default::default()
    };

    let dry_run = storage
        .with_connection(|conn| run_embedding_queue_hygiene(conn, &config, true, false, true))
        .unwrap();
    assert_eq!(dry_run.requeued_stale, 1);
    assert_eq!(dry_run.failed_exhausted, 1);
    assert_eq!(dry_run.requeued_failed, 1);
    assert_eq!(dry_run.pruned_complete, 1);

    let before = storage
        .with_connection(|conn| {
            let stale_retryable_state = queue_state(conn, stale_retryable)?;
            let stale_exhausted_state = queue_state(conn, stale_exhausted)?;
            let stale_fresh_state = queue_state(conn, stale_fresh)?;
            let failed_retryable_state = queue_state(conn, failed_retryable)?;
            let old_complete = conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete' AND memory_id = ?",
                params![complete_old],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((
                stale_retryable_state,
                stale_exhausted_state,
                stale_fresh_state,
                failed_retryable_state,
                old_complete,
            ))
        })
        .unwrap();

    assert_eq!(before.0, ("processing".to_string(), 1));
    assert_eq!(before.1, ("processing".to_string(), 3));
    assert_eq!(before.2, ("processing".to_string(), 0));
    assert_eq!(before.3, ("failed".to_string(), 1));
    assert_eq!(before.4, 1);

    let applied = storage
        .with_connection(|conn| run_embedding_queue_hygiene(conn, &config, true, true, true))
        .unwrap();
    assert_eq!(applied.requeued_stale, 1);
    assert_eq!(applied.failed_exhausted, 1);
    assert_eq!(applied.requeued_failed, 1);
    assert_eq!(applied.pruned_complete, 1);

    let after = storage.with_connection(|conn| {
        let stale_retryable_state = queue_state(conn, stale_retryable)?;
        let stale_exhausted_state = queue_state(conn, stale_exhausted)?;
        let stale_fresh_state = queue_state(conn, stale_fresh)?;
        let failed_retryable_state = queue_state(conn, failed_retryable)?;
        let complete_count = conn.query_row(
            "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete' AND memory_id IN (?, ?)",
            params![complete_recent, complete_old],
            |row| row.get::<_, i64>(0),
        )?;
        Ok((
            stale_retryable_state,
            stale_exhausted_state,
            stale_fresh_state,
            failed_retryable_state,
            complete_count,
        ))
    }).unwrap();

    assert_eq!(after.0, ("pending".to_string(), 2));
    assert_eq!(after.1, ("failed".to_string(), 3));
    assert_eq!(after.2, ("processing".to_string(), 0));
    assert_eq!(after.3, ("pending".to_string(), 2));
    assert_eq!(after.4, 1);
}

#[test]
fn test_drain_does_not_requeue_stale_processing_rows() {
    let storage = Storage::open_in_memory().unwrap();
    let memory_id = storage
        .with_connection(|conn| {
            let memory = create_memory(conn, &test_memory_input("stale processing"))?;
            let old_started_at = (Utc::now() - ChronoDuration::minutes(30)).to_rfc3339();
            conn.execute(
                "UPDATE embedding_queue
                     SET status = 'processing', started_at = ?, retry_count = 1
                     WHERE memory_id = ?",
                params![old_started_at, memory.id],
            )?;
            Ok(memory.id)
        })
        .unwrap();

    let embedder = crate::embedding::TfIdfEmbedder::new(8);
    let processed = drain_pending_embeddings(&storage, &embedder, 10).unwrap();
    assert_eq!(processed, 0);

    let state = storage
        .with_connection(|conn| queue_state(conn, memory_id))
        .unwrap();
    assert_eq!(state, ("processing".to_string(), 1));
}

// ---------------------------------------------------------------------------
// C7: end-to-end coherence of embedding row, `has_embedding` flag and job state
// ---------------------------------------------------------------------------

/// Snapshot of everything that must stay coherent for one memory.
#[derive(Debug, PartialEq, Eq)]
struct EmbeddingFacts {
    rows: i64,
    flag: i64,
    queue_rows: i64,
    job: Option<(String, i32)>,
}

fn embedding_facts(storage: &Storage, memory_id: MemoryId) -> EmbeddingFacts {
    storage
        .with_connection(|conn| {
            let rows = conn.query_row(
                "SELECT COUNT(*) FROM embeddings WHERE memory_id = ?",
                params![memory_id],
                |row| row.get(0),
            )?;
            let flag = conn.query_row(
                "SELECT has_embedding FROM memories WHERE id = ?",
                params![memory_id],
                |row| row.get(0),
            )?;
            let queue_rows = conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE memory_id = ?",
                params![memory_id],
                |row| row.get(0),
            )?;
            let job = queue_state(conn, memory_id).ok();
            Ok(EmbeddingFacts {
                rows,
                flag,
                queue_rows,
                job,
            })
        })
        .unwrap()
}

fn embedding_health(storage: &Storage) -> crate::storage::DerivedIndexHealth {
    crate::storage::health_check_storage(storage)
        .unwrap()
        .derived_indexes
        .into_iter()
        .find(|index| index.name == "embeddings")
        .expect("embedding health")
}

fn queued_memory(storage: &Storage, content: &str) -> MemoryId {
    storage
        .with_transaction(|conn| Ok(create_memory(conn, &test_memory_input(content))?.id))
        .unwrap()
}

#[test]
fn test_drain_injected_failures_leave_no_partial_state_and_converge() {
    // Failure injected at each local write of the persistence step: the embedding
    // row, the `has_embedding` flag and the job completion. None may leave a
    // half-written memory behind, and a retry must converge without duplicates.
    let triggers = [
        (
            "embedding row",
            "CREATE TRIGGER inject_fail BEFORE INSERT ON embeddings
             BEGIN SELECT RAISE(ABORT, 'injected embedding row failure'); END;",
        ),
        (
            "flag",
            "CREATE TRIGGER inject_fail BEFORE UPDATE OF has_embedding ON memories
             WHEN NEW.has_embedding = 1
             BEGIN SELECT RAISE(ABORT, 'injected flag failure'); END;",
        ),
        (
            "completion",
            "CREATE TRIGGER inject_fail BEFORE UPDATE OF status ON embedding_queue
             WHEN NEW.status = 'complete'
             BEGIN SELECT RAISE(ABORT, 'injected completion failure'); END;",
        ),
    ];

    for (step, trigger_sql) in triggers {
        let storage = Storage::open_in_memory().unwrap();
        let id = queued_memory(&storage, "converge after injected failure");
        storage
            .with_connection(|conn| Ok(conn.execute_batch(trigger_sql)?))
            .unwrap();

        let embedder = crate::embedding::TfIdfEmbedder::new(8);
        let failed = drain_pending_embeddings(&storage, &embedder, 10);
        assert!(failed.is_err(), "{step}: drain must report the failure");

        // Atomicity: nothing partial, and the job is explicitly failed (not stuck).
        assert_eq!(
            embedding_facts(&storage, id),
            EmbeddingFacts {
                rows: 0,
                flag: 0,
                queue_rows: 1,
                job: Some(("failed".to_string(), 1)),
            },
            "{step}: failure must roll back row + flag + completion together"
        );
        assert_eq!(
            embedding_health(&storage).status,
            crate::storage::DerivedIndexStatus::Degraded,
            "{step}: failed job must be visible in health"
        );

        // Retry through the explicit repair path converges.
        storage
            .with_connection(|conn| {
                conn.execute_batch("DROP TRIGGER inject_fail")?;
                let config = EmbeddingQueueHygieneConfig::default();
                run_embedding_queue_hygiene(conn, &config, true, true, false)?;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            drain_pending_embeddings(&storage, &embedder, 10).unwrap(),
            1
        );
        assert_eq!(
            embedding_facts(&storage, id),
            EmbeddingFacts {
                rows: 1,
                flag: 1,
                queue_rows: 1,
                job: Some(("complete".to_string(), 2)),
            },
            "{step}: retry must converge without duplicating rows or jobs"
        );
        assert_eq!(
            embedding_health(&storage).status,
            crate::storage::DerivedIndexStatus::Healthy,
            "{step}: converged state is healthy"
        );

        // Idempotent: a further drain finds nothing and changes nothing.
        assert_eq!(
            drain_pending_embeddings(&storage, &embedder, 10).unwrap(),
            0
        );
        assert_eq!(embedding_facts(&storage, id).rows, 1);
    }
}

/// Returns fewer embeddings than requested, like a misbehaving provider.
struct ShortBatchEmbedder;

impl crate::embedding::Embedder for ShortBatchEmbedder {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.5; 8])
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().skip(1).map(|_| vec![0.5; 8]).collect())
    }

    fn dimensions(&self) -> usize {
        8
    }

    fn model_name(&self) -> &str {
        "short-batch-test"
    }
}

#[test]
fn test_drain_short_provider_batch_fails_jobs_instead_of_stranding_processing() {
    let storage = Storage::open_in_memory().unwrap();
    let first = queued_memory(&storage, "short batch first");
    let second = queued_memory(&storage, "short batch second");

    let result = drain_pending_embeddings(&storage, &ShortBatchEmbedder, 10);
    assert!(result.is_err(), "a short provider batch must be an error");

    for id in [first, second] {
        let facts = embedding_facts(&storage, id);
        assert_eq!(
            facts.rows, 0,
            "no embedding row may be written for a bad batch"
        );
        assert_eq!(facts.flag, 0);
        assert_eq!(
            facts.job,
            Some(("failed".to_string(), 1)),
            "job must be failed (explicit), not stranded in processing"
        );
    }
}

/// Rewrites the memory content while the provider call is in flight, then
/// returns the embedding of the content it was originally given.
struct ContentChangingEmbedder {
    storage: Storage,
    memory_id: MemoryId,
}

impl crate::embedding::Embedder for ContentChangingEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        crate::embedding::TfIdfEmbedder::new(8).embed(text)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.storage.with_transaction(|conn| {
            crate::storage::queries::update_memory(
                conn,
                self.memory_id,
                &crate::types::UpdateMemoryInput {
                    content: Some("content rewritten during the provider call".to_string()),
                    memory_type: None,
                    tags: None,
                    metadata: None,
                    importance: None,
                    scope: None,
                    ttl_seconds: None,
                    event_time: None,
                    trigger_pattern: None,
                    media_url: None,
                },
            )?;
            Ok(())
        })?;
        texts.iter().map(|t| self.embed(t)).collect()
    }

    fn dimensions(&self) -> usize {
        8
    }

    fn model_name(&self) -> &str {
        "content-changing-test"
    }
}

#[test]
fn test_drain_does_not_persist_embedding_for_content_that_changed_in_flight() {
    let storage = Storage::open_in_memory().unwrap();
    let id = queued_memory(&storage, "original content to embed");
    let embedder = ContentChangingEmbedder {
        storage: storage.clone(),
        memory_id: id,
    };

    drain_pending_embeddings(&storage, &embedder, 10).unwrap();

    // The re-queue made by the content update owns the memory now: the stale
    // embedding of the old content must not be stored, flagged or completed.
    assert_eq!(
        embedding_facts(&storage, id),
        EmbeddingFacts {
            rows: 0,
            flag: 0,
            queue_rows: 1,
            job: Some(("pending".to_string(), 0)),
        }
    );
}

fn disk_storage(path: &std::path::Path) -> Storage {
    Storage::open(crate::types::StorageConfig {
        db_path: path.to_string_lossy().into_owned(),
        storage_mode: crate::types::StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .expect("open on-disk storage")
}

fn no_observer(_: MemoryId, _: &[f32]) {}

#[test]
fn test_crash_after_claim_then_reopen_recovers_and_drains_on_disk() {
    let dir = tempfile::tempdir().expect("caller-owned temp dir");
    let db_path = dir.path().join("engram.db");
    let embedder = crate::embedding::TfIdfEmbedder::new(8);
    let config = EmbeddingQueueHygieneConfig::default();

    // Process 1: memory committed with its job; the worker claims it and then
    // "crashes" while the provider call is in flight (storage dropped).
    let id = {
        let storage = disk_storage(&db_path);
        let id = queued_memory(&storage, "survives a worker crash");
        let claimed = super::drain::claim_pending_embeddings(&storage, 10).unwrap();
        assert_eq!(claimed.len(), 1);
        id
    };

    // Process 2: reopen the same file. The job is still `processing` and health
    // says so: not healthy just because the pending backlog is empty.
    let storage = disk_storage(&db_path);
    assert_eq!(
        embedding_facts(&storage, id),
        EmbeddingFacts {
            rows: 0,
            flag: 0,
            queue_rows: 1,
            job: Some(("processing".to_string(), 0)),
        }
    );
    let after_reopen = embedding_health(&storage);
    assert_eq!(
        after_reopen.status,
        crate::storage::DerivedIndexStatus::Backlogged
    );
    assert_eq!(after_reopen.pending_count, 1);
    assert_eq!(after_reopen.details["processing"], "1");
    assert_eq!(after_reopen.details["stale_processing"], "0");
    assert_eq!(after_reopen.indexed_count, 0);

    // Lease not expired yet: the cycle must not steal an in-flight job.
    let early = run_embedding_drain_cycle(&storage, &embedder, 10, &config, &no_observer).unwrap();
    assert_eq!(early, EmbeddingDrainReport::default());
    assert_eq!(
        embedding_facts(&storage, id).job,
        Some(("processing".to_string(), 0))
    );

    // Lease expired: visible as stale (degraded) until recovery runs.
    let expired = (Utc::now() - ChronoDuration::minutes(30)).to_rfc3339();
    storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE embedding_queue SET started_at = ? WHERE memory_id = ?",
                params![expired, id],
            )?;
            Ok(())
        })
        .unwrap();
    let stale = embedding_health(&storage);
    assert_eq!(stale.status, crate::storage::DerivedIndexStatus::Degraded);
    assert_eq!(stale.stale_count, 1);

    // The drain cycle (what the server worker runs on start) recovers and drains.
    let report = run_embedding_drain_cycle(&storage, &embedder, 10, &config, &no_observer).unwrap();
    assert_eq!(
        report,
        EmbeddingDrainReport {
            requeued_stale: 1,
            failed_exhausted: 0,
            drained: 1,
        }
    );
    let healed = EmbeddingFacts {
        rows: 1,
        flag: 1,
        queue_rows: 1,
        job: Some(("complete".to_string(), 1)),
    };
    assert_eq!(embedding_facts(&storage, id), healed);
    let health = embedding_health(&storage);
    assert_eq!(health.status, crate::storage::DerivedIndexStatus::Healthy);
    assert_eq!(
        (
            health.pending_count,
            health.stale_count,
            health.indexed_count
        ),
        (0, 0, 1)
    );
    drop(storage);

    // Process 3: reopen again; the converged state is durable and a further
    // cycle is a no-op (idempotent: no duplicate row, no duplicate job).
    let storage = disk_storage(&db_path);
    assert_eq!(embedding_facts(&storage, id), healed);
    assert_eq!(
        run_embedding_drain_cycle(&storage, &embedder, 10, &config, &no_observer).unwrap(),
        EmbeddingDrainReport::default()
    );
    assert_eq!(embedding_facts(&storage, id), healed);
    let stored = storage
        .with_connection(|conn| get_embedding(conn, id))
        .unwrap()
        .expect("embedding readback");
    assert_eq!(stored.len(), 8);
}

#[test]
fn test_drain_cycle_fails_stale_job_after_retry_budget() {
    let storage = Storage::open_in_memory().unwrap();
    let id = queued_memory(&storage, "exhausted lease");
    let expired = (Utc::now() - ChronoDuration::minutes(30)).to_rfc3339();
    storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE embedding_queue
                 SET status = 'processing', started_at = ?, retry_count = ?
                 WHERE memory_id = ?",
                params![expired, DEFAULT_MAX_EMBEDDING_RETRIES, id],
            )?;
            Ok(())
        })
        .unwrap();

    let embedder = crate::embedding::TfIdfEmbedder::new(8);
    let report = run_embedding_drain_cycle(
        &storage,
        &embedder,
        10,
        &EmbeddingQueueHygieneConfig::default(),
        &no_observer,
    )
    .unwrap();

    assert_eq!(report.failed_exhausted, 1);
    assert_eq!(report.drained, 0);
    assert_eq!(
        embedding_facts(&storage, id).job,
        Some(("failed".to_string(), DEFAULT_MAX_EMBEDDING_RETRIES))
    );
    assert_eq!(
        embedding_health(&storage).status,
        crate::storage::DerivedIndexStatus::Degraded
    );
}

#[test]
fn test_drain_observer_sees_only_persisted_embeddings() {
    let storage = Storage::open_in_memory().unwrap();
    let id = queued_memory(&storage, "observed once");
    let seen = Mutex::new(Vec::new());
    let observer = |memory_id: MemoryId, embedding: &[f32]| {
        seen.lock().push((memory_id, embedding.len()));
    };

    let embedder = crate::embedding::TfIdfEmbedder::new(8);
    drain_pending_embeddings_observed(&storage, &embedder, 10, &observer).unwrap();
    drain_pending_embeddings_observed(&storage, &embedder, 10, &observer).unwrap();

    assert_eq!(*seen.lock(), vec![(id, 8)]);
}

#[test]
fn test_enqueue_embedding_job_is_idempotent_and_never_resurrects_jobs() {
    let storage = Storage::open_in_memory().unwrap();
    let mut deferred_input = test_memory_input("enqueued later");
    deferred_input.defer_embedding = true;
    let id = storage
        .with_transaction(|conn| Ok(create_memory(conn, &deferred_input)?.id))
        .unwrap();
    assert_eq!(embedding_facts(&storage, id).queue_rows, 0);

    let enqueue = |memory_id| {
        storage
            .with_transaction(|conn| enqueue_embedding_job(conn, memory_id))
            .unwrap()
    };
    assert!(enqueue(id), "first enqueue creates the pending job");
    assert!(!enqueue(id), "second enqueue is a no-op");
    assert!(!enqueue(987_654), "unknown memory is never enqueued");

    // A failed job is left to explicit hygiene, never silently reset.
    storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 2 WHERE memory_id = ?",
                params![id],
            )?;
            Ok(())
        })
        .unwrap();
    assert!(!enqueue(id));
    assert_eq!(
        embedding_facts(&storage, id).job,
        Some(("failed".to_string(), 2))
    );

    // Once an embedding exists, no new job is created for the memory.
    let embedder = crate::embedding::TfIdfEmbedder::new(8);
    storage
        .with_connection(|conn| {
            conn.execute(
                "DELETE FROM embedding_queue WHERE memory_id = ?",
                params![id],
            )?;
            Ok(())
        })
        .unwrap();
    storage
        .with_transaction(|conn| {
            persist_computed_embedding(
                conn,
                id,
                "enqueued later",
                &crate::embedding::Embedder::embed(&embedder, "enqueued later")?,
                "test",
            )
        })
        .unwrap();
    assert!(!enqueue(id));
    assert_eq!(embedding_facts(&storage, id).queue_rows, 0);
}

#[tokio::test]
async fn test_embedding_worker_persists_row_flag_and_completion_atomically() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    crate::storage::migrations::run_migrations(&conn).unwrap();
    let id = create_memory(&conn, &test_memory_input("worker atomic persistence"))
        .unwrap()
        .id;
    conn.execute_batch(
        "CREATE TRIGGER inject_fail BEFORE UPDATE OF has_embedding ON memories
         WHEN NEW.has_embedding = 1
         BEGIN SELECT RAISE(ABORT, 'injected flag failure'); END;",
    )
    .unwrap();
    let worker = EmbeddingWorker {
        embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(8)),
        queue: EmbeddingQueue::new(1),
        conn: Arc::new(Mutex::new(conn)),
        batch_size: 1,
        batch_timeout: Duration::from_secs(1),
    };
    let request = || EmbeddingRequest {
        memory_id: id,
        content: "worker atomic persistence".to_string(),
    };

    let mut batch = vec![request()];
    let failed = worker.process_batch(&mut batch).await;
    assert!(
        matches!(failed, Err(EngramError::Embedding(ref m)) if m.contains("injected flag failure")),
        "expected the injected flag failure, got {failed:?}"
    );

    let facts = |conn: &Connection| -> (i64, i64, (String, i32)) {
        (
            conn.query_row("SELECT COUNT(*) FROM embeddings", [], |r| r.get(0))
                .unwrap(),
            conn.query_row(
                "SELECT has_embedding FROM memories WHERE id = ?",
                [id],
                |r| r.get(0),
            )
            .unwrap(),
            queue_state(conn, id).unwrap(),
        )
    };
    assert_eq!(
        facts(&worker.conn.lock()),
        (0, 0, ("failed".to_string(), 1))
    );

    // Repair and retry through the explicit path: converges, no duplicates.
    {
        let conn = worker.conn.lock();
        conn.execute_batch("DROP TRIGGER inject_fail").unwrap();
        run_embedding_queue_hygiene(
            &conn,
            &EmbeddingQueueHygieneConfig::default(),
            true,
            true,
            false,
        )
        .unwrap();
    }
    let mut batch = vec![request()];
    worker.process_batch(&mut batch).await.unwrap();
    assert_eq!(
        facts(&worker.conn.lock()),
        (1, 1, ("complete".to_string(), 2))
    );
}

#[test]
fn test_rebuild_embeddings_repairs_every_degraded_diagnostic_then_drains_healthy() {
    use crate::storage::queries::rebuild_derived_indexes;

    let storage = Storage::open_in_memory().unwrap();
    let make = |content: &str| {
        let mut input = test_memory_input(content);
        input.defer_embedding = true; // no job created by storage
        storage
            .with_transaction(|conn| Ok(create_memory(conn, &input)?.id))
            .unwrap()
    };
    // A: no row, no flag, no job.            -> missing_embedding_unqueued
    let missing_unqueued = make("rebuild missing unqueued");
    // B: flag set, no row, no job.           -> flagged_without_embedding_row
    let flagged_no_row = make("rebuild flagged without row");
    // C: flag set, job complete, no row.     -> complete_job_without_embedding_row
    let complete_no_row = make("rebuild complete job without row");
    storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET has_embedding = 1 WHERE id IN (?, ?)",
                params![flagged_no_row, complete_no_row],
            )?;
            conn.execute(
                "INSERT INTO embedding_queue (memory_id, status, queued_at, completed_at)
                 VALUES (?, 'complete', datetime('now'), datetime('now'))",
                params![complete_no_row],
            )?;
            Ok(())
        })
        .unwrap();

    let before = embedding_health(&storage);
    assert_eq!(before.status, crate::storage::DerivedIndexStatus::Degraded);
    assert_eq!(before.details["missing_embedding_unqueued"], "1");
    assert_eq!(before.details["flagged_without_embedding_row"], "2");
    assert_eq!(before.details["complete_job_without_embedding_row"], "1");

    // Dry-run reports the three memories and changes nothing.
    let dry = storage
        .with_transaction(|conn| rebuild_derived_indexes(conn, false, true, false))
        .unwrap();
    assert_eq!(dry.embeddings_missing, 3);
    assert_eq!(dry.embeddings_requeued, 0);
    assert_eq!(
        embedding_health(&storage).details["flagged_without_embedding_row"],
        "2"
    );

    // Apply: one transaction resets the flags and (re)queues all three.
    let applied = storage
        .with_transaction(|conn| rebuild_derived_indexes(conn, false, true, true))
        .unwrap();
    assert_eq!(applied.embeddings_requeued, 3);
    for id in [missing_unqueued, flagged_no_row, complete_no_row] {
        assert_eq!(
            embedding_facts(&storage, id),
            EmbeddingFacts {
                rows: 0,
                flag: 0,
                queue_rows: 1,
                job: Some(("pending".to_string(), 0)),
            }
        );
    }
    assert_eq!(
        embedding_health(&storage).status,
        crate::storage::DerivedIndexStatus::Backlogged
    );

    // Drain converges everything; rebuilding again is a no-op.
    let embedder = crate::embedding::TfIdfEmbedder::new(8);
    assert_eq!(
        drain_pending_embeddings(&storage, &embedder, 10).unwrap(),
        3
    );
    for id in [missing_unqueued, flagged_no_row, complete_no_row] {
        let facts = embedding_facts(&storage, id);
        assert_eq!((facts.rows, facts.flag, facts.queue_rows), (1, 1, 1));
        assert_eq!(facts.job.map(|j| j.0), Some("complete".to_string()));
    }
    assert_eq!(
        embedding_health(&storage).status,
        crate::storage::DerivedIndexStatus::Healthy
    );
    let again = storage
        .with_transaction(|conn| rebuild_derived_indexes(conn, false, true, true))
        .unwrap();
    assert_eq!(again.embeddings_requeued, 0);
    assert_eq!(again.embeddings_missing, 0);
}

fn queue_state(conn: &Connection, memory_id: MemoryId) -> Result<(String, i32)> {
    Ok(conn.query_row(
        "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
        params![memory_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?)
}

struct FailingEmbedder;

impl crate::embedding::Embedder for FailingEmbedder {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(EngramError::Embedding("forced embed failure".to_string()))
    }

    fn dimensions(&self) -> usize {
        8
    }

    fn model_name(&self) -> &str {
        "failing-test"
    }
}

fn test_memory_input(content: &str) -> CreateMemoryInput {
    CreateMemoryInput {
        content: content.to_string(),
        memory_type: MemoryType::Note,
        tags: vec![],
        metadata: HashMap::new(),
        importance: None,
        scope: Default::default(),
        workspace: None,
        tier: Default::default(),
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    }
}
