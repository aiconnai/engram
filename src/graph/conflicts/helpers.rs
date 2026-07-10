use std::collections::{HashMap, HashSet};

use crate::error::{EngramError, Result};
use rusqlite::{Connection, Row};

use super::{Conflict, ConflictType, ResolutionStrategy, Severity};

// =============================================================================
// Private helpers
// =============================================================================

/// Minimal representation of a row in `cross_references`.
#[derive(Debug)]
pub(super) struct EdgeRow {
    pub(crate) id: i64,
    pub(crate) from_id: i64,
    pub(crate) to_id: i64,
    pub(crate) relation_type: String,
    /// Stored for temporal ordering; not read directly in Rust but used in SQL
    /// ordering.
    #[allow(dead_code)]
    created_at: String,
}

/// Load all rows from `cross_references`. Returns empty vec if table does not
/// exist.
pub(super) fn load_all_edges(conn: &Connection) -> Result<Vec<EdgeRow>> {
    if !table_exists(conn, "cross_references")? {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, from_id, to_id, relation_type, created_at
             FROM   cross_references
             ORDER  BY id ASC",
        )
        .map_err(EngramError::Database)?;

    let rows = stmt
        .query_map([], |row| {
            Ok(EdgeRow {
                id: row.get(0)?,
                from_id: row.get(1)?,
                to_id: row.get(2)?,
                relation_type: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(EngramError::Database)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(EngramError::Database)?;

    Ok(rows)
}

/// Iterative DFS for cycle detection. Detected cycles are appended to
/// `conflicts`.
pub(super) fn dfs_detect_cycle(
    start: i64,
    adj: &HashMap<i64, Vec<(i64, i64)>>,
    edge_map: &HashMap<(i64, i64), i64>,
    visited: &mut HashSet<i64>,
    rec_stack: &mut HashSet<i64>,
    conflicts: &mut Vec<Conflict>,
) {
    // Stack items: (node, index into adj[node], parent_edge_id)
    let mut stack: Vec<(i64, usize, Option<i64>)> = vec![(start, 0, None)];
    let mut path: Vec<i64> = vec![start];

    visited.insert(start);
    rec_stack.insert(start);

    while let Some((node, idx, _parent_edge)) = stack.last_mut() {
        let node = *node;
        let neighbors = adj.get(&node).map(|v| v.as_slice()).unwrap_or(&[]);

        if *idx < neighbors.len() {
            let (neighbor, edge_id) = neighbors[*idx];
            *idx += 1;

            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                rec_stack.insert(neighbor);
                path.push(neighbor);
                stack.push((neighbor, 0, Some(edge_id)));
            } else if rec_stack.contains(&neighbor) {
                // Cycle detected — collect the edge IDs that form the cycle.
                let cycle_start_pos = path.iter().position(|&n| n == neighbor).unwrap_or(0);
                let cycle_nodes = &path[cycle_start_pos..];
                let mut cycle_edge_ids: Vec<i64> = Vec::new();
                for window in cycle_nodes.windows(2) {
                    if let Some(&eid) = edge_map.get(&(window[0], window[1])) {
                        cycle_edge_ids.push(eid);
                    }
                }
                // Close the cycle: last node -> neighbor
                if let Some(&eid) =
                    edge_map.get(&(*cycle_nodes.last().unwrap_or(&neighbor), neighbor))
                {
                    cycle_edge_ids.push(eid);
                }

                if !cycle_edge_ids.is_empty() {
                    conflicts.push(Conflict {
                        id: 0,
                        conflict_type: ConflictType::CyclicDependency,
                        edge_ids: cycle_edge_ids.clone(),
                        description: format!("Cycle detected involving nodes: {:?}", cycle_nodes),
                        severity: Severity::Medium,
                        resolved_at: None,
                        resolution_strategy: None,
                    });
                }
            }
        } else {
            // Done with this node — pop.
            stack.pop();
            path.pop();
            rec_stack.remove(&node);
        }
    }
}

/// Check whether a table exists in the current SQLite database.
pub(super) fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [name],
            |r| r.get(0),
        )
        .map_err(EngramError::Database)?;
    Ok(count > 0)
}

/// Map a rusqlite row to a `Conflict`.
pub(super) fn row_to_conflict(row: &Row<'_>) -> rusqlite::Result<Conflict> {
    let id: i64 = row.get(0)?;
    let conflict_type_str: String = row.get(1)?;
    let edge_ids_str: String = row.get(2)?;
    let description: String = row.get(3)?;
    let severity_str: String = row.get(4)?;
    let resolved_at: Option<String> = row.get(5)?;
    let resolution_strategy_str: Option<String> = row.get(6)?;

    let conflict_type =
        ConflictType::from_str(&conflict_type_str).unwrap_or(ConflictType::DirectContradiction);
    let edge_ids: Vec<i64> = serde_json::from_str(&edge_ids_str).unwrap_or_default();
    let severity = Severity::from_str(&severity_str).unwrap_or(Severity::Low);
    let resolution_strategy = resolution_strategy_str
        .as_deref()
        .and_then(ResolutionStrategy::from_str);

    Ok(Conflict {
        id,
        conflict_type,
        edge_ids,
        description,
        severity,
        resolved_at,
        resolution_strategy,
    })
}

/// Return the current UTC timestamp in RFC3339 format.
pub(super) fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Use chrono if available; otherwise fall back to a formatted timestamp.
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
        .unwrap_or(chrono::Utc::now());
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
