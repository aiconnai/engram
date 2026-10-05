use super::*;

#[test]
fn test_memory_ttl_creation() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create daily memory with TTL of 1 hour
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Temporary memory".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    workspace: None,
                    tier: MemoryTier::Daily, // Daily tier for expiring memories
                    defer_embedding: true,
                    ttl_seconds: Some(3600), // 1 hour
                    dedup_mode: Default::default(),
                    dedup_threshold: None,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                },
            )?;

            // Verify expires_at is set and tier is daily
            assert!(memory.expires_at.is_some());
            assert_eq!(memory.tier, MemoryTier::Daily);
            let expires_at = memory.expires_at.unwrap();
            let now = Utc::now();

            // Should expire approximately 1 hour from now (within 5 seconds tolerance)
            let diff = (expires_at - now).num_seconds();
            assert!(
                (3595..=3605).contains(&diff),
                "Expected ~3600 seconds, got {}",
                diff
            );

            // Create memory without TTL
            let permanent = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Permanent memory".to_string(),
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

            // Verify expires_at is None for permanent memory
            assert!(permanent.expires_at.is_none());

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_expired_memories_excluded_from_queries() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Create a daily memory with TTL (will expire)
            let memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Memory to expire".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["test".to_string()],
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

            // Create a permanent memory
            let active = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Active memory".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["test".to_string()],
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

            // Both should be visible initially
            let results = list_memories(conn, &ListOptions::default())?;
            assert_eq!(results.len(), 2);

            // Manually expire memory1 by setting expires_at to the past
            let past = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
            conn.execute(
                "UPDATE memories SET expires_at = ? WHERE id = ?",
                params![past, memory1.id],
            )?;

            // List should only return active memory now
            let results = list_memories(conn, &ListOptions::default())?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, active.id);

            // Direct get_memory should fail for expired
            let get_result = get_memory(conn, memory1.id);
            assert!(get_result.is_err());

            // Direct get_memory should succeed for active
            let get_result = get_memory(conn, active.id);
            assert!(get_result.is_ok());

            Ok(())
        })
        .unwrap();
}

#[test]
fn out_of_range_ttl_is_a_typed_error_not_a_chrono_panic() {
    let storage = open_test_storage();
    storage
        .with_connection(|conn| {
            let mut input = test_memory_input("daily huge ttl");
            input.tier = MemoryTier::Daily;
            input.ttl_seconds = Some(i64::MAX);
            assert!(matches!(
                create_memory(conn, &input),
                Err(EngramError::InvalidInput(_))
            ));

            let id = make_memory(conn);
            assert!(matches!(
                set_memory_expiration(conn, id, Some(i64::MIN)),
                Err(EngramError::InvalidInput(_))
            ));
            assert!(matches!(
                set_memory_expiration(conn, id, Some(i64::MAX)),
                Err(EngramError::InvalidInput(_))
            ));

            assert!(matches!(
                acquire_dream_lock(conn, "lock", "owner", u64::MAX),
                Err(EngramError::InvalidInput(_))
            ));
            Ok(())
        })
        .unwrap();
}
