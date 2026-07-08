//! Read-only health summaries for the durable SQL embedding queue.

use chrono::Utc;
use rusqlite::{params, Connection};
use std::time::Duration;

use super::types::{EmbeddingQueueHealth, EmbeddingQueueHygieneConfig, DEFAULT_COMPLETE_RETENTION};
use super::util::stale_cutoff_rfc3339;
use crate::error::Result;

/// Retry count bucket threshold for health reporting.
const RETRY_COUNT_BUCKET_3_PLUS: i32 = 3;

/// Return a read-only health summary for the durable embedding queue.
pub fn get_embedding_queue_health(
    conn: &Connection,
    stale_after: Duration,
    max_retries: i32,
) -> Result<EmbeddingQueueHealth> {
    let config = EmbeddingQueueHygieneConfig {
        stale_processing_after: stale_after,
        max_retries,
        complete_retention: DEFAULT_COMPLETE_RETENTION,
    };
    get_embedding_queue_health_with_config(conn, &config)
}

/// Return a read-only health summary for the durable embedding queue using
/// an explicit hygiene policy.
pub fn get_embedding_queue_health_with_config(
    conn: &Connection,
    config: &EmbeddingQueueHygieneConfig,
) -> Result<EmbeddingQueueHealth> {
    let stale_cutoff = stale_cutoff_rfc3339(config.stale_processing_after)?;

    let pending = count_queue_status(conn, "pending")?;
    let processing = count_queue_status(conn, "processing")?;
    let complete = count_queue_status(conn, "complete")?;
    let failed = count_queue_status(conn, "failed")?;

    let stale_processing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'processing' AND started_at IS NOT NULL AND started_at <= ?",
        params![stale_cutoff],
        |row| row.get(0),
    )?;

    let retryable_failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'failed' AND retry_count < ?",
        params![config.max_retries],
        |row| row.get(0),
    )?;

    let exhausted_failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'failed' AND retry_count >= ?",
        params![config.max_retries],
        |row| row.get(0),
    )?;

    let zero_retry_failed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue
         WHERE status = 'failed' AND retry_count = 0",
        [],
        |row| row.get(0),
    )?;

    let oldest_processing_age_seconds =
        oldest_timestamp_seconds(conn, "processing", Some("started_at"))?;

    let oldest_failed_age_seconds = oldest_timestamp_seconds(
        conn,
        "failed",
        Some("COALESCE(completed_at, started_at, queued_at)"),
    )?;

    let max_retry_count = conn
        .query_row(
            "SELECT COALESCE(MAX(retry_count), 0) FROM embedding_queue",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let retry_count_0 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count = 0",
        [],
        |row| row.get(0),
    )?;
    let retry_count_1 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count = 1",
        [],
        |row| row.get(0),
    )?;
    let retry_count_2 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count = 2",
        [],
        |row| row.get(0),
    )?;
    let retry_count_3_plus = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed' AND retry_count >= ?",
        params![RETRY_COUNT_BUCKET_3_PLUS],
        |row| row.get(0),
    )?;

    let oldest_pending_at: Option<String> = conn.query_row(
        "SELECT MIN(queued_at) FROM embedding_queue WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let oldest_pending_seconds = oldest_pending_at.and_then(|queued_at| {
        chrono::DateTime::parse_from_rfc3339(&queued_at)
            .ok()
            .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0))
    });

    Ok(EmbeddingQueueHealth {
        pending,
        processing,
        stale_processing,
        complete,
        failed,
        retryable_failed,
        exhausted_failed,
        zero_retry_failed,
        max_retry_count,
        oldest_pending_seconds,
        oldest_processing_age_seconds,
        oldest_failed_age_seconds,
        retry_count_0,
        retry_count_1,
        retry_count_2,
        retry_count_3_plus,
    })
}

fn count_queue_status(conn: &Connection, status: &str) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = ?",
        params![status],
        |row| row.get(0),
    )?)
}

fn oldest_timestamp_seconds(
    conn: &Connection,
    status: &str,
    ts_expr: Option<&str>,
) -> Result<Option<i64>> {
    let ts_sql = ts_expr.unwrap_or("queued_at");
    let oldest: Option<String> = conn.query_row(
        &format!(
            "SELECT MIN({}) FROM embedding_queue WHERE status = ? AND {} IS NOT NULL",
            ts_sql, ts_sql
        ),
        params![status],
        |row| row.get(0),
    )?;
    Ok(oldest.and_then(|ts| {
        chrono::DateTime::parse_from_rfc3339(&ts)
            .ok()
            .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0))
    }))
}
