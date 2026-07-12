use duckdb::params;
use tracing::debug;

use crate::error::{EngramError, Result};

use super::types::{PathStep, TemporalGraph};

impl TemporalGraph {
    /// Find all shortest paths (up to `max_hops`) between two nodes in
    /// the given scope using a recursive CTE.
    ///
    /// Only currently-valid edges (`valid_to IS NULL`) are traversed.
    /// Cycle prevention is done by checking whether the destination node ID
    /// already appears in the accumulated path string.
    ///
    /// Returns at most 10 paths ordered by hop-count ascending. Returns an
    /// empty `Vec` when no path exists.
    pub fn find_connection(
        &self,
        scope: &str,
        start_id: i64,
        end_id: i64,
        max_hops: u8,
    ) -> Result<Vec<PathStep>> {
        let scope_pattern = format!("{}%", scope);

        let sql = "
            WITH RECURSIVE paths AS (
                SELECT
                    from_id,
                    to_id,
                    relation,
                    1                                                        AS depth,
                    CAST(from_id AS VARCHAR) || ' -[' || relation || ']-> '
                        || CAST(to_id AS VARCHAR)                           AS path
                FROM engram.temporal_edges
                WHERE from_id = $1
                  AND scope_path LIKE $2
                  AND valid_to IS NULL

                UNION ALL

                SELECT
                    p.from_id,
                    e.to_id,
                    e.relation,
                    p.depth + 1,
                    p.path || ' -[' || e.relation || ']-> ' || CAST(e.to_id AS VARCHAR)
                FROM paths p
                JOIN engram.temporal_edges e ON p.to_id = e.from_id
                WHERE p.depth < $3
                  AND e.scope_path LIKE $4
                  AND e.valid_to IS NULL
                  AND POSITION(CAST(e.to_id AS VARCHAR) IN p.path) = 0
            )
            SELECT path, depth
            FROM paths
            WHERE to_id = $5
            ORDER BY depth
            LIMIT 10
        ";

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![
                start_id,
                scope_pattern,
                max_hops as i32,
                scope_pattern,
                end_id
            ],
            |row| {
                Ok(PathStep {
                    path: row.get(0)?,
                    depth: row.get(1)?,
                })
            },
        )?;

        let mut steps = Vec::new();
        for row in rows {
            steps.push(row.map_err(|e| EngramError::Storage(format!("DuckDB row error: {}", e)))?);
        }

        debug!(
            "find_connection({} -> {}, max_hops={}): {} paths found",
            start_id,
            end_id,
            max_hops,
            steps.len()
        );
        Ok(steps)
    }

    /// Return all nodes reachable from `node_id` within `max_depth` hops in
    /// the given scope.
    ///
    /// Useful for neighbourhood exploration ("what's connected to X?").
    /// Like `find_connection`, only currently-valid edges are traversed and
    /// cycles are prevented via path-string containment checks.
    ///
    /// Results are ordered by depth ascending (closest nodes first).
    pub fn find_neighbors(
        &self,
        scope: &str,
        node_id: i64,
        max_depth: u8,
    ) -> Result<Vec<PathStep>> {
        let scope_pattern = format!("{}%", scope);

        let sql = "
            WITH RECURSIVE paths AS (
                SELECT
                    from_id,
                    to_id,
                    relation,
                    1                                                        AS depth,
                    CAST(from_id AS VARCHAR) || ' -[' || relation || ']-> '
                        || CAST(to_id AS VARCHAR)                           AS path
                FROM engram.temporal_edges
                WHERE from_id = $1
                  AND scope_path LIKE $2
                  AND valid_to IS NULL

                UNION ALL

                SELECT
                    p.from_id,
                    e.to_id,
                    e.relation,
                    p.depth + 1,
                    p.path || ' -[' || e.relation || ']-> ' || CAST(e.to_id AS VARCHAR)
                FROM paths p
                JOIN engram.temporal_edges e ON p.to_id = e.from_id
                WHERE p.depth < $3
                  AND e.scope_path LIKE $4
                  AND e.valid_to IS NULL
                  AND POSITION(CAST(e.to_id AS VARCHAR) IN p.path) = 0
            )
            SELECT path, depth
            FROM paths
            ORDER BY depth
        ";

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![node_id, scope_pattern, max_depth as i32, scope_pattern],
            |row| {
                Ok(PathStep {
                    path: row.get(0)?,
                    depth: row.get(1)?,
                })
            },
        )?;

        let mut steps = Vec::new();
        for row in rows {
            steps.push(row.map_err(|e| EngramError::Storage(format!("DuckDB row error: {}", e)))?);
        }

        debug!(
            "find_neighbors(node={}, max_depth={}): {} reachable nodes",
            node_id,
            max_depth,
            steps.len()
        );
        Ok(steps)
    }
}
