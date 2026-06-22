use super::*;
use crate::storage::Storage;
use serde_json::json;
use std::collections::HashMap;

fn test_memory_input(content: &str) -> CreateMemoryInput {
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

#[test]
fn test_compact_dry_run_is_noop() {
    let storage = Storage::open_in_memory().unwrap();
    let report = storage.compact(false).unwrap();
    assert!(!report.applied);
    assert!(report.db_size_bytes > 0);
    assert!(report.operations.iter().all(|op| !op.applied));
    assert_eq!(report.queue_complete_prunable, 0);
    assert_eq!(report.orphan_embeddings, 0);
}

#[test]
fn test_compact_prunes_complete_queue_on_apply() {
    let storage = Storage::open_in_memory().unwrap();
    storage
            .with_connection(|conn| {
                let m = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "queued".to_string(),
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
                conn.execute(
                    "INSERT OR REPLACE INTO embedding_queue (memory_id, status) VALUES (?1, 'complete')",
                    params![m.id],
                )?;
                Ok(())
            })
            .unwrap();

    assert_eq!(
        storage.compact(false).unwrap().queue_complete_prunable,
        1,
        "dry-run should see one prunable completed row"
    );

    let applied = storage.compact(true).unwrap();
    assert!(applied.applied);
    let op = applied
        .operations
        .iter()
        .find(|o| o.name == "prune_complete_queue")
        .expect("prune_complete_queue op present");
    assert!(op.applied, "completed-queue prune should run on apply");

    assert_eq!(
        storage.compact(false).unwrap().queue_complete_prunable,
        0,
        "completed rows should be gone after apply"
    );
}

#[test]
fn test_rebuild_derived_indexes() {
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            for content in ["x", "y"] {
                create_memory(
                    conn,
                    &CreateMemoryInput {
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
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();

    // Dry-run reports without mutating.
    let dry = storage
        .with_transaction(|conn| rebuild_derived_indexes(conn, true, true, false))
        .unwrap();
    assert!(!dry.applied);
    assert_eq!(dry.memories, 2);
    assert_eq!(dry.embeddings_missing, 2);
    assert!(!dry.fts_rebuilt);
    assert_eq!(dry.embeddings_requeued, 0);

    // Apply rebuilds FTS and requeues the two unembedded memories.
    let applied = storage
        .with_transaction(|conn| rebuild_derived_indexes(conn, true, true, true))
        .unwrap();
    assert!(applied.applied);
    assert!(applied.fts_rebuilt);
    assert_eq!(applied.embeddings_requeued, 2);
    assert_eq!(applied.fts_drift_after, 0);
    assert_eq!(applied.memories, 2, "canonical memories are preserved");

    // The two memories now have pending queue rows.
    let pending: i64 = storage
        .with_connection(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(pending, 2);
}

#[test]
fn test_list_memories_metadata_filter_types() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let mut metadata1 = HashMap::new();
            metadata1.insert("status".to_string(), json!("active"));
            metadata1.insert("count".to_string(), json!(3));
            metadata1.insert("flag".to_string(), json!(true));

            let mut metadata2 = HashMap::new();
            metadata2.insert("status".to_string(), json!("inactive"));
            metadata2.insert("count".to_string(), json!(5));
            metadata2.insert("flag".to_string(), json!(false));
            metadata2.insert("optional".to_string(), json!("set"));

            let memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "First".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: metadata1,
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
            let memory2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Second".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: metadata2,
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

            let mut filter = HashMap::new();
            filter.insert("status".to_string(), json!("active"));
            let results = list_memories(
                conn,
                &ListOptions {
                    metadata_filter: Some(filter),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, memory1.id);

            let mut filter = HashMap::new();
            filter.insert("count".to_string(), json!(5));
            let results = list_memories(
                conn,
                &ListOptions {
                    metadata_filter: Some(filter),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, memory2.id);

            let mut filter = HashMap::new();
            filter.insert("flag".to_string(), json!(true));
            let results = list_memories(
                conn,
                &ListOptions {
                    metadata_filter: Some(filter),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, memory1.id);

            let mut filter = HashMap::new();
            filter.insert("optional".to_string(), serde_json::Value::Null);
            let results = list_memories(
                conn,
                &ListOptions {
                    metadata_filter: Some(filter),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, memory1.id);

            Ok(())
        })
        .unwrap();
}

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

// ========== Advanced Filter Tests (RML-932) ==========

#[test]
fn test_advanced_filter_eq() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            // Create test memories
            let mut metadata1 = HashMap::new();
            metadata1.insert("project".to_string(), json!("engram"));
            metadata1.insert("priority".to_string(), json!(1));

            let mut metadata2 = HashMap::new();
            metadata2.insert("project".to_string(), json!("other"));
            metadata2.insert("priority".to_string(), json!(2));

            let _m1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Engram project note".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["rust".to_string()],
                    metadata: metadata1,
                    importance: Some(0.8),
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            let _m2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Other project note".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["python".to_string()],
                    metadata: metadata2,
                    importance: Some(0.5),
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            // Test eq filter
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"metadata.project": {"eq": "engram"}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("Engram"));

            Ok(())
        })
        .unwrap();
}

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

#[test]
fn test_advanced_filter_comparison_operators() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            for i in 1..=5 {
                let mut metadata = HashMap::new();
                metadata.insert("priority".to_string(), json!(i));

                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: format!("Memory with priority {}", i),
                        memory_type: MemoryType::Note,
                        tags: vec![],
                        metadata,
                        importance: Some(i as f32 / 10.0),
                        scope: Default::default(),
                        defer_embedding: true,
                        ..Default::default()
                    },
                )?;
            }

            // Test gte
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"metadata.priority": {"gte": 3}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 3); // 3, 4, 5

            // Test lt
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"metadata.priority": {"lt": 3}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 2); // 1, 2

            // Test importance gte
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"importance": {"gte": 0.4}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 2); // 0.4 and 0.5

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_advanced_filter_and_or() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            // Memory 1: rust, high priority
            let mut m1 = HashMap::new();
            m1.insert("lang".to_string(), json!("rust"));
            m1.insert("priority".to_string(), json!(5));
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Rust high priority".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["performance".to_string()],
                    metadata: m1,
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            // Memory 2: rust, low priority
            let mut m2 = HashMap::new();
            m2.insert("lang".to_string(), json!("rust"));
            m2.insert("priority".to_string(), json!(1));
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Rust low priority".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: m2,
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            // Memory 3: python, high priority
            let mut m3 = HashMap::new();
            m3.insert("lang".to_string(), json!("python"));
            m3.insert("priority".to_string(), json!(5));
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Python high priority".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["performance".to_string()],
                    metadata: m3,
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            // Test AND: rust AND high priority
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({
                        "AND": [
                            {"metadata.lang": {"eq": "rust"}},
                            {"metadata.priority": {"gte": 3}}
                        ]
                    })),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("Rust high"));

            // Test OR: rust OR high priority
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({
                        "OR": [
                            {"metadata.lang": {"eq": "rust"}},
                            {"metadata.priority": {"gte": 5}}
                        ]
                    })),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 3); // All 3 match

            Ok(())
        })
        .unwrap();
}

