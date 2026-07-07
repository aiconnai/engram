use super::*;

pub(super) fn test_memory_input(content: &str) -> CreateMemoryInput {
    CreateMemoryInput {
        content: content.to_string(),
        memory_type: MemoryType::Note,
        tags: vec![],
        metadata: HashMap::new(),
        importance: None,
        scope: Default::default(),
        workspace: None,
        tier: Default::default(),
        defer_embedding: true,
        ttl_seconds: None,
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    }
}

pub(super) fn open_test_storage() -> crate::storage::Storage {
    crate::storage::Storage::open_in_memory().expect("in-memory storage")
}

pub(super) fn make_memory(conn: &Connection) -> i64 {
    create_memory(
        conn,
        &CreateMemoryInput {
            content: "test memory".to_string(),
            memory_type: MemoryType::Note,
            tags: vec![],
            metadata: HashMap::new(),
            importance: None,
            scope: Default::default(),
            workspace: None,
            tier: Default::default(),
            defer_embedding: true,
            ttl_seconds: None,
            dedup_mode: Default::default(),
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        },
    )
    .unwrap()
    .id
}

pub(super) fn link_supersedes(conn: &Connection, from_id: i64, to_id: i64) {
    conn.execute(
            "INSERT INTO crossrefs (from_id, to_id, edge_type, score, strength, source, created_at, valid_from) \
             VALUES (?1, ?2, 'supersedes', 1.0, 1.0, 'test', datetime('now'), datetime('now'))",
            params![from_id, to_id],
        )
        .unwrap();
}
