use super::*;

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
fn list_memories_errors_instead_of_dropping_an_undecodable_row() {
    let storage = open_test_storage();
    storage
        .with_connection(|conn| {
            let good = create_memory(conn, &test_memory_input("healthy row"))?;
            let bad = create_memory(conn, &test_memory_input("corrupt row"))?;
            conn.execute(
                "UPDATE memories SET importance = 'not-a-number' WHERE id = ?",
                [bad.id],
            )?;
            let total: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?;
            assert_eq!(total, 2);

            let result = list_memories(conn, &ListOptions::default());

            let error = result
                .as_ref()
                .err()
                .unwrap_or_else(|| {
                    panic!(
                        "an undecodable row must surface an error, not shrink the list: {:?}",
                        result
                            .as_ref()
                            .map(|rows| rows.iter().map(|m| m.id).collect::<Vec<_>>())
                    )
                })
                .to_string();
            assert!(
                error.contains(&format!("id {}", bad.id)),
                "the error must name the corrupt row: {error}"
            );
            // Rows that decode are still listed once the corruption is gone.
            conn.execute(
                "UPDATE memories SET importance = 0.5 WHERE id = ?",
                [bad.id],
            )?;
            let ids: Vec<i64> = list_memories(conn, &ListOptions::default())?
                .iter()
                .map(|m| m.id)
                .collect();
            assert!(ids.contains(&good.id) && ids.contains(&bad.id));
            Ok(())
        })
        .unwrap();
}

#[test]
fn list_memories_errors_when_tags_cannot_be_loaded() {
    let storage = open_test_storage();
    storage
        .with_connection(|conn| {
            create_memory(conn, &test_memory_input("tag table will vanish"))?;
            conn.execute_batch("DROP TABLE memory_tags;")?;

            let result = list_memories(conn, &ListOptions::default());

            assert!(
                result.is_err(),
                "a failed tag load must not be swallowed into an empty tag list"
            );
            Ok(())
        })
        .unwrap();
}