// ========== Deduplication Tests (RML-931) ==========

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

#[test]
fn test_update_memory_lifecycle_state_records_update_side_effects() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Lifecycle candidate".to_string(),
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
            let event_count_before: i64 =
                conn.query_row("SELECT COUNT(*) FROM memory_events", [], |row| row.get(0))?;
            let version_count_before: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memory_versions WHERE memory_id = ?1",
                params![memory.id],
                |row| row.get(0),
            )?;

            let updated = update_memory_lifecycle_state(conn, memory.id, LifecycleState::Archived)?;

            assert_eq!(updated.lifecycle_state, LifecycleState::Archived);
            assert_eq!(updated.version, memory.version + 1);

            let event_count_after: i64 =
                conn.query_row("SELECT COUNT(*) FROM memory_events", [], |row| row.get(0))?;
            assert_eq!(event_count_after, event_count_before + 1);

            let version_count_after: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memory_versions WHERE memory_id = ?1",
                params![memory.id],
                |row| row.get(0),
            )?;
            assert_eq!(version_count_after, version_count_before + 1);

            let changed_fields: String = conn.query_row(
                "SELECT data FROM memory_events
                 WHERE memory_id = ?1
                 ORDER BY id DESC
                 LIMIT 1",
                params![memory.id],
                |row| row.get(0),
            )?;
            assert!(changed_fields.contains("lifecycle_state"));

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

