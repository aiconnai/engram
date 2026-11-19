use super::*;

#[test]
fn test_text_similarity() {
    let a = "The quick brown fox jumps over the lazy dog";
    let b = "The quick brown fox jumps over the lazy cat";
    let c = "Something completely different";

    let sim_ab = calculate_text_similarity(a, b);
    let sim_ac = calculate_text_similarity(a, c);

    assert!(sim_ab > 0.8, "Similar texts should have high similarity");
    assert!(sim_ac < 0.3, "Different texts should have low similarity");
}

#[test]
fn test_cosine_similarity() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let c = vec![0.0, 1.0, 0.0];

    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    assert!(cosine_similarity(&a, &c).abs() < 0.001);
}

#[test]
fn test_conflict_type_parsing() {
    assert_eq!(
        "contradiction".parse::<ConflictType>().unwrap(),
        ConflictType::Contradiction
    );
    assert_eq!(
        "duplicate".parse::<ConflictType>().unwrap(),
        ConflictType::Duplicate
    );
}

fn seed_conflict_pair(conn: &rusqlite::Connection) -> (i64, i64, i64) {
    use crate::storage::queries::create_memory;
    use crate::types::CreateMemoryInput;

    let mut ids = Vec::new();
    for content in ["conflict side alpha", "conflict side beta"] {
        let memory = create_memory(
            conn,
            &CreateMemoryInput {
                content: content.to_string(),
                ..Default::default()
            },
        )
        .expect("create memory");
        ids.push(memory.id);
    }
    conn.execute(
        "INSERT INTO memory_conflicts (memory_a_id, memory_b_id) VALUES (?1, ?2)",
        rusqlite::params![ids[0], ids[1]],
    )
    .expect("insert conflict");
    (conn.last_insert_rowid(), ids[0], ids[1])
}

#[test]
fn test_resolve_conflict_keep_a_archives_b_with_normal_update_bookkeeping() {
    use crate::storage::queries::update_footprint;
    use crate::storage::Storage;

    let storage = Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let (conflict_id, keep, archive) = seed_conflict_pair(conn);
            let kept_before = update_footprint(conn, keep)?;
            let archived_before = update_footprint(conn, archive)?;

            resolve_conflict(conn, conflict_id, ResolutionType::KeepA, None)?;

            let state: String = conn.query_row(
                "SELECT lifecycle_state FROM memories WHERE id = ?1",
                rusqlite::params![archive],
                |row| row.get(0),
            )?;
            assert_eq!(state, "archived");
            let archived_after = update_footprint(conn, archive)?;
            assert_eq!(archived_after.0, archived_before.0 + 1, "version bump");
            assert_eq!(
                archived_after.1,
                archived_before.1 + 1,
                "memory_versions row"
            );
            assert_eq!(archived_after.2, archived_before.2 + 1, "change event");
            assert_eq!(
                archived_after.3,
                archived_before.3 + 1,
                "sync_state pending"
            );
            assert_eq!(update_footprint(conn, keep)?.0, kept_before.0);
            Ok(())
        })
        .expect("keep_a");
}

#[test]
fn test_resolve_conflict_keep_b_archives_a_with_normal_update_bookkeeping() {
    use crate::storage::queries::update_footprint;
    use crate::storage::Storage;

    let storage = Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let (conflict_id, archive, keep) = seed_conflict_pair(conn);
            let archived_before = update_footprint(conn, archive)?;

            resolve_conflict(conn, conflict_id, ResolutionType::KeepB, None)?;

            let archived_after = update_footprint(conn, archive)?;
            assert_eq!(archived_after.0, archived_before.0 + 1, "version bump");
            assert_eq!(
                archived_after.1,
                archived_before.1 + 1,
                "memory_versions row"
            );
            assert_eq!(archived_after.2, archived_before.2 + 1, "change event");
            let kept_state: String = conn.query_row(
                "SELECT lifecycle_state FROM memories WHERE id = ?1",
                rusqlite::params![keep],
                |row| row.get(0),
            )?;
            assert_ne!(kept_state, "archived");
            Ok(())
        })
        .expect("keep_b");
}

