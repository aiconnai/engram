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
        conn.query_row("SELECT 1", [], |_| Ok(()))?;

        let quick_check: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
        let quick_check_ok = quick_check == "ok";
        let quick_check_status = if quick_check_ok {
            "ok".to_string()
        } else {
            quick_check
        };

        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;
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
    Ok(conn.query_row(sql, [], |row| row.get(0))?)
}

fn sqlite_table_exists(conn: &rusqlite::Connection, table_name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        [table_name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::storage::backend::StorageBackend;
    use crate::storage::backend::{DerivedIndexKind, DerivedIndexStatus};
    use crate::storage::sqlite_backend::SqliteBackend;
    use crate::types::EdgeType;
    use crate::types::{CreateMemoryInput, MemoryScope, MemoryTier, MemoryType};
    use rusqlite::params;

    fn test_memory_input(content: &str) -> CreateMemoryInput {
        CreateMemoryInput {
            content: content.to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["health".to_string()],
            metadata: HashMap::new(),
            importance: Some(0.5),
            scope: MemoryScope::Global,
            workspace: Some("default".to_string()),
            tier: MemoryTier::Permanent,
            defer_embedding: true,
            ttl_seconds: None,
            dedup_mode: crate::types::DedupMode::Allow,
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        }
    }

    #[test]
    fn test_create_in_memory() {
        let backend = SqliteBackend::in_memory().unwrap();
        assert_eq!(backend.backend_name(), "sqlite");
    }

    #[test]
    fn test_health_check() {
        let backend = SqliteBackend::in_memory().unwrap();
        let health = backend.health_check().unwrap();
        assert!(health.healthy, "health check failed: {:?}", health.error);
        assert!(health.latency_ms >= 0.0);
    }

    #[test]
    fn test_health_check_reports_derived_index_contract() {
        let backend = SqliteBackend::in_memory().unwrap();
        backend
            .create_memory(CreateMemoryInput {
                content: "contract health memory".to_string(),
                memory_type: MemoryType::Note,
                tags: vec!["health".to_string()],
                metadata: HashMap::new(),
                importance: Some(0.5),
                scope: MemoryScope::Global,
                workspace: Some("default".to_string()),
                tier: MemoryTier::Permanent,
                defer_embedding: false,
                ttl_seconds: None,
                dedup_mode: crate::types::DedupMode::Allow,
                dedup_threshold: None,
                event_time: None,
                event_duration_seconds: None,
                trigger_pattern: None,
                summary_of_id: None,
                media_url: None,
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        assert!(health.healthy, "health check failed: {:?}", health.error);

        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embeddings health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Backlogged);
        assert_eq!(embeddings.pending_count, 1);

        let fts = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "memories_fts")
            .expect("fts health");
        assert_eq!(fts.kind, DerivedIndexKind::FullText);
        assert_eq!(fts.status, DerivedIndexStatus::Healthy);

        let graph = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "crossrefs")
            .expect("graph health");
        assert_eq!(graph.kind, DerivedIndexKind::Graph);
        assert_eq!(graph.status, DerivedIndexStatus::Healthy);
    }

    #[test]
    fn test_health_check_reports_fts_degraded_when_rows_missing() {
        let backend = SqliteBackend::in_memory().unwrap();
        backend
            .create_memory(test_memory_input("fts-1 missing row"))
            .unwrap();
        backend
            .create_memory(test_memory_input("fts-2 missing row"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                // Remove all indexed rows to make FTS source-index drift visible.
                conn.execute(
                    "INSERT INTO memories_fts(memories_fts) VALUES('delete-all')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let fts = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "memories_fts")
            .expect("fts health");
        assert_eq!(fts.kind, DerivedIndexKind::FullText);
        assert_eq!(fts.status, DerivedIndexStatus::Degraded);
        assert_eq!(fts.stale_count, 2);
    }

    #[test]
    fn test_health_check_reports_graph_degraded_for_orphaned_crossrefs() {
        let backend = SqliteBackend::in_memory().unwrap();
        let source = backend
            .create_memory(test_memory_input("crossref source"))
            .unwrap();
        let target = backend
            .create_memory(test_memory_input("crossref target"))
            .unwrap();

        backend
            .create_crossref(source.id, target.id, EdgeType::RelatedTo, 0.8)
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "UPDATE memories SET valid_to = ? WHERE id = ?",
                    params![chrono::Utc::now().to_rfc3339(), source.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let graph = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "crossrefs")
            .expect("graph health");
        assert_eq!(graph.kind, DerivedIndexKind::Graph);
        assert_eq!(graph.status, DerivedIndexStatus::Degraded);
        assert_eq!(graph.orphaned_count, 1);
    }

    #[test]
    fn test_health_check_reports_embedding_degraded_for_failed_queue_rows() {
        let backend = SqliteBackend::in_memory().unwrap();
        let memory = backend
            .create_memory(test_memory_input("failed queue row"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO embedding_queue (memory_id, status, queued_at, retry_count)
                     VALUES (?, 'failed', datetime('now'), 0)",
                    params![memory.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.failed_count, 1);
    }

    #[test]
    fn test_health_check_reports_embedding_degraded_for_stale_queue_rows() {
        let backend = SqliteBackend::in_memory().unwrap();
        let memory = backend
            .create_memory(test_memory_input("stale queue row"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO embedding_queue (memory_id, status, queued_at, started_at, retry_count)
                     VALUES (?, 'processing', datetime('now'), datetime('now','-1 hour'), 0)",
                    params![memory.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.stale_count, 1);
    }

    #[test]
    fn test_health_check_embedding_details_include_queue_state_counters() {
        let backend = SqliteBackend::in_memory().unwrap();
        let pending = backend
            .create_memory(test_memory_input("state counter pending"))
            .unwrap();
        let processing = backend
            .create_memory(test_memory_input("state counter processing"))
            .unwrap();
        let retryable_failed = backend
            .create_memory(test_memory_input("state counter retryable failed"))
            .unwrap();
        let exhausted_failed = backend
            .create_memory(test_memory_input("state counter exhausted failed"))
            .unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        backend
            .storage()
            .with_connection(|conn| {
                let stale_started = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
                let old_pending = (chrono::Utc::now() - chrono::Duration::minutes(15)).to_rfc3339();

                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at)
                     VALUES (?, 'pending', ?)",
                    params![pending.id, old_pending],
                )?;
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at, started_at, retry_count)
                     VALUES (?, 'processing', ?, ?, 0)",
                    params![processing.id, now, stale_started],
                )?;
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at, retry_count)
                     VALUES (?, 'failed', ?, 1)",
                    params![retryable_failed.id, now],
                )?;
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at, retry_count)
                     VALUES (?, 'failed', ?, 4)",
                    params![exhausted_failed.id, now],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");

        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.details["pending"], "1");
        assert_eq!(embeddings.details["processing"], "1");
        assert_eq!(embeddings.details["stale_processing"], "1");
        assert_eq!(embeddings.details["failed"], "2");
        assert_eq!(embeddings.details["retryable_failed"], "1");
        assert_eq!(embeddings.details["exhausted_failed"], "1");
        assert_eq!(embeddings.details["max_retry_count"], "4");
        assert_ne!(embeddings.details["oldest_pending_age"], "none");
        assert_ne!(embeddings.details["oldest_pending_age_seconds"], "none");
    }

    #[test]
    fn test_health_check_reports_embedding_degraded_for_flag_mismatch() {
        let backend = SqliteBackend::in_memory().unwrap();
        let memory = backend
            .create_memory(test_memory_input("flag mismatch"))
            .unwrap();

        backend
            .storage()
            .with_connection(|conn| {
                // Mark as embedded without an embeddings row.
                conn.execute(
                    "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                    params![memory.id],
                )?;
                Ok(())
            })
            .unwrap();

        let health = backend.health_check().unwrap();
        let embeddings = health
            .derived_indexes
            .iter()
            .find(|index| index.name == "embeddings")
            .expect("embedding health");
        assert_eq!(embeddings.kind, DerivedIndexKind::Embedding);
        assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
        assert_eq!(embeddings.pending_count, 0);
        assert_eq!(embeddings.indexed_count, 0);
        assert_eq!(embeddings.stale_count, 0);
        assert_eq!(embeddings.orphaned_count, 0);
    }
}
