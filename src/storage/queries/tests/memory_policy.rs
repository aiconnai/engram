use super::*;

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
