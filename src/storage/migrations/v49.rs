//! Database migration v49: Workspace-scoped media_assets deduplication
//!
//! Adds `workspace` column to `media_assets`, backfills it from linked `memories`,
//! and replaces the global `idx_media_assets_hash` unique index with
//! `idx_media_assets_workspace_hash` on `(workspace, file_hash)`.

use rusqlite::Connection;

use crate::error::Result;

pub(super) fn migrate_v49(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v49: Adding workspace column and scoped index to media_assets...");

    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='media_assets'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if table_exists {
        let mut stmt = conn.prepare("PRAGMA table_info(media_assets)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut has_workspace = false;
        for col in columns {
            if col.unwrap_or_default() == "workspace" {
                has_workspace = true;
                break;
            }
        }

        if !has_workspace {
            conn.execute(
                "ALTER TABLE media_assets ADD COLUMN workspace TEXT NOT NULL DEFAULT 'default';",
                [],
            )?;
        }

        conn.execute(
            "UPDATE media_assets SET workspace = COALESCE((SELECT m.workspace FROM memories m WHERE m.id = media_assets.memory_id), 'default');",
            [],
        )?;

        conn.execute_batch(
            "DROP INDEX IF EXISTS idx_media_assets_hash;
             CREATE UNIQUE INDEX IF NOT EXISTS idx_media_assets_workspace_hash ON media_assets(workspace, file_hash);",
        )?;
    }

    conn.execute("INSERT INTO schema_version (version) VALUES (49);", [])?;

    tracing::info!("Migration v49 complete: media_assets scoped by workspace");
    Ok(())
}
