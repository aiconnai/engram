use std::collections::HashMap;
use std::time::Instant;

use crate::embedding::{
    get_embedding_queue_health, DEFAULT_MAX_EMBEDDING_RETRIES, DEFAULT_STALE_PROCESSING_AFTER,
};
use crate::error::Result;

use super::super::backend::{
    DerivedIndexHealth, DerivedIndexKind, DerivedIndexStatus, HealthStatus,
};
use super::super::connection::Storage;
use super::super::db::DbConnectionExt;

/// Check SQLite storage health using an already-open storage handle.
///
/// This is intentionally separate from `SqliteBackend::new(...).health_check()`
/// so read-only callers, such as CLI status reporting, do not reopen the
/// database and trigger migrations or connection pragmas.
pub fn health_check_storage(storage: &Storage) -> Result<HealthStatus> {
    let start = Instant::now();

    let storage_mode_warning = storage.storage_mode_warning();
    let db_path = storage.db_path().to_string();

    let result = storage.with_connection(|conn| {
        conn.query_exists("SELECT 1", &[])?;

        let quick_check: String = conn.query_scalar_0("PRAGMA quick_check")?;
        let quick_check_ok = quick_check == "ok";
        let quick_check_status = if quick_check_ok {
            "ok".to_string()
        } else {
            quick_check
        };

        let page_size: i64 = conn.query_scalar_0("PRAGMA page_size")?;
        let page_count: i64 = conn.query_scalar_0("PRAGMA page_count")?;
        let freelist_count: i64 = conn.query_scalar_0("PRAGMA freelist_count")?;
        let reclaimable_bytes = page_size * freelist_count;
        let db_size_bytes = page_size * page_count;

        let derived_indexes = sqlite_derived_index_health(conn)?;

        Ok((
            derived_indexes,
            quick_check_status,
            quick_check_ok,
            page_size,
            page_count,
            db_size_bytes,
            freelist_count,
            reclaimable_bytes,
        ))
    });

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok((
            derived_indexes,
            quick_check,
            quick_check_ok,
            page_size,
            page_count,
            db_size_bytes,
            freelist_count,
            reclaimable_bytes,
        )) => {
            let mut details = HashMap::from([
                ("db_path".to_string(), db_path),
                (
                    "storage_mode".to_string(),
                    format!("{:?}", storage.storage_mode()),
                ),
                ("quick_check".to_string(), quick_check.clone()),
                ("page_size".to_string(), page_size.to_string()),
                ("page_count".to_string(), page_count.to_string()),
                ("db_size_bytes".to_string(), db_size_bytes.to_string()),
                ("freelist_count".to_string(), freelist_count.to_string()),
                (
                    "reclaimable_bytes".to_string(),
                    reclaimable_bytes.to_string(),
                ),
            ]);
            if let Some(warning) = storage_mode_warning {
                details.insert("warning".to_string(), warning);
            }

            let healthy = quick_check_ok;
            Ok(HealthStatus {
                healthy,
                latency_ms,
                error: if healthy {
                    None
                } else {
                    Some(format!("quick_check failed: {quick_check}"))
                },
                details,
                derived_indexes,
            })
        }
        Err(e) => Ok(HealthStatus {
            healthy: false,
            latency_ms,
            error: Some(e.to_string()),
            details: HashMap::from([("db_path".to_string(), db_path)]),
            derived_indexes: Vec::new(),
        }),
    }
}

fn sqlite_derived_index_health(conn: &rusqlite::Connection) -> Result<Vec<DerivedIndexHealth>> {
    Ok(vec![
        sqlite_embedding_health(conn)?,
        sqlite_fts_health(conn)?,
        sqlite_graph_health(conn)?,
    ])
}

