use std::collections::{HashMap, HashSet};

use crate::error::{EngramError, Result};
use rusqlite::Connection;

use super::{Conflict, ConflictType, Severity};
use crate::graph::conflicts::helpers::{dfs_detect_cycle, load_all_edges, table_exists};
use crate::graph::conflicts::types::CONTRADICTING_PAIRS;

// =============================================================================
// ConflictDetector
// =============================================================================

/// Detects conflicts in the `cross_references` graph.
pub struct ConflictDetector;

impl ConflictDetector {
    /// Run all detectors and return a combined, deduplicated list of conflicts.
    pub fn detect_all(conn: &Connection) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        conflicts.extend(Self::detect_contradictions(conn)?);
        conflicts.extend(Self::detect_temporal_inconsistencies(conn)?);
        conflicts.extend(Self::detect_cycles(conn)?);
        conflicts.extend(Self::detect_orphans(conn)?);
        Ok(conflicts)
    }

    /// Find edges where A→B has contradicting relation types
    /// (e.g. both "supports" and "contradicts" for the same pair).
    pub fn detect_contradictions(conn: &Connection) -> Result<Vec<Conflict>> {
        // Load all edges from cross_references.
        let edges = load_all_edges(conn)?;

        // Group by (from_id, to_id).
        let mut by_pair: HashMap<(i64, i64), Vec<super::helpers::EdgeRow>> = HashMap::new();
        for edge in edges {
            by_pair
                .entry((edge.from_id, edge.to_id))
                .or_default()
                .push(edge);
        }

        let mut conflicts = Vec::new();

        for ((from_id, to_id), group) in &by_pair {
            let relations: Vec<&str> = group.iter().map(|e| e.relation_type.as_str()).collect();

            for &(a, b) in CONTRADICTING_PAIRS {
                if relations.contains(&a) && relations.contains(&b) {
                    let involved_ids: Vec<i64> = group.iter().map(|e| e.id).collect();
                    conflicts.push(Conflict {
                        id: 0,
                        conflict_type: ConflictType::DirectContradiction,
                        edge_ids: involved_ids,
                        description: format!(
                            "Contradicting relations '{}' and '{}' between nodes {} and {}",
                            a, b, from_id, to_id
                        ),
                        severity: Severity::High,
                        resolved_at: None,
                        resolution_strategy: None,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Find edges with overlapping validity periods for the same entity pair.
    ///
    /// Queries the `cross_references` table and treats the `created_at` column
    /// as a proxy for validity start. If two edges share the same
    /// `(from_id, to_id, relation_type)` triple, that is considered a temporal
    /// inconsistency — one should have been closed when the next was created.
    pub fn detect_temporal_inconsistencies(conn: &Connection) -> Result<Vec<Conflict>> {
        // Self-join: same triple, both are unresolved / open, different IDs.
        let sql = "
            SELECT a.id, b.id, a.from_id, a.to_id, a.relation_type
            FROM   cross_references a
            JOIN   cross_references b
              ON   a.from_id       = b.from_id
             AND   a.to_id         = b.to_id
             AND   a.relation_type = b.relation_type
             AND   a.id < b.id
        ";

        let table_exists = table_exists(conn, "cross_references")?;
        if !table_exists {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare(sql).map_err(EngramError::Database)?;

        let pairs = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(EngramError::Database)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(EngramError::Database)?;

        let conflicts = pairs
            .into_iter()
            .map(|(id_a, id_b, from_id, to_id, rel)| Conflict {
                id: 0,
                conflict_type: ConflictType::TemporalInconsistency,
                edge_ids: vec![id_a, id_b],
                description: format!(
                    "Duplicate '{}' edges between nodes {} and {} (ids {} and {}); possible temporal overlap",
                    rel, from_id, to_id, id_a, id_b
                ),
                severity: Severity::Medium,
                resolved_at: None,
                resolution_strategy: None,
            })
            .collect();

        Ok(conflicts)
    }

    /// Detect cycles in the directed edge graph using iterative DFS.
    ///
    /// Returns one conflict per cycle found, listing the edge IDs that form
    /// that cycle.
    pub fn detect_cycles(conn: &Connection) -> Result<Vec<Conflict>> {
        let table_exists = table_exists(conn, "cross_references")?;
        if !table_exists {
            return Ok(Vec::new());
        }

        let edges = load_all_edges(conn)?;

        // Build adjacency list: from_id -> Vec<(to_id, edge_id)>
        let mut adj: HashMap<i64, Vec<(i64, i64)>> = HashMap::new();
        for edge in &edges {
            adj.entry(edge.from_id)
                .or_default()
                .push((edge.to_id, edge.id));
        }

        // Build edge lookup: (from_id, to_id) -> edge_id for path reconstruction.
        let mut edge_map: HashMap<(i64, i64), i64> = HashMap::new();
        for edge in &edges {
            edge_map.insert((edge.from_id, edge.to_id), edge.id);
        }

        let all_nodes: HashSet<i64> = edges.iter().flat_map(|e| [e.from_id, e.to_id]).collect();

        let mut visited: HashSet<i64> = HashSet::new();
        let mut rec_stack: HashSet<i64> = HashSet::new();
        let mut conflicts = Vec::new();

        for &start in &all_nodes {
            if !visited.contains(&start) {
                dfs_detect_cycle(
                    start,
                    &adj,
                    &edge_map,
                    &mut visited,
                    &mut rec_stack,
                    &mut conflicts,
                );
            }
        }

        Ok(conflicts)
    }

    /// Find edges whose `from_id` or `to_id` do not exist in the `memories`
    /// table.
    pub fn detect_orphans(conn: &Connection) -> Result<Vec<Conflict>> {
        let cr_exists = table_exists(conn, "cross_references")?;
        let mem_exists = table_exists(conn, "memories")?;

        if !cr_exists || !mem_exists {
            return Ok(Vec::new());
        }

        let sql = "
            SELECT cr.id, cr.from_id, cr.to_id
            FROM   cross_references cr
            WHERE  NOT EXISTS (SELECT 1 FROM memories m WHERE m.id = cr.from_id)
               OR  NOT EXISTS (SELECT 1 FROM memories m WHERE m.id = cr.to_id)
        ";

        let mut stmt = conn.prepare(sql).map_err(EngramError::Database)?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(EngramError::Database)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(EngramError::Database)?;

        let conflicts = rows
            .into_iter()
            .map(|(edge_id, from_id, to_id)| Conflict {
                id: 0,
                conflict_type: ConflictType::OrphanedReference,
                edge_ids: vec![edge_id],
                description: format!(
                    "Edge {} references non-existent memory node(s): from_id={}, to_id={}",
                    edge_id, from_id, to_id
                ),
                severity: Severity::Critical,
                resolved_at: None,
                resolution_strategy: None,
            })
            .collect();

        Ok(conflicts)
    }
}
