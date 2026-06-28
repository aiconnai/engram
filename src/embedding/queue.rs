//! Async embedding queue with batch processing (RML-873)
//!
//! Embeddings are computed in the background to avoid blocking writes.
//! The queue supports batching for efficient API usage.

use async_channel::{bounded, Receiver, Sender};
use chrono::{Duration as ChronoDuration, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

use super::{create_embedder, Embedder};
use crate::error::{EngramError, Result};
use crate::types::{EmbeddingConfig, EmbeddingState, EmbeddingStatus, MemoryId};

/// Default age after which an in-flight SQL queue row is considered abandoned.
pub const DEFAULT_STALE_PROCESSING_AFTER: Duration = Duration::from_secs(15 * 60);

/// Default retry budget for queue hygiene and health accounting.
pub const DEFAULT_MAX_EMBEDDING_RETRIES: i32 = 3;

/// Default age after which completed embedding rows are eligible for retention pruning.
pub const DEFAULT_COMPLETE_RETENTION: Duration = Duration::from_secs(14 * 24 * 60 * 60);

/// Retry count bucket threshold for health reporting.
const RETRY_COUNT_BUCKET_3_PLUS: i32 = 3;

/// Parameters that govern explicit embedding-queue hygiene.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddingQueueHygieneConfig {
    pub stale_processing_after: Duration,
    pub max_retries: i32,
    pub complete_retention: Duration,
}

impl Default for EmbeddingQueueHygieneConfig {
    fn default() -> Self {
        Self {
            stale_processing_after: DEFAULT_STALE_PROCESSING_AFTER,
            max_retries: DEFAULT_MAX_EMBEDDING_RETRIES,
            complete_retention: DEFAULT_COMPLETE_RETENTION,
        }
    }
}

/// Message for the embedding queue
#[derive(Debug)]
pub struct EmbeddingRequest {
    pub memory_id: MemoryId,
    pub content: String,
}

/// Read-only summary of durable embedding queue health.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingQueueHealth {
    pub pending: i64,
    pub processing: i64,
    pub stale_processing: i64,
    pub complete: i64,
    pub failed: i64,
    pub retryable_failed: i64,
    pub exhausted_failed: i64,
    pub zero_retry_failed: i64,
    pub max_retry_count: i32,
    pub oldest_pending_seconds: Option<i64>,
    pub oldest_processing_age_seconds: Option<i64>,
    pub oldest_failed_age_seconds: Option<i64>,
    pub retry_count_0: i64,
    pub retry_count_1: i64,
    pub retry_count_2: i64,
    pub retry_count_3_plus: i64,
}

/// Result of an explicit queue hygiene pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingQueueHygieneReport {
    pub requeued_stale: i64,
    pub failed_exhausted: i64,
    pub requeued_failed: i64,
    pub pruned_complete: i64,
}

/// Embedding queue for async processing
pub struct EmbeddingQueue {
    sender: Sender<EmbeddingRequest>,
    receiver: Receiver<EmbeddingRequest>,
    batch_size: usize,
}

impl EmbeddingQueue {
    /// Create a new embedding queue
    pub fn new(batch_size: usize) -> Self {
        let (sender, receiver) = bounded(10000); // Buffer up to 10k requests
        Self {
            sender,
            receiver,
            batch_size,
        }
    }

    /// Queue a memory for embedding
    pub async fn queue(&self, memory_id: MemoryId, content: String) -> Result<()> {
        self.sender
            .send(EmbeddingRequest { memory_id, content })
            .await
            .map_err(|e| EngramError::Embedding(format!("Queue send error: {}", e)))?;
        Ok(())
    }

    /// Queue a memory (blocking version for sync contexts)
    pub fn queue_blocking(&self, memory_id: MemoryId, content: String) -> Result<()> {
        self.sender
            .send_blocking(EmbeddingRequest { memory_id, content })
            .map_err(|e| EngramError::Embedding(format!("Queue send error: {}", e)))?;
        Ok(())
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.receiver.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }

    /// Get receiver for worker
    pub fn receiver(&self) -> Receiver<EmbeddingRequest> {
        self.receiver.clone()
    }
}

impl Clone for EmbeddingQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            batch_size: self.batch_size,
        }
    }
}

