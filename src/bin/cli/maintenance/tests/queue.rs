use rusqlite::params;

use super::support::{create_test_memory, test_storage};
use crate::maintenance::queue::run_embedding_queue_maintenance;

#[test]
fn maintenance_queue_hygiene_dry_run_does_not_mutate_and_apply_updates() {
    let (_dir, storage) = test_storage();
    let (
        stale_retryable,
        stale_exhausted,
        stale_fresh,
        failed_retryable,
        complete_recent,
        complete_old,
    ) = storage
        .with_connection(|conn| {
            let stale_retryable = create_test_memory(conn, "queue-hygiene stale retryable")?;
            let stale_exhausted = create_test_memory(conn, "queue-hygiene stale exhausted")?;
            let stale_fresh = create_test_memory(conn, "queue-hygiene stale fresh")?;
            let failed_retryable = create_test_memory(conn, "queue-hygiene failed retryable")?;
            let complete_recent = create_test_memory(conn, "queue-hygiene complete recent")?;
            let complete_old = create_test_memory(conn, "queue-hygiene complete old")?;

            let old_started_at =
                (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
            let fresh_started_at = chrono::Utc::now().to_rfc3339();
            let old_completed = (chrono::Utc::now() - chrono::Duration::days(30)).to_rfc3339();
            let new_completed =
                (chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();

            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 1 WHERE memory_id = ?",
                params![old_started_at, stale_retryable.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 3 WHERE memory_id = ?",
                params![old_started_at, stale_exhausted.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 0 WHERE memory_id = ?",
                params![fresh_started_at, stale_fresh.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                params![failed_retryable.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'complete', queued_at = ?, completed_at = ? WHERE memory_id = ?",
                params![old_completed, old_completed, complete_old.id],
            )?;
            conn.execute(
                "UPDATE embedding_queue SET status = 'complete', queued_at = ?, completed_at = ? WHERE memory_id = ?",
                params![new_completed, new_completed, complete_recent.id],
            )?;
            Ok((
                stale_retryable.id,
                stale_exhausted.id,
                stale_fresh.id,
                failed_retryable.id,
                complete_recent.id,
                complete_old.id,
            ))
        })
        .unwrap();

    let dry_run = run_embedding_queue_maintenance(&storage, true, false).unwrap();
    assert_eq!(dry_run.requeued_stale, 1);
    assert_eq!(dry_run.failed_exhausted, 1);
    assert_eq!(dry_run.requeued_failed, 1);
    assert_eq!(dry_run.pruned_complete, 1);

    let before = storage
        .with_connection(|conn| {
            let stale_retryable_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![stale_retryable],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let stale_exhausted_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![stale_exhausted],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let stale_fresh_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![stale_fresh],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let failed_retryable_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![failed_retryable],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let old_complete = conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete' AND memory_id = ?",
                params![complete_old],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((
                stale_retryable_state,
                stale_exhausted_state,
                stale_fresh_state,
                failed_retryable_state,
                old_complete,
            ))
        })
        .unwrap();
    assert_eq!(before.0, ("processing".to_string(), 1));
    assert_eq!(before.1, ("processing".to_string(), 3));
    assert_eq!(before.2, ("processing".to_string(), 0));
    assert_eq!(before.3, ("failed".to_string(), 1));
    assert_eq!(before.4, 1);

    let applied = run_embedding_queue_maintenance(&storage, true, true).unwrap();
    assert_eq!(applied.requeued_stale, 1);
    assert_eq!(applied.failed_exhausted, 1);
    assert_eq!(applied.requeued_failed, 1);
    assert_eq!(applied.pruned_complete, 1);

    let after = storage
        .with_connection(|conn| {
            let stale_retryable_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![stale_retryable],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let stale_exhausted_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![stale_exhausted],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let stale_fresh_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![stale_fresh],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let failed_retryable_state = conn.query_row(
                "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                params![failed_retryable],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
            )?;
            let complete_count = conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete' AND memory_id IN (?, ?)",
                params![complete_recent, complete_old],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((
                stale_retryable_state,
                stale_exhausted_state,
                stale_fresh_state,
                failed_retryable_state,
                complete_count,
            ))
        })
        .unwrap();

    assert_eq!(after.0, ("pending".to_string(), 2));
    assert_eq!(after.1, ("failed".to_string(), 3));
    assert_eq!(after.2, ("processing".to_string(), 0));
    assert_eq!(after.3, ("pending".to_string(), 2));
    assert_eq!(after.4, 1);
}
