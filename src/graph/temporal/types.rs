//! DDL, core types and row mapping for the temporal knowledge graph.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// =============================================================================
// DDL
// =============================================================================

/// SQL that creates the `temporal_edges` table and its supporting indexes.
///
/// Safe to run on an existing database — all statements use `IF NOT EXISTS`.
///
/// Note: the `scope_path` column was added in migration v33. This constant
/// reflects the canonical schema; production databases gain the column via
/// the migration runner.
pub const CREATE_TEMPORAL_EDGES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS temporal_edges (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id     INTEGER NOT NULL,
    to_id       INTEGER NOT NULL,
    relation    TEXT    NOT NULL,
    properties  TEXT    NOT NULL DEFAULT '{}',
    valid_from  TEXT    NOT NULL,
    valid_to    TEXT,
    confidence  REAL    NOT NULL DEFAULT 1.0,
    source      TEXT    NOT NULL DEFAULT '',
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    scope_path  TEXT    NOT NULL DEFAULT 'global'
);
CREATE INDEX IF NOT EXISTS idx_temporal_edges_from       ON temporal_edges(from_id);
CREATE INDEX IF NOT EXISTS idx_temporal_edges_to         ON temporal_edges(to_id);
CREATE INDEX IF NOT EXISTS idx_temporal_edges_valid      ON temporal_edges(valid_from, valid_to);
CREATE INDEX IF NOT EXISTS idx_temporal_edges_scope_path ON temporal_edges(scope_path);
"#;

// =============================================================================
// Types
// =============================================================================

/// A directed edge in the temporal knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEdge {
    /// Row identifier.
    pub id: i64,
    /// Source memory / node.
    pub from_id: i64,
    /// Target memory / node.
    pub to_id: i64,
    /// Semantic label for the relationship (e.g. `"works_at"`, `"reports_to"`).
    pub relation: String,
    /// Arbitrary key-value metadata stored as JSON.
    pub properties: Value,
    /// Start of validity period (RFC3339 UTC).
    pub valid_from: String,
    /// End of validity period (RFC3339 UTC), `None` means still valid.
    pub valid_to: Option<String>,
    /// Confidence in this edge (0.0–1.0).
    pub confidence: f32,
    /// Provenance string (e.g. document name, agent ID).
    pub source: String,
    /// Wall-clock creation time (RFC3339 UTC).
    pub created_at: String,
    /// Hierarchical scope path (e.g. `"global"`, `"global/org:acme/user:alice"`).
    /// Added in schema v33. Defaults to `"global"` for backward compatibility.
    pub scope_path: String,
}

/// Summary of how the graph changed between two timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    /// Edges present at `t2` but not at `t1`.
    pub added: Vec<TemporalEdge>,
    /// Edges present at `t1` but not at `t2`.
    pub removed: Vec<TemporalEdge>,
    /// Edges whose properties or confidence changed between `t1` and `t2`.
    ///
    /// Each tuple is `(old_edge_at_t1, new_edge_at_t2)`.
    pub changed: Vec<(TemporalEdge, TemporalEdge)>,
}

// =============================================================================
// Row mapper helpers
// =============================================================================

/// Build a `TemporalEdge` from a rusqlite row.
///
/// Expected column order:
/// 0: id, 1: from_id, 2: to_id, 3: properties, 4: valid_from, 5: valid_to,
/// 6: confidence, 7: source, 8: relation, 9: created_at, 10: scope_path
pub(super) fn row_to_edge(row: &rusqlite::Row<'_>) -> rusqlite::Result<TemporalEdge> {
    let props_str: String = row.get(3)?;
    let properties: Value =
        serde_json::from_str(&props_str).unwrap_or(Value::Object(Default::default()));

    Ok(TemporalEdge {
        id: row.get(0)?,
        from_id: row.get(1)?,
        to_id: row.get(2)?,
        relation: row.get(8)?,
        properties,
        valid_from: row.get(4)?,
        valid_to: row.get(5)?,
        confidence: row.get(6)?,
        source: row.get(7)?,
        created_at: row.get(9)?,
        scope_path: row.get(10)?,
    })
}