/// Background worker for processing embeddings
pub struct EmbeddingWorker {
    embedder: Arc<dyn Embedder>,
    queue: EmbeddingQueue,
    conn: Arc<Mutex<Connection>>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl EmbeddingWorker {
    /// Create a new embedding worker
    pub fn new(
        config: EmbeddingConfig,
        queue: EmbeddingQueue,
        conn: Arc<Mutex<Connection>>,
    ) -> Result<Self> {
        let embedder = create_embedder(&config)?;
        let batch_size = config.batch_size;

        Ok(Self {
            embedder,
            queue,
            conn,
            batch_size,
            batch_timeout: Duration::from_secs(5),
        })
    }

    /// Run the worker (call in a spawned task)
    pub async fn run(&self) {
        let receiver = self.queue.receiver();
        let mut batch: Vec<EmbeddingRequest> = Vec::with_capacity(self.batch_size);
        let mut batch_timer = interval(self.batch_timeout);

        loop {
            tokio::select! {
                // Receive new request
                Ok(request) = receiver.recv() => {
                    batch.push(request);

                    // Process if batch is full
                    if batch.len() >= self.batch_size {
                        if let Err(e) = self.process_batch(&mut batch).await {
                            tracing::error!("Embedding batch processing failed: {}", e);
                        }
                    }
                }

                // Process on timeout even if batch isn't full
                _ = batch_timer.tick() => {
                    if !batch.is_empty() {
                        if let Err(e) = self.process_batch(&mut batch).await {
                            tracing::error!("Embedding batch processing failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Process a batch of embedding requests
    async fn process_batch(&self, batch: &mut Vec<EmbeddingRequest>) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let memory_ids: Vec<MemoryId> = batch.iter().map(|r| r.memory_id).collect();
        let contents: Vec<&str> = batch.iter().map(|r| r.content.as_str()).collect();

        let result = (|| -> Result<()> {
            // Mark as processing
            {
                let conn = self.conn.lock();
                let now = Utc::now().to_rfc3339();
                for &id in &memory_ids {
                    conn.execute(
                        "UPDATE embedding_queue SET status = 'processing', started_at = ? WHERE memory_id = ?",
                        params![now, id],
                    )
                    .map_err(|e| {
                        embedding_queue_db_write_error(
                            "mark embedding queue row as processing",
                            id,
                            e,
                        )
                    })?;
                }
            }

            // Generate embeddings
            match self.embedder.embed_batch(&contents) {
                Ok(embeddings) => {
                    let conn = self.conn.lock();
                    let now = Utc::now().to_rfc3339();
                    let model = self.embedder.model_name();
                    let dimensions = self.embedder.dimensions();

                    for (id, embedding) in memory_ids.iter().zip(embeddings.iter()) {
                        // Serialize embedding to bytes
                        let embedding_bytes: Vec<u8> =
                            embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

                        // Store embedding
                        conn.execute(
                            "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                         VALUES (?, ?, ?, ?, ?)",
                            params![id, embedding_bytes, model, dimensions, now],
                        )
                        .map_err(|e| embedding_queue_db_write_error("store embedding row", *id, e))?;

                        // Update memory
                        conn.execute(
                            "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                            params![id],
                        )
                        .map_err(|e| {
                            embedding_queue_db_write_error(
                                "mark memory as having embedding",
                                *id,
                                e,
                            )
                        })?;

                        // Mark as complete
                        conn.execute(
                            "UPDATE embedding_queue SET status = 'complete', completed_at = ? WHERE memory_id = ?",
                            params![now, id],
                        )
                        .map_err(|e| {
                            embedding_queue_db_write_error(
                                "mark embedding queue row as complete",
                                *id,
                                e,
                            )
                        })?;
                    }

                    tracing::info!("Processed {} embeddings", memory_ids.len());
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Embedding batch failed: {}", e);

                    let conn = self.conn.lock();
                    let error_msg = e.to_string();

                    for &id in &memory_ids {
                        conn.execute(
                            "UPDATE embedding_queue SET status = 'failed', error = ?, retry_count = retry_count + 1 WHERE memory_id = ?",
                            params![error_msg, id],
                        )
                        .map_err(|db_error| {
                            embedding_queue_db_write_error(
                                "mark embedding queue row as failed",
                                id,
                                db_error,
                            )
                        })?;
                    }

                    Err(EngramError::Embedding(error_msg))
                }
            }
        })();

        batch.clear();
        result
    }
}

fn embedding_queue_db_write_error(
    operation: &str,
    memory_id: MemoryId,
    error: rusqlite::Error,
) -> EngramError {
    EngramError::Embedding(format!(
        "database write failed while {operation} for memory_id={memory_id}: {error}"
    ))
}

/// Get embedding status for a memory
pub fn get_embedding_status(conn: &Connection, memory_id: MemoryId) -> Result<EmbeddingStatus> {
    let row = conn.query_row(
        "SELECT status, queued_at, completed_at, error FROM embedding_queue WHERE memory_id = ?",
        params![memory_id],
        |row| {
            let status_str: String = row.get(0)?;
            let queued_at: Option<String> = row.get(1)?;
            let completed_at: Option<String> = row.get(2)?;
            let error: Option<String> = row.get(3)?;

            let status = match status_str.as_str() {
                "pending" => EmbeddingState::Pending,
                "processing" => EmbeddingState::Processing,
                "complete" => EmbeddingState::Complete,
                "failed" => EmbeddingState::Failed,
                _ => EmbeddingState::Pending,
            };

            Ok(EmbeddingStatus {
                memory_id,
                status,
                queued_at: queued_at.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                completed_at: completed_at.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                error,
            })
        },
    );

    match row {
        Ok(status) => Ok(status),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Check if memory has embedding
            let has_embedding: bool = conn
                .query_row(
                    "SELECT has_embedding FROM memories WHERE id = ?",
                    params![memory_id],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            Ok(EmbeddingStatus {
                memory_id,
                status: if has_embedding {
                    EmbeddingState::Complete
                } else {
                    EmbeddingState::Pending
                },
                queued_at: None,
                completed_at: None,
                error: None,
            })
        }
        Err(e) => Err(EngramError::Database(e)),
    }
}

/// Get embedding for a memory
pub fn get_embedding(conn: &Connection, memory_id: MemoryId) -> Result<Option<Vec<f32>>> {
    let row = conn.query_row(
        "SELECT embedding, dimensions FROM embeddings WHERE memory_id = ?",
        params![memory_id],
        |row| {
            let bytes: Vec<u8> = row.get(0)?;
            let dimensions: usize = row.get(1)?;
            Ok((bytes, dimensions))
        },
    );

    match row {
        Ok((bytes, dimensions)) => {
            let expected_len = dimensions.checked_mul(4).ok_or_else(|| {
                EngramError::InvalidInput("Embedding dimensions too large".to_string())
            })?;
            if bytes.len() != expected_len {
                return Err(EngramError::InvalidInput(format!(
                    "Embedding byte length {} does not match dimensions {}",
                    bytes.len(),
                    dimensions
                )));
            }

            // Deserialize from bytes
            let mut embedding = Vec::with_capacity(dimensions);
            for chunk in bytes.chunks_exact(4) {
                let arr: [u8; 4] = chunk.try_into().unwrap();
                embedding.push(f32::from_le_bytes(arr));
            }
            Ok(Some(embedding))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(EngramError::Database(e)),
    }
}

/// Retry failed embeddings
#[allow(dead_code)]
pub fn retry_failed_embeddings(conn: &Connection, max_retries: i32) -> Result<Vec<MemoryId>> {
    let mut stmt = conn.prepare(
        "SELECT eq.memory_id, m.content FROM embedding_queue eq
         JOIN memories m ON eq.memory_id = m.id
         WHERE eq.status = 'failed' AND eq.retry_count < ?",
    )?;

    let failed: Vec<(MemoryId, String)> = stmt
        .query_map([max_retries], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let ids: Vec<MemoryId> = failed.iter().map(|(id, _)| *id).collect();

    // Reset status to pending
    for &id in &ids {
        conn.execute(
            "UPDATE embedding_queue SET status = 'pending', error = NULL WHERE memory_id = ?",
            params![id],
        )?;
    }

    Ok(ids)
}

/// Return a read-only health summary for the durable embedding queue.
pub fn get_embedding_queue_health(
    conn: &Connection,
    stale_after: Duration,
    max_retries: i32,
) -> Result<EmbeddingQueueHealth> {
    let config = EmbeddingQueueHygieneConfig {
        stale_processing_after: stale_after,
        max_retries,
        complete_retention: DEFAULT_COMPLETE_RETENTION,
    };
    get_embedding_queue_health_with_config(conn, &config)
}

/// Return a read-only health summary for the durable embedding queue using
/// an explicit hygiene policy.
pub fn get_embedding_queue_health_with_config(
    conn: &Connection,
    config: &EmbeddingQueueHygieneConfig,
) -> Result<EmbeddingQueueHealth> {
    let stale_cutoff = stale_cutoff_rfc3339(config.stale_processing_after)?;

    let pending = count_queue_status(conn, "pending")?;
    let processing = count_queue_status(conn, "processing")?;
    let complete = count_queue_status(conn, "complete")?;
    let failed = count_queue_status(conn, "failed")?;

    let stale_processing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'processing' AND started_at IS NOT NULL AND started_at <= ?",
        params![stale_cutoff],
        |row| row.get(0),
    )?;

    let retryable_failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'failed' AND retry_count < ?",
        params![config.max_retries],
        |row| row.get(0),
    )?;

    let exhausted_failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'failed' AND retry_count >= ?",
        params![config.max_retries],
        |row| row.get(0),
    )?;

    let zero_retry_failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'failed' AND retry_count = 0",
        [],
        |row| row.get(0),
    )?;

    let oldest_processing_age_seconds =
        oldest_timestamp_seconds(conn, "processing", Some("started_at"))?;

    let oldest_failed_age_seconds = oldest_timestamp_seconds(
        conn,
        "failed",
        Some("COALESCE(completed_at, started_at, queued_at)"),
    )?;

    let max_retry_count = conn
        .query_row(
            "SELECT COALESCE(MAX(retry_count), 0) FROM embedding_queue",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let retry_count_0 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count = 0",
        [],
        |row| row.get(0),
    )?;
    let retry_count_1 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count = 1",
        [],
        |row| row.get(0),
    )?;
    let retry_count_2 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count = 2",
        [],
        |row| row.get(0),
    )?;
    let retry_count_3_plus = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count >= ?",
        params![RETRY_COUNT_BUCKET_3_PLUS],
        |row| row.get(0),
    )?;

    let oldest_pending_at: Option<String> = conn.query_row(
        "SELECT MIN(queued_at) FROM embedding_queue WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let oldest_pending_seconds = oldest_pending_at.and_then(|queued_at| {
        chrono::DateTime::parse_from_rfc3339(&queued_at)
            .ok()
            .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0))
    });

    Ok(EmbeddingQueueHealth {
        pending,
        processing,
        stale_processing,
        complete,
        failed,
        retryable_failed,
        exhausted_failed,
        zero_retry_failed,
        max_retry_count,
        oldest_pending_seconds,
        oldest_processing_age_seconds,
        oldest_failed_age_seconds,
        retry_count_0,
        retry_count_1,
        retry_count_2,
        retry_count_3_plus,
    })
}

