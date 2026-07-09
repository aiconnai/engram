use rusqlite::{params, Connection};

use crate::error::{EngramError, Result};

use super::{Conflict, ResolutionResult, ResolutionStrategy};
use crate::graph::conflicts::helpers::{chrono_now, row_to_conflict};

// =============================================================================
// ConflictResolver
// =============================================================================

/// Resolves conflicts and persists them to the `graph_conflicts` table.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve a saved conflict by its ID using the given strategy.
    pub fn resolve(
        conn: &Connection,
        conflict_id: i64,
        strategy: ResolutionStrategy,
    ) -> Result<ResolutionResult> {
        let conflict = Self::get_conflict(conn, conflict_id)?
            .ok_or_else(|| EngramError::NotFound(conflict_id))?;

        if conflict.resolved_at.is_some() {
            return Err(EngramError::InvalidInput(format!(
                "Conflict {} is already resolved",
                conflict_id
            )));
        }

        let edge_ids = &conflict.edge_ids;

        let (edges_removed, edges_kept) = match strategy {
            ResolutionStrategy::KeepNewer => resolve_keep_newer(conn, edge_ids)?,
            ResolutionStrategy::KeepHigherConfidence => {
                resolve_keep_higher_confidence(conn, edge_ids)?
            }
            ResolutionStrategy::Merge => resolve_merge(conn, edge_ids)?,
            ResolutionStrategy::Manual => {
                // No edge modifications — just mark resolved.
                (Vec::new(), edge_ids.clone())
            }
        };

        // Mark the conflict as resolved.
        let now = chrono_now();
        conn.execute(
            "UPDATE graph_conflicts
             SET resolved_at = ?1, resolution_strategy = ?2
             WHERE id = ?3",
            params![now, strategy.as_str(), conflict_id],
        )
        .map_err(EngramError::Database)?;

        Ok(ResolutionResult {
            conflict_id,
            strategy,
            edges_removed,
            edges_kept,
        })
    }

    /// Persist a detected conflict to the `graph_conflicts` table.
    ///
    /// Returns the generated row ID.
    pub fn save_conflict(conn: &Connection, conflict: &Conflict) -> Result<i64> {
        let edge_ids_json = serde_json::to_string(&conflict.edge_ids)?;
        let resolution_strategy = conflict.resolution_strategy.as_ref().map(|s| s.as_str());

        conn.execute(
            "INSERT INTO graph_conflicts
                 (conflict_type, edge_ids, description, severity, resolved_at, resolution_strategy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                conflict.conflict_type.as_str(),
                edge_ids_json,
                conflict.description,
                conflict.severity.as_str(),
                conflict.resolved_at,
                resolution_strategy,
            ],
        )
        .map_err(EngramError::Database)?;

        Ok(conn.last_insert_rowid())
    }

    /// List conflicts from the `graph_conflicts` table.
    ///
    /// - `resolved = Some(true)`  — only resolved conflicts.
    /// - `resolved = Some(false)` — only unresolved conflicts.
    /// - `resolved = None`        — all conflicts.
    pub fn list_conflicts(conn: &Connection, resolved: Option<bool>) -> Result<Vec<Conflict>> {
        let sql = match resolved {
            Some(true) => {
                "SELECT id, conflict_type, edge_ids, description, severity,
                        resolved_at, resolution_strategy
                 FROM   graph_conflicts
                 WHERE  resolved_at IS NOT NULL
                 ORDER  BY id ASC"
            }
            Some(false) => {
                "SELECT id, conflict_type, edge_ids, description, severity,
                        resolved_at, resolution_strategy
                 FROM   graph_conflicts
                 WHERE  resolved_at IS NULL
                 ORDER  BY id ASC"
            }
            None => {
                "SELECT id, conflict_type, edge_ids, description, severity,
                        resolved_at, resolution_strategy
                 FROM   graph_conflicts
                 ORDER  BY id ASC"
            }
        };

        let mut stmt = conn.prepare(sql).map_err(EngramError::Database)?;

        let rows = stmt
            .query_map([], row_to_conflict)
            .map_err(EngramError::Database)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(EngramError::Database)?;

        Ok(rows)
    }

    /// Retrieve a single conflict by ID.
    pub fn get_conflict(conn: &Connection, id: i64) -> Result<Option<Conflict>> {
        let mut stmt = conn
            .prepare(
                "SELECT id, conflict_type, edge_ids, description, severity,
                        resolved_at, resolution_strategy
                 FROM   graph_conflicts
                 WHERE  id = ?1",
            )
            .map_err(EngramError::Database)?;

        let mut rows = stmt
            .query_map(params![id], row_to_conflict)
            .map_err(EngramError::Database)?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(EngramError::Database)?)),
            None => Ok(None),
        }
    }
}