#[test]
fn test_resolve_conflict_keep_a_is_noop_for_missing_or_already_archived_target() {
    use crate::storage::queries::{update_footprint, update_memory_lifecycle_state};
    use crate::storage::Storage;
    use crate::types::LifecycleState;

    let storage = Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let (conflict_id, _keep, archive) = seed_conflict_pair(conn);
            update_memory_lifecycle_state(conn, archive, LifecycleState::Archived)?;
            let before = update_footprint(conn, archive)?;

            resolve_conflict(conn, conflict_id, ResolutionType::KeepA, None)?;

            assert_eq!(update_footprint(conn, archive)?, before);
            Ok(())
        })
        .expect("already archived target");
}

#[test]
fn test_detect_conflicts_and_delete_both_use_live_schema_and_workspace_isolation() {
    use crate::storage::queries::{create_memory, get_memory_readonly};
    use crate::storage::Storage;
    use crate::types::CreateMemoryInput;

    let storage = Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let a = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Service port configuration uses TCP 8080 in production".to_string(),
                    tags: vec!["network".to_string()],
                    workspace: Some("alpha".to_string()),
                    ..Default::default()
                },
            )?;
            let b = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Service port configuration uses TCP 8080 in production".to_string(),
                    tags: vec!["network".to_string()],
                    workspace: Some("alpha".to_string()),
                    ..Default::default()
                },
            )?;
            // Identical content + identical tag in another workspace must never pair with `alpha`.
            let cross_ws = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Service port configuration uses TCP 8080 in production".to_string(),
                    tags: vec!["network".to_string()],
                    workspace: Some("beta".to_string()),
                    ..Default::default()
                },
            )?;

            let config = ContextQualityConfig::default();
            let conflicts = detect_conflicts(conn, a.id, &config)?;
            assert_eq!(conflicts.len(), 1, "must find only same-workspace conflict");
            assert_eq!(conflicts[0].memory_a_id, a.id);
            assert_eq!(conflicts[0].memory_b_id, b.id);
            assert!(conflicts[0].id > 0, "must return persisted conflict row id");

            // Scanning conflicts must not inflate `access_count`.
            assert_eq!(get_memory_readonly(conn, a.id)?.access_count, 0);

            // Re-running while unresolved stays idempotent and returns the same conflict id.
            let rerun = detect_conflicts(conn, a.id, &config)?;
            assert_eq!(rerun.len(), 1);
            assert_eq!(rerun[0].id, conflicts[0].id);

            let events_before: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memory_events WHERE event_type = 'deleted'",
                [],
                |row| row.get(0),
            )?;
            let pending_before: i64 = conn.query_row(
                "SELECT pending_changes FROM sync_state WHERE id = 1",
                [],
                |row| row.get(0),
            )?;

            resolve_conflict(
                conn,
                conflicts[0].id,
                ResolutionType::DeleteBoth,
                Some("obsolete duplicate pair"),
            )?;

            // Both alpha memories are soft-deleted (`valid_to` set), with delete events and sync_state bumped.
            assert!(get_memory_readonly(conn, a.id).is_err());
            assert!(get_memory_readonly(conn, b.id).is_err());
            assert!(
                get_memory_readonly(conn, cross_ws.id).is_ok(),
                "other workspace memory untouched"
            );

            let events_after: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memory_events WHERE event_type = 'deleted'",
                [],
                |row| row.get(0),
            )?;
            let pending_after: i64 = conn.query_row(
                "SELECT pending_changes FROM sync_state WHERE id = 1",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(events_after, events_before + 2);
            assert_eq!(pending_after, pending_before + 2);

            // Repeat DeleteBoth on already soft-deleted targets is a safe no-op.
            resolve_conflict(conn, conflicts[0].id, ResolutionType::DeleteBoth, None)?;
            Ok(())
        })
        .expect("detect_conflicts + delete_both");
}

