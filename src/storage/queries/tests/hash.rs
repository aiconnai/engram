use super::*;

#[test]
fn test_content_hash_computation() {
    // Test that content hash is consistent and normalized
    let hash1 = compute_content_hash("Hello World");
    let hash2 = compute_content_hash("hello world"); // Different case
    let hash3 = compute_content_hash("  hello   world  "); // Extra whitespace
    let hash4 = compute_content_hash("Hello World!"); // Different content
    let raw_hash1 = compute_content_hash_raw("Hello World");
    let raw_hash2 = compute_content_hash_raw("hello world");

    // Same normalized content should produce same hash
    assert_eq!(hash1, hash2, "case change should be ignored");
    assert_eq!(hash2, hash3, "whitespace changes should be ignored");

    // Different content should produce different hash
    assert_ne!(hash1, hash4, "different content → different hash");
    assert_ne!(
        raw_hash1, raw_hash2,
        "raw hash should keep byte-level casing"
    );

    // Dedup helper remains compatible with compute_content_hash
    assert_eq!(hash1, compute_dedup_hash("Hello World"));

    // Hash should be prefixed with algorithm
    assert!(hash1.starts_with("sha256:"));
}

#[test]
fn test_content_hash_stored_on_create() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Test content for hash".to_string(),
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
            )?;

            // Content hash should be set
            assert!(memory.content_hash.is_some());
            let hash = memory.content_hash.as_ref().unwrap();
            assert!(hash.starts_with("sha256:"));

            // Fetch from DB and verify hash is persisted
            let fetched = get_memory(conn, memory.id)?;
            assert_eq!(fetched.content_hash, memory.content_hash);

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_update_memory_recalculates_hash() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create a memory
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Original content".to_string(),
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
            )?;

            let original_hash = memory.content_hash.clone();

            // Update the content
            let updated = update_memory(
                conn,
                memory.id,
                &UpdateMemoryInput {
                    content: Some("Updated content".to_string()),
                    memory_type: None,
                    tags: None,
                    metadata: None,
                    importance: None,
                    scope: None,
                    ttl_seconds: None,
                    event_time: None,
                    trigger_pattern: None,
                    media_url: None,
                },
            )?;

            // Hash should be different
            assert_ne!(updated.content_hash, original_hash);
            assert!(updated.content_hash.is_some());

            // Verify against expected dedup hash
            let expected_hash = compute_dedup_hash("Updated content");
            assert_eq!(updated.content_hash.as_ref().unwrap(), &expected_hash);

            Ok(())
        })
        .unwrap();
}
