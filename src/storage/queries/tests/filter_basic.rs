use super::*;

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
