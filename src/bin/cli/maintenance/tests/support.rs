use std::collections::HashMap;

use engram::error::Result;
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::*;
use rusqlite::Connection;
use tempfile::TempDir;

pub(super) fn memory_input(content: &str) -> CreateMemoryInput {
    CreateMemoryInput {
        content: content.to_string(),
        memory_type: MemoryType::Note,
        tags: vec![],
        metadata: HashMap::new(),
        importance: None,
        scope: Default::default(),
        workspace: None,
        tier: Default::default(),
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: DedupMode::Allow,
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    }
}

pub(super) fn test_storage() -> (TempDir, Storage) {
    let dir = tempfile::tempdir().expect("temporary directory should be created");
    let db_path = dir.path().join("memories.db").to_string_lossy().to_string();
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
    (dir, storage)
}

pub(super) fn table_counts(conn: &Connection) -> Result<(i64, i64, i64)> {
    Ok((
        conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?,
        conn.query_row("SELECT COUNT(*) FROM memory_versions", [], |row| row.get(0))?,
        conn.query_row("SELECT COUNT(*) FROM embedding_queue", [], |row| row.get(0))?,
    ))
}

pub(super) fn create_test_memory(
    conn: &Connection,
    content: &str,
) -> Result<engram::types::Memory> {
    create_memory(conn, &memory_input(content))
}
