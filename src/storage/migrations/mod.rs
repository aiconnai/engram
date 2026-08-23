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

/// Run all migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version > SCHEMA_VERSION {
        return Err(EngramError::Storage(format!(
            "Database schema version {} is newer than supported version {}",
            current_version, SCHEMA_VERSION
        )));
    }

    if current_version < 1 {
        migrate_v1(conn)?;
    }

    if current_version < 2 {
        migrate_v2(conn)?;
    }

    if current_version < 3 {
        migrate_v3(conn)?;
    }

    if current_version < 4 {
        migrate_v4(conn)?;
    }

    if current_version < 5 {
        migrate_v5(conn)?;
    }

    if current_version < 6 {
        migrate_v6(conn)?;
    }

    if current_version < 7 {
        migrate_v7(conn)?;
    }

    if current_version < 8 {
        migrate_v8(conn)?;
    }

    if current_version < 9 {
        migrate_v9(conn)?;
    }

    if current_version < 10 {
        migrate_v10(conn)?;
    }

    if current_version < 11 {
        migrate_v11(conn)?;
    }

    if current_version < 12 {
        migrate_v12(conn)?;
    }

    if current_version < 13 {
        migrate_v13(conn)?;
    }

    if current_version < 14 {
        migrate_v14(conn)?;
    }

    if current_version < 15 {
        migrate_v15(conn)?;
    }

    if current_version < 16 {
        migrate_v16(conn)?;
    }

    if current_version < 17 {
        migrate_v17(conn)?;
    }

    if current_version < 18 {
        migrate_v18(conn)?;
    }

    if current_version < 19 {
        migrate_v19(conn)?;
    }

    if current_version < 20 {
        migrate_v20(conn)?;
    }

    if current_version < 21 {
        migrate_v21(conn)?;
    }

    if current_version < 22 {
        migrate_v22(conn)?;
    }

    if current_version < 23 {
        migrate_v23(conn)?;
    }

    if current_version < 24 {
        migrate_v24(conn)?;
    }

    if current_version < 25 {
        migrate_v25(conn)?;
    }

    if current_version < 26 {
        migrate_v26(conn)?;
    }

    if current_version < 27 {
        migrate_v27(conn)?;
    }

    if current_version < 28 {
        migrate_v28(conn)?;
    }

    if current_version < 29 {
        migrate_v29(conn)?;
    }

    if current_version < 30 {
        migrate_v30(conn)?;
    }

    if current_version < 31 {
        migrate_v31(conn)?;
    }

    if current_version < 32 {
        migrate_v32(conn)?;
    }

    if current_version < 33 {
        migrate_v33(conn)?;
    }

    if current_version < 34 {
        migrate_v34(conn)?;
    }

    if current_version < 35 {
        migrate_v35(conn)?;
    }

    if current_version < 36 {
        migrate_v36(conn)?;
    }

    if current_version < 37 {
        migrate_v37(conn)?;
    }

    if current_version < 38 {
        migrate_v38(conn)?;
    }

    if current_version < 39 {
        migrate_v39(conn)?;
    }

    if current_version < 40 {
        migrate_v40(conn)?;
    }

    if current_version < 41 {
        migrate_v41(conn)?;
    }

    if current_version < 42 {
        migrate_v42(conn)?;
    }

    if current_version < 43 {
        migrate_v43(conn)?;
    }

    if current_version < 44 {
        migrate_v44(conn)?;
    }

    if current_version < 45 {
        migrate_v45(conn)?;
    }

    if current_version < 46 {
        migrate_v46(conn)?;
    }

    if current_version < 47 {
        migrate_v47(conn)?;
    }

    if current_version < 48 {
        migrate_v48(conn)?;
    }

    Ok(())
}
