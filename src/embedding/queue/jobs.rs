//! Per-memory embedding job transitions shared by every writer.
//!
//! One memory has at most one row in `embedding_queue` (primary key
//! `memory_id`). The lifecycle of that row, and the local writes that move it,
//! are defined here so the immediate path (MCP create), the storage drain and
//! the in-process worker cannot drift apart:
//!
//! ```text
//!  enqueue            claim (drain/worker)         persist_computed_embedding
//!  -------> pending ------------------> processing --------------------------> complete
//!              ^                            |  \__ provider/persist error ____> failed
//!              |  stale lease expired       |
//!              +----------------------------+
//!
//!  failed   --(explicit hygiene `--requeue-failed`)-----------------------------> pending
//!  any state --(`update_memory` content change: INSERT OR REPLACE, flag reset)---> pending
//! ```
//!
//! Rules:
//! - The provider call is never made inside a transaction. Only the local
//!   writes (embedding row, `has_embedding` flag, job completion) are, and they
//!   commit or roll back together.
//! - Persisting is idempotent per memory: a second persist of an already
//!   completed job, of a memory whose content changed after the embedding was
//!   requested, or of a memory that no longer exists, writes nothing.
//! - A job moves backwards only through (a) the stale-`processing` lease
//!   recovery in [`super::hygiene`], which the drain cycle runs on start and on
//!   every interval, (b) explicit hygiene for retryable `failed` jobs (never
//!   automatic), and (c) a content update re-queuing the memory, which also
//!   invalidates any embedding persisted for the old content.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use super::util::embedding_queue_db_write_error;
use crate::error::Result;
use crate::types::MemoryId;

/// Serialize an embedding as little-endian `f32` bytes (the storage format).
pub(crate) fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Enqueue a background embedding job for `memory_id`.
///
/// Idempotent and non-destructive: nothing is written when the memory does not
/// exist, already has an embedding row, or already has a job in any state
/// (a `failed` job is only retried through explicit hygiene). Returns `true`
/// when a new pending job was created.
///
/// Call it inside the same transaction that created the memory so a memory
/// that is promised to the background queue always has its job.
pub fn enqueue_embedding_job(conn: &Connection, memory_id: MemoryId) -> Result<bool> {
    let inserted = conn
        .execute(
            "INSERT INTO embedding_queue (memory_id, status, queued_at)
             SELECT ?1, 'pending', ?2
             WHERE EXISTS (SELECT 1 FROM memories WHERE id = ?1)
               AND NOT EXISTS (SELECT 1 FROM embeddings WHERE memory_id = ?1)
             ON CONFLICT(memory_id) DO NOTHING",
            params![memory_id, Utc::now().to_rfc3339()],
        )
        .map_err(|e| embedding_queue_db_write_error("enqueue embedding job", memory_id, e))?;
    Ok(inserted > 0)
}

/// Persist one computed embedding: embedding row, `has_embedding` flag and job
/// completion. **Must run inside a transaction** (for example
/// `Storage::with_transaction`) so the three writes are atomic.
///
/// `expected_content` is the text the embedding was computed from. It is
/// compared with the stored content so an embedding of stale text is never
/// stored (the content update that changed it already re-queued the memory).
///
/// Returns `Ok(true)` when the embedding was persisted and `Ok(false)` when
/// nothing was written because the memory is gone, its content changed, or its
/// job was already completed/failed by someone else.
pub fn persist_computed_embedding(
    conn: &Connection,
    memory_id: MemoryId,
    expected_content: &str,
    embedding: &[f32],
    model: &str,
) -> Result<bool> {
    let stored_content: Option<String> = conn
        .query_row(
            "SELECT content FROM memories WHERE id = ?",
            params![memory_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| embedding_queue_db_write_error("read memory content", memory_id, e))?;
    if stored_content.as_deref() != Some(expected_content) {
        return Ok(false);
    }

    let now = Utc::now().to_rfc3339();
    let completed = conn
        .execute(
            "UPDATE embedding_queue
             SET status = 'complete', completed_at = ?, error = NULL
             WHERE memory_id = ? AND status IN ('pending', 'processing')",
            params![now, memory_id],
        )
        .map_err(|e| {
            embedding_queue_db_write_error("mark embedding queue row as complete", memory_id, e)
        })?;
    if completed == 0 {
        let has_job: bool = conn
            .query_row(
                "SELECT 1 FROM embedding_queue WHERE memory_id = ?",
                params![memory_id],
                |_| Ok(true),
            )
            .optional()
            .map_err(|e| embedding_queue_db_write_error("read embedding queue row", memory_id, e))?
            .unwrap_or(false);
        if has_job {
            // Already complete (idempotent retry) or failed (explicit repair owns it).
            return Ok(false);
        }
        // No job at all: a caller-owned embedding (no queue involved). Persist it.
    }

    conn.execute(
        "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
         VALUES (?, ?, ?, ?, ?)",
        params![
            memory_id,
            embedding_to_bytes(embedding),
            model,
            embedding.len(),
            now
        ],
    )
    .map_err(|e| embedding_queue_db_write_error("store embedding row", memory_id, e))?;

    conn.execute(
        "UPDATE memories SET has_embedding = 1 WHERE id = ?",
        params![memory_id],
    )
    .map_err(|e| embedding_queue_db_write_error("mark memory as having embedding", memory_id, e))?;

    Ok(true)
}
