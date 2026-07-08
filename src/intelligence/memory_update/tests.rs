use super::detector::MIN_CONFIDENCE;
use super::*;
use rusqlite::{params, Connection};

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

/// Create an in-memory database with the minimal `memories` table schema
/// and the `update_log` table.
fn in_memory_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memories (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            content      TEXT    NOT NULL,
            memory_type  TEXT    NOT NULL DEFAULT 'note',
            tags         TEXT    NOT NULL DEFAULT '[]',
            workspace    TEXT    NOT NULL DEFAULT 'default',
            created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );",
    )
    .expect("create memories table");
    conn.execute_batch(CREATE_UPDATE_LOG_TABLE)
        .expect("create update_log table");
    conn
}

fn insert_memory(conn: &Connection, content: &str, workspace: &str) -> i64 {
    conn.execute(
        "INSERT INTO memories (content, workspace) VALUES (?1, ?2)",
        params![content, workspace],
    )
    .expect("insert memory");
    conn.last_insert_rowid()
}

fn get_content(conn: &Connection, id: i64) -> String {
    conn.query_row(
        "SELECT content FROM memories WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .expect("get content")
}

fn get_memory_type(conn: &Connection, id: i64) -> String {
    conn.query_row(
        "SELECT memory_type FROM memories WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .expect("get memory_type")
}

fn get_tags(conn: &Connection, id: i64) -> Vec<String> {
    let raw: String = conn
        .query_row(
            "SELECT tags FROM memories WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .expect("get tags");
    serde_json::from_str(&raw).unwrap_or_default()
}

// -------------------------------------------------------------------------
// Detection tests — one per conflict type
// -------------------------------------------------------------------------

#[test]
fn test_detect_contradiction() {
    let conn = in_memory_conn();
    let _id = insert_memory(
        &conn,
        "Alice works at Anthropic as a senior engineer.",
        "work",
    );

    let detector = UpdateDetector::new();
    let candidates = detector
        .detect_updates(&conn, "Alice no longer works at Anthropic.", "work")
        .expect("detect_updates should succeed");

    assert!(
        !candidates.is_empty(),
        "Expected at least one contradiction candidate"
    );
    let cand = candidates
        .iter()
        .find(|c| c.conflict_type == ConflictType::Contradiction);
    assert!(
        cand.is_some(),
        "Expected a Contradiction candidate, got: {:?}",
        candidates
    );
    assert!(
        cand.unwrap().confidence >= MIN_CONFIDENCE,
        "Confidence too low"
    );
}

#[test]
fn test_detect_supplement() {
    let conn = in_memory_conn();
    let _id = insert_memory(
        &conn,
        "Alice works at Anthropic as a senior engineer.",
        "work",
    );

    let detector = UpdateDetector::new();
    let candidates = detector
        .detect_updates(
            &conn,
            "Alice works at Anthropic and also leads the safety team.",
            "work",
        )
        .expect("detect_updates should succeed");

    let cand = candidates
        .iter()
        .find(|c| c.conflict_type == ConflictType::Supplement);
    assert!(
        cand.is_some(),
        "Expected a Supplement candidate, got: {:?}",
        candidates
    );
}

#[test]
fn test_detect_correction() {
    let conn = in_memory_conn();
    let _id = insert_memory(
        &conn,
        "The project deadline is Friday the 20th.",
        "schedule",
    );

    let detector = UpdateDetector::new();
    let candidates = detector
        .detect_updates(
            &conn,
            "Actually, the project deadline is Thursday the 19th.",
            "schedule",
        )
        .expect("detect_updates should succeed");

    let cand = candidates
        .iter()
        .find(|c| c.conflict_type == ConflictType::Correction);
    assert!(
        cand.is_some(),
        "Expected a Correction candidate, got: {:?}",
        candidates
    );
    assert_eq!(
        cand.unwrap().suggested_action,
        UpdateAction::Replace,
        "Correction should suggest Replace"
    );
}

#[test]
fn test_detect_obsolescence() {
    let conn = in_memory_conn();
    let _id = insert_memory(
        &conn,
        "In 2020, the team was using Python 3.6 for all services.",
        "tech",
    );

    let detector = UpdateDetector::new();
    let candidates = detector
        .detect_updates(
            &conn,
            "The team is currently using Python 3.12 for all services.",
            "tech",
        )
        .expect("detect_updates should succeed");

    let cand = candidates
        .iter()
        .find(|c| c.conflict_type == ConflictType::Obsolescence);
    assert!(
        cand.is_some(),
        "Expected an Obsolescence candidate, got: {:?}",
        candidates
    );
    assert_eq!(
        cand.unwrap().suggested_action,
        UpdateAction::Archive,
        "Obsolescence should suggest Archive"
    );
}

// -------------------------------------------------------------------------
// Apply-action tests — one per UpdateAction variant
// -------------------------------------------------------------------------

#[test]
fn test_apply_replace() {
    let conn = in_memory_conn();
    let id = insert_memory(&conn, "Old content about the project.", "notes");

    let candidate = UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Correction,
        confidence: 0.8,
        suggested_action: UpdateAction::Replace,
        reason: "test".to_string(),
    };

    let result = apply_update(
        &conn,
        &candidate,
        UpdateAction::Replace,
        "New content about the project.",
    )
    .expect("apply_update should succeed");

    assert_eq!(result.memory_id, id);
    assert_eq!(result.action_taken, UpdateAction::Replace);
    assert_ne!(result.old_content_hash, result.new_content_hash);
    assert_eq!(get_content(&conn, id), "New content about the project.");
}

#[test]
fn test_apply_merge() {
    let conn = in_memory_conn();
    let id = insert_memory(&conn, "Alice works at Anthropic.", "notes");

    let candidate = UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Supplement,
        confidence: 0.6,
        suggested_action: UpdateAction::Merge,
        reason: "test".to_string(),
    };

    let result = apply_update(
        &conn,
        &candidate,
        UpdateAction::Merge,
        "She leads the safety team.",
    )
    .expect("apply_update should succeed");

    assert_eq!(result.action_taken, UpdateAction::Merge);
    let merged = get_content(&conn, id);
    assert!(
        merged.contains("Alice works at Anthropic."),
        "Merged content should retain old content"
    );
    assert!(
        merged.contains("She leads the safety team."),
        "Merged content should include new content"
    );
}

#[test]
fn test_apply_archive() {
    let conn = in_memory_conn();
    let id = insert_memory(&conn, "We use Python 3.6.", "tech");

    let candidate = UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Obsolescence,
        confidence: 0.7,
        suggested_action: UpdateAction::Archive,
        reason: "test".to_string(),
    };

    let result = apply_update(
        &conn,
        &candidate,
        UpdateAction::Archive,
        "We now use Python 3.12.",
    )
    .expect("apply_update should succeed");

    assert_eq!(result.action_taken, UpdateAction::Archive);
    assert_eq!(get_memory_type(&conn, id), "archived");
}

