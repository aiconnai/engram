use super::*;

#[test]
fn test_find_duplicates_exact_hash() {
    use crate::types::DedupMode;

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create two memories with same content (exact hash duplicates)
            let _memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Duplicate content".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["first".to_string()],
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

            let _memory2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Duplicate content".to_string(), // Same content
                    memory_type: MemoryType::Note,
                    tags: vec!["second".to_string()],
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

            // Create a unique memory (not a duplicate)
            let _memory3 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Unique content".to_string(),
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

            // Find duplicates
            let duplicates = find_duplicates(conn, 0.9)?;

            // Should find one duplicate pair
            assert_eq!(duplicates.len(), 1);

            // Should be exact hash match
            assert_eq!(duplicates[0].match_type, DuplicateMatchType::ExactHash);
            assert!((duplicates[0].similarity_score - 1.0).abs() < 0.01);

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_dedup_scope_isolation() {
    use crate::types::{DedupMode, MemoryScope};

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create memory in user-1 scope
            let _user1_memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Shared content".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["user1".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::user("user-1"),
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

            // Create same content in user-2 scope with Reject mode
            // Should succeed because scopes are different
            let user2_result = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Shared content".to_string(), // Same content!
                    memory_type: MemoryType::Note,
                    tags: vec!["user2".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::user("user-2"), // Different scope
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Reject, // Should not reject - different scope
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            );

            // Should succeed - different scopes are not considered duplicates
            assert!(user2_result.is_ok());
            let _user2_memory = user2_result.unwrap();

            // Now try to create duplicate in same scope (user-2)
            let duplicate_result = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Shared content".to_string(), // Same content
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::user("user-2"), // Same scope as user2_memory
                    workspace: None,
                    tier: Default::default(),
                    defer_embedding: true,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Reject, // Should reject - same scope
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            );

            // Should fail - same scope with same content
            assert!(duplicate_result.is_err());
            assert!(matches!(
                duplicate_result.unwrap_err(),
                crate::error::EngramError::Duplicate { .. }
            ));

            // Verify we have exactly 2 memories (one per user)
            let all = list_memories(conn, &ListOptions::default())?;
            assert_eq!(all.len(), 2);

            Ok(())
        })
        .unwrap();
}
