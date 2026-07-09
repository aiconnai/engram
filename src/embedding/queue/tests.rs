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
