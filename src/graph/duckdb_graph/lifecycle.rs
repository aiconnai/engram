use duckdb::Connection as DuckdbConnection;

use crate::error::Result;
use tracing::{debug, warn};

use super::types::{validate_sqlite_path, TemporalGraph};

impl TemporalGraph {
    /// Open an in-memory DuckDB session attached to `sqlite_path`.
    ///
    /// Steps performed:
    /// - Validate `sqlite_path` (no single quotes, no null bytes, no `..` components).
    /// - Install + load the bundled `sqlite` scanner extension.
    /// - Attach the SQLite database as the catalog `engram` (read-only).
    /// - Attempt to install + load `duckpgq`; failures are non-fatal.
    /// - If PGQ loaded, register a property graph over `graph_entities` /
    ///   `temporal_edges`.
    pub fn new(sqlite_path: &str) -> Result<Self> {
        validate_sqlite_path(sqlite_path)?;

        let conn = DuckdbConnection::open_in_memory()?;

        // --- SQLite scanner extension -----------------------------------------
        // The bundled DuckDB already ships the sqlite extension; INSTALL is
        // effectively a no-op when it is already present.
        conn.execute_batch("INSTALL sqlite; LOAD sqlite;")?;

        // --- Attach the SQLite file read-only --------------------------------
        // Defense-in-depth: escape single quotes even though validate_sqlite_path
        // already rejects them.
        let safe_path = sqlite_path.replace('\'', "''");
        conn.execute_batch(&format!(
            "ATTACH '{path}' AS engram (TYPE SQLITE, READ_ONLY);",
            path = safe_path
        ))?;

        // --- Optional: duckpgq extension -------------------------------------
        let has_pgq = Self::try_load_pgq(&conn);

        Ok(Self {
            conn,
            has_pgq,
            sqlite_path: sqlite_path.to_string(),
        })
    }

    /// Attempt to install and load `duckpgq`, then register the property graph.
    ///
    /// Returns `true` on full success, `false` on any error (with a warning
    /// logged so the caller gets visibility without a hard failure).
    fn try_load_pgq(conn: &DuckdbConnection) -> bool {
        // Install extension — may fail if the registry is unavailable.
        if let Err(e) = conn.execute_batch("INSTALL duckpgq FROM community;") {
            warn!(
                "duckpgq install failed (graph pattern matching unavailable): {}",
                e
            );
            return false;
        }

        if let Err(e) = conn.execute_batch("LOAD duckpgq;") {
            warn!(
                "duckpgq load failed (graph pattern matching unavailable): {}",
                e
            );
            return false;
        }

        // Register the property graph over the attached SQLite tables.
        let pgq_ddl = r#"
            CREATE OR REPLACE PROPERTY GRAPH knowledge_graph
            VERTEX TABLES (engram.graph_entities)
            EDGE TABLES (
                engram.temporal_edges
                SOURCE KEY (from_id) REFERENCES graph_entities(id)
                DESTINATION KEY (to_id) REFERENCES graph_entities(id)
                LABEL relation
            );
        "#;

        if let Err(e) = conn.execute_batch(pgq_ddl) {
            warn!("duckpgq property graph creation failed: {}", e);
            return false;
        }

        debug!("duckpgq property graph 'knowledge_graph' created successfully");
        true
    }

    /// Whether the `duckpgq` extension loaded and the property graph is active.
    ///
    /// When `false`, graph pattern (`MATCH`) queries are unavailable; use
    /// standard SQL over `engram.temporal_edges` / `engram.graph_entities`.
    pub fn has_pgq(&self) -> bool {
        self.has_pgq
    }

    /// Re-attach the SQLite file to reflect writes committed since the last
    /// attach.
    ///
    /// DuckDB caches the SQLite file at attach time; this detach + re-attach
    /// cycle is the canonical way to pick up new data without restarting the
    /// DuckDB session.
    pub fn refresh(&self) -> Result<()> {
        // Detach the existing catalog.
        self.conn.execute_batch("DETACH engram;")?;

        // Re-attach read-only.  Path was validated in new(); escape as
        // defense-in-depth.
        let safe_path = self.sqlite_path.replace('\'', "''");
        self.conn.execute_batch(&format!(
            "ATTACH '{path}' AS engram (TYPE SQLITE, READ_ONLY);",
            path = safe_path
        ))?;

        debug!(
            "TemporalGraph: re-attached SQLite at '{}'",
            self.sqlite_path
        );
        Ok(())
    }
}
