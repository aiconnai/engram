use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::core::MemoryId;

/// Statistics for a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStats {
    /// Workspace name
    pub workspace: String,
    /// Total number of memories
    pub memory_count: i64,
    /// Number of permanent memories
    pub permanent_count: i64,
    /// Number of daily (ephemeral) memories
    pub daily_count: i64,
    /// Timestamp of first memory
    pub first_memory_at: Option<DateTime<Utc>>,
    /// Timestamp of last memory
    pub last_memory_at: Option<DateTime<Utc>>,
    /// Top tags in this workspace (tag, count)
    #[serde(default)]
    pub top_tags: Vec<(String, i64)>,
    /// Average importance score
    pub avg_importance: Option<f32>,
}

/// Statistics about the memory store
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    pub total_memories: i64,
    pub total_tags: i64,
    pub total_crossrefs: i64,
    pub total_versions: i64,
    pub total_identities: i64,
    pub total_entities: i64,
    pub db_size_bytes: i64,
    pub memories_with_embeddings: i64,
    pub memories_pending_embedding: i64,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_pending: bool,
    pub storage_mode: String,
    pub schema_version: i32,
    pub workspaces: HashMap<String, i64>,
    pub type_counts: HashMap<String, i64>,
    pub tier_counts: HashMap<String, i64>,
}

/// A single operation within a [`CompactReport`] (issue #22).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactOp {
    /// Operation name, e.g. `prune_complete_queue`, `checkpoint_wal`, `vacuum`.
    pub name: String,
    /// Rows or bytes the operation would affect (or did affect when applied).
    pub candidates: i64,
    /// Whether the operation actually ran (false in dry-run or when skipped).
    pub applied: bool,
    /// Why the operation was skipped, when applicable.
    pub skipped_reason: Option<String>,
}

/// Result of a storage compaction (issue #22), in dry-run or apply mode.
///
/// Building it in dry-run mode never mutates the database. `VACUUM` is only
/// executed in apply mode and only when there is enough free disk space.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactReport {
    /// False for a dry-run (no mutations performed).
    pub applied: bool,
    /// On-disk size in bytes (`page_count * page_size`).
    pub db_size_bytes: i64,
    /// Size of the `-wal` sidecar file in bytes (0 if absent).
    pub wal_bytes: i64,
    /// Size of the `-shm` sidecar file in bytes (0 if absent).
    pub shm_bytes: i64,
    /// Pages on the freelist.
    pub freelist_count: i64,
    /// Bytes reclaimable by `VACUUM` (`freelist_count * page_size`).
    pub reclaimable_bytes: i64,
    /// Completed embedding-queue rows that can be pruned.
    pub queue_complete_prunable: i64,
    /// Failed embedding-queue rows that can be pruned.
    pub queue_failed_prunable: i64,
    /// Embedding rows whose owning memory no longer exists.
    pub orphan_embeddings: i64,
    /// Free bytes on the database's filesystem (-1 when unknown).
    pub free_space_bytes: i64,
    /// Whether a `VACUUM` is considered safe (enough free space for a rewrite).
    pub vacuum_safe: bool,
    /// Individual operations and their status.
    pub operations: Vec<CompactOp>,
}

/// Result of rebuilding derived indexes (issue #23), dry-run or applied.
///
/// Derived indexes (FTS, embeddings) are disposable; rebuilding never touches
/// canonical `memories` or their versions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RebuildReport {
    /// False for a dry-run (no mutations performed).
    pub applied: bool,
    /// Live canonical memories (preserved by any rebuild).
    pub memories: i64,
    pub fts_targeted: bool,
    pub fts_indexed_before: i64,
    pub fts_indexed_after: i64,
    pub fts_drift_before: i64,
    pub fts_drift_after: i64,
    pub fts_rebuilt: bool,
    pub embeddings_targeted: bool,
    /// Live memories that currently have an embedding.
    pub embeddings_present: i64,
    /// Live memories missing an embedding (requeue candidates).
    pub embeddings_missing: i64,
    /// Memories re-enqueued for embedding (apply mode).
    pub embeddings_requeued: i64,
}

/// Memory version for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVersion {
    /// Version number (1, 2, 3, ...)
    pub version: i32,
    /// Content at this version
    pub content: String,
    /// Tags at this version
    pub tags: Vec<String>,
    /// Metadata at this version
    pub metadata: HashMap<String, serde_json::Value>,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// Who created this version
    pub created_by: Option<String>,
    /// Summary of changes
    pub change_summary: Option<String>,
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub pending_changes: i64,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub is_syncing: bool,
}

/// Embedding queue status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStatus {
    pub memory_id: MemoryId,
    pub status: EmbeddingState,
    pub queued_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// State of embedding computation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingState {
    Pending,
    Processing,
    Complete,
    Failed,
}
