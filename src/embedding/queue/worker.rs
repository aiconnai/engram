//! Background worker that batches queued requests and persists embeddings.

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

use super::core::EmbeddingQueue;
use super::jobs::persist_computed_embedding;
use super::types::EmbeddingRequest;
use super::util::embedding_queue_db_write_error;
use crate::embedding::{create_embedder, Embedder};
use crate::error::{EngramError, Result};
use crate::types::{EmbeddingConfig, MemoryId};

/// Background worker for processing embeddings
pub struct EmbeddingWorker {
    pub(super) embedder: Arc<dyn Embedder>,
    pub(super) queue: EmbeddingQueue,
    pub(super) conn: Arc<Mutex<Connection>>,
    pub(super) batch_size: usize,
    pub(super) batch_timeout: Duration,
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
                            tracing::error!(
                                error = %crate::observability::redact::redacted(&e),
                                "Embedding batch processing failed"
                            );
                        }
                    }
                }

                // Process on timeout even if batch isn't full
                _ = batch_timer.tick() => {
                    if !batch.is_empty() {
                        if let Err(e) = self.process_batch(&mut batch).await {
                            tracing::error!(
                                error = %crate::observability::redact::redacted(&e),
                                "Embedding batch processing failed"
                            );
                        }
                    }
                }
            }
        }
    }

    /// Process a batch of embedding requests
    ///
    /// `pub(super)` so the queue test module can exercise it directly.
    pub(super) async fn process_batch(&self, batch: &mut Vec<EmbeddingRequest>) -> Result<()> {
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

            // Generate embeddings (no lock held, no transaction open).
            crate::observability::record_provider_call(
                crate::observability::ProviderCall::Embedding,
            );
            match self.embedder.embed_batch(&contents) {
                Ok(embeddings) if embeddings.len() == memory_ids.len() => {
                    let model = self.embedder.model_name();
                    let mut conn = self.conn.lock();
                    let mut failures: Vec<(MemoryId, String)> = Vec::new();

                    // One transaction per memory: embedding row + flag + job
                    // completion commit together or not at all.
                    for ((id, content), embedding) in
                        memory_ids.iter().zip(&contents).zip(&embeddings)
                    {
                        let outcome = conn
                            .transaction()
                            .map_err(|e| {
                                embedding_queue_db_write_error(
                                    "begin embedding persistence transaction",
                                    *id,
                                    e,
                                )
                            })
                            .and_then(|tx| {
                                let persisted = persist_computed_embedding(
                                    &tx, *id, content, embedding, model,
                                )?;
                                tx.commit().map_err(|e| {
                                    embedding_queue_db_write_error(
                                        "commit embedding persistence transaction",
                                        *id,
                                        e,
                                    )
                                })?;
                                Ok(persisted)
                            });
                        if let Err(e) = outcome {
                            failures.push((*id, e.to_string()));
                        }
                    }

                    if failures.is_empty() {
                        tracing::info!("Processed {} embeddings", memory_ids.len());
                        return Ok(());
                    }
                    let first = failures[0].1.clone();
                    let total = failures.len();
                    for (id, message) in &failures {
                        mark_job_failed(&conn, *id, message)?;
                    }
                    Err(EngramError::Embedding(format!(
                        "{total} of {} embeddings failed to persist (first: {first})",
                        memory_ids.len()
                    )))
                }
                other => {
                    // `error_msg` is persisted on the job and returned to the
                    // caller; logs only get the class (provider bodies echo prompts).
                    let (error_msg, error_class) = match other {
                        Ok(embeddings) => (
                            format!(
                                "embedding provider returned {} embeddings for {} texts",
                                embeddings.len(),
                                memory_ids.len()
                            ),
                            "embedding",
                        ),
                        Err(e) => {
                            crate::observability::record_provider_failure(
                                crate::observability::ProviderCall::Embedding,
                                &e,
                            );
                            (e.to_string(), crate::observability::redact::error_class(&e))
                        }
                    };
                    tracing::error!(
                        error_class,
                        batch = memory_ids.len(),
                        "Embedding batch failed"
                    );

                    let conn = self.conn.lock();
                    for &id in &memory_ids {
                        mark_job_failed(&conn, id, &error_msg)?;
                    }

                    Err(EngramError::Embedding(error_msg))
                }
            }
        })();

        batch.clear();
        result
    }
}

/// Mark a job failed (retry_count + 1). Only a job still in `processing` is
/// touched, so a job re-queued by a content update is not clobbered.
fn mark_job_failed(conn: &Connection, memory_id: MemoryId, error: &str) -> Result<()> {
    conn.execute(
        "UPDATE embedding_queue SET status = 'failed', error = ?, retry_count = retry_count + 1
         WHERE memory_id = ? AND status = 'processing'",
        params![error, memory_id],
    )
    .map_err(|db_error| {
        embedding_queue_db_write_error("mark embedding queue row as failed", memory_id, db_error)
    })?;
    Ok(())
}