#[test]
fn test_apply_flag() {
    let conn = in_memory_conn();
    let id = insert_memory(&conn, "The budget is $50k.", "finance");

    let candidate = UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Contradiction,
        confidence: 0.65,
        suggested_action: UpdateAction::Flag,
        reason: "test".to_string(),
    };

    let result = apply_update(
        &conn,
        &candidate,
        UpdateAction::Flag,
        "The budget is not $50k.",
    )
    .expect("apply_update should succeed");

    assert_eq!(result.action_taken, UpdateAction::Flag);
    let tags = get_tags(&conn, id);
    assert!(
        tags.contains(&"needs-review".to_string()),
        "Tagged memory should contain 'needs-review'"
    );
}

// -------------------------------------------------------------------------
// Edge-case tests
// -------------------------------------------------------------------------

#[test]
fn test_no_conflict_when_unrelated() {
    let conn = in_memory_conn();
    // Insert a memory about cooking — completely unrelated to software.
    let _id = insert_memory(
        &conn,
        "The best way to make pasta is to boil water and add salt.",
        "kitchen",
    );

    let detector = UpdateDetector::new();
    let candidates = detector
        .detect_updates(
            &conn,
            "Alice no longer works at Anthropic as an engineer.",
            "kitchen",
        )
        .expect("detect_updates should succeed");

    // No significant overlap → no candidates above threshold.
    assert!(
        candidates.is_empty(),
        "Expected no candidates for unrelated content, got: {:?}",
        candidates
    );
}

#[test]
fn test_empty_workspace_returns_empty() {
    let conn = in_memory_conn();
    // No memories in "empty-ws".
    let detector = UpdateDetector::new();
    let candidates = detector
        .detect_updates(&conn, "Some new information.", "empty-ws")
        .expect("detect_updates should succeed");

    assert!(
        candidates.is_empty(),
        "Empty workspace must return empty candidates"
    );
}

// -------------------------------------------------------------------------
// Log storage tests
// -------------------------------------------------------------------------

#[test]
fn test_create_and_list_update_log() {
    let conn = in_memory_conn();
    let id = insert_memory(&conn, "Original content.", "notes");

    let candidate = UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Correction,
        confidence: 0.9,
        suggested_action: UpdateAction::Replace,
        reason: "explicit correction".to_string(),
    };

    let result = apply_update(
        &conn,
        &candidate,
        UpdateAction::Replace,
        "Corrected content.",
    )
    .expect("apply_update should succeed");

    let log_entry = create_update_log(&conn, &result, "explicit correction")
        .expect("create_update_log should succeed");

    assert_eq!(log_entry.memory_id, id);
    assert_eq!(log_entry.action, UpdateAction::Replace);
    assert!(!log_entry.old_hash.is_empty());
    assert!(!log_entry.new_hash.is_empty());
    assert_ne!(log_entry.old_hash, log_entry.new_hash);

    // list_update_logs filtered by memory_id
    let logs = list_update_logs(&conn, Some(id), 10).expect("list_update_logs should succeed");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].id, log_entry.id);

    // list_update_logs unfiltered
    let all_logs = list_update_logs(&conn, None, 0).expect("list_update_logs should succeed");
    assert_eq!(all_logs.len(), 1);
}
