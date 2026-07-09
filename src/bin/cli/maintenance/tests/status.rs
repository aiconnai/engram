use rusqlite::params;

use super::support::{create_test_memory, table_counts, test_storage};
use crate::maintenance::status::{maintenance_status, write_maintenance_status};
use engram::storage::Storage;
use engram::types::{StorageConfig, StorageMode};

#[test]
fn maintenance_status_matches_storage_health_shape() {
    let (_dir, storage) = test_storage();

    let status = maintenance_status(&storage).expect("status should be collected");
    let json = serde_json::to_value(status).expect("status should serialize");

    assert!(json["healthy"].is_boolean());
    assert!(json["latency_ms"].is_number());
    assert!(json["error"].is_null() || json["error"].is_string());
    assert!(json["details"]["db_path"]
        .as_str()
        .expect("db_path should be a string")
        .ends_with("memories.db"));
    if let Some(storage_mode) = json["details"]["storage_mode"].as_str() {
        assert_eq!(storage_mode, "Local");
    }
    assert_eq!(json["details"]["quick_check"].as_str(), Some("ok"));
    assert!(json["details"]["page_size"].is_string());
    assert!(json["details"]["page_count"].is_string());
    assert!(json["details"]["db_size_bytes"].is_string());
    assert!(json["details"]["freelist_count"].is_string());
    assert!(json["details"]["reclaimable_bytes"].is_string());
    assert!(json["derived_indexes"].is_array());
    assert_eq!(json["stats"]["total_memories"], 0);
    assert!(json["stats"]["schema_version"].is_number());

    let embedding_index = json["derived_indexes"]
        .as_array()
        .and_then(|indexes| {
            indexes
                .iter()
                .find(|index| index["name"].as_str() == Some("embeddings"))
        })
        .expect("embedding derived index should be present in health payload");

    let details = embedding_index["details"]
        .as_object()
        .expect("details should be object");
    for key in [
        "pending",
        "processing",
        "stale_processing",
        "failed",
        "zero_retry_failed",
        "retryable_failed",
        "exhausted_failed",
        "max_retry_count",
        "oldest_pending_age",
        "oldest_pending_age_seconds",
        "oldest_processing_age",
        "oldest_failed_age",
        "retry_count_0",
        "retry_count_1",
        "retry_count_2",
        "retry_count_3_plus",
        "embedding_profile_rows",
        "embedding_profile_bytes_total",
        "embedding_profile_bytes_avg",
        "embedding_profile_bytes_min",
        "embedding_profile_bytes_max",
    ] {
        assert!(
            details.contains_key(key),
            "details missing queue state key: {key}"
        );
    }
}

#[test]
fn maintenance_status_is_read_only_for_storage_tables() {
    let (_dir, storage) = test_storage();

    let before = storage
        .with_connection(table_counts)
        .expect("initial counts should be readable");
    let _ = maintenance_status(&storage).expect("status should be collected");
    let after = storage
        .with_connection(table_counts)
        .expect("final counts should be readable");

    assert_eq!(before, after);
}

#[test]
fn maintenance_status_includes_sqlite_health_contract() {
    let (_dir, storage) = test_storage();
    let status = maintenance_status(&storage).expect("status should be collected");
    let mut output = Vec::new();

    write_maintenance_status(&mut output, &status).expect("status should render");
    let text = String::from_utf8(output).expect("output should be utf8");

    assert!(text.contains("PRAGMA quick_check: ok"));
    assert!(text.contains("Database pages:"));
    assert!(text.contains("embedding profile: rows="));
    assert!(text.contains("drift:"));
}

#[test]
fn maintenance_status_reports_warning_for_cloud_path() {
    let dir = tempfile::tempdir().expect("temporary directory should be created");
    let db_path = dir
        .path()
        .join("my_dropbox_backup")
        .join("memories.db")
        .to_string_lossy()
        .to_string();
    let config = StorageConfig {
        db_path,
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    };
    let storage = Storage::open(config).expect("file storage should open");
    let status = maintenance_status(&storage).expect("status should be collected");

    let warning = status
        .health
        .details
        .get("warning")
        .expect("warning should be present for cloud-like path");
    assert!(warning.contains("WAL mode"));
}

#[test]
fn maintenance_status_human_output_includes_derived_indexes() {
    let (_dir, storage) = test_storage();
    let status = maintenance_status(&storage).expect("status should be collected");
    let mut output = Vec::new();

    write_maintenance_status(&mut output, &status).expect("status should render");
    let text = String::from_utf8(output).expect("output should be utf8");

    assert!(text.contains("Derived indexes:"));
    assert!(text.contains("embeddings (embedding):"));
    assert!(text.contains("memories_fts (full_text):"));
    assert!(text.contains("crossrefs (graph):"));
    assert!(text.contains("source="));
    assert!(text.contains("indexed="));
    assert!(text.contains("orphaned="));
}

#[test]
fn maintenance_status_human_output_includes_embedding_queue_state_counters() {
    let (_dir, storage) = test_storage();
    let stale_time = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
    let old_pending_time = (chrono::Utc::now() - chrono::Duration::minutes(20)).to_rfc3339();

    storage
        .with_connection(|conn| {
            let pending = create_test_memory(conn, "state counter pending")?;
            let processing = create_test_memory(conn, "state counter processing")?;
            let retryable_failed = create_test_memory(conn, "state counter retryable")?;
            let exhausted_failed = create_test_memory(conn, "state counter exhausted")?;
            let zero_retry_failed = create_test_memory(conn, "state counter zero retry")?;

            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 0 WHERE memory_id = ?",
                params![stale_time, processing.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET queued_at = ?, status = 'pending' WHERE memory_id = ?",
                params![old_pending_time, pending.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                params![retryable_failed.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 4 WHERE memory_id = ?",
                params![exhausted_failed.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 0 WHERE memory_id = ?",
                params![zero_retry_failed.id],
            )?;
            Ok(())
        })
        .unwrap();

    let status = maintenance_status(&storage).expect("status should be collected");
    let mut output = Vec::new();

    write_maintenance_status(&mut output, &status).expect("status should render");
    let text = String::from_utf8(output).expect("output should be utf8");

    assert!(text.contains(
        "queue-state: pending=1 processing=1 stale_processing=1 failed=3 zero_retry_failed=1 retryable_failed=2 exhausted_failed=1 max_retry_count=4 oldest_pending_age="
    ));
    assert!(text.contains("retry_count_0=1"));
    assert!(text.contains("retry_count_1=1"));
    assert!(text.contains("retry_count_2=0"));
    assert!(text.contains("retry_count_3+=1"));
}