fn sqlite_embedding_health(conn: &rusqlite::Connection) -> Result<DerivedIndexHealth> {
    let queue = get_embedding_queue_health(
        conn,
        DEFAULT_STALE_PROCESSING_AFTER,
        DEFAULT_MAX_EMBEDDING_RETRIES,
    )?;

    let live_memories = count_i64(conn, "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL")?;
    let indexed = count_i64(
        conn,
        "SELECT COUNT(*) FROM embeddings e
         JOIN memories m ON m.id = e.memory_id
         WHERE m.valid_to IS NULL",
    )?;
    let flagged_without_row = count_i64(
        conn,
        "SELECT COUNT(*) FROM memories m
         LEFT JOIN embeddings e ON e.memory_id = m.id
         WHERE m.valid_to IS NULL AND m.has_embedding = 1 AND e.memory_id IS NULL",
    )?;
    let row_without_flag = count_i64(
        conn,
        "SELECT COUNT(*) FROM embeddings e
         JOIN memories m ON m.id = e.memory_id
         WHERE m.valid_to IS NULL AND m.has_embedding = 0",
    )?;
    let orphaned = count_i64(
        conn,
        "SELECT COUNT(*) FROM embeddings e
         LEFT JOIN memories m ON m.id = e.memory_id
         WHERE m.id IS NULL OR m.valid_to IS NOT NULL",
    )?;
    // Memories that have no embedding row, no flag and no job that could produce
    // one. An empty backlog must never be read as "everything is embedded". These
    // are distinct from pending/stale jobs (backlog) and from flag/row
    // mismatches (a flag with no row is `flagged_without_embedding_row`, counted
    // once there). `maintenance rebuild --embeddings` repairs both.
    let missing_unqueued = count_i64(
        conn,
        "SELECT COUNT(*) FROM memories m
         WHERE m.valid_to IS NULL AND m.has_embedding = 0
           AND NOT EXISTS (SELECT 1 FROM embeddings e WHERE e.memory_id = m.id)
           AND NOT EXISTS (SELECT 1 FROM embedding_queue q WHERE q.memory_id = m.id)",
    )?;
    // Job says complete but the embedding row it should have produced is absent.
    let complete_without_row = count_i64(
        conn,
        "SELECT COUNT(*) FROM memories m
         JOIN embedding_queue q ON q.memory_id = m.id AND q.status = 'complete'
         WHERE m.valid_to IS NULL
           AND NOT EXISTS (SELECT 1 FROM embeddings e WHERE e.memory_id = m.id)",
    )?;
    let (embedding_profile_rows, embedding_profile_bytes_total, embedding_profile_bytes_avg) = conn
        .query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(LENGTH(embedding)), 0),
                COALESCE(CAST(AVG(LENGTH(embedding)) AS INTEGER), 0)
             FROM embeddings",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )?;
    let (embedding_profile_bytes_min, embedding_profile_bytes_max) = conn.query_row(
        "SELECT
            COALESCE(MIN(LENGTH(embedding)), 0),
            COALESCE(MAX(LENGTH(embedding)), 0)
         FROM embeddings",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )?;

    let status = if queue.stale_processing > 0
        || queue.failed > 0
        || flagged_without_row > 0
        || row_without_flag > 0
        || orphaned > 0
        || missing_unqueued > 0
        || complete_without_row > 0
    {
        DerivedIndexStatus::Degraded
    } else if queue.pending > 0 || queue.processing > 0 {
        DerivedIndexStatus::Backlogged
    } else {
        DerivedIndexStatus::Healthy
    };

    let oldest_pending_age = match queue.oldest_pending_seconds {
        Some(age) => age.to_string(),
        None => "none".to_string(),
    };

    Ok(DerivedIndexHealth {
        name: "embeddings".to_string(),
        kind: DerivedIndexKind::Embedding,
        status,
        source_count: live_memories,
        indexed_count: indexed,
        pending_count: queue.pending + queue.processing,
        stale_count: queue.stale_processing,
        failed_count: queue.failed,
        orphaned_count: orphaned,
        details: HashMap::from([
            ("pending".to_string(), queue.pending.to_string()),
            ("processing".to_string(), queue.processing.to_string()),
            (
                "stale_processing".to_string(),
                queue.stale_processing.to_string(),
            ),
            ("failed".to_string(), queue.failed.to_string()),
            (
                "zero_retry_failed".to_string(),
                queue.zero_retry_failed.to_string(),
            ),
            (
                "retryable_failed".to_string(),
                queue.retryable_failed.to_string(),
            ),
            (
                "exhausted_failed".to_string(),
                queue.exhausted_failed.to_string(),
            ),
            (
                "max_retry_count".to_string(),
                queue.max_retry_count.to_string(),
            ),
            ("oldest_pending_age".to_string(), oldest_pending_age.clone()),
            ("oldest_pending_age_seconds".to_string(), oldest_pending_age),
            (
                "oldest_processing_age".to_string(),
                queue
                    .oldest_processing_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "oldest_processing_age_seconds".to_string(),
                queue
                    .oldest_processing_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "oldest_failed_age".to_string(),
                queue
                    .oldest_failed_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "oldest_failed_age_seconds".to_string(),
                queue
                    .oldest_failed_age_seconds
                    .map(|age| age.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            ("retry_count_0".to_string(), queue.retry_count_0.to_string()),
            ("retry_count_1".to_string(), queue.retry_count_1.to_string()),
            ("retry_count_2".to_string(), queue.retry_count_2.to_string()),
            (
                "retry_count_3_plus".to_string(),
                queue.retry_count_3_plus.to_string(),
            ),
            (
                "embedding_profile_rows".to_string(),
                embedding_profile_rows.to_string(),
            ),
            (
                "embedding_profile_bytes_total".to_string(),
                embedding_profile_bytes_total.to_string(),
            ),
            (
                "embedding_profile_bytes_avg".to_string(),
                embedding_profile_bytes_avg.to_string(),
            ),
            (
                "embedding_profile_bytes_min".to_string(),
                embedding_profile_bytes_min.to_string(),
            ),
            (
                "embedding_profile_bytes_max".to_string(),
                embedding_profile_bytes_max.to_string(),
            ),
            (
                "flagged_without_embedding_row".to_string(),
                flagged_without_row.to_string(),
            ),
            (
                "embedding_row_without_flag".to_string(),
                row_without_flag.to_string(),
            ),
            (
                "missing_embedding_unqueued".to_string(),
                missing_unqueued.to_string(),
            ),
            (
                "complete_job_without_embedding_row".to_string(),
                complete_without_row.to_string(),
            ),
        ]),
    })
}

fn sqlite_fts_health(conn: &rusqlite::Connection) -> Result<DerivedIndexHealth> {
    let source_count = count_i64(conn, "SELECT COUNT(*) FROM memories")?;
    let (rowid_source, rowid_column) = if sqlite_table_exists(conn, "memories_fts_docsize")? {
        ("memories_fts_docsize", "id")
    } else {
        ("memories_fts", "rowid")
    };
    let indexed_count = count_i64(conn, &format!("SELECT COUNT(*) FROM {rowid_source}"))?;
    let missing = count_i64(
        conn,
        &format!(
            "SELECT COUNT(*) FROM memories m
             WHERE m.id NOT IN (SELECT {rowid_column} FROM {rowid_source})"
        ),
    )?;
    let orphaned = count_i64(
        conn,
        &format!(
            "SELECT COUNT(*) FROM {rowid_source}
             WHERE {rowid_column} NOT IN (SELECT id FROM memories)"
        ),
    )?;
    let status = if missing > 0 || orphaned > 0 {
        DerivedIndexStatus::Degraded
    } else {
        DerivedIndexStatus::Healthy
    };

    Ok(DerivedIndexHealth {
        name: "memories_fts".to_string(),
        kind: DerivedIndexKind::FullText,
        status,
        source_count,
        indexed_count,
        pending_count: 0,
        stale_count: missing,
        failed_count: 0,
        orphaned_count: orphaned,
        details: HashMap::from([
            ("missing_rows".to_string(), missing.to_string()),
            ("drift_rows".to_string(), missing.to_string()),
        ]),
    })
}

fn sqlite_graph_health(conn: &rusqlite::Connection) -> Result<DerivedIndexHealth> {
    let source_count = count_i64(conn, "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL")?;
    let indexed_count = count_i64(
        conn,
        "SELECT COUNT(*) FROM crossrefs WHERE valid_to IS NULL",
    )?;
    let orphaned = count_i64(
        conn,
        "SELECT COUNT(*) FROM crossrefs c
         LEFT JOIN memories mf ON mf.id = c.from_id
         LEFT JOIN memories mt ON mt.id = c.to_id
         WHERE c.valid_to IS NULL
           AND (mf.id IS NULL OR mt.id IS NULL OR mf.valid_to IS NOT NULL OR mt.valid_to IS NOT NULL)",
    )?;
    let status = if orphaned > 0 {
        DerivedIndexStatus::Degraded
    } else {
        DerivedIndexStatus::Healthy
    };

    Ok(DerivedIndexHealth {
        name: "crossrefs".to_string(),
        kind: DerivedIndexKind::Graph,
        status,
        source_count,
        indexed_count,
        pending_count: 0,
        stale_count: 0,
        failed_count: 0,
        orphaned_count: orphaned,
        details: HashMap::new(),
    })
}

fn count_i64(conn: &rusqlite::Connection, sql: &str) -> Result<i64> {
    Ok(conn.query_scalar_0(sql)?)
}

fn sqlite_table_exists(conn: &rusqlite::Connection, table_name: &str) -> Result<bool> {
    Ok(conn.query_exists(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?",
        &[&table_name],
    )?)
}

#[cfg(test)]
mod tests;
