//! Storage-scoped drain of pending SQL queue rows.

use chrono::Utc;

use crate::embedding::Embedder;
use crate::error::{EngramError, Result};
use crate::types::MemoryId;

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

    let limit = batch_size as i64;

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
            .query_map([limit], |row| {
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
