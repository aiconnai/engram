use super::*;

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
