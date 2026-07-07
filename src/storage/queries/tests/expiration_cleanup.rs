use super::*;

#[test]
fn test_set_memory_expiration() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create a permanent memory
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Initially permanent".to_string(),
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

            assert!(memory.expires_at.is_none());

            // Set expiration to 30 minutes
            let updated = set_memory_expiration(conn, memory.id, Some(1800))?;
            assert!(updated.expires_at.is_some());

            // Remove expiration (make permanent again) - use Some(0) to clear
            let permanent_again = set_memory_expiration(conn, memory.id, Some(0))?;
            assert!(permanent_again.expires_at.is_none());

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_cleanup_expired_memories() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create 3 daily memories that we'll expire manually
            let mut expired_ids = vec![];
            for i in 0..3 {
                let mem = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: format!("To expire {}", i),
                        memory_type: MemoryType::Note,
                        tags: vec![],
                        metadata: HashMap::new(),
                        importance: None,
                        scope: Default::default(),
                        workspace: None,
                        tier: MemoryTier::Daily, // Daily tier for expiring memories
                        defer_embedding: true,
                        ttl_seconds: Some(3600), // 1 hour TTL
                        dedup_mode: Default::default(),
                        dedup_threshold: None,
                        event_time: None,
                        event_duration_seconds: None,
                        trigger_pattern: None,
                        summary_of_id: None,
                        media_url: None,
                    },
                )?;
                expired_ids.push(mem.id);
            }

            // Create 2 active memories (permanent)
            for i in 0..2 {
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: format!("Active {}", i),
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
            }

            // All 5 should be visible initially
            let results = list_memories(conn, &ListOptions::default())?;
            assert_eq!(results.len(), 5);

            // Manually expire the first 3 memories
            let past = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
            for id in &expired_ids {
                conn.execute(
                    "UPDATE memories SET expires_at = ? WHERE id = ?",
                    params![past, id],
                )?;
            }

            // Count expired
            let expired_count = count_expired_memories(conn)?;
            assert_eq!(expired_count, 3);

            // Cleanup should delete 3
            let deleted = cleanup_expired_memories(conn)?;
            assert_eq!(deleted, 3);

            // Verify only 2 remain
            let remaining = list_memories(conn, &ListOptions::default())?;
            assert_eq!(remaining.len(), 2);

            // No more expired
            let expired_count = count_expired_memories(conn)?;
            assert_eq!(expired_count, 0);

            Ok(())
        })
        .unwrap();
}
