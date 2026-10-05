//! Shared timestamp/cutoff helpers for queue health and hygiene.

use chrono::{Duration as ChronoDuration, Utc};
use std::time::Duration;

use crate::error::{EngramError, Result};

pub(super) fn stale_cutoff_rfc3339(stale_after: Duration) -> Result<String> {
    let stale_after = ChronoDuration::from_std(stale_after)
        .map_err(|_| EngramError::InvalidInput("stale_after duration is too large".to_string()))?;
    Ok((Utc::now() - stale_after).to_rfc3339())
}

pub(super) fn complete_retention_cutoff_rfc3339(complete_retention: Duration) -> Result<String> {
    let complete_retention = ChronoDuration::from_std(complete_retention).map_err(|_| {
        EngramError::InvalidInput("complete_retention duration is too large".to_string())
    })?;
    Ok((Utc::now() - complete_retention).to_rfc3339())
}

pub(super) fn embedding_queue_db_write_error(
    operation: &str,
    memory_id: crate::types::MemoryId,
    error: rusqlite::Error,
) -> EngramError {
    EngramError::Embedding(format!(
        "database write failed while {operation} for memory_id={memory_id}: {error}"
    ))
}
