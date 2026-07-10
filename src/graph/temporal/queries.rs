//! Point-in-time queries: snapshots, per-memory edges and timelines.

use rusqlite::{params, Connection};

use super::types::{row_to_edge, TemporalEdge};
use crate::error::{EngramError, Result};

/// Return all edges that were valid at `timestamp`.
///
/// An edge is valid at `t` when `valid_from <= t` AND (`valid_to IS NULL` OR
/// `valid_to > t`).
///
/// When `scope_path` is `Some(prefix)`, only edges whose `scope_path` equals
/// `prefix` or starts with `prefix/` are returned (hierarchical prefix
/// matching). When `None`, edges from all scopes are returned (backward
/// compatible).
pub fn snapshot_at(
    conn: &Connection,
    timestamp: &str,
    scope_path: Option<&str>,
) -> Result<Vec<TemporalEdge>> {
    match scope_path {
        None => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, from_id, to_id, properties, valid_from, valid_to,
                            confidence, source, relation, created_at, scope_path
                     FROM   temporal_edges
                     WHERE  valid_from <= ?1
                       AND  (valid_to IS NULL OR valid_to > ?1)
                     ORDER  BY from_id, to_id, relation",
                )
                .map_err(EngramError::Database)?;

            let edges = stmt
                .query_map(params![timestamp], row_to_edge)
                .map_err(EngramError::Database)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(EngramError::Database)?;

            Ok(edges)
        }
        Some(scope) => {
            let pattern = format!("{}/%", scope);
            let mut stmt = conn
                .prepare(
                    "SELECT id, from_id, to_id, properties, valid_from, valid_to,
                            confidence, source, relation, created_at, scope_path
                     FROM   temporal_edges
                     WHERE  valid_from <= ?1
                       AND  (valid_to IS NULL OR valid_to > ?1)
                       AND  (scope_path = ?2 OR scope_path LIKE ?3)
                     ORDER  BY from_id, to_id, relation",
                )
                .map_err(EngramError::Database)?;

            let edges = stmt
                .query_map(params![timestamp, scope, pattern], row_to_edge)
                .map_err(EngramError::Database)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(EngramError::Database)?;

            Ok(edges)
        }
    }
}

/// Return all temporal edges where `memory_id` is either endpoint, valid at `timestamp`.
///
/// Unlike `snapshot_at`, this filters in SQL rather than loading the whole graph —
/// O(K) where K is the number of edges for that memory, not O(N_total).
pub fn edges_for_memory_at(
    conn: &Connection,
    memory_id: i64,
    timestamp: &str,
) -> Result<Vec<TemporalEdge>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, from_id, to_id, properties, valid_from, valid_to,
                    confidence, source, relation, created_at, scope_path
             FROM   temporal_edges
             WHERE  (from_id = ?1 OR to_id = ?1)
               AND  valid_from <= ?2
               AND  (valid_to IS NULL OR valid_to > ?2)
             ORDER  BY from_id, to_id, relation",
        )
        .map_err(EngramError::Database)?;

    let edges = stmt
        .query_map(params![memory_id, timestamp], row_to_edge)
        .map_err(EngramError::Database)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(EngramError::Database)?;

    Ok(edges)
}

/// Return the complete edit history for a `(from_id, to_id)` pair, ordered
/// chronologically (`valid_from ASC`, then `created_at ASC`).
///
/// When `scope_path` is `Some(prefix)`, only edges whose `scope_path` equals
/// `prefix` or starts with `prefix/` are returned. When `None`, all scopes
/// are included (backward compatible).
pub fn relationship_timeline(
    conn: &Connection,
    from_id: i64,
    to_id: i64,
    scope_path: Option<&str>,
) -> Result<Vec<TemporalEdge>> {
    match scope_path {
        None => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, from_id, to_id, properties, valid_from, valid_to,
                            confidence, source, relation, created_at, scope_path
                     FROM   temporal_edges
                     WHERE  from_id = ?1 AND to_id = ?2
                     ORDER  BY valid_from ASC, created_at ASC",
                )
                .map_err(EngramError::Database)?;

            let edges = stmt
                .query_map(params![from_id, to_id], row_to_edge)
                .map_err(EngramError::Database)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(EngramError::Database)?;

            Ok(edges)
        }
        Some(scope) => {
            let pattern = format!("{}/%", scope);
            let mut stmt = conn
                .prepare(
                    "SELECT id, from_id, to_id, properties, valid_from, valid_to,
                            confidence, source, relation, created_at, scope_path
                     FROM   temporal_edges
                     WHERE  from_id    = ?1
                       AND  to_id      = ?2
                       AND  (scope_path = ?3 OR scope_path LIKE ?4)
                     ORDER  BY valid_from ASC, created_at ASC",
                )
                .map_err(EngramError::Database)?;

            let edges = stmt
                .query_map(params![from_id, to_id, scope, pattern], row_to_edge)
                .map_err(EngramError::Database)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(EngramError::Database)?;

            Ok(edges)
        }
    }
}
