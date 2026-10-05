//! Core domain types and data models for Engram.
//!
//! Provides pure-Rust data structures with zero database, networking, or I/O dependencies.

pub mod config;
pub mod core;
pub mod memory;
pub mod search;
pub mod stats;

pub use config::{
    CreateCrossRefInput, CreateMemoryInput, DedupMode, EmbeddingConfig, ListOptions, StorageConfig,
    StorageMode, UpdateMemoryInput,
};
pub use core::{
    normalize_workspace, LifecycleState, Memory, MemoryId, WorkspaceError, MAX_WORKSPACE_LENGTH,
    RESERVED_WORKSPACES,
};
pub use memory::{
    CrossReference, EdgeType, MemoryScope, MemoryTier, MemoryType, RelationSource, Visibility,
};
pub use search::{MatchInfo, SearchOptions, SearchResult, SearchStrategy, SortField, SortOrder};
pub use stats::{
    CompactOp, CompactReport, EmbeddingState, EmbeddingStatus, MemoryVersion, RebuildReport,
    StorageStats, SyncStatus, WorkspaceStats,
};