#[test]
fn test_duplicates_and_quality_report_respect_valid_to_and_workspace() {
    use crate::storage::queries::{create_memory, delete_memory};
    use crate::storage::Storage;
    use crate::types::CreateMemoryInput;

    let storage = Storage::open_in_memory().expect("storage");
    storage
        .with_transaction(|conn| {
            let a1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Database connection pool max size is 32 connections".to_string(),
                    workspace: Some("alpha".to_string()),
                    ..Default::default()
                },
            )?;
            let a2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Database connection pool max size is 32 connections".to_string(),
                    workspace: Some("alpha".to_string()),
                    ..Default::default()
                },
            )?;
            let deleted_alpha = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Database connection pool max size is 32 connections".to_string(),
                    workspace: Some("alpha".to_string()),
                    ..Default::default()
                },
            )?;
            delete_memory(conn, deleted_alpha.id)?;

            // Same text in workspace `beta` must not cross-link with `alpha` or inflate `alpha` report counts.
            let b1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Database connection pool max size is 32 connections".to_string(),
                    workspace: Some("beta".to_string()),
                    ..Default::default()
                },
            )?;
            let b2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Database connection pool max size is 32 connections".to_string(),
                    workspace: Some("beta".to_string()),
                    ..Default::default()
                },
            )?;
            let _ = detect_conflicts(conn, b1.id, &ContextQualityConfig::default())?;

            let dups_first = find_near_duplicates(conn, 0.9, 50)?;
            // One pair in alpha (a2, a1) and one pair in beta (b2, b1); soft-deleted row excluded and no cross-workspace pair.
            assert_eq!(dups_first.len(), 2);
            for cand in &dups_first {
                assert!(cand.id > 0);
                let pair = (cand.memory_a_id, cand.memory_b_id);
                assert!(
                    pair == (a2.id, a1.id)
                        || pair == (a1.id, a2.id)
                        || pair == (b2.id, b1.id)
                        || pair == (b1.id, b2.id),
                    "unexpected cross-workspace or soft-deleted duplicate pair: {pair:?}"
                );
            }
            // Calling `find_near_duplicates` again while candidates remain pending is idempotent.
            let dups_second = find_near_duplicates(conn, 0.9, 50)?;
            assert_eq!(dups_second.len(), 2);

            // Semantic duplicates query also respects `valid_to IS NULL`.
            for id in [a1.id, deleted_alpha.id] {
                let bytes: Vec<u8> = [1.0_f32, 0.0, 0.0]
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect();
                conn.execute(
                    "INSERT INTO embeddings (memory_id, embedding, model, dimensions) VALUES (?1, ?2, 'test', 3)",
                    rusqlite::params![id, bytes],
                )?;
            }
            let sem_dups = find_semantic_duplicates(conn, &[1.0, 0.0, 0.0], 0.95, 10)?;
            assert_eq!(sem_dups.len(), 1);
            assert_eq!(sem_dups[0].memory_b_id, a1.id);

            let alpha_report = generate_quality_report(conn, Some("alpha"))?;
            assert_eq!(alpha_report.total_memories, 2);
            assert_eq!(
                alpha_report.conflicts_count, 0,
                "beta conflicts must not leak into alpha report"
            );
            assert_eq!(
                alpha_report.duplicates_count, 1,
                "beta duplicates must not leak into alpha report"
            );

            let beta_report = generate_quality_report(conn, Some("beta"))?;
            assert_eq!(beta_report.total_memories, 2);
            assert_eq!(beta_report.conflicts_count, 1);
            assert_eq!(beta_report.duplicates_count, 1);
            Ok(())
        })
        .expect("duplicates + quality_report");
}
