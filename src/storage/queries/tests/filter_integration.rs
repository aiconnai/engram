use super::*;

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
