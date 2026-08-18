//! Storage queries for HNSW index checkpoints.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Result;

/// Record of an HNSW index checkpoint saved in SQLite.
#[derive(Debug, Clone)]
pub struct HnswCheckpointRecord {
    pub id: i64,
    pub model: String,
    pub dimensions: usize,
    pub metric: String,
    pub vector_count: usize,
    pub checkpoint_blob: Vec<u8>,
    pub created_at: String,
}

/// Save an HNSW index binary checkpoint BLOB to the database.
pub fn save_hnsw_checkpoint(
    conn: &Connection,
    model: &str,
    dimensions: usize,
    metric: &str,
    vector_count: usize,
    checkpoint_blob: &[u8],
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO hnsw_checkpoints (model, dimensions, metric, vector_count, checkpoint_blob, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![model, dimensions as i64, metric, vector_count as i64, checkpoint_blob, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get the latest HNSW checkpoint for a given model and dimension.
pub fn get_latest_hnsw_checkpoint(
    conn: &Connection,
    model: &str,
    dimensions: usize,
) -> Result<Option<HnswCheckpointRecord>> {
    conn.query_row(
        "SELECT id, model, dimensions, metric, vector_count, checkpoint_blob, created_at
         FROM hnsw_checkpoints
         WHERE model = ?1 AND dimensions = ?2
         ORDER BY id DESC
         LIMIT 1",
        params![model, dimensions as i64],
        |row| {
            let dim_i64: i64 = row.get(2)?;
            let count_i64: i64 = row.get(4)?;
            Ok(HnswCheckpointRecord {
                id: row.get(0)?,
                model: row.get(1)?,
                dimensions: dim_i64 as usize,
                metric: row.get(3)?,
                vector_count: count_i64 as usize,
                checkpoint_blob: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

/// Prune older checkpoints, keeping only the most recent `keep_count` entries per model and dimension.
pub fn prune_old_hnsw_checkpoints(
    conn: &Connection,
    model: &str,
    dimensions: usize,
    keep_count: usize,
) -> Result<usize> {
    let deleted = conn.execute(
        "DELETE FROM hnsw_checkpoints
         WHERE model = ?1 AND dimensions = ?2
           AND id NOT IN (
               SELECT id FROM hnsw_checkpoints
               WHERE model = ?1 AND dimensions = ?2
               ORDER BY id DESC
               LIMIT ?3
           )",
        params![model, dimensions as i64, keep_count as i64],
    )?;
    Ok(deleted)
}
