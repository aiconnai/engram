//! Background worker that batches queued requests and persists embeddings.

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

use super::core::EmbeddingQueue;
use super::types::EmbeddingRequest;
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
