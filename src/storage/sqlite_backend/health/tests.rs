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
fn embeddings_health(backend: &SqliteBackend) -> crate::storage::DerivedIndexHealth {
    backend
        .health_check()
        .unwrap()
        .derived_indexes
        .into_iter()
        .find(|index| index.name == "embeddings")
        .expect("embedding health")
}

#[test]
fn test_health_check_embedding_not_healthy_when_memory_has_no_embedding_and_no_job() {
    // An empty backlog must not be read as "all embedded": this memory has no
    // embedding row, no flag and no job that would ever produce one.
    let backend = SqliteBackend::in_memory().unwrap();
    backend
        .create_memory(test_memory_input("never embedded, never queued"))
        .unwrap();

    let embeddings = embeddings_health(&backend);
    assert_eq!(embeddings.pending_count, 0);
    assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
    assert_eq!(embeddings.details["missing_embedding_unqueued"], "1");
    assert_eq!(
        embeddings.details["complete_job_without_embedding_row"],
        "0"
    );
}

#[test]
fn test_health_check_embedding_not_healthy_when_complete_job_has_no_row() {
    let backend = SqliteBackend::in_memory().unwrap();
    let memory = backend
        .create_memory(test_memory_input("complete job without row"))
        .unwrap();
    backend
        .storage()
        .with_connection(|conn| {
            conn.execute(
                "INSERT INTO embedding_queue (memory_id, status, queued_at, completed_at)
                 VALUES (?, 'complete', datetime('now'), datetime('now'))",
                params![memory.id],
            )?;
            Ok(())
        })
        .unwrap();

    let embeddings = embeddings_health(&backend);
    assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
    assert_eq!(
        embeddings.details["complete_job_without_embedding_row"],
        "1"
    );
    assert_eq!(embeddings.details["missing_embedding_unqueued"], "0");
}

#[test]
fn test_health_check_embedding_diagnostics_are_distinct() {
    let backend = SqliteBackend::in_memory().unwrap();
    let flagged = backend
        .create_memory(test_memory_input("flag without row"))
        .unwrap();
    let unflagged = backend
        .create_memory(test_memory_input("row without flag"))
        .unwrap();
    let superseded = backend
        .create_memory(test_memory_input("orphan row for a superseded memory"))
        .unwrap();
    let healthy = backend
        .create_memory(test_memory_input("coherent embedding"))
        .unwrap();

    backend
        .storage()
        .with_connection(|conn| {
            let insert_row = |id: i64| {
                conn.execute(
                    "INSERT INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                     VALUES (?, zeroblob(8), 'test', 2, datetime('now'))",
                    params![id],
                )
            };
            conn.execute(
                "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                params![flagged.id],
            )?;
            insert_row(unflagged.id)?;
            insert_row(superseded.id)?;
            conn.execute(
                "UPDATE memories SET has_embedding = 1, valid_to = datetime('now') WHERE id = ?",
                params![superseded.id],
            )?;
            insert_row(healthy.id)?;
            conn.execute(
                "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                params![healthy.id],
            )?;
            Ok(())
        })
        .unwrap();

    let embeddings = embeddings_health(&backend);
    assert_eq!(embeddings.status, DerivedIndexStatus::Degraded);
    assert_eq!(embeddings.details["flagged_without_embedding_row"], "1");
    assert_eq!(embeddings.details["embedding_row_without_flag"], "1");
    assert_eq!(embeddings.orphaned_count, 1);
    // A flag with no row is counted once, as `flagged_without_embedding_row`,
    // not again as missing-and-unqueued.
    assert_eq!(embeddings.details["missing_embedding_unqueued"], "0");
}
