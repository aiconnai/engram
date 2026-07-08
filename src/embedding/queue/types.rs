//! Shared types and defaults for the embedding queue.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::MemoryId;

/// Default age after which an in-flight SQL queue row is considered abandoned.
pub const DEFAULT_STALE_PROCESSING_AFTER: Duration = Duration::from_secs(15 * 60);

/// Default retry budget for queue hygiene and health accounting.
pub const DEFAULT_MAX_EMBEDDING_RETRIES: i32 = 3;

/// Default age after which completed embedding rows are eligible for retention pruning.
pub const DEFAULT_COMPLETE_RETENTION: Duration = Duration::from_secs(14 * 24 * 60 * 60);

/// Parameters that govern explicit embedding-queue hygiene.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddingQueueHygieneConfig {
    pub stale_processing_after: Duration,
    pub max_retries: i32,
    pub complete_retention: Duration,
}

impl Default for EmbeddingQueueHygieneConfig {
    fn default() -> Self {
        Self {
            stale_processing_after: DEFAULT_STALE_PROCESSING_AFTER,
            max_retries: DEFAULT_MAX_EMBEDDING_RETRIES,
            complete_retention: DEFAULT_COMPLETE_RETENTION,
        }
    }
}

/// Message for the embedding queue
#[derive(Debug)]
pub struct EmbeddingRequest {
    pub memory_id: MemoryId,
    pub content: String,
}

/// Read-only summary of durable embedding queue health.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingQueueHealth {
    pub pending: i64,
    pub processing: i64,
    pub stale_processing: i64,
    pub complete: i64,
    pub failed: i64,
    pub retryable_failed: i64,
    pub exhausted_failed: i64,
    pub zero_retry_failed: i64,
    pub max_retry_count: i32,
    pub oldest_pending_seconds: Option<i64>,
    pub oldest_processing_age_seconds: Option<i64>,
    pub oldest_failed_age_seconds: Option<i64>,
    pub retry_count_0: i64,
    pub retry_count_1: i64,
    pub retry_count_2: i64,
    pub retry_count_3_plus: i64,
}

/// Result of an explicit queue hygiene pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingQueueHygieneReport {
    pub requeued_stale: i64,
    pub failed_exhausted: i64,
    pub requeued_failed: i64,
    pub pruned_complete: i64,
}
