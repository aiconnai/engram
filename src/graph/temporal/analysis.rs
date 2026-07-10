//! Graph analysis: contradiction detection and point-in-time diffing.

use rusqlite::Connection;
use serde_json::Value;

use super::queries::snapshot_at;
use super::types::{GraphDiff, TemporalEdge};
use crate::error::{EngramError, Result};

/// Detect edges that share the same `(from_id, to_id, relation)` triple and
/// have **overlapping** validity periods — which should not exist under normal
/// operation.
///
/// Returns pairs `(edge_a, edge_b)` where `edge_a.id < edge_b.id`.
pub fn detect_contradictions(conn: &Connection) -> Result<Vec<(TemporalEdge, TemporalEdge)>> {
    // Self-join: find pairs that share the triple and overlap.
    // Overlap condition: a.valid_from < b.valid_to_or_max AND b.valid_from < a.valid_to_or_max
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.from_id, a.to_id, a.properties, a.valid_from, a.valid_to,
                    a.confidence, a.source, a.relation, a.created_at, a.scope_path,
                    b.id, b.from_id, b.to_id, b.properties, b.valid_from, b.valid_to,
                    b.confidence, b.source, b.relation, b.created_at, b.scope_path
             FROM   temporal_edges a
             JOIN   temporal_edges b
               ON   a.from_id  = b.from_id
              AND   a.to_id    = b.to_id
              AND   a.relation = b.relation
              AND   a.id < b.id
             WHERE  a.valid_from < COALESCE(b.valid_to, '9999-12-31T23:59:59Z')
               AND  b.valid_from < COALESCE(a.valid_to, '9999-12-31T23:59:59Z')",
        )
        .map_err(EngramError::Database)?;

    let pairs = stmt
        .query_map([], |row| {
            // First edge columns: 0..10
            let props_a: String = row.get(3)?;
            // Second edge columns: 11..21
            let props_b: String = row.get(14)?;

            let edge_a = TemporalEdge {
                id: row.get(0)?,
                from_id: row.get(1)?,
                to_id: row.get(2)?,
                properties: serde_json::from_str(&props_a)
                    .unwrap_or(Value::Object(Default::default())),
                valid_from: row.get(4)?,
                valid_to: row.get(5)?,
                confidence: row.get(6)?,
                source: row.get(7)?,
                relation: row.get(8)?,
                created_at: row.get(9)?,
                scope_path: row.get(10)?,
            };

            let edge_b = TemporalEdge {
                id: row.get(11)?,
                from_id: row.get(12)?,
                to_id: row.get(13)?,
                properties: serde_json::from_str(&props_b)
                    .unwrap_or(Value::Object(Default::default())),
                valid_from: row.get(15)?,
                valid_to: row.get(16)?,
                confidence: row.get(17)?,
                source: row.get(18)?,
                relation: row.get(19)?,
                created_at: row.get(20)?,
                scope_path: row.get(21)?,
            };

            Ok((edge_a, edge_b))
        })
        .map_err(EngramError::Database)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(EngramError::Database)?;

    Ok(pairs)
}

/// Compare the graph state at two different timestamps.
///
/// - `added`   — edges valid at `t2` whose `(from_id, to_id, relation)` triple
///   was not present at `t1`.
/// - `removed` — edges valid at `t1` whose triple was not present at `t2`.
/// - `changed` — triples present at both `t1` and `t2` but with a different
///   `id` (i.e. the edge was superseded), implying the properties
///   or confidence changed.
///
/// When `scope_path` is `Some(prefix)`, the diff is limited to edges within
/// that scope hierarchy. When `None`, all scopes are compared (backward
/// compatible).
pub fn diff(conn: &Connection, t1: &str, t2: &str, scope_path: Option<&str>) -> Result<GraphDiff> {
    let snap1 = snapshot_at(conn, t1, scope_path)?;
    let snap2 = snapshot_at(conn, t2, scope_path)?;

    // Key: (from_id, to_id, relation)
    type Key = (i64, i64, String);

    let map1: std::collections::HashMap<Key, TemporalEdge> = snap1
        .into_iter()
        .map(|e| ((e.from_id, e.to_id, e.relation.clone()), e))
        .collect();

    let map2: std::collections::HashMap<Key, TemporalEdge> = snap2
        .into_iter()
        .map(|e| ((e.from_id, e.to_id, e.relation.clone()), e))
        .collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (key, edge2) in &map2 {
        match map1.get(key) {
            None => added.push(edge2.clone()),
            Some(edge1) if edge1.id != edge2.id => {
                changed.push((edge1.clone(), edge2.clone()));
            }
            _ => {} // same edge, no change
        }
    }

    for (key, edge1) in &map1 {
        if !map2.contains_key(key) {
            removed.push(edge1.clone());
        }
    }

    Ok(GraphDiff {
        added,
        removed,
        changed,
    })
}
