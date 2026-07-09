use super::*;

#[test]
fn test_dedup_mode_reject() {
    use crate::types::DedupMode;

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create first memory
            let _memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Unique content for testing".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow, // First one allows
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Try to create duplicate with reject mode
            let result = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Unique content for testing".to_string(), // Same content
                    memory_type: MemoryType::Note,
                    tags: vec!["new-tag".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Reject,
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            );

            // Should fail with Duplicate error
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, crate::error::EngramError::Duplicate { .. }));

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_dedup_mode_skip() {
    use crate::types::DedupMode;

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create first memory
            let memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Skip test content".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["original".to_string()],
                    metadata: HashMap::new(),
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

            // Try to create duplicate with skip mode
            let memory2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Skip test content".to_string(), // Same content
                    memory_type: MemoryType::Note,
                    tags: vec!["new-tag".to_string()], // Different tags
                    metadata: HashMap::new(),
                    importance: Some(0.9), // Different importance
                    scope: Default::default(),
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Skip,
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Should return existing memory unchanged
            assert_eq!(memory1.id, memory2.id);
            assert_eq!(memory2.tags, vec!["original".to_string()]); // Original tags
            assert!((memory2.importance - 0.5).abs() < 0.01); // Original importance

            // Only one memory should exist
            let all = list_memories(conn, &ListOptions::default())?;
            assert_eq!(all.len(), 1);

            Ok(())
        })
        .unwrap();
}
