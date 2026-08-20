//! Cloud sync functionality (RML-875)
//!
//! Non-blocking S3/R2/GCS sync with debouncing.
//!
//! # Feature Flags
//!
//! This module requires the `cloud` feature to be enabled for cloud storage backends.
//! The conflict resolution logic is always available.

#[cfg(feature = "cloud")]
mod cloud;
pub mod conflict;
#[cfg(feature = "cloud")]
mod encryption;
#[cfg(feature = "cloud")]
pub mod key_config;
#[cfg(all(test, feature = "cloud"))]
mod key_config_tests;
pub mod wal_replication;
#[cfg(feature = "cloud")]
mod worker;

#[cfg(feature = "cloud")]
pub use cloud::CloudStorage;
pub use conflict::{
    Conflict, ConflictDetector, ConflictInfo, ConflictQueue, ConflictResolver, ConflictType,
    MergeResult, Resolution, ResolutionStrategy, SyncMemoryVersion, ThreeWayMerge,
};
pub use wal_replication::{
    compute_wal_checksum, RecoveryOptions, RecoveryReport, ReplicationLag,
    ReplicationStatus as WalReplicationStatus, WalDelta, WalDeltaPack, WalDeltaReader, WalFrame,
    WalHeader, WalRecoveryEngine, WalReplicationError, WalReplicationStreamer,
};
#[cfg(feature = "cloud")]
pub use worker::{get_sync_status, SyncWorker};

use chrono::{DateTime, Utc};

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

/// Sync event for logging/notifications
#[derive(Debug, Clone)]
pub struct SyncEvent {
    pub direction: SyncDirection,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub bytes_transferred: u64,
    pub success: bool,
    pub error: Option<String>,
}
