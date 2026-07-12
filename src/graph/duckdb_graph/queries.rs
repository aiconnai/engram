use duckdb::params;

use crate::error::Result;

use super::types::{DuckDbGraphDiff, DuckDbTemporalEdge, TemporalGraph};

impl TemporalGraph {
    /// Return all edges whose validity window includes `timestamp` within
    /// the given `scope` prefix.
    ///
    /// Edges are included when:
    /// - `scope_path` starts with `scope`
    /// - `valid_from <= timestamp`
    /// - `valid_to IS NULL OR valid_to >= timestamp`
    pub fn snapshot_at(&self, scope: &str, timestamp: &str) -> Result<Vec<DuckDbTemporalEdge>> {
        let scope_pattern = format!("{}%", scope);
        let sql = "
            SELECT id, from_id, to_id, relation, valid_from, valid_to, confidence, scope_path
            FROM engram.temporal_edges
            WHERE scope_path LIKE ?
              AND valid_from <= ?
              AND (valid_to IS NULL OR valid_to >= ?)
            ORDER BY id ASC
        ";
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![scope_pattern, timestamp, timestamp], |row| {
            Ok(DuckDbTemporalEdge {
                id: row.get(0)?,
                from_id: row.get(1)?,
                to_id: row.get(2)?,
                relation: row.get(3)?,
                valid_from: row.get(4)?,
                valid_to: row.get(5)?,
                confidence: row.get::<_, f64>(6)? as f32,
                scope_path: row.get(7)?,
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Compute the structural difference between two graph snapshots at `t1`
    /// and `t2` within the given `scope` prefix.
    ///
    /// - `added`   — edges present in t2 but not t1 (matched by (from_id, to_id, relation))
    /// - `removed` — edges present in t1 but not t2
    /// - `changed` — edges present in both snapshots but with differing
    ///   `confidence` or `valid_to`; tuple is (t1_edge, t2_edge)
    pub fn graph_diff(&self, scope: &str, t1: &str, t2: &str) -> Result<DuckDbGraphDiff> {
        let snap1 = self.snapshot_at(scope, t1)?;
        let snap2 = self.snapshot_at(scope, t2)?;

        // Build a lookup key: (from_id, to_id, relation) -> edge
        use std::collections::HashMap;

        let key = |e: &DuckDbTemporalEdge| (e.from_id, e.to_id, e.relation.clone());

        let map1: HashMap<_, _> = snap1.iter().map(|e| (key(e), e.clone())).collect();
        let map2: HashMap<_, _> = snap2.iter().map(|e| (key(e), e.clone())).collect();

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        for (k, e2) in &map2 {
            match map1.get(k) {
                None => added.push(e2.clone()),
                Some(e1) => {
                    // Consider an edge "changed" when confidence or valid_to differ.
                    let conf_changed = (e1.confidence - e2.confidence).abs() > f32::EPSILON;
                    let valid_to_changed = e1.valid_to != e2.valid_to;
                    if conf_changed || valid_to_changed {
                        changed.push((e1.clone(), e2.clone()));
                    }
                }
            }
        }

        for (k, e1) in &map1 {
            if !map2.contains_key(k) {
                removed.push(e1.clone());
            }
        }

        Ok(DuckDbGraphDiff {
            added,
            removed,
            changed,
        })
    }

    /// Return the full history of edges between `from_id` and `to_id` within
    /// the given `scope` prefix, ordered from most-recent to oldest.
    pub fn relationship_timeline(
        &self,
        scope: &str,
        from_id: i64,
        to_id: i64,
    ) -> Result<Vec<DuckDbTemporalEdge>> {
        let scope_pattern = format!("{}%", scope);
        let sql = "
            SELECT id, from_id, to_id, relation, valid_from, valid_to, confidence, scope_path
            FROM engram.temporal_edges
            WHERE scope_path LIKE ?
              AND from_id = ?
              AND to_id   = ?
            ORDER BY valid_from DESC
        ";
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![scope_pattern, from_id, to_id], |row| {
            Ok(DuckDbTemporalEdge {
                id: row.get(0)?,
                from_id: row.get(1)?,
                to_id: row.get(2)?,
                relation: row.get(3)?,
                valid_from: row.get(4)?,
                valid_to: row.get(5)?,
                confidence: row.get::<_, f64>(6)? as f32,
                scope_path: row.get(7)?,
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}