/// Explicit repair pass over stale queue work.
///
/// This function is non-invasive when `apply == false` (dry-run).
/// In read-only mode it only reports candidate counts.
pub fn run_embedding_queue_hygiene(
    conn: &Connection,
    config: &EmbeddingQueueHygieneConfig,
    requeue_retryable_failed: bool,
    apply: bool,
    prune_complete: bool,
) -> Result<EmbeddingQueueHygieneReport> {
    let stale_cutoff = stale_cutoff_rfc3339(config.stale_processing_after)?;
    let now = Utc::now().to_rfc3339();
    let retention_cutoff = complete_retention_cutoff_rfc3339(config.complete_retention)?;
    let stale_processing_where =
        "status = 'processing' AND started_at IS NOT NULL AND started_at <= ? AND retry_count < ?";
    let failed_retryable_where = "status = 'failed' AND retry_count < ? AND retry_count >= 0";
    let failed_exhausted_where =
        "status = 'processing' AND started_at IS NOT NULL AND started_at <= ? AND retry_count >= ?";
    let complete_prunable_where =
        "status = 'complete' AND COALESCE(completed_at, queued_at) IS NOT NULL AND COALESCE(completed_at, queued_at) <= ?";

    let requeued_stale = if apply {
        conn.execute(
            &format!(
                "UPDATE embedding_queue SET status = 'pending', started_at = NULL, error = NULL, \
                 retry_count = retry_count + 1, queued_at = ? WHERE {stale_processing_where}"
            ),
            params![now, stale_cutoff, config.max_retries],
        )? as i64
    } else {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM embedding_queue WHERE {stale_processing_where}"),
            params![stale_cutoff, config.max_retries],
            |row| row.get(0),
        )?
    };

    let failed_exhausted = if apply {
        conn.execute(
            &format!(
                "UPDATE embedding_queue
                 SET status = 'failed',
                     error = 'embedding processing lease expired after retry budget',
                     completed_at = ?
                 WHERE {failed_exhausted_where}"
            ),
            params![now, stale_cutoff, config.max_retries],
        )? as i64
    } else {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM embedding_queue WHERE {failed_exhausted_where}"),
            params![stale_cutoff, config.max_retries],
            |row| row.get(0),
        )?
    };

    let requeued_failed = if requeue_retryable_failed {
        if apply {
            conn.execute(
                &format!(
                    "UPDATE embedding_queue
                     SET status = 'pending',
                         error = NULL,
                         retry_count = retry_count + 1,
                         queued_at = ?,
                         started_at = NULL
                     WHERE {failed_retryable_where}"
                ),
                params![now, config.max_retries],
            )? as i64
        } else {
            conn.query_row(
                &format!("SELECT COUNT(*) FROM embedding_queue WHERE {failed_retryable_where}"),
                params![config.max_retries],
                |row| row.get(0),
            )?
        }
    } else {
        0
    };

    let pruned_complete = if prune_complete && config.complete_retention.as_secs() > 0 {
        if apply {
            conn.execute(
                &format!("DELETE FROM embedding_queue WHERE {complete_prunable_where}"),
                params![retention_cutoff],
            )? as i64
        } else {
            conn.query_row(
                &format!("SELECT COUNT(*) FROM embedding_queue WHERE {complete_prunable_where}"),
                params![retention_cutoff],
                |row| row.get(0),
            )?
        }
    } else {
        0
    };

    Ok(EmbeddingQueueHygieneReport {
        requeued_stale,
        failed_exhausted,
        requeued_failed,
        pruned_complete,
    })
}