#[test]
fn test_find_similar_by_embedding() {
    // Helper to store embedding (convert f32 vec to bytes for SQLite)
    fn store_test_embedding(
        conn: &Connection,
        memory_id: i64,
        embedding: &[f32],
    ) -> crate::error::Result<()> {
        let bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                 VALUES (?, ?, ?, ?, datetime('now'))",
            params![memory_id, bytes, "test", embedding.len() as i32],
        )?;
        // Mark memory as having embedding
        conn.execute(
            "UPDATE memories SET has_embedding = 1 WHERE id = ?",
            params![memory_id],
        )?;
        Ok(())
    }

    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_transaction(|conn| {
            // Create a memory with an embedding
            let memory1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Rust is a systems programming language".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["rust".to_string()],
                    metadata: std::collections::HashMap::new(),
                    importance: None,
                    scope: MemoryScope::Global,
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
                },
            )?;

            // Store an embedding for it (simple test embedding)
            let embedding1 = vec![0.8, 0.4, 0.2, 0.1]; // Normalized-ish vector
            store_test_embedding(conn, memory1.id, &embedding1)?;

            // Create another memory with different embedding
            let memory2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Python is a scripting language".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["python".to_string()],
                    metadata: std::collections::HashMap::new(),
                    importance: None,
                    scope: MemoryScope::Global,
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
                },
            )?;

            // Store a very different embedding
            let embedding2 = vec![0.1, 0.2, 0.8, 0.4]; // Different direction
            store_test_embedding(conn, memory2.id, &embedding2)?;

            // Test 1: Query with embedding similar to memory1
            let query_similar_to_1 = vec![0.79, 0.41, 0.21, 0.11]; // Very similar to embedding1
            let result = find_similar_by_embedding(
                conn,
                &query_similar_to_1,
                &MemoryScope::Global,
                None, // default workspace
                0.95, // High threshold
            )?;
            assert!(result.is_some());
            let (found_memory, similarity) = result.unwrap();
            assert_eq!(found_memory.id, memory1.id);
            assert!(similarity > 0.95);

            // Test 2: Query with low threshold should still find memory1
            let result_low_threshold = find_similar_by_embedding(
                conn,
                &query_similar_to_1,
                &MemoryScope::Global,
                None,
                0.5,
            )?;
            assert!(result_low_threshold.is_some());

            // Test 3: Query with embedding not similar to anything (threshold too high)
            let query_orthogonal = vec![0.0, 0.0, 0.0, 1.0]; // Different direction
            let result_no_match = find_similar_by_embedding(
                conn,
                &query_orthogonal,
                &MemoryScope::Global,
                None,
                0.99, // Very high threshold
            )?;
            assert!(result_no_match.is_none());

            // Test 4: Different scope should not find anything
            let result_wrong_scope = find_similar_by_embedding(
                conn,
                &query_similar_to_1,
                &MemoryScope::User {
                    user_id: "other-user".to_string(),
                },
                None,
                0.5,
            )?;
            assert!(result_wrong_scope.is_none());

            Ok(())
        })
        .unwrap();
}

// ── T1: Multimodal MemoryType tests ──────────────────────────────────────

