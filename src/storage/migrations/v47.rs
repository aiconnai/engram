//! Database migration v47: HNSW index checkpoints table

use rusqlite::Connection;

use crate::error::Result;

pub(super) fn migrate_v47(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v47: Creating hnsw_checkpoints table...");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS hnsw_checkpoints (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            model TEXT NOT NULL DEFAULT 'default',
            dimensions INTEGER NOT NULL,
            metric TEXT NOT NULL DEFAULT 'cosine',
            vector_count INTEGER NOT NULL DEFAULT 0,
            checkpoint_blob BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_hnsw_checkpoints_lookup
            ON hnsw_checkpoints(model, dimensions, created_at DESC);

        INSERT INTO schema_version (version) VALUES (47);
        "#,
    )?;

    tracing::info!("Migration v47 complete: hnsw_checkpoints table created");
    Ok(())
}
