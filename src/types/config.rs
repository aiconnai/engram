use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::core::MemoryId;
use super::memory::{EdgeType, MemoryScope, MemoryTier, MemoryType};
use super::search::SortField;
use super::search::SortOrder;

/// Configuration for the storage engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path to SQLite database
    pub db_path: String,
    /// Storage mode (local or cloud-safe)
    #[serde(default)]
    pub storage_mode: StorageMode,
    /// Cloud storage URI (s3://bucket/path)
    pub cloud_uri: Option<String>,
    /// Enable encryption for cloud storage
    #[serde(default)]
    pub encrypt_cloud: bool,
    /// Confidence decay half-life in days
    #[serde(default = "default_half_life")]
    pub confidence_half_life_days: f32,
    /// Auto-sync after writes
    #[serde(default = "default_true")]
    pub auto_sync: bool,
    /// Sync debounce delay in milliseconds
    #[serde(default = "default_sync_debounce")]
    pub sync_debounce_ms: u64,
}

fn default_half_life() -> f32 {
    30.0
}

fn default_true() -> bool {
    true
}

fn default_sync_debounce() -> u64 {
    5000
}

/// Storage mode for SQLite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StorageMode {
    #[default]
    Local,
    CloudSafe,
}

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Model to use: "openai", "local", "tfidf"
    pub model: String,
    /// OpenAI API key (for openai model)
    pub api_key: Option<String>,
    /// OpenAI-compatible API base URL (for OpenRouter, Azure, etc.)
    /// Default: `<https://api.openai.com/v1>`
    pub base_url: Option<String>,
    /// Embedding model name override (e.g., "text-embedding-3-small", "openai/text-embedding-3-small")
    pub embedding_model: Option<String>,
    /// Local model path (for local model)
    pub model_path: Option<String>,
    /// Embedding dimensions (must match model output)
    /// Default: 384 for TF-IDF, 1536 for text-embedding-3-small
    pub dimensions: usize,
    /// Batch size for async queue
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    100
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: "tfidf".to_string(),
            api_key: None,
            base_url: None,
            embedding_model: None,
            model_path: None,
            dimensions: 384,
            batch_size: 100,
        }
    }
}

/// Deduplication mode when creating memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DedupMode {
    /// Return error if duplicate found
    Reject,
    /// Merge with existing memory (update metadata, tags)
    Merge,
    /// Silently skip creation, return existing memory
    Skip,
    /// Allow duplicate creation (default, current behavior)
    #[default]
    Allow,
}

/// Input for creating a new memory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateMemoryInput {
    pub content: String,
    #[serde(default, alias = "type")]
    pub memory_type: MemoryType,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    pub importance: Option<f32>,
    /// Memory scope for isolation (user/session/agent/global)
    #[serde(default)]
    pub scope: MemoryScope,
    /// Workspace for project-based isolation (will be normalized)
    pub workspace: Option<String>,
    /// Memory tier (permanent or daily)
    #[serde(default)]
    pub tier: MemoryTier,
    /// Defer embedding computation to background queue
    #[serde(default)]
    pub defer_embedding: bool,
    /// Time-to-live in seconds (None = use tier default, Some(0) = never expires)
    /// For daily tier: defaults to 24 hours if not specified
    /// For permanent tier: must be None (enforced at write-time)
    pub ttl_seconds: Option<i64>,
    /// Deduplication mode (default: allow)
    #[serde(default)]
    pub dedup_mode: DedupMode,
    /// Similarity threshold for semantic deduplication (0.0-1.0, default: 0.95)
    pub dedup_threshold: Option<f32>,
    // Phase 1 - Cognitive memory fields (ENG-33)
    /// Timestamp when the event occurred (for Episodic memories)
    pub event_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration of the event in seconds (for Episodic memories)
    pub event_duration_seconds: Option<i64>,
    /// Pattern that triggers this procedure (for Procedural memories)
    pub trigger_pattern: Option<String>,
    /// ID of the memory this is a summary of (for Summary memories)
    pub summary_of_id: Option<MemoryId>,
    /// URL or local path to the primary media asset (for Image/Audio/Video memories)
    pub media_url: Option<String>,
}

/// Input for updating a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryInput {
    pub content: Option<String>,
    #[serde(alias = "type")]
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub importance: Option<f32>,
    /// Memory scope for isolation (user/session/agent/global)
    pub scope: Option<MemoryScope>,
    /// Time-to-live in seconds (None = no change, Some(0) = remove expiration, Some(n) = set to n seconds from now)
    pub ttl_seconds: Option<i64>,
    // Phase 1 - Cognitive memory fields (ENG-33)
    /// Timestamp when the event occurred (for Episodic memories)
    /// Use Some(None) to clear the value
    pub event_time: Option<Option<chrono::DateTime<chrono::Utc>>>,
    /// Pattern that triggers this procedure (for Procedural memories)
    /// Use Some(None) to clear the value
    pub trigger_pattern: Option<Option<String>>,
    /// URL or local path to the primary media asset (for Image/Audio/Video memories)
    /// Use Some(None) to clear, Some(Some(url)) to set
    pub media_url: Option<Option<String>>,
}

/// Input for creating a cross-reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCrossRefInput {
    pub from_id: MemoryId,
    pub to_id: MemoryId,
    #[serde(default)]
    pub edge_type: EdgeType,
    pub strength: Option<f32>,
    pub source_context: Option<String>,
    #[serde(default)]
    pub pinned: bool,
}

/// Options for listing memories
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListOptions {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub tags: Option<Vec<String>>,
    #[serde(alias = "type")]
    pub memory_type: Option<MemoryType>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
    /// Legacy metadata filter (simple key-value equality)
    /// Deprecated: Use `filter` for advanced queries
    pub metadata_filter: Option<HashMap<String, serde_json::Value>>,
    /// Filter by memory scope
    pub scope: Option<MemoryScope>,
    /// Filter by workspace (single workspace)
    pub workspace: Option<String>,
    /// Filter by multiple workspaces (OR logic)
    pub workspaces: Option<Vec<String>>,
    /// Filter by memory tier
    pub tier: Option<MemoryTier>,
    /// Advanced filter expression with AND/OR/comparison operators (RML-932)
    /// Example: {"AND": [{"metadata.project": {"eq": "engram"}}, {"importance": {"gte": 0.5}}]}
    pub filter: Option<serde_json::Value>,
    // Phase 5 - Lifecycle management (ENG-37)
    /// Include archived memories in results (default: false)
    #[serde(default)]
    pub include_archived: bool,
}