/// Requeue abandoned `processing` rows, or fail them if their retry budget is exhausted.
///
/// This is an explicit repair path for stale in-flight work. Health checks do
/// not call this function.
pub fn requeue_stale_processing_embeddings(
    conn: &Connection,
    stale_after: Duration,
    max_retries: i32,
) -> Result<EmbeddingQueueHygieneReport> {
    let stale_cutoff = stale_cutoff_rfc3339(stale_after)?;
    let now = Utc::now().to_rfc3339();

    let requeued_stale = conn.execute(
        "UPDATE embedding_queue
         SET status = 'pending',
             started_at = NULL,
             error = NULL,
             retry_count = retry_count + 1,
             queued_at = ?
         WHERE status = 'processing'
           AND started_at IS NOT NULL
           AND started_at <= ?
           AND retry_count < ?",
        params![now, stale_cutoff, max_retries],
    )? as i64;

    let failed_exhausted = conn.execute(
        "UPDATE embedding_queue
         SET status = 'failed',
             error = 'embedding processing lease expired after retry budget',
             completed_at = ?
         WHERE status = 'processing'
           AND started_at IS NOT NULL
           AND started_at <= ?
           AND retry_count >= ?",
        params![now, stale_cutoff, max_retries],
    )? as i64;

    Ok(EmbeddingQueueHygieneReport {
        requeued_stale,
        failed_exhausted,
        requeued_failed: 0,
        pruned_complete: 0,
    })
}