// =============================================================================
// Resolution helpers
// =============================================================================

/// Keep the edge with the highest ID (most recently inserted) and remove the
/// rest.  Returns `(removed, kept)`.
fn resolve_keep_newer(conn: &Connection, edge_ids: &[i64]) -> Result<(Vec<i64>, Vec<i64>)> {
    if edge_ids.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Load creation timestamps from cross_references.
    let mut id_times: Vec<(i64, String)> = edge_ids
        .iter()
        .filter_map(|&id| {
            let ts: rusqlite::Result<String> = conn.query_row(
                "SELECT created_at FROM cross_references WHERE id = ?1",
                params![id],
                |r| r.get(0),
            );
            ts.ok().map(|t| (id, t))
        })
        .collect();

    // Sort ascending; the last element is the newest.
    id_times.sort_by(|a, b| a.1.cmp(&b.1));

    if id_times.is_empty() {
        return Ok((Vec::new(), edge_ids.to_vec()));
    }

    let newest_id = id_times.last().unwrap().0;
    let to_remove: Vec<i64> = id_times
        .iter()
        .filter(|(id, _)| *id != newest_id)
        .map(|(id, _)| *id)
        .collect();

    for &id in &to_remove {
        conn.execute("DELETE FROM cross_references WHERE id = ?1", params![id])
            .map_err(EngramError::Database)?;
    }

    Ok((to_remove, vec![newest_id]))
}

/// Keep the edge with the highest `strength` (confidence proxy) and remove the
/// rest.  Returns `(removed, kept)`.
fn resolve_keep_higher_confidence(
    conn: &Connection,
    edge_ids: &[i64],
) -> Result<(Vec<i64>, Vec<i64>)> {
    if edge_ids.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Load strength from cross_references.
    let mut id_strengths: Vec<(i64, f64)> = edge_ids
        .iter()
        .filter_map(|&id| {
            let s: rusqlite::Result<f64> = conn.query_row(
                "SELECT strength FROM cross_references WHERE id = ?1",
                params![id],
                |r| r.get(0),
            );
            s.ok().map(|strength| (id, strength))
        })
        .collect();

    // Sort ascending; last element has highest strength.
    id_strengths.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if id_strengths.is_empty() {
        return Ok((Vec::new(), edge_ids.to_vec()));
    }

    let best_id = id_strengths.last().unwrap().0;
    let to_remove: Vec<i64> = id_strengths
        .iter()
        .filter(|(id, _)| *id != best_id)
        .map(|(id, _)| *id)
        .collect();

    for &id in &to_remove {
        conn.execute("DELETE FROM cross_references WHERE id = ?1", params![id])
            .map_err(EngramError::Database)?;
    }

    Ok((to_remove, vec![best_id]))
}

/// Merge edges: keep the one with the highest strength, update its metadata to
/// be the JSON merge of all involved edges, then delete the rest.
/// Returns `(removed, kept)`.
fn resolve_merge(conn: &Connection, edge_ids: &[i64]) -> Result<(Vec<i64>, Vec<i64>)> {
    if edge_ids.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Load all edge metadata.
    let mut rows: Vec<(i64, f64, String)> = edge_ids
        .iter()
        .filter_map(|&id| {
            conn.query_row(
                "SELECT id, strength, metadata FROM cross_references WHERE id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, f64>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .ok()
        })
        .collect();

    if rows.is_empty() {
        return Ok((Vec::new(), edge_ids.to_vec()));
    }

    // Sort by strength desc; first element is the keeper.
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let (keep_id, keep_strength, keep_meta_str) = rows.remove(0);

    // Merge metadata JSON objects.
    let mut merged: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&keep_meta_str).unwrap_or_default();

    for (_, _, meta_str) in &rows {
        if let Ok(serde_json::Value::Object(extra)) = serde_json::from_str(meta_str) {
            for (k, v) in extra {
                merged.entry(k).or_insert(v);
            }
        }
    }

    let merged_str = serde_json::to_string(&serde_json::Value::Object(merged))?;

    conn.execute(
        "UPDATE cross_references SET metadata = ?1, strength = ?2 WHERE id = ?3",
        params![merged_str, keep_strength, keep_id],
    )
    .map_err(EngramError::Database)?;

    let to_remove: Vec<i64> = rows.iter().map(|(id, _, _)| *id).collect();

    for &id in &to_remove {
        conn.execute("DELETE FROM cross_references WHERE id = ?1", params![id])
            .map_err(EngramError::Database)?;
    }

    Ok((to_remove, vec![keep_id]))
}
