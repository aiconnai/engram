//! Edge mutations: adding, invalidating and fetching single edges.

use rusqlite::{params, Connection};
use serde_json::Value;

use super::types::{row_to_edge, TemporalEdge};
use crate::error::{EngramError, Result};

/// Add a new temporal edge.
///
/// If an open edge (`valid_to IS NULL`) already exists for the same
/// `(from_id, to_id, relation)` triple **within the same scope**, it is
/// automatically closed by setting its `valid_to` to the `valid_from` of the
/// new edge before inserting.
///
/// `scope_path` defaults to `"global"` when `None`.
///
/// Returns the newly inserted edge with its generated `id` and `created_at`.
#[allow(clippy::too_many_arguments)]
pub fn add_edge(
    conn: &Connection,
    from_id: i64,
    to_id: i64,
    relation: &str,
    properties: &Value,
    valid_from: &str,
    confidence: f32,
    source: &str,
    scope_path: Option<&str>,
) -> Result<TemporalEdge> {
    let scope = scope_path.unwrap_or("global");
    let props_str = serde_json::to_string(properties)?;

    // Auto-invalidate any currently-open edges for the same triple within
    // the same scope.
    conn.execute(
        "UPDATE temporal_edges
         SET valid_to = ?1
         WHERE from_id    = ?2
           AND to_id      = ?3
           AND relation   = ?4
           AND scope_path = ?5
           AND valid_to IS NULL",
        params![valid_from, from_id, to_id, relation, scope],
    )
    .map_err(EngramError::Database)?;

    // Insert the new edge.
    conn.execute(
        "INSERT INTO temporal_edges
             (from_id, to_id, relation, properties, valid_from, confidence, source, scope_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![from_id, to_id, relation, props_str, valid_from, confidence, source, scope],
    )
    .map_err(EngramError::Database)?;

    let id = conn.last_insert_rowid();
    get_edge_by_id(conn, id)?
        .ok_or_else(|| EngramError::Internal(format!("Edge {} disappeared after insert", id)))
}

/// Set the `valid_to` timestamp on an existing edge, effectively closing it.
pub fn invalidate_edge(conn: &Connection, edge_id: i64, valid_to: &str) -> Result<()> {
    let affected = conn
        .execute(
            "UPDATE temporal_edges SET valid_to = ?1 WHERE id = ?2",
            params![valid_to, edge_id],
        )
        .map_err(EngramError::Database)?;

    if affected == 0 {
        return Err(EngramError::NotFound(edge_id));
    }
    Ok(())
}

/// Fetch a single edge by row id.
pub(super) fn get_edge_by_id(conn: &Connection, id: i64) -> Result<Option<TemporalEdge>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, from_id, to_id, properties, valid_from, valid_to,
                    confidence, source, relation, created_at, scope_path
             FROM   temporal_edges
             WHERE  id = ?1",
        )
        .map_err(EngramError::Database)?;

    let mut rows = stmt
        .query_map(params![id], row_to_edge)
        .map_err(EngramError::Database)?;

    match rows.next() {
        Some(row) => Ok(Some(row.map_err(EngramError::Database)?)),
        None => Ok(None),
    }
}