fn open_test_storage() -> crate::storage::Storage {
    crate::storage::Storage::open_in_memory().expect("in-memory storage")
}

#[test]
fn test_create_image_memory_with_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "A screenshot of the dashboard".to_string(),
                    memory_type: MemoryType::Image,
                    media_url: Some("local:///tmp/dashboard.png".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create image memory");
    assert_eq!(memory.memory_type, MemoryType::Image);
    assert_eq!(
        memory.media_url.as_deref(),
        Some("local:///tmp/dashboard.png")
    );
}

#[test]
fn test_create_audio_memory_with_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Meeting recording transcript".to_string(),
                    memory_type: MemoryType::Audio,
                    media_url: Some("local:///tmp/meeting.mp3".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create audio memory");
    assert_eq!(memory.memory_type, MemoryType::Audio);
    assert_eq!(
        memory.media_url.as_deref(),
        Some("local:///tmp/meeting.mp3")
    );
}

#[test]
fn test_create_video_memory_with_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Keynote presentation video".to_string(),
                    memory_type: MemoryType::Video,
                    media_url: Some("https://cdn.example.com/keynote.mp4".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create video memory");
    assert_eq!(memory.memory_type, MemoryType::Video);
    assert_eq!(
        memory.media_url.as_deref(),
        Some("https://cdn.example.com/keynote.mp4")
    );
}

#[test]
fn test_get_memory_returns_media_url() {
    let storage = open_test_storage();
    let created = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Image with URL".to_string(),
                    memory_type: MemoryType::Image,
                    media_url: Some("local:///tmp/image.png".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create");
    let fetched = storage
        .with_connection(|conn| get_memory(conn, created.id))
        .expect("get");
    assert_eq!(fetched.media_url, created.media_url);
}

#[test]
fn test_create_image_memory_without_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Image described in text only".to_string(),
                    memory_type: MemoryType::Image,
                    media_url: None,
                    ..Default::default()
                },
            )
        })
        .expect("create image memory without media_url");
    assert_eq!(memory.memory_type, MemoryType::Image);
    assert!(memory.media_url.is_none());
}

#[test]
fn test_update_memory_sets_media_url() {
    let storage = open_test_storage();
    let created = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Image memory".to_string(),
                    memory_type: MemoryType::Image,
                    ..Default::default()
                },
            )
        })
        .expect("create");
    let updated = storage
        .with_transaction(|conn| {
            update_memory(
                conn,
                created.id,
                &UpdateMemoryInput {
                    media_url: Some(Some("local:///tmp/updated.png".to_string())),
                    content: None,
                    memory_type: None,
                    tags: None,
                    metadata: None,
                    importance: None,
                    scope: None,
                    ttl_seconds: None,
                    event_time: None,
                    trigger_pattern: None,
                },
            )
        })
        .expect("update");
    assert_eq!(
        updated.media_url.as_deref(),
        Some("local:///tmp/updated.png")
    );
}

#[test]
fn test_memory_type_is_multimodal() {
    assert!(MemoryType::Image.is_multimodal());
    assert!(MemoryType::Audio.is_multimodal());
    assert!(MemoryType::Video.is_multimodal());
    assert!(!MemoryType::Note.is_multimodal());
    assert!(!MemoryType::Episodic.is_multimodal());
}

#[test]
fn test_schema_migration_v34_idempotent() {
    use crate::storage::migrations::run_migrations;
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    run_migrations(&conn).expect("run migrations");
    // Running again should be a no-op
    run_migrations(&conn).expect("idempotent second run");
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .expect("query version");
    assert_eq!(version, crate::storage::migrations::SCHEMA_VERSION);
}

