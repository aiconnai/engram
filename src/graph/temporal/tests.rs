//! Tests for the temporal knowledge graph.

use super::edges::get_edge_by_id;
use super::*;
use crate::error::EngramError;
use rusqlite::Connection;
use serde_json::json;

/// Open an in-memory SQLite database and create the temporal_edges table.
fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory DB");
    conn.execute_batch(CREATE_TEMPORAL_EDGES_TABLE)
        .expect("create table");
    conn
}

// -------------------------------------------------------------------------
// Test 1: Add edge and retrieve it
// -------------------------------------------------------------------------
#[test]
fn test_add_edge_and_retrieve() {
    let conn = setup_db();

    let edge = add_edge(
        &conn,
        1,
        2,
        "works_at",
        &json!({}),
        "2024-01-01T00:00:00Z",
        0.9,
        "test",
        None,
    )
    .expect("add_edge");

    assert_eq!(edge.from_id, 1);
    assert_eq!(edge.to_id, 2);
    assert_eq!(edge.relation, "works_at");
    assert!(edge.valid_to.is_none());
    assert_eq!(edge.confidence, 0.9);
    assert_eq!(edge.source, "test");
    assert_eq!(edge.scope_path, "global");
}

// -------------------------------------------------------------------------
// Test 2: Auto-invalidation of conflicting edges
// -------------------------------------------------------------------------
#[test]
fn test_auto_invalidation_on_new_edge() {
    let conn = setup_db();

    let first = add_edge(
        &conn,
        1,
        2,
        "works_at",
        &json!({"role": "engineer"}),
        "2023-01-01T00:00:00Z",
        1.0,
        "hr",
        None,
    )
    .expect("first edge");

    assert!(first.valid_to.is_none(), "first edge should be open");

    // Adding a new edge for the same triple must close the first one.
    let _second = add_edge(
        &conn,
        1,
        2,
        "works_at",
        &json!({"role": "manager"}),
        "2024-06-01T00:00:00Z",
        1.0,
        "hr",
        None,
    )
    .expect("second edge");

    // Re-fetch first edge to confirm it was closed.
    let updated = get_edge_by_id(&conn, first.id)
        .expect("query")
        .expect("edge still exists");

    assert_eq!(
        updated.valid_to.as_deref(),
        Some("2024-06-01T00:00:00Z"),
        "first edge should have been closed at the second edge's valid_from"
    );
}

