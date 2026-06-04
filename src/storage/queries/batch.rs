use super::*;

/// Result of a batch create operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchCreateResult {
    pub created: Vec<Memory>,
    pub failed: Vec<BatchError>,
    pub total_created: usize,
    pub total_failed: usize,
}

/// Result of a batch delete operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchDeleteResult {
    pub deleted: Vec<i64>,
    pub failed: Vec<BatchError>,
    pub total_deleted: usize,
    pub total_failed: usize,
}

/// Error information for batch operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchError {
    pub index: usize,
    pub id: Option<i64>,
    pub error: String,
}

/// Create multiple memories in a single transaction
pub fn create_memory_batch(
    conn: &Connection,
    inputs: &[CreateMemoryInput],
) -> Result<BatchCreateResult> {
    let mut created = Vec::new();
    let mut failed = Vec::new();

    for (index, input) in inputs.iter().enumerate() {
        match create_memory(conn, input) {
            Ok(memory) => created.push(memory),
            Err(e) => failed.push(BatchError {
                index,
                id: None,
                error: e.to_string(),
            }),
        }
    }

    Ok(BatchCreateResult {
        total_created: created.len(),
        total_failed: failed.len(),
        created,
        failed,
    })
}

/// Collect all memory IDs in a "supersedes" chain starting from `root_id`.
///
/// Returns `root_id` plus all ancestors (memories that `root_id` supersedes,
/// recursively), following `crossrefs` rows where `edge_type = 'supersedes'`
/// and `from_id = current`.  Capped at 100 hops to prevent infinite loops.
pub fn collect_supersedes_chain(conn: &Connection, root_id: i64) -> Result<Vec<i64>> {
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    visited.insert(root_id);
    let mut chain: Vec<i64> = vec![root_id];
    let mut current_ids = vec![root_id];

    for _ in 0..100usize {
        if current_ids.is_empty() {
            break;
        }

        // Build placeholder list for the IN clause using positional params
        let placeholders: String = current_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "SELECT to_id FROM crossrefs \
             WHERE edge_type = 'supersedes' \
               AND valid_to IS NULL \
               AND from_id IN ({placeholders})",
            placeholders = placeholders
        );

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = current_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        // Use HashSet for O(1) dedup — guards against both cycles and
        // diamond topologies where multiple parents share the same ancestor.
        let next_ids: Vec<i64> = stmt
            .query_map(params_refs.as_slice(), |row| row.get(0))?
            .filter_map(|r| r.ok())
            .filter(|id| visited.insert(*id)) // insert returns false if already present
            .collect();

        if next_ids.is_empty() {
            break;
        }

        chain.extend_from_slice(&next_ids);
        current_ids = next_ids;
    }

    Ok(chain)
}

/// Delete multiple memories in a single transaction
pub fn delete_memory_batch(conn: &Connection, ids: &[i64]) -> Result<BatchDeleteResult> {
    let mut deleted = Vec::new();
    let mut failed = Vec::new();

    for (index, &id) in ids.iter().enumerate() {
        match delete_memory(conn, id) {
            Ok(()) => deleted.push(id),
            Err(e) => failed.push(BatchError {
                index,
                id: Some(id),
                error: e.to_string(),
            }),
        }
    }

    Ok(BatchDeleteResult {
        total_deleted: deleted.len(),
        total_failed: failed.len(),
        deleted,
        failed,
    })
}
