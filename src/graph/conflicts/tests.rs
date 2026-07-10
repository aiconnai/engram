use super::*;
use rusqlite::{params, Connection};

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

const CREATE_CROSS_REFS: &str = "
    CREATE TABLE IF NOT EXISTS cross_references (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        from_id         INTEGER NOT NULL,
        to_id           INTEGER NOT NULL,
        relation_type   TEXT    NOT NULL DEFAULT 'related',
        strength        REAL    NOT NULL DEFAULT 0.5,
        metadata        TEXT    DEFAULT '{}',
        created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
";

const CREATE_MEMORIES: &str = "
    CREATE TABLE IF NOT EXISTS memories (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        content    TEXT    NOT NULL,
        created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
";

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory DB");
    conn.execute_batch(CREATE_CROSS_REFS)
        .expect("create cross_references");
    conn.execute_batch(CREATE_MEMORIES)
        .expect("create memories");
    conn.execute_batch(CREATE_CONFLICTS_TABLE)
        .expect("create graph_conflicts");
    conn
}

fn insert_edge(conn: &Connection, from_id: i64, to_id: i64, rel: &str, strength: f64) -> i64 {
    conn.execute(
        "INSERT INTO cross_references (from_id, to_id, relation_type, strength)
         VALUES (?1, ?2, ?3, ?4)",
        params![from_id, to_id, rel, strength],
    )
    .expect("insert edge");
    conn.last_insert_rowid()
}

fn insert_memory(conn: &Connection, id: i64) {
    conn.execute(
        "INSERT INTO memories (id, content) VALUES (?1, 'test')",
        params![id],
    )
    .expect("insert memory");
}

// -------------------------------------------------------------------------
// Test 1: detect_contradictions — finds contradicting relation pair
// -------------------------------------------------------------------------
#[test]
fn test_detect_contradiction() {
    let conn = setup_db();

    // A --supports--> B
    insert_edge(&conn, 1, 2, "supports", 0.8);
    // A --contradicts--> B  (same pair, contradicting semantics)
    insert_edge(&conn, 1, 2, "contradicts", 0.8);

    let conflicts = ConflictDetector::detect_contradictions(&conn).expect("detect_contradictions");

    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        ConflictType::DirectContradiction
    );
    assert_eq!(conflicts[0].severity, Severity::High);
    assert!(conflicts[0].edge_ids.len() >= 2);
    assert!(conflicts[0].description.contains("Contradicting"));
}

// -------------------------------------------------------------------------
// Test 2: detect_temporal_inconsistencies — duplicate triple
// -------------------------------------------------------------------------
#[test]
fn test_detect_temporal_inconsistency() {
    let conn = setup_db();

    // Two edges for the exact same (from, to, relation) triple.
    let id_a = insert_edge(&conn, 10, 20, "works_at", 0.9);
    let id_b = insert_edge(&conn, 10, 20, "works_at", 0.7);

    let conflicts = ConflictDetector::detect_temporal_inconsistencies(&conn)
        .expect("detect_temporal_inconsistencies");

    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        ConflictType::TemporalInconsistency
    );
    assert_eq!(conflicts[0].severity, Severity::Medium);
    assert!(conflicts[0].edge_ids.contains(&id_a));
    assert!(conflicts[0].edge_ids.contains(&id_b));
}

// -------------------------------------------------------------------------
// Test 3: detect_cycles — simple A→B→C→A cycle
// -------------------------------------------------------------------------
#[test]
fn test_detect_cycle() {
    let conn = setup_db();

    // A→B, B→C, C→A forms a cycle.
    insert_edge(&conn, 1, 2, "depends_on", 0.9);
    insert_edge(&conn, 2, 3, "depends_on", 0.9);
    insert_edge(&conn, 3, 1, "depends_on", 0.9); // closes the cycle

    let conflicts = ConflictDetector::detect_cycles(&conn).expect("detect_cycles");

    assert!(
        !conflicts.is_empty(),
        "expected at least one cycle conflict"
    );
    assert_eq!(conflicts[0].conflict_type, ConflictType::CyclicDependency);
    assert!(conflicts[0].description.contains("Cycle"));
}

