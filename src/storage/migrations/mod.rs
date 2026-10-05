//! Database migrations for Engram

mod v16_v25;
mod v1_v15;
mod v26_v33;
mod v34_v46;
mod v47;
mod v48;

#[cfg(test)]
mod tests;

use rusqlite::Connection;

use crate::error::{EngramError, Result};
use v16_v25::*;
use v1_v15::*;
use v26_v33::*;
use v34_v46::*;
use v47::*;
use v48::*;

/// Current schema version
pub const SCHEMA_VERSION: i32 = 48;

/// A single schema migration step.
type Migration = fn(&Connection) -> Result<()>;

/// Every migration, in order. Each step records its own version row.
const MIGRATIONS: [(i32, Migration); SCHEMA_VERSION as usize] = [
    (1, migrate_v1),
    (2, migrate_v2),
    (3, migrate_v3),
    (4, migrate_v4),
    (5, migrate_v5),
    (6, migrate_v6),
    (7, migrate_v7),
    (8, migrate_v8),
    (9, migrate_v9),
    (10, migrate_v10),
    (11, migrate_v11),
    (12, migrate_v12),
    (13, migrate_v13),
    (14, migrate_v14),
    (15, migrate_v15),
    (16, migrate_v16),
    (17, migrate_v17),
    (18, migrate_v18),
    (19, migrate_v19),
    (20, migrate_v20),
    (21, migrate_v21),
    (22, migrate_v22),
    (23, migrate_v23),
    (24, migrate_v24),
    (25, migrate_v25),
    (26, migrate_v26),
    (27, migrate_v27),
    (28, migrate_v28),
    (29, migrate_v29),
    (30, migrate_v30),
    (31, migrate_v31),
    (32, migrate_v32),
    (33, migrate_v33),
    (34, migrate_v34),
    (35, migrate_v35),
    (36, migrate_v36),
    (37, migrate_v37),
    (38, migrate_v38),
    (39, migrate_v39),
    (40, migrate_v40),
    (41, migrate_v41),
    (42, migrate_v42),
    (43, migrate_v43),
    (44, migrate_v44),
    (45, migrate_v45),
    (46, migrate_v46),
    (47, migrate_v47),
    (48, migrate_v48),
];

/// Migrations whose SQL toggles `PRAGMA foreign_keys`. That pragma is a no-op
/// inside a transaction, so the runner disables foreign keys *before* opening
/// the step's transaction and restores the previous setting afterwards
/// (SQLite's documented table-rebuild procedure).
const FOREIGN_KEYS_OFF_DURING: [i32; 1] = [45];

/// Run all pending migrations.
///
/// Each step runs in its own `BEGIN IMMEDIATE` transaction and re-reads the
/// schema version under the write lock, so:
/// - concurrent openers (threads or processes) serialize instead of applying
///   the same step twice;
/// - a step that fails is rolled back entirely (no half-applied schema) and
///   can be re-run on the next open;
/// - a database newer than [`SCHEMA_VERSION`] is refused without changes.
///
/// Must be called outside any open transaction.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    if !conn.is_autocommit() {
        return Err(EngramError::Storage(
            "run_migrations must not be called inside an open transaction".to_string(),
        ));
    }
    ensure_schema_version_table(conn)?;

    let current_version = read_schema_version(conn)?;
    ensure_supported_version(current_version)?;

    for (version, migrate) in MIGRATIONS {
        if version > current_version {
            apply_migration(conn, version, migrate)?;
        }
    }
    Ok(())
}

fn ensure_schema_version_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

fn read_schema_version(conn: &Connection) -> Result<i32> {
    Ok(conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?)
}

fn ensure_supported_version(current_version: i32) -> Result<()> {
    if current_version > SCHEMA_VERSION {
        return Err(EngramError::Storage(format!(
            "Database schema version {} is newer than supported version {}",
            current_version, SCHEMA_VERSION
        )));
    }
    Ok(())
}

/// Apply one step atomically, skipping it if another connection already did.
fn apply_migration(conn: &Connection, version: i32, migrate: Migration) -> Result<()> {
    let restore_fk = if FOREIGN_KEYS_OFF_DURING.contains(&version) {
        let enabled: bool = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
        conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
        Some(enabled)
    } else {
        None
    };

    let result = run_step_in_transaction(conn, version, migrate, restore_fk.is_some());

    if let Some(enabled) = restore_fk {
        let pragma = if enabled {
            "PRAGMA foreign_keys = ON;"
        } else {
            "PRAGMA foreign_keys = OFF;"
        };
        if let Err(err) = conn.execute_batch(pragma) {
            tracing::error!(version, error = %err, "failed to restore foreign_keys after migration");
            return result.and(Err(err.into()));
        }
    }
    result
}

fn run_step_in_transaction(
    conn: &Connection,
    version: i32,
    migrate: Migration,
    check_foreign_keys: bool,
) -> Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    let outcome = (|| {
        let current = read_schema_version(conn)?;
        ensure_supported_version(current)?;
        if current >= version {
            return Ok(());
        }
        // With foreign keys disabled for this step, refuse any violation the
        // step introduces (pre-existing ones are not this step's doing).
        let before = if check_foreign_keys {
            foreign_key_violations(conn)?
        } else {
            0
        };
        migrate(conn)?;
        if check_foreign_keys {
            let after = foreign_key_violations(conn)?;
            if after > before {
                return Err(EngramError::Storage(format!(
                    "migration v{version} introduced {} foreign key violation(s)",
                    after - before
                )));
            }
        }
        Ok(())
    })();

    let outcome = outcome.and_then(|()| conn.execute_batch("COMMIT;").map_err(Into::into));
    if outcome.is_err() && !conn.is_autocommit() {
        // Covers both a failed step and a failed COMMIT (e.g. deferred
        // constraint), which leaves the transaction open.
        if let Err(rollback_err) = conn.execute_batch("ROLLBACK;") {
            tracing::error!(
                version,
                error = %rollback_err,
                "rollback after failed migration also failed"
            );
        }
    }
    outcome
}

/// Number of rows reported by `PRAGMA foreign_key_check`.
fn foreign_key_violations(conn: &Connection) -> Result<usize> {
    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let mut rows = stmt.query([])?;
    let mut count = 0usize;
    while rows.next()?.is_some() {
        count += 1;
    }
    Ok(count)
}
