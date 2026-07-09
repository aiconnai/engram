use super::*;

#[test]
fn test_memory_scope_isolation() {
    use crate::types::MemoryScope;

    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            // Create memory with user scope
            let user1_memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "User 1 memory".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["test".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::user("user-1"),
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

            // Create memory with different user scope
            let user2_memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "User 2 memory".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["test".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::user("user-2"),
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

            // Create memory with session scope
            let session_memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Session memory".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["test".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::session("session-abc"),
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

            // Create memory with global scope
            let global_memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Global memory".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["test".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: MemoryScope::Global,
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

            // Test: List all memories (no scope filter) should return all 4
            let all_results = list_memories(conn, &ListOptions::default())?;
            assert_eq!(all_results.len(), 4);

            // Test: Filter by user-1 scope should return only user-1's memory
            let user1_results = list_memories(
                conn,
                &ListOptions {
                    scope: Some(MemoryScope::user("user-1")),
                    ..Default::default()
                },
            )?;
            assert_eq!(user1_results.len(), 1);
            assert_eq!(user1_results[0].id, user1_memory.id);
            assert_eq!(user1_results[0].scope, MemoryScope::user("user-1"));

            // Test: Filter by user-2 scope should return only user-2's memory
            let user2_results = list_memories(
                conn,
                &ListOptions {
                    scope: Some(MemoryScope::user("user-2")),
                    ..Default::default()
                },
            )?;
            assert_eq!(user2_results.len(), 1);
            assert_eq!(user2_results[0].id, user2_memory.id);

            // Test: Filter by session scope should return only session memory
            let session_results = list_memories(
                conn,
                &ListOptions {
                    scope: Some(MemoryScope::session("session-abc")),
                    ..Default::default()
                },
            )?;
            assert_eq!(session_results.len(), 1);
            assert_eq!(session_results[0].id, session_memory.id);

            // Test: Filter by global scope should return only global memory
            let global_results = list_memories(
                conn,
                &ListOptions {
                    scope: Some(MemoryScope::Global),
                    ..Default::default()
                },
            )?;
            assert_eq!(global_results.len(), 1);
            assert_eq!(global_results[0].id, global_memory.id);

            // Test: Verify scope is correctly stored and retrieved
            let retrieved = get_memory(conn, user1_memory.id)?;
            assert_eq!(retrieved.scope, MemoryScope::user("user-1"));

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_memory_scope_can_access() {
    use crate::types::MemoryScope;

    // Global can access everything
    assert!(MemoryScope::Global.can_access(&MemoryScope::user("user-1")));
    assert!(MemoryScope::Global.can_access(&MemoryScope::session("session-1")));
    assert!(MemoryScope::Global.can_access(&MemoryScope::agent("agent-1")));
    assert!(MemoryScope::Global.can_access(&MemoryScope::Global));

    // Same scope can access
    assert!(MemoryScope::user("user-1").can_access(&MemoryScope::user("user-1")));
    assert!(MemoryScope::session("s1").can_access(&MemoryScope::session("s1")));
    assert!(MemoryScope::agent("a1").can_access(&MemoryScope::agent("a1")));

    // Different scope IDs cannot access each other
    assert!(!MemoryScope::user("user-1").can_access(&MemoryScope::user("user-2")));
    assert!(!MemoryScope::session("s1").can_access(&MemoryScope::session("s2")));
    assert!(!MemoryScope::agent("a1").can_access(&MemoryScope::agent("a2")));

    // Different scope types cannot access each other
    assert!(!MemoryScope::user("user-1").can_access(&MemoryScope::session("s1")));
    assert!(!MemoryScope::session("s1").can_access(&MemoryScope::agent("a1")));

    // Anyone can access global memories
    assert!(MemoryScope::user("user-1").can_access(&MemoryScope::Global));
    assert!(MemoryScope::session("s1").can_access(&MemoryScope::Global));
    assert!(MemoryScope::agent("a1").can_access(&MemoryScope::Global));
}