// -------------------------------------------------------------------------
// Test 4: detect_orphans — edge references missing memory
// -------------------------------------------------------------------------
#[test]
fn test_detect_orphan() {
    let conn = setup_db();

    // Only memory 1 exists; edge references memory 99 which doesn't exist.
    insert_memory(&conn, 1);
    let edge_id = insert_edge(&conn, 1, 99, "related", 0.5); // to_id=99 is orphan

    let conflicts = ConflictDetector::detect_orphans(&conn).expect("detect_orphans");

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].conflict_type, ConflictType::OrphanedReference);
    assert_eq!(conflicts[0].severity, Severity::Critical);
    assert!(conflicts[0].edge_ids.contains(&edge_id));
}

// -------------------------------------------------------------------------
// Test 5: resolve with KeepNewer removes older edges
// -------------------------------------------------------------------------
#[test]
fn test_resolve_keep_newer() {
    let conn = setup_db();

    let id_old = insert_edge(&conn, 5, 6, "supports", 0.5);
    // Ensure the second edge has a later created_at by updating it.
    conn.execute(
        "UPDATE cross_references SET created_at = '2099-01-01T00:00:00.000Z' WHERE id = ?1",
        params![id_old + 1],
    )
    .ok();
    let id_new = insert_edge(&conn, 5, 6, "supports", 0.5);
    // Make the new edge newer.
    conn.execute(
        "UPDATE cross_references SET created_at = '2099-01-02T00:00:00.000Z' WHERE id = ?1",
        params![id_new],
    )
    .expect("update ts");

    // Save a conflict manually.
    let conflict = Conflict {
        id: 0,
        conflict_type: ConflictType::TemporalInconsistency,
        edge_ids: vec![id_old, id_new],
        description: "duplicate triple".to_string(),
        severity: Severity::Medium,
        resolved_at: None,
        resolution_strategy: None,
    };
    let cid = ConflictResolver::save_conflict(&conn, &conflict).expect("save");

    let result =
        ConflictResolver::resolve(&conn, cid, ResolutionStrategy::KeepNewer).expect("resolve");

    assert_eq!(result.conflict_id, cid);
    assert_eq!(result.strategy, ResolutionStrategy::KeepNewer);
    assert_eq!(result.edges_removed.len(), 1);
    assert_eq!(result.edges_kept.len(), 1);
    assert!(result.edges_kept.contains(&id_new));
    assert!(result.edges_removed.contains(&id_old));

    // Verify the conflict is marked resolved.
    let saved = ConflictResolver::get_conflict(&conn, cid)
        .expect("get")
        .expect("exists");
    assert!(saved.resolved_at.is_some());
}

// -------------------------------------------------------------------------
// Test 6: no conflicts when graph is clean
// -------------------------------------------------------------------------
#[test]
fn test_no_conflicts_clean_graph() {
    let conn = setup_db();

    // Insert valid memories and non-contradicting edges.
    insert_memory(&conn, 1);
    insert_memory(&conn, 2);
    insert_memory(&conn, 3);
    insert_edge(&conn, 1, 2, "supports", 0.9);
    insert_edge(&conn, 2, 3, "related", 0.7);

    let all = ConflictDetector::detect_all(&conn).expect("detect_all");

    // No cycles (1→2→3, no back-edge), no orphans, no contradictions, no temporal.
    assert!(all.is_empty(), "expected zero conflicts, got: {:?}", all);
}

