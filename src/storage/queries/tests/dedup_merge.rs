use super::*;

#[test]
fn test_dedup_mode_merge() {
    use crate::types::DedupMode;

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create first memory with some tags and metadata
            let memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Merge test content".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["tag1".to_string(), "tag2".to_string()],
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("key1".to_string(), serde_json::json!("value1"));
                        m
                    },
                    importance: Some(0.5),
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Try to create duplicate with merge mode
            let memory2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Merge test content".to_string(), // Same content
                    memory_type: MemoryType::Note,
                    tags: vec!["tag2".to_string(), "tag3".to_string()], // Overlapping + new
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("key2".to_string(), serde_json::json!("value2"));
                        m
                    },
                    importance: Some(0.8), // Higher importance
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Merge,
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Should return same memory ID
            assert_eq!(memory1.id, memory2.id);

            // Tags should be merged (no duplicates)
            assert!(memory2.tags.contains(&"tag1".to_string()));
            assert!(memory2.tags.contains(&"tag2".to_string()));
            assert!(memory2.tags.contains(&"tag3".to_string()));
            assert_eq!(memory2.tags.len(), 3);

            // Metadata should be merged
            assert!(memory2.metadata.contains_key("key1"));
            assert!(memory2.metadata.contains_key("key2"));

            // Only one memory should exist
            let all = list_memories(conn, &ListOptions::default())?;
            assert_eq!(all.len(), 1);

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_dedup_mode_allow() {
    use crate::types::DedupMode;

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create first memory
            let memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Allow duplicates content".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Create duplicate with allow mode (default)
            let memory2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Allow duplicates content".to_string(), // Same content
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Should create separate memory
            assert_ne!(memory1.id, memory2.id);

            // Both memories should exist
            let all = list_memories(conn, &ListOptions::default())?;
            assert_eq!(all.len(), 2);

            // Both should have same content hash
            assert_eq!(memory1.content_hash, memory2.content_hash);

            Ok(())
        })
        .unwrap();
}