// -------------------------------------------------------------------------
// Test 3: Snapshot at a specific timestamp
// -------------------------------------------------------------------------
#[test]
fn test_snapshot_at() {
    let conn = setup_db();

    // Edge valid in 2023 only.
    add_edge(
        &conn,
        1,
        2,
        "rel",
        &json!({}),
        "2023-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();
    // Manually close it via a second edge (auto-invalidation).
    add_edge(
        &conn,
        1,
        2,
        "rel",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();

    // Snapshot mid-2023 should return exactly 1 edge.
    let snap = snapshot_at(&conn, "2023-07-01T00:00:00Z", None).expect("snapshot");
    assert_eq!(snap.len(), 1);
    assert_eq!(snap[0].valid_from, "2023-01-01T00:00:00Z");

    // Snapshot mid-2024 should return the second edge.
    let snap2 = snapshot_at(&conn, "2024-07-01T00:00:00Z", None).expect("snapshot");
    assert_eq!(snap2.len(), 1);
    assert_eq!(snap2[0].valid_from, "2024-01-01T00:00:00Z");
}

// -------------------------------------------------------------------------
// Test 4: Timeline shows chronological history
// -------------------------------------------------------------------------
#[test]
fn test_relationship_timeline_chronological() {
    let conn = setup_db();

    add_edge(
        &conn,
        10,
        20,
        "partner",
        &json!({}),
        "2020-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();
    add_edge(
        &conn,
        10,
        20,
        "partner",
        &json!({}),
        "2021-06-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();
    add_edge(
        &conn,
        10,
        20,
        "partner",
        &json!({}),
        "2022-09-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();

    let timeline = relationship_timeline(&conn, 10, 20, None).expect("timeline");
    assert_eq!(timeline.len(), 3);

    // Verify ascending order.
    assert!(timeline[0].valid_from <= timeline[1].valid_from);
    assert!(timeline[1].valid_from <= timeline[2].valid_from);
}

// -------------------------------------------------------------------------
// Test 5: Detect contradictions (manually injected overlap)
// -------------------------------------------------------------------------
#[test]
fn test_detect_contradictions() {
    let conn = setup_db();

    // Insert two edges with overlapping validity directly (bypassing
    // the auto-invalidation logic that `add_edge` provides).
    conn.execute(
        "INSERT INTO temporal_edges
             (from_id, to_id, relation, properties, valid_from, valid_to, confidence, source)
         VALUES (1, 2, 'rel', '{}', '2023-01-01T00:00:00Z', NULL, 1.0, '')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO temporal_edges
             (from_id, to_id, relation, properties, valid_from, valid_to, confidence, source)
         VALUES (1, 2, 'rel', '{}', '2023-06-01T00:00:00Z', NULL, 1.0, '')",
        [],
    )
    .unwrap();

    let contradictions = detect_contradictions(&conn).expect("detect");
    assert_eq!(contradictions.len(), 1);

    let (a, b) = &contradictions[0];
    assert!(a.id < b.id);
}

// -------------------------------------------------------------------------
// Test 6: Diff between two timestamps
// -------------------------------------------------------------------------
#[test]
fn test_diff_between_timestamps() {
    let conn = setup_db();

    // Edge A: exists in 2023 and 2024.
    add_edge(
        &conn,
        1,
        2,
        "knows",
        &json!({}),
        "2022-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();

    // Edge B: appears in 2024 only.
    add_edge(
        &conn,
        3,
        4,
        "likes",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();

    let d = diff(&conn, "2023-01-01T00:00:00Z", "2025-01-01T00:00:00Z", None).expect("diff");

    // "knows" was present at both; "likes" was added.
    assert_eq!(d.added.len(), 1);
    assert_eq!(d.added[0].relation, "likes");
    assert_eq!(d.removed.len(), 0);
    // "knows" same edge, not changed.
    assert_eq!(d.changed.len(), 0);
}

// -------------------------------------------------------------------------
// Test 7: Empty graph operations
// -------------------------------------------------------------------------
#[test]
fn test_empty_graph_operations() {
    let conn = setup_db();

    let snap = snapshot_at(&conn, "2024-01-01T00:00:00Z", None).expect("snapshot");
    assert!(snap.is_empty());

    let timeline = relationship_timeline(&conn, 99, 100, None).expect("timeline");
    assert!(timeline.is_empty());

    let contradictions = detect_contradictions(&conn).expect("detect");
    assert!(contradictions.is_empty());

    let d = diff(&conn, "2024-01-01T00:00:00Z", "2025-01-01T00:00:00Z", None).expect("diff");
    assert!(d.added.is_empty());
    assert!(d.removed.is_empty());
    assert!(d.changed.is_empty());
}

// -------------------------------------------------------------------------
// Test 8: Edge with rich JSON properties
// -------------------------------------------------------------------------
#[test]
fn test_edge_with_json_properties() {
    let conn = setup_db();

    let props = json!({
        "title": "Senior Engineer",
        "department": "R&D",
        "salary": 120_000,
        "remote": true,
        "skills": ["Rust", "Python"]
    });

    let edge = add_edge(
        &conn,
        5,
        6,
        "employed_by",
        &props,
        "2024-03-01T00:00:00Z",
        0.95,
        "payroll",
        None,
    )
    .expect("add");

    assert_eq!(edge.properties["title"], "Senior Engineer");
    assert_eq!(edge.properties["salary"], 120_000);
    assert_eq!(edge.properties["remote"], true);
    assert_eq!(edge.properties["skills"][0], "Rust");
}

// -------------------------------------------------------------------------
// Test 9: Invalidate edge manually
// -------------------------------------------------------------------------
#[test]
fn test_invalidate_edge_manually() {
    let conn = setup_db();

    let edge = add_edge(
        &conn,
        7,
        8,
        "owns",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "legal",
        None,
    )
    .expect("add");

    assert!(edge.valid_to.is_none());

    invalidate_edge(&conn, edge.id, "2024-12-31T23:59:59Z").expect("invalidate");

    let updated = get_edge_by_id(&conn, edge.id)
        .expect("query")
        .expect("still exists");

    assert_eq!(updated.valid_to.as_deref(), Some("2024-12-31T23:59:59Z"));
}

// -------------------------------------------------------------------------
// Test 10: Invalidating a non-existent edge returns NotFound
// -------------------------------------------------------------------------
#[test]
fn test_invalidate_nonexistent_edge_returns_not_found() {
    let conn = setup_db();

    let result = invalidate_edge(&conn, 99999, "2025-01-01T00:00:00Z");
    assert!(
        matches!(result, Err(EngramError::NotFound(99999))),
        "expected NotFound(99999), got {:?}",
        result
    );
}

// -------------------------------------------------------------------------
// Test 11: Diff detects edge supersession as "changed"
// -------------------------------------------------------------------------
#[test]
fn test_diff_detects_changed_edge() {
    let conn = setup_db();

    // First version of the edge.
    add_edge(
        &conn,
        1,
        2,
        "role",
        &json!({"level": "junior"}),
        "2022-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();

    // Supersede it (auto-invalidation closes the first).
    add_edge(
        &conn,
        1,
        2,
        "role",
        &json!({"level": "senior"}),
        "2023-06-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .unwrap();

    let d = diff(&conn, "2022-07-01T00:00:00Z", "2024-01-01T00:00:00Z", None).expect("diff");

    // The triple is present at both timestamps, but via a different edge id.
    assert_eq!(d.changed.len(), 1);
    let (old, new) = &d.changed[0];
    assert_eq!(old.properties["level"], "junior");
    assert_eq!(new.properties["level"], "senior");
}

// -------------------------------------------------------------------------
// Test 12: Add edge with explicit scope, verify scope is stored
// -------------------------------------------------------------------------
#[test]
fn test_add_edge_with_scope() {
    let conn = setup_db();

    // Edge in the default (global) scope.
    let global_edge = add_edge(
        &conn,
        1,
        2,
        "knows",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        None,
    )
    .expect("global edge");
    assert_eq!(global_edge.scope_path, "global");

    // Edge in a tenant-specific scope.
    let tenant_edge = add_edge(
        &conn,
        3,
        4,
        "manages",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/org:acme"),
    )
    .expect("tenant edge");
    assert_eq!(tenant_edge.scope_path, "global/org:acme");

    // Edge in a deeper scope.
    let user_edge = add_edge(
        &conn,
        5,
        6,
        "reports_to",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/org:acme/user:alice"),
    )
    .expect("user edge");
    assert_eq!(user_edge.scope_path, "global/org:acme/user:alice");

    // Auto-invalidation is scope-aware: adding another edge for the same
    // triple in a DIFFERENT scope must NOT close the first-scope edge.
    let acme_edge_1 = add_edge(
        &conn,
        10,
        20,
        "partner",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/org:acme"),
    )
    .expect("acme edge 1");

    let _acme_edge_2 = add_edge(
        &conn,
        10,
        20,
        "partner",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/org:beta"), // different scope — must not close acme_edge_1
    )
    .expect("beta edge");

    let refetched = get_edge_by_id(&conn, acme_edge_1.id)
        .expect("query")
        .expect("still exists");
    assert!(
        refetched.valid_to.is_none(),
        "edge in org:acme must not be closed by edge in org:beta"
    );
}

// -------------------------------------------------------------------------
// Test 13: snapshot_at with scope_path filter
// -------------------------------------------------------------------------
#[test]
fn test_snapshot_at_with_scope_filter() {
    let conn = setup_db();

    // Add one edge in "global" scope and one in "global/org:acme".
    add_edge(
        &conn,
        1,
        2,
        "rel",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        None, // defaults to "global"
    )
    .unwrap();

    add_edge(
        &conn,
        3,
        4,
        "rel",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/org:acme"),
    )
    .unwrap();

    // No scope filter → both edges visible.
    let all = snapshot_at(&conn, "2025-01-01T00:00:00Z", None).unwrap();
    assert_eq!(all.len(), 2);

    // Filter to "global" includes all descendants (hierarchical prefix matching).
    // "global" matches exactly, and "global/org:acme" matches via LIKE 'global/%'.
    let global_tree = snapshot_at(&conn, "2025-01-01T00:00:00Z", Some("global")).unwrap();
    assert_eq!(
        global_tree.len(),
        2,
        "global scope tree should include its child org:acme"
    );

    // Filter to "global/org:acme" → only the acme edge (no further children here).
    let acme_only = snapshot_at(&conn, "2025-01-01T00:00:00Z", Some("global/org:acme")).unwrap();
    assert_eq!(acme_only.len(), 1);
    assert_eq!(acme_only[0].from_id, 3);

    // Demonstrate that "global" exact match can be queried by adding a non-child scope.
    add_edge(
        &conn,
        7,
        8,
        "rel",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/org:beta"),
    )
    .unwrap();

    // "global/org:acme" filter should still only return the one acme edge.
    let acme_only2 = snapshot_at(&conn, "2025-01-01T00:00:00Z", Some("global/org:acme")).unwrap();
    assert_eq!(acme_only2.len(), 1);
    assert_eq!(acme_only2[0].from_id, 3);
}

// -------------------------------------------------------------------------
// Test 14: scope prefix matching — hierarchy traversal
// -------------------------------------------------------------------------
#[test]
fn test_scope_prefix_matching() {
    let conn = setup_db();

    // Three edges at different scope depths.
    add_edge(
        &conn,
        1,
        2,
        "a",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/mbras"),
    )
    .unwrap();

    add_edge(
        &conn,
        3,
        4,
        "b",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/mbras/broker_alice"),
    )
    .unwrap();

    add_edge(
        &conn,
        5,
        6,
        "c",
        &json!({}),
        "2024-01-01T00:00:00Z",
        1.0,
        "",
        Some("global/other"),
    )
    .unwrap();

    // Filtering on "global/mbras" should return:
    //   - the exact "global/mbras" edge
    //   - "global/mbras/broker_alice" (child)
    // but NOT "global/other".
    let mbras_snap = snapshot_at(&conn, "2025-01-01T00:00:00Z", Some("global/mbras")).unwrap();
    assert_eq!(
        mbras_snap.len(),
        2,
        "expected 2 edges under global/mbras, got: {:?}",
        mbras_snap.iter().map(|e| &e.scope_path).collect::<Vec<_>>()
    );

    let scope_paths: Vec<&str> = mbras_snap.iter().map(|e| e.scope_path.as_str()).collect();
    assert!(scope_paths.contains(&"global/mbras"));
    assert!(scope_paths.contains(&"global/mbras/broker_alice"));
}
