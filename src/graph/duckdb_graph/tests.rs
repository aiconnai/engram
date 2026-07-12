use super::*;
use crate::error::{EngramError, Result};

/// Create a minimal SQLite database at `path` that satisfies the schema
/// expected by `TemporalGraph` (v33 tables).
fn setup_sqlite(path: &str) -> rusqlite::Connection {
    let conn = rusqlite::Connection::open(path).expect("open sqlite");
    conn.execute_batch(
        r#"
            CREATE TABLE IF NOT EXISTS graph_entities (
                id          TEXT PRIMARY KEY,
                scope_path  TEXT NOT NULL DEFAULT 'global',
                name        TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                metadata    TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS temporal_edges (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                from_id     INTEGER NOT NULL,
                to_id       INTEGER NOT NULL,
                relation    TEXT NOT NULL,
                properties  TEXT,
                valid_from  TEXT NOT NULL,
                valid_to    TEXT,
                confidence  REAL NOT NULL DEFAULT 1.0,
                source      TEXT,
                scope_path  TEXT NOT NULL DEFAULT 'global',
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
        "#,
    )
    .expect("create tables");
    conn
}

/// Insert a single edge into `temporal_edges` and return its rowid.
fn insert_edge(
    conn: &rusqlite::Connection,
    from_id: i64,
    to_id: i64,
    relation: &str,
    valid_from: &str,
    valid_to: Option<&str>,
    confidence: f64,
    scope_path: &str,
) {
    conn.execute(
        "INSERT INTO temporal_edges
                (from_id, to_id, relation, valid_from, valid_to, confidence, scope_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![from_id, to_id, relation, valid_from, valid_to, confidence, scope_path],
    )
    .expect("insert edge");
}

// -----------------------------------------------------------------------
// Security tests (H4)
// -----------------------------------------------------------------------

fn assert_invalid_input(result: Result<TemporalGraph>) {
    match result {
        Err(EngramError::InvalidInput(_)) => {}
        Err(e) => panic!("expected InvalidInput, got: {}", e),
        Ok(_) => panic!("expected error but got Ok"),
    }
}

#[test]
fn test_new_rejects_path_with_single_quote() {
    assert_invalid_input(TemporalGraph::new("/tmp/evil'path.sqlite"));
}

#[test]
fn test_new_rejects_path_with_null_byte() {
    assert_invalid_input(TemporalGraph::new("/tmp/evil\0path.sqlite"));
}

#[test]
fn test_new_rejects_path_with_dotdot() {
    assert_invalid_input(TemporalGraph::new("../../../etc/passwd"));
}

// -----------------------------------------------------------------------

#[test]
fn test_temporal_graph_new() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_new.sqlite");
    let path_str = path.to_str().unwrap();

    // Remove any leftovers from a previous run.
    let _ = std::fs::remove_file(path_str);

    setup_sqlite(path_str);

    let graph = TemporalGraph::new(path_str);
    assert!(
        graph.is_ok(),
        "TemporalGraph::new should succeed: {:?}",
        graph.err()
    );

    // Cleanup.
    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_temporal_graph_refresh() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_refresh.sqlite");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(path_str);
    setup_sqlite(path_str);

    let graph = TemporalGraph::new(path_str).expect("new");

    // First refresh should succeed (detach + re-attach).
    let r1 = graph.refresh();
    assert!(r1.is_ok(), "first refresh failed: {:?}", r1.err());

    // Second refresh should also succeed (idempotent).
    let r2 = graph.refresh();
    assert!(r2.is_ok(), "second refresh failed: {:?}", r2.err());

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_has_pgq_false_without_extension() {
    // In most CI environments duckpgq is not installed.  We verify that
    // the constructor still succeeds and `has_pgq` returns a bool (true
    // only if the extension happened to be available).
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_pgq.sqlite");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(path_str);
    setup_sqlite(path_str);

    let graph = TemporalGraph::new(path_str).expect("new");

    // The important invariant: even if PGQ is unavailable the struct is
    // valid and `has_pgq()` returns a stable boolean without panicking.
    let _ = graph.has_pgq(); // just assert it doesn't panic

    let _ = std::fs::remove_file(path_str);
}

// -----------------------------------------------------------------------
// Temporal query method tests
// -----------------------------------------------------------------------

#[test]
fn test_snapshot_at() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_snapshot_at.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    let sqlite = setup_sqlite(path_str);

    // Edge A: active 2024-01-01 → 2024-06-30 (closed)
    insert_edge(
        &sqlite,
        1,
        2,
        "knows",
        "2024-01-01",
        Some("2024-06-30"),
        0.9,
        "global",
    );
    // Edge B: active 2024-01-01 → open (still active)
    insert_edge(&sqlite, 1, 3, "follows", "2024-01-01", None, 0.8, "global");
    // Edge C: starts in the future (2025-01-01), should not appear at 2024-03-01
    insert_edge(&sqlite, 2, 3, "linked", "2025-01-01", None, 0.7, "global");
    drop(sqlite);

    let graph = TemporalGraph::new(path_str).expect("new");

    // Snapshot mid-year 2024: should see edges A and B, not C.
    let snap = graph
        .snapshot_at("global", "2024-03-01")
        .expect("snapshot_at");
    assert_eq!(snap.len(), 2, "expected 2 edges active at 2024-03-01");
    let relations: Vec<&str> = snap.iter().map(|e| e.relation.as_str()).collect();
    assert!(relations.contains(&"knows"), "edge A should be included");
    assert!(relations.contains(&"follows"), "edge B should be included");

    // Snapshot after edge A expired: should see only B and C.
    let snap2 = graph
        .snapshot_at("global", "2024-08-01")
        .expect("snapshot_at late");
    assert_eq!(snap2.len(), 1, "expected 1 edge active at 2024-08-01");
    assert_eq!(snap2[0].relation, "follows");

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_graph_diff() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_graph_diff.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    let sqlite = setup_sqlite(path_str);

    // Edge present at both t1 and t2 (no change).
    insert_edge(&sqlite, 1, 2, "knows", "2024-01-01", None, 1.0, "global");
    // Edge present at t1 but expired before t2 (removed).
    insert_edge(
        &sqlite,
        1,
        3,
        "follows",
        "2024-01-01",
        Some("2024-03-31"),
        1.0,
        "global",
    );
    // Edge starting after t1 (added at t2).
    insert_edge(&sqlite, 2, 3, "linked", "2024-06-01", None, 0.5, "global");
    drop(sqlite);

    let graph = TemporalGraph::new(path_str).expect("new");

    let diff = graph
        .graph_diff("global", "2024-02-01", "2024-07-01")
        .expect("graph_diff");

    assert_eq!(diff.added.len(), 1, "one edge added between t1 and t2");
    assert_eq!(diff.added[0].relation, "linked");

    assert_eq!(diff.removed.len(), 1, "one edge removed between t1 and t2");
    assert_eq!(diff.removed[0].relation, "follows");

    assert_eq!(diff.changed.len(), 0, "no edges changed");

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_relationship_timeline() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_timeline.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    let sqlite = setup_sqlite(path_str);

    // Three versions of the same 1→2 relationship, plus an unrelated edge.
    insert_edge(
        &sqlite,
        1,
        2,
        "knows",
        "2022-01-01",
        Some("2022-12-31"),
        0.5,
        "global",
    );
    insert_edge(
        &sqlite,
        1,
        2,
        "knows",
        "2023-01-01",
        Some("2023-12-31"),
        0.75,
        "global",
    );
    insert_edge(&sqlite, 1, 2, "knows", "2024-01-01", None, 0.9, "global");
    // Unrelated: different pair.
    insert_edge(&sqlite, 3, 4, "linked", "2024-01-01", None, 1.0, "global");
    drop(sqlite);

    let graph = TemporalGraph::new(path_str).expect("new");

    let timeline = graph
        .relationship_timeline("global", 1, 2)
        .expect("timeline");

    assert_eq!(timeline.len(), 3, "three versions of the 1→2 relationship");

    // Results should be ordered by valid_from DESC.
    assert_eq!(timeline[0].valid_from, "2024-01-01", "most recent first");
    assert_eq!(timeline[1].valid_from, "2023-01-01");
    assert_eq!(timeline[2].valid_from, "2022-01-01", "oldest last");

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_scope_filtering_in_snapshot() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_scope_filter.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    let sqlite = setup_sqlite(path_str);

    // Two edges in "project/alpha" scope.
    insert_edge(
        &sqlite,
        1,
        2,
        "depends",
        "2024-01-01",
        None,
        1.0,
        "project/alpha",
    );
    insert_edge(
        &sqlite,
        2,
        3,
        "depends",
        "2024-01-01",
        None,
        1.0,
        "project/alpha/sub",
    );
    // One edge in a sibling scope — must NOT appear in "project/alpha" snapshots.
    insert_edge(
        &sqlite,
        3,
        4,
        "depends",
        "2024-01-01",
        None,
        1.0,
        "project/beta",
    );
    // One edge in parent scope — must NOT appear (prefix match is strict on scope arg).
    insert_edge(&sqlite, 4, 5, "depends", "2024-01-01", None, 1.0, "project");
    drop(sqlite);

    let graph = TemporalGraph::new(path_str).expect("new");

    // Snapshot scoped to "project/alpha" should return both alpha + alpha/sub edges.
    let snap = graph
        .snapshot_at("project/alpha", "2024-06-01")
        .expect("snapshot_at scoped");

    assert_eq!(snap.len(), 2, "only edges under project/alpha scope");
    for edge in &snap {
        assert!(
            edge.scope_path.starts_with("project/alpha"),
            "unexpected scope: {}",
            edge.scope_path
        );
    }

    let _ = std::fs::remove_file(path_str);
}

// -----------------------------------------------------------------------
// Path-finding tests
// -----------------------------------------------------------------------

/// Build the example graph used in path-finding tests.
///
/// Graph (all edges open / valid_to IS NULL):
///   1 (Alice) --[works_at]--> 2 (MBRAS) --[located_in]--> 3 (Sao Paulo)
///   1 (Alice) --[knows]-----> 4 (Bob)   --[works_at]----> 5 (Competitor)
fn setup_pathfinding_db(path: &str) {
    let conn = setup_sqlite(path);
    let scope = "global";
    let vf = "2024-01-01";
    insert_edge(&conn, 1, 2, "works_at", vf, None, 1.0, scope);
    insert_edge(&conn, 2, 3, "located_in", vf, None, 1.0, scope);
    insert_edge(&conn, 1, 4, "knows", vf, None, 1.0, scope);
    insert_edge(&conn, 4, 5, "works_at", vf, None, 1.0, scope);
}

#[test]
fn test_find_connection_direct() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_pathfind_direct.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    setup_pathfinding_db(path_str);

    let graph = TemporalGraph::new(path_str).expect("new");

    // Alice (1) -> MBRAS (2) is a single-hop connection.
    let paths = graph
        .find_connection("global", 1, 2, 3)
        .expect("find_connection direct");

    assert!(!paths.is_empty(), "should find a direct path 1->2");
    assert_eq!(paths[0].depth, 1, "direct connection has depth 1");
    assert!(
        paths[0].path.contains("-[works_at]->"),
        "path should traverse works_at edge"
    );

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_find_connection_two_hops() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_pathfind_twohop.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    setup_pathfinding_db(path_str);

    let graph = TemporalGraph::new(path_str).expect("new");

    // Alice (1) -> MBRAS (2) -> Sao Paulo (3) — two hops.
    let paths = graph
        .find_connection("global", 1, 3, 5)
        .expect("find_connection two hops");

    assert!(!paths.is_empty(), "should find a 2-hop path 1->2->3");
    let best = &paths[0];
    assert_eq!(best.depth, 2, "two-hop path has depth 2");
    assert!(
        best.path.contains("-[works_at]->") && best.path.contains("-[located_in]->"),
        "path should contain both edge labels"
    );

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_find_connection_no_path() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_pathfind_nopath.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    setup_pathfinding_db(path_str);

    let graph = TemporalGraph::new(path_str).expect("new");

    // Sao Paulo (3) is a sink — no outgoing edges — so 3->1 should return empty.
    let paths = graph
        .find_connection("global", 3, 1, 5)
        .expect("find_connection no path");

    assert!(paths.is_empty(), "no path from sink node 3 back to 1");

    let _ = std::fs::remove_file(path_str);
}

#[test]
fn test_find_neighbors() {
    let dir = std::env::temp_dir();
    let path = dir.join("engram_test_neighbors.sqlite");
    let path_str = path.to_str().unwrap();
    let _ = std::fs::remove_file(path_str);

    setup_pathfinding_db(path_str);

    let graph = TemporalGraph::new(path_str).expect("new");

    // From Alice (1), within 2 hops, should reach:
    //   depth 1: 2 (MBRAS), 4 (Bob)
    //   depth 2: 3 (Sao Paulo), 5 (Competitor)
    let neighbors = graph
        .find_neighbors("global", 1, 2)
        .expect("find_neighbors");

    assert_eq!(
        neighbors.len(),
        4,
        "4 nodes reachable within 2 hops from Alice"
    );

    let depth1: Vec<_> = neighbors.iter().filter(|n| n.depth == 1).collect();
    let depth2: Vec<_> = neighbors.iter().filter(|n| n.depth == 2).collect();
    assert_eq!(depth1.len(), 2, "2 direct neighbours");
    assert_eq!(depth2.len(), 2, "2 two-hop neighbours");

    let _ = std::fs::remove_file(path_str);
}
