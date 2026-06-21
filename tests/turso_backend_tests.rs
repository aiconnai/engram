//! Turso backend integration tests.
//!
//! These tests run in a separate process to avoid the libsql/rusqlite
//! SQLite initialization conflict that occurs under --all-features.
//!
//! Run with: cargo test --test turso_backend_tests --features turso

#![cfg(feature = "turso")]

use std::collections::HashMap;

use engram::error::EngramError;
use engram::storage::{
    CloudSyncBackend, DerivedIndexKind, DerivedIndexStatus, StorageBackend, TransactionalBackend,
    TursoBackend,
};
use engram::types::*;

#[tokio::test]
async fn test_turso_in_memory() {
    let backend = TursoBackend::in_memory().await.unwrap();
    assert_eq!(backend.backend_name(), "turso");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_turso_health_check() {
    let backend = TursoBackend::in_memory().await.unwrap();
    let health = backend.health_check().unwrap();
    assert!(health.healthy);
    assert!(!health.derived_indexes.is_empty());

    let memories_index = health
        .derived_indexes
        .iter()
        .find(|index| index.name == "memories")
        .unwrap();
    assert_eq!(memories_index.kind, DerivedIndexKind::External);
    assert_eq!(memories_index.source_count, 0);
    assert_eq!(memories_index.indexed_count, 0);
    assert_eq!(memories_index.pending_count, 0);
    assert!(matches!(
        memories_index.status,
        DerivedIndexStatus::Healthy | DerivedIndexStatus::Unavailable
    ));
    assert_eq!(
        memories_index
            .details
            .get("index")
            .expect("detail key should include index name"),
        "memories"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_turso_crud() {
    let backend = TursoBackend::in_memory().await.unwrap();

    // Create
    let input = CreateMemoryInput {
        content: "Test memory for Turso".to_string(),
        memory_type: MemoryType::Note,
        tags: vec!["test".to_string()],
        metadata: HashMap::new(),
        importance: Some(0.7),
        scope: MemoryScope::Global,
        workspace: Some("test".to_string()),
        tier: MemoryTier::Permanent,
        defer_embedding: true,
        ttl_seconds: None,
        dedup_mode: engram::types::DedupMode::Allow,
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    let memory = backend.create_memory(input).unwrap();
    assert_eq!(memory.content, "Test memory for Turso");

    // Read
    let retrieved = backend.get_memory(memory.id).unwrap();
    assert!(retrieved.is_some());

    // Update
    let update = UpdateMemoryInput {
        content: Some("Updated Turso memory".to_string()),
        memory_type: None,
        tags: None,
        metadata: None,
        importance: None,
        scope: None,
        ttl_seconds: None,
        event_time: None,
        trigger_pattern: None,
        media_url: None,
    };
    let updated = backend.update_memory(memory.id, update).unwrap();
    assert_eq!(updated.content, "Updated Turso memory");

    // Delete
    backend.delete_memory(memory.id).unwrap();
    let deleted = backend.get_memory(memory.id).unwrap();
    assert!(deleted.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_turso_with_transaction_reports_unsupported_wrapper_when_called() {
    let backend = TursoBackend::in_memory().await.unwrap();
    let mut called = false;

    let result = backend.with_transaction(|_| -> Result<(), EngramError> {
        called = true;
        Ok(())
    });

    assert!(!called);
    assert!(
        matches!(result, Err(EngramError::Storage(message)) if message.contains("transaction-scoped StorageBackend"))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_turso_savepoint_rejects_invalid_names_before_sql() {
    let backend = TursoBackend::in_memory().await.unwrap();

    for name in ["", "1bad", "bad-name", "bad name", "bad;DROP"] {
        let savepoint = backend.savepoint(name);
        let release = backend.release_savepoint(name);
        let rollback = backend.rollback_to_savepoint(name);

        assert!(
            matches!(savepoint, Err(EngramError::InvalidInput(message)) if message.contains("savepoint name"))
        );
        assert!(
            matches!(release, Err(EngramError::InvalidInput(message)) if message.contains("savepoint name"))
        );
        assert!(
            matches!(rollback, Err(EngramError::InvalidInput(message)) if message.contains("savepoint name"))
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_turso_sync_delta_and_sync_state_report_unsupported_extensions() {
    let backend = TursoBackend::in_memory().await.unwrap();

    let delta = backend.sync_delta(0);
    let state = backend.sync_state();

    assert!(matches!(delta, Err(EngramError::Sync(message)) if message.contains("Turso backend")));
    assert!(matches!(state, Err(EngramError::Sync(message)) if message.contains("Turso backend")));
}
