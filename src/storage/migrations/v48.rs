//! Database migration v48: Attestation hash version column
//!
//! Adds `hash_version` to `attestation_log` to support versioned hash schemes
//! (v1 legacy delimiter format, v2 canonical length-prefixed format)
//! without breaking existing chains or invalidating Ed25519 signatures.

use rusqlite::Connection;

use crate::error::Result;

pub(super) fn migrate_v48(conn: &Connection) -> Result<()> {
    tracing::info!("Migration v48: Adding hash_version to attestation_log...");

    conn.execute_batch(
        r#"
        ALTER TABLE attestation_log ADD COLUMN hash_version INTEGER NOT NULL DEFAULT 1;

        INSERT INTO schema_version (version) VALUES (48);
        "#,
    )?;

    tracing::info!("Migration v48 complete: attestation_log updated with hash_version");
    Ok(())
}
