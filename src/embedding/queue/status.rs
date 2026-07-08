//! Per-memory embedding status/readback and failed-row retry.

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::{EngramError, Result};
use crate::types::{EmbeddingState, EmbeddingStatus, MemoryId};

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
