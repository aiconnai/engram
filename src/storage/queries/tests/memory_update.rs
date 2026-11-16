use super::*;

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
fn test_archive_memory_if_active_guards_workspace_deleted_and_archived() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Archive guard candidate".to_string(),
                    workspace: Some("ws-guard".to_string()),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;
            let before = update_footprint(conn, memory.id)?;

            // Wrong workspace: untouched.
            assert!(!archive_memory_if_active(conn, memory.id, Some("other"))?);
            assert_eq!(update_footprint(conn, memory.id)?, before);

            // Missing memory: untouched, no error.
            assert!(!archive_memory_if_active(conn, i64::MAX, None)?);

            // Matching workspace: archived once through the normal update path.
            assert!(archive_memory_if_active(conn, memory.id, Some("ws-guard"))?);
            let after = update_footprint(conn, memory.id)?;
            assert_eq!(after.0, before.0 + 1);
            assert_eq!(after.1, before.1 + 1);
            assert_eq!(after.2, before.2 + 1);
            assert_eq!(after.3, before.3 + 1);

            // Already archived: idempotent no-op.
            assert!(!archive_memory_if_active(conn, memory.id, None)?);
            assert_eq!(update_footprint(conn, memory.id)?, after);

            // Soft-deleted: untouched.
            let other = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Archive guard deleted".to_string(),
                    defer_embedding: true,
                    ..Default::default()
                },
            )?;
            delete_memory(conn, other.id)?;
            assert!(!archive_memory_if_active(conn, other.id, None)?);
            Ok(())
        })
        .unwrap();
}
