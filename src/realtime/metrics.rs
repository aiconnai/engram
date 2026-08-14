//! Aggregate realtime resource counters (no PII dimensions).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Aggregate resource counters. They intentionally contain no principal,
/// credential, workspace, or message data.
#[derive(Default)]
pub(super) struct RealtimeMetrics {
    pub(super) active_connections: AtomicU64,
    pub(super) accepted_connections: AtomicU64,
    pub(super) connection_cap_rejections: AtomicU64,
    pub(super) oversized_messages: AtomicU64,
    pub(super) idle_disconnects: AtomicU64,
    pub(super) completed_disconnects: AtomicU64,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct RealtimeMetricsSnapshot {
    pub(super) active_connections: u64,
    pub(super) accepted_connections: u64,
    pub(super) connection_cap_rejections: u64,
    pub(super) oversized_messages: u64,
    pub(super) idle_disconnects: u64,
    pub(super) completed_disconnects: u64,
}

impl RealtimeMetrics {
    pub(super) fn snapshot(&self) -> RealtimeMetricsSnapshot {
        RealtimeMetricsSnapshot {
            active_connections: self.active_connections.load(Ordering::Relaxed),
            accepted_connections: self.accepted_connections.load(Ordering::Relaxed),
            connection_cap_rejections: self.connection_cap_rejections.load(Ordering::Relaxed),
            oversized_messages: self.oversized_messages.load(Ordering::Relaxed),
            idle_disconnects: self.idle_disconnects.load(Ordering::Relaxed),
            completed_disconnects: self.completed_disconnects.load(Ordering::Relaxed),
        }
    }
}

pub(super) struct ActiveConnectionGuard {
    metrics: Arc<RealtimeMetrics>,
}

impl ActiveConnectionGuard {
    pub(super) fn new(metrics: Arc<RealtimeMetrics>) -> Self {
        metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        metrics.accepted_connections.fetch_add(1, Ordering::Relaxed);
        Self { metrics }
    }
}

impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        self.metrics
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
        self.metrics
            .completed_disconnects
            .fetch_add(1, Ordering::Relaxed);
    }
}
