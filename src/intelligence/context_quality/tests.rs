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
