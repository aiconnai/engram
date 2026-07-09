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
