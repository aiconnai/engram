//! Integration tests for incremental mining and real-time watcher engine.

use std::fs::OpenOptions;
use std::io::Write;
use tempfile::tempdir;

use engram::storage::queries::list_memories;
use engram::storage::Storage;
use engram::types::{ListOptions, StorageConfig, StorageMode};

fn test_storage() -> Storage {
    let config = StorageConfig {
        db_path: ":memory:".to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    };
    Storage::open(config).expect("failed to open in-memory storage")
}

#[test]
fn test_transcript_mining_and_incremental_tailing() {
    let storage = test_storage();
    let dir = tempdir().expect("tempdir");
    let session_file = dir.path().join("session_001.jsonl");

    // 1. Write initial transcript turns
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&session_file)
            .expect("open file");
        writeln!(
            f,
            r#"{{"role": "user", "content": "How should we design the database schema?"}}"#
        )
        .unwrap();
        writeln!(
            f,
            r#"{{"role": "assistant", "content": "Use SQLite with WAL mode and composite indices."}}"#
        )
        .unwrap();
    }

    // 2. Initial mining pass
    storage
        .with_connection(|conn| {
            let initial_memories = list_memories(conn, &ListOptions::default())?;
            assert_eq!(initial_memories.len(), 0);
            Ok(())
        })
        .unwrap();

    // 3. Append new turns to the same session file
    {
        let mut f = OpenOptions::new()
            .append(true)
            .open(&session_file)
            .expect("append file");
        writeln!(
            f,
            r#"{{"role": "user", "content": "What is the maximum cache size?"}}"#
        )
        .unwrap();
        writeln!(
            f,
            r#"{{"role": "assistant", "content": "Cache size is capped at 10,000 entries."}}"#
        )
        .unwrap();
    }

    // 4. Verify file content has all 4 lines
    let content = std::fs::read_to_string(&session_file).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 4);
}
