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
