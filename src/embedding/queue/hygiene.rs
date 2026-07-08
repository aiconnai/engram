//! Explicit repair passes over stale/failed/completed queue rows.

use chrono::Utc;
use rusqlite::{params, Connection};
use std::time::Duration;

use super::types::{EmbeddingQueueHygieneConfig, EmbeddingQueueHygieneReport};
use super::util::{complete_retention_cutoff_rfc3339, stale_cutoff_rfc3339};
use crate::error::Result;

/// Explicit repair pass over stale queue work.
///
/// This function is non-invasive when `apply == false` (dry-run).
/// In read-only mode it only reports candidate counts.
pub fn run_embedding_queue_hygiene(
    conn: &Connection,
    config: &EmbeddingQueueHygieneConfig,
    requeue_retryable_failed: bool,
    apply: bool,
    prune_complete: bool,
) -> Result<EmbeddingQueueHygieneReport> {
    let stale_cutoff = stale_cutoff_rfc3339(config.stale_processing_after)?;
    let now = Utc::now().to_rfc3339();
    let retention_cutoff = complete_retention_cutoff_rfc3339(config.complete_retention)?;
    let stale_processing_where =
        "status = 'processing' AND started_at IS NOT NULL AND started_at <= ? AND retry_count < ?";
    let failed_retryable_where = "status = 'failed' AND retry_count < ? AND retry_count >= 0";
    let failed_exhausted_where =
        "status = 'processing' AND started_at IS NOT NULL AND started_at <= ? AND retry_count >= ?";
    let complete_prunable_where =
        "status = 'complete' AND COALESCE(completed_at, queued_at) IS NOT NULL AND COALESCE(completed_at, queued_at) <= ?";

    let requeued_stale = if apply {
        conn.execute(
            &format!(
                "UPDATE embedding_queue SET status = 'pending', started_at = NULL, error = NULL, \
                 retry_count = retry_count + 1, queued_at = ? WHERE {stale_processing_where}"
            ),
            params![now, stale_cutoff, config.max_retries],
        )? as i64
    } else {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM embedding_queue WHERE {stale_processing_where}"),
            params![stale_cutoff, config.max_retries],
            |row| row.get(0),
        )?
    };

    let failed_exhausted = if apply {
        conn.execute(
            &format!(
                "UPDATE embedding_queue
                 SET status = 'failed',
                     error = 'embedding processing lease expired after retry budget',
                     completed_at = ?
                 WHERE {failed_exhausted_where}"
            ),
            params![now, stale_cutoff, config.max_retries],
        )? as i64
    } else {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM embedding_queue WHERE {failed_exhausted_where}"),
            params![stale_cutoff, config.max_retries],
            |row| row.get(0),
        )?
    };

    let requeued_failed = if requeue_retryable_failed {
        if apply {
            conn.execute(
                &format!(
                    "UPDATE embedding_queue
                     SET status = 'pending',
                         error = NULL,
                         retry_count = retry_count + 1,
                         queued_at = ?,
                         started_at = NULL
                     WHERE {failed_retryable_where}"
                ),
                params![now, config.max_retries],
            )? as i64
        } else {
            conn.query_row(
                &format!("SELECT COUNT(*) FROM embedding_queue WHERE {failed_retryable_where}"),
                params![config.max_retries],
                |row| row.get(0),
            )?
        }
    } else {
        0
    };

    let pruned_complete = if prune_complete && config.complete_retention.as_secs() > 0 {
        if apply {
            conn.execute(
                &format!("DELETE FROM embedding_queue WHERE {complete_prunable_where}"),
                params![retention_cutoff],
            )? as i64
        } else {
            conn.query_row(
                &format!("SELECT COUNT(*) FROM embedding_queue WHERE {complete_prunable_where}"),
                params![retention_cutoff],
                |row| row.get(0),
            )?
        }
    } else {
        0
    };

    Ok(EmbeddingQueueHygieneReport {
        requeued_stale,
        failed_exhausted,
        requeued_failed,
        pruned_complete,
    })
}

/// Requeue abandoned `processing` rows, or fail them if their retry budget is exhausted.
///
/// This is an explicit repair path for stale in-flight work. Health checks do
/// not call this function.
pub fn requeue_stale_processing_embeddings(
    conn: &Connection,
    stale_after: Duration,
    max_retries: i32,
) -> Result<EmbeddingQueueHygieneReport> {
    let stale_cutoff = stale_cutoff_rfc3339(stale_after)?;
    let now = Utc::now().to_rfc3339();

    let requeued_stale = conn.execute(
        "UPDATE embedding_queue
         SET status = 'pending',
             started_at = NULL,
             error = NULL,
             retry_count = retry_count + 1,
             queued_at = ?
         WHERE status = 'processing'
           AND started_at IS NOT NULL
           AND started_at <= ?
           AND retry_count < ?",
        params![now, stale_cutoff, max_retries],
    )? as i64;

    let failed_exhausted = conn.execute(
        "UPDATE embedding_queue
         SET status = 'failed',
             error = 'embedding processing lease expired after retry budget',
             completed_at = ?
         WHERE status = 'processing'
           AND started_at IS NOT NULL
           AND started_at <= ?
           AND retry_count >= ?",
        params![now, stale_cutoff, max_retries],
    )? as i64;

    Ok(EmbeddingQueueHygieneReport {
        requeued_stale,
        failed_exhausted,
        requeued_failed: 0,
        pruned_complete: 0,
    })
}