#[test]
fn memory_policy_record_round_trips_and_clamps_scores() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let memory = create_memory(conn, &test_memory_input("policy clamp"))?;
            let record = upsert_policy_record(
                conn,
                PolicyRecordInput {
                    memory_id: memory.id,
                    salience_score: -10.0,
                    retention_score: 4.0,
                    retrieval_priority: 0.75,
                    policy_version: "heuristic-test".to_string(),
                    policy_reason: "test clamp".to_string(),
                },
            )?;

            assert_eq!(record.memory_id, memory.id);
            assert_eq!(record.salience_score, 0.0);
            assert_eq!(record.retention_score, 1.0);
            assert_eq!(record.retrieval_priority, 0.75);
            assert_eq!(record.policy_version, "heuristic-test");
            assert_eq!(record.policy_reason, "test clamp");

            let fetched = get_policy_record(conn, memory.id)?.expect("policy record should exist");
            assert_eq!(fetched.memory_id, memory.id);
            assert_eq!(fetched.salience_score, 0.0);
            assert_eq!(fetched.retention_score, 1.0);
            assert_eq!(fetched.retrieval_priority, 0.75);
            Ok(())
        })
        .unwrap();
}

#[test]
fn memory_policy_reinforcement_updates_count_and_timestamp() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let memory = create_memory(conn, &test_memory_input("policy reinforcement"))?;
            let record = record_reinforcement(conn, memory.id, 0.2, "test_reinforcement")?;

            assert_eq!(record.memory_id, memory.id);
            assert_eq!(record.reinforcement_count, 1);
            assert!(record.last_reinforced_at.is_some());
            assert!(record.salience_score > 0.5);
            assert!(record.retention_score > 0.5);
            assert!(record.retrieval_priority > 0.5);

            let events: i64 = conn.query_row(
                "SELECT COUNT(*) FROM enrichment_events
                 WHERE memory_id = ?1
                   AND event_type = 'memory_policy'
                   AND triggered_by = 'test_reinforcement'",
                params![memory.id],
                |row| row.get(0),
            )?;
            assert_eq!(events, 1);
            Ok(())
        })
        .unwrap();
}

#[test]
fn memory_policy_contradiction_increments_count_and_demotes_without_deleting_memory() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let memory = create_memory(conn, &test_memory_input("policy contradiction"))?;
            let record = record_contradiction(
                conn,
                memory.id,
                "test_contradiction",
                "conflicts with newer source",
            )?;

            assert_eq!(record.memory_id, memory.id);
            assert_eq!(record.contradiction_count, 1);
            assert_eq!(record.policy_reason, "conflicts with newer source");
            assert!(record.salience_score < 0.5);
            assert!(record.retention_score < 0.5);
            assert!(record.retrieval_priority < 0.5);

            let still_present = get_memory(conn, memory.id)?;
            assert_eq!(still_present.id, memory.id);

            let events: i64 = conn.query_row(
                "SELECT COUNT(*) FROM enrichment_events
                 WHERE memory_id = ?1 AND event_type = 'memory_policy_conflict'",
                params![memory.id],
                |row| row.get(0),
            )?;
            assert_eq!(events, 1);
            Ok(())
        })
        .unwrap();
}

// ========== Advanced Filter Integration Tests (RML-932) ==========

