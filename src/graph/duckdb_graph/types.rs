use duckdb::Connection as DuckdbConnection;
use serde::{Deserialize, Serialize};

use crate::error::EngramError;

/// A step (or complete result) in a graph path traversal.
///
/// Each value returned by `find_connection` or `find_neighbors` carries a
/// human-readable path string and the hop-count from the origin node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStep {
    /// Human-readable representation of the full path from origin to this node,
    /// e.g. `"1 -[works_at]-> 2 -[located_in]-> 3"`.
    pub path: String,
    /// Number of hops from the origin node.
    pub depth: i32,
}

/// A temporal edge returned from DuckDB queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckDbTemporalEdge {
    pub id: i64,
    pub from_id: i64,
    pub to_id: i64,
    pub relation: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub confidence: f32,
    pub scope_path: String,
}

/// Diff between two graph snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckDbGraphDiff {
    pub added: Vec<DuckDbTemporalEdge>,
    pub removed: Vec<DuckDbTemporalEdge>,
    pub changed: Vec<(DuckDbTemporalEdge, DuckDbTemporalEdge)>,
}

/// DuckDB-backed analytical graph over Engram's temporal edges.
///
/// Lifecycle:
/// 1. `new(sqlite_path)` — opens an in-memory DuckDB, attaches the SQLite
///    file read-only, optionally loads `duckpgq` and creates a property graph.
/// 2. `refresh()` — detaches and re-attaches SQLite to pick up writes that
///    have been committed since the last attach.
pub struct TemporalGraph {
    pub(crate) conn: DuckdbConnection,
    /// Whether the `duckpgq` extension loaded successfully.
    pub(crate) has_pgq: bool,
    /// The SQLite path kept for re-attach on `refresh`.
    pub(crate) sqlite_path: String,
}

/// Convert a DuckDB error into a public `EngramError`.
impl From<duckdb::Error> for EngramError {
    fn from(e: duckdb::Error) -> Self {
        EngramError::Storage(format!("DuckDB error: {}", e))
    }
}

pub(super) fn validate_sqlite_path(path: &str) -> crate::error::Result<()> {
    if path.contains('\'') {
        return Err(EngramError::InvalidInput(
            "sqlite_path must not contain single quotes".to_string(),
        ));
    }
    if path.contains('\0') {
        return Err(EngramError::InvalidInput(
            "sqlite_path must not contain null bytes".to_string(),
        ));
    }
    for component in path.split(['/', '\\']) {
        if component == ".." {
            return Err(EngramError::InvalidInput(
                "sqlite_path must not contain '..' path components".to_string(),
            ));
        }
    }
    Ok(())
}