fn count_queue_status(conn: &Connection, status: &str) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = ?",
        params![status],
        |row| row.get(0),
    )?)
}

fn stale_cutoff_rfc3339(stale_after: Duration) -> Result<String> {
    let stale_after = ChronoDuration::from_std(stale_after)
        .map_err(|_| EngramError::InvalidInput("stale_after duration is too large".to_string()))?;
    Ok((Utc::now() - stale_after).to_rfc3339())
}

fn complete_retention_cutoff_rfc3339(complete_retention: Duration) -> Result<String> {
    let complete_retention = ChronoDuration::from_std(complete_retention).map_err(|_| {
        EngramError::InvalidInput("complete_retention duration is too large".to_string())
    })?;
    Ok((Utc::now() - complete_retention).to_rfc3339())
}

fn oldest_timestamp_seconds(
    conn: &Connection,
    status: &str,
    ts_expr: Option<&str>,
) -> Result<Option<i64>> {
    let ts_sql = ts_expr.unwrap_or("queued_at");
    let oldest: Option<String> = conn.query_row(
        &format!(
            "SELECT MIN({}) FROM embedding_queue WHERE status = ? AND {} IS NOT NULL",
            ts_sql, ts_sql
        ),
        params![status],
        |row| row.get(0),
    )?;
    Ok(oldest.and_then(|ts| {
        chrono::DateTime::parse_from_rfc3339(&ts)
            .ok()
            .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0))
    }))
}