#[test]
fn test_advanced_filter_tags_contains() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Has rust tag".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["rust".to_string(), "performance".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Has python tag".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec!["python".to_string()],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            // Test tags contains
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"tags": {"contains": "rust"}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("rust"));

            // Test tags not_contains
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"tags": {"not_contains": "rust"}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("python"));

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_advanced_filter_exists() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            let mut m1 = HashMap::new();
            m1.insert("optional_field".to_string(), json!("present"));
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Has optional field".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: m1,
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Missing optional field".to_string(),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: HashMap::new(),
                    importance: None,
                    scope: Default::default(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;

            // Test exists: true
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"metadata.optional_field": {"exists": true}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("Has optional"));

            // Test exists: false
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({"metadata.optional_field": {"exists": false}})),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("Missing optional"));

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_advanced_filter_nested_and_or() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_connection(|conn| {
            // Create diverse test data
            let test_data = vec![
                ("A", "rust", 5, vec!["perf"]),
                ("B", "rust", 1, vec![]),
                ("C", "python", 5, vec!["perf"]),
                ("D", "python", 1, vec![]),
            ];

            for (name, lang, priority, tags) in test_data {
                let mut m = HashMap::new();
                m.insert("lang".to_string(), json!(lang));
                m.insert("priority".to_string(), json!(priority));
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: format!("Memory {}", name),
                        memory_type: MemoryType::Note,
                        tags: tags.into_iter().map(|s| s.to_string()).collect(),
                        metadata: m,
                        importance: None,
                        scope: Default::default(),
                        defer_embedding: true,
                        ..Default::default()
                    },
                )?;
            }

            // Complex filter: (rust AND high) OR (python AND perf tag)
            let results = list_memories(
                conn,
                &ListOptions {
                    filter: Some(json!({
                        "OR": [
                            {
                                "AND": [
                                    {"metadata.lang": {"eq": "rust"}},
                                    {"metadata.priority": {"gte": 5}}
                                ]
                            },
                            {
                                "AND": [
                                    {"metadata.lang": {"eq": "python"}},
                                    {"tags": {"contains": "perf"}}
                                ]
                            }
                        ]
                    })),
                    ..Default::default()
                },
            )?;
            assert_eq!(results.len(), 2); // A and C

            Ok(())
        })
        .unwrap();
}

fn make_memory(conn: &Connection) -> i64 {
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

fn link_supersedes(conn: &Connection, from_id: i64, to_id: i64) {
    conn.execute(
            "INSERT INTO crossrefs (from_id, to_id, edge_type, score, strength, source, created_at, valid_from) \
             VALUES (?1, ?2, 'supersedes', 1.0, 1.0, 'test', datetime('now'), datetime('now'))",
            params![from_id, to_id],
        )
        .unwrap();
}

#[test]
fn test_collect_supersedes_chain_three_nodes() {
    // C supersedes B supersedes A
    // collect_supersedes_chain(C) should return [C, B, A]
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let id_b = make_memory(conn);
            let id_c = make_memory(conn);
            link_supersedes(conn, id_b, id_a); // B supersedes A
            link_supersedes(conn, id_c, id_b); // C supersedes B

            let chain = collect_supersedes_chain(conn, id_c).unwrap();
            assert!(chain.contains(&id_c), "chain must include root");
            assert!(chain.contains(&id_b), "chain must include B");
            assert!(chain.contains(&id_a), "chain must include A");
            assert_eq!(chain.len(), 3);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_collect_supersedes_chain_no_ancestors() {
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let chain = collect_supersedes_chain(conn, id_a).unwrap();
            assert_eq!(chain, vec![id_a]);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_delete_memory_cascade_chain_true() {
    // C supersedes B supersedes A; deleting C with cascade_chain=true removes all three
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let id_b = make_memory(conn);
            let id_c = make_memory(conn);
            link_supersedes(conn, id_b, id_a);
            link_supersedes(conn, id_c, id_b);

            let chain = collect_supersedes_chain(conn, id_c).unwrap();
            for &id in &chain {
                delete_memory(conn, id).unwrap();
            }

            // All three should be soft-deleted (valid_to is set)
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM memories WHERE id IN (?1, ?2, ?3) AND valid_to IS NULL",
                    params![id_a, id_b, id_c],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0, "all three memories should be deleted");
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_delete_memory_cascade_chain_false_leaves_ancestors() {
    // C supersedes B supersedes A; deleting C without cascade leaves B and A intact
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let id_b = make_memory(conn);
            let id_c = make_memory(conn);
            link_supersedes(conn, id_b, id_a);
            link_supersedes(conn, id_c, id_b);

            delete_memory(conn, id_c).unwrap();

            // B and A should still be alive
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM memories WHERE id IN (?1, ?2) AND valid_to IS NULL",
                    params![id_a, id_b],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 2, "ancestors must survive when cascade_chain=false");
            Ok(())
        })
        .unwrap();
}
