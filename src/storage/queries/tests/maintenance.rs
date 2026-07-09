use super::*;

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