/// Drain up to `batch_size` pending entries from the SQL `embedding_queue`
/// table, compute their embeddings, and persist them.
///
/// Lock discipline: this function takes `&Storage` rather than `&Connection`
/// so it can scope each `with_connection` acquisition narrowly. The
/// `embedder.embed_batch()` call (which is a blocking HTTP request for cloud
/// backends like OpenAI) MUST run with the connection lock released —
/// otherwise every other DB operation in the server stalls behind every drain
/// cycle.
///
/// Flow:
///   1. Lock acquired briefly: SELECT pending rows + mark them 'processing'
///   2. Lock RELEASED — embed_batch runs the network call
///   3. Lock re-acquired briefly: persist embeddings + mark 'complete', or
///      mark 'failed' on error
///
/// On success, each processed memory has:
///   - a row in `embeddings`
///   - `memories.has_embedding = 1`
///   - `embedding_queue.status = 'complete'`
///
/// On failure, the queue rows are marked `failed` with the error message and
/// `retry_count` is incremented so `retry_failed_embeddings` can re-queue
/// them later.
///
/// Returns the number of memories processed (success or failure). Returns 0
/// when the queue is empty.
///
/// Fixes #10 sintoma A.
pub fn drain_pending_embeddings(
    storage: &crate::storage::Storage,
    embedder: &dyn Embedder,
    batch_size: usize,
) -> Result<usize> {
    use rusqlite::params;

    // ── Phase 1: claim a batch atomically ───────────────────────────────────
    // Wrap SELECT + mark-as-processing in a transaction so a hypothetical
    // second drainer can't claim the same rows between the two statements.
    // Today only one drain thread is spawned, but the transaction is cheap
    // and removes the race as a class.
    let claimed: Vec<(MemoryId, String)> = storage.with_transaction(|tx| {
        let mut stmt = tx.prepare(
            "SELECT eq.memory_id, m.content
             FROM embedding_queue eq
             JOIN memories m ON eq.memory_id = m.id
             WHERE eq.status = 'pending' AND m.valid_to IS NULL
             ORDER BY eq.queued_at
             LIMIT ?",
        )?;

        let rows: Vec<(MemoryId, String)> = stmt
            .query_map(params![batch_size as i64], |row| {
                Ok((row.get::<_, MemoryId>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        drop(stmt);

        if !rows.is_empty() {
            let now = Utc::now().to_rfc3339();
            for &(id, _) in &rows {
                tx.execute(
                    "UPDATE embedding_queue SET status = 'processing', started_at = ?
                     WHERE memory_id = ?",
                    params![now, id],
                )?;
            }
        }

        Ok(rows)
    })?;

    if claimed.is_empty() {
        return Ok(0);
    }

    let memory_ids: Vec<MemoryId> = claimed.iter().map(|(id, _)| *id).collect();
    let contents: Vec<&str> = claimed.iter().map(|(_, c)| c.as_str()).collect();

    // ── Phase 2: NO LOCK HELD — run the (potentially slow) network call ─────
    let embed_result = embedder.embed_batch(&contents);
    let model = embedder.model_name().to_string();
    let dimensions = embedder.dimensions();

    // ── Phase 3: re-acquire lock to persist results ─────────────────────────
    storage.with_connection(|conn| match &embed_result {
        Ok(embeddings) => {
            let now = Utc::now().to_rfc3339();
            for (id, embedding) in memory_ids.iter().zip(embeddings.iter()) {
                let embedding_bytes: Vec<u8> =
                    embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

                conn.execute(
                    "INSERT OR REPLACE INTO embeddings
                         (memory_id, embedding, model, dimensions, created_at)
                         VALUES (?, ?, ?, ?, ?)",
                    params![id, embedding_bytes, &model, dimensions, now],
                )?;

                conn.execute(
                    "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                    params![id],
                )?;

                conn.execute(
                    "UPDATE embedding_queue SET status = 'complete', completed_at = ?
                         WHERE memory_id = ?",
                    params![now, id],
                )?;
            }
            Ok(memory_ids.len())
        }
        Err(e) => {
            let error_msg = e.to_string();
            for &id in &memory_ids {
                conn.execute(
                    "UPDATE embedding_queue SET status = 'failed', error = ?,
                         retry_count = retry_count + 1
                         WHERE memory_id = ?",
                    params![error_msg, id],
                )?;
            }
            Err(EngramError::Embedding(error_msg))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::create_memory;
    use crate::storage::Storage;
    use crate::types::{CreateMemoryInput, MemoryType};
    use std::collections::HashMap;

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
}
