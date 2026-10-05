#[test]
fn test_schema_migration_v34_idempotent() {
    use crate::storage::migrations::run_migrations;
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    run_migrations(&conn).expect("run migrations");
    // Running again should be a no-op
    run_migrations(&conn).expect("idempotent second run");
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .expect("query version");
    assert_eq!(version, crate::storage::migrations::SCHEMA_VERSION);
}

fn max_schema_version(conn: &rusqlite::Connection) -> i32 {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .expect("query version")
}

fn has_column(conn: &rusqlite::Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("table_info");
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("columns")
        .collect::<std::result::Result<Vec<_>, _>>()
        .expect("column names");
    names.iter().any(|name| name == column)
}

#[test]
fn test_failed_migration_rolls_back_and_reruns_from_predecessor() {
    use crate::storage::migrations::{run_migrations, SCHEMA_VERSION};
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    run_migrations(&conn).expect("migrate to current");

    // Return to the supported predecessor (v47) state.
    conn.execute_batch(
        "DELETE FROM schema_version WHERE version = 48;
         ALTER TABLE attestation_log DROP COLUMN hash_version;",
    )
    .expect("downgrade fixture to v47");
    assert_eq!(max_schema_version(&conn), SCHEMA_VERSION - 1);

    // Inject a failure after v48's ALTER TABLE but before it records itself.
    conn.execute_batch(
        "CREATE TRIGGER fail_v48 BEFORE INSERT ON schema_version
         WHEN NEW.version = 48
         BEGIN SELECT RAISE(ABORT, 'injected v48 failure'); END;",
    )
    .expect("install failure trigger");
    let err = run_migrations(&conn).expect_err("v48 must fail");
    assert!(err.to_string().contains("injected v48 failure"), "{err}");
    assert!(
        !has_column(&conn, "attestation_log", "hash_version"),
        "failed step must be rolled back entirely"
    );
    assert_eq!(max_schema_version(&conn), SCHEMA_VERSION - 1);
    assert!(conn.is_autocommit(), "no transaction left open");

    // Re-running after the cause is removed completes the upgrade.
    conn.execute_batch("DROP TRIGGER fail_v48;")
        .expect("drop trigger");
    run_migrations(&conn).expect("rerun succeeds");
    assert!(has_column(&conn, "attestation_log", "hash_version"));
    assert_eq!(max_schema_version(&conn), SCHEMA_VERSION);
}

#[test]
fn test_newer_schema_version_is_refused_without_changes() {
    use crate::storage::migrations::{run_migrations, SCHEMA_VERSION};
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    run_migrations(&conn).expect("migrate");
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?)",
        [SCHEMA_VERSION + 1],
    )
    .expect("simulate newer binary");
    let objects_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM sqlite_master", [], |r| r.get(0))
        .expect("count objects");

    let err = run_migrations(&conn).expect_err("newer schema refused");
    assert!(err.to_string().contains("newer than supported"), "{err}");
    let objects_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM sqlite_master", [], |r| r.get(0))
        .expect("count objects");
    assert_eq!(objects_before, objects_after);
    assert_eq!(max_schema_version(&conn), SCHEMA_VERSION + 1);
}

#[test]
fn test_run_migrations_refuses_open_transaction() {
    use crate::storage::migrations::run_migrations;
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    conn.execute_batch("BEGIN;").expect("begin");
    assert!(run_migrations(&conn).is_err());
    conn.execute_batch("ROLLBACK;").expect("rollback");
}
