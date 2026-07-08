//! Async embedding queue with batch processing (RML-873)
//!
//! Embeddings are computed in the background to avoid blocking writes.
//! The queue supports batching for efficient API usage.
//!
//! Module layout (ADR-CLEANUP-20260708-2 row 2):
//! - [`types`]: shared structs, reports, and default constants
//! - [`core`]: in-process async channel queue ([`EmbeddingQueue`])
//! - [`worker`]: background batch worker ([`EmbeddingWorker`])
//! - [`status`]: per-memory status/embedding readback and failed-row retry
//! - [`health`]: read-only durable queue health summaries
//! - [`hygiene`]: explicit repair passes over stale/failed/complete rows
//! - [`drain`]: storage-scoped drain of pending SQL queue rows

mod core;
mod drain;
mod health;
mod hygiene;
mod status;
mod types;
mod util;
mod worker;

pub use core::EmbeddingQueue;
pub use drain::drain_pending_embeddings;
pub use health::get_embedding_queue_health;
pub use hygiene::{requeue_stale_processing_embeddings, run_embedding_queue_hygiene};
pub use status::{get_embedding, get_embedding_status};
pub use types::{
    EmbeddingQueueHealth, EmbeddingQueueHygieneConfig, EmbeddingQueueHygieneReport,
    DEFAULT_COMPLETE_RETENTION, DEFAULT_MAX_EMBEDDING_RETRIES, DEFAULT_STALE_PROCESSING_AFTER,
};
pub use worker::EmbeddingWorker;

// Currently only exercised by tests, but part of the queue's intended surface.
#[allow(unused_imports)]
pub use health::get_embedding_queue_health_with_config;
#[allow(unused_imports)]
pub use status::retry_failed_embeddings;
#[allow(unused_imports)]
pub use types::EmbeddingRequest;

#[cfg(test)]
mod tests;
