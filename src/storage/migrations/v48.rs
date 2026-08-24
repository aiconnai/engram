//! Database migration v48: Attestation hash version column
//!
//! Adds `hash_version` to `attestation_log` to support versioned hash schemes
//! (v1 legacy delimiter format, v2 canonical length-prefixed format)
//! without breaking existing chains or invalidating Ed25519 signatures.

use rusqlite::Connection;

use crate::error::Result;

pub(super) fn migrate_v48(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v48: Adding hash_version to attestation_log...");

    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='attestation_log'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if table_exists {
        let mut stmt = conn.prepare("PRAGMA table_info(attestation_log)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut has_hash_version = false;
        for col in columns {
            if col.unwrap_or_default() == "hash_version" {
                has_hash_version = true;
                break;
            }
        }

        if !has_hash_version {
            conn.execute(
                "ALTER TABLE attestation_log ADD COLUMN hash_version INTEGER NOT NULL DEFAULT 1;",
                [],
            )?;
        }
    }

    conn.execute("INSERT INTO schema_version (version) VALUES (48);", [])?;

    tracing::info!("Migration v48 complete: attestation_log updated with hash_version");
    Ok(())
}