// -------------------------------------------------------------------------
// Test 7: save and list conflicts
// -------------------------------------------------------------------------
#[test]
fn test_save_and_list_conflicts() {
    let conn = setup_db();

    let c1 = Conflict {
        id: 0,
        conflict_type: ConflictType::DirectContradiction,
        edge_ids: vec![1, 2],
        description: "supports vs contradicts".to_string(),
        severity: Severity::High,
        resolved_at: None,
        resolution_strategy: None,
    };
    let c2 = Conflict {
        id: 0,
        conflict_type: ConflictType::OrphanedReference,
        edge_ids: vec![3],
        description: "missing node 99".to_string(),
        severity: Severity::Critical,
        resolved_at: None,
        resolution_strategy: None,
    };

    let id1 = ConflictResolver::save_conflict(&conn, &c1).expect("save c1");
    let id2 = ConflictResolver::save_conflict(&conn, &c2).expect("save c2");

    let all = ConflictResolver::list_conflicts(&conn, None).expect("list all");
    assert_eq!(all.len(), 2);

    let unresolved = ConflictResolver::list_conflicts(&conn, Some(false)).expect("list unresolved");
    assert_eq!(unresolved.len(), 2);

    let resolved = ConflictResolver::list_conflicts(&conn, Some(true)).expect("list resolved");
    assert_eq!(resolved.len(), 0);

    // Verify we can retrieve by ID.
    let fetched = ConflictResolver::get_conflict(&conn, id1)
        .expect("get c1")
        .expect("exists");
    assert_eq!(fetched.conflict_type, ConflictType::DirectContradiction);
    assert_eq!(fetched.severity, Severity::High);

    let fetched2 = ConflictResolver::get_conflict(&conn, id2)
        .expect("get c2")
        .expect("exists");
    assert_eq!(fetched2.conflict_type, ConflictType::OrphanedReference);
}

// -------------------------------------------------------------------------
// Test 8: multiple conflict types in one scan
// -------------------------------------------------------------------------
#[test]
fn test_detect_all_multiple_types() {
    let conn = setup_db();

    // Memory 1 exists, 99 does not → orphan.
    insert_memory(&conn, 1);
    insert_memory(&conn, 2);

    // Contradiction: supports + contradicts between same pair.
    insert_edge(&conn, 1, 2, "supports", 0.8);
    insert_edge(&conn, 1, 2, "contradicts", 0.6);

    // Orphan: edge references non-existent memory 99.
    insert_edge(&conn, 1, 99, "related", 0.5);

    let all = ConflictDetector::detect_all(&conn).expect("detect_all");

    let types: Vec<&ConflictType> = all.iter().map(|c| &c.conflict_type).collect();

    assert!(
        types.contains(&&ConflictType::DirectContradiction),
        "expected DirectContradiction in {:?}",
        types
    );
    assert!(
        types.contains(&&ConflictType::OrphanedReference),
        "expected OrphanedReference in {:?}",
        types
    );
}

// -------------------------------------------------------------------------
// Test 9: resolve already-resolved conflict returns error
// -------------------------------------------------------------------------
#[test]
fn test_resolve_already_resolved_returns_error() {
    let conn = setup_db();

    let id_a = insert_edge(&conn, 5, 6, "rel", 0.5);
    let id_b = insert_edge(&conn, 5, 6, "rel", 0.5);

    let conflict = Conflict {
        id: 0,
        conflict_type: ConflictType::TemporalInconsistency,
        edge_ids: vec![id_a, id_b],
        description: "dup".to_string(),
        severity: Severity::Medium,
        resolved_at: None,
        resolution_strategy: None,
    };
    let cid = ConflictResolver::save_conflict(&conn, &conflict).expect("save");

    // First resolution should succeed.
    ConflictResolver::resolve(&conn, cid, ResolutionStrategy::Manual).expect("first resolve");

    // Second resolution on the same conflict should fail.
    let result = ConflictResolver::resolve(&conn, cid, ResolutionStrategy::Manual);
    assert!(result.is_err(), "expected error on double-resolve");
}
