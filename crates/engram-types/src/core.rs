use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::memory::{MemoryScope, MemoryTier, MemoryType, Visibility};

/// Unique identifier for a memory
pub type MemoryId = i64;

/// A memory entry in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier
    pub id: MemoryId,
    /// Main content of the memory
    pub content: String,
    /// Memory type (e.g., "note", "todo", "issue", "decision")
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Arbitrary metadata as JSON
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Importance score (0.0 - 1.0)
    #[serde(default = "default_importance")]
    pub importance: f32,
    /// Number of times accessed
    #[serde(default)]
    pub access_count: i32,
    /// When the memory was created
    pub created_at: DateTime<Utc>,
    /// When the memory was last updated
    pub updated_at: DateTime<Utc>,
    /// When the memory was last accessed
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Owner ID for multi-user support
    pub owner_id: Option<String>,
    /// Visibility level
    #[serde(default)]
    pub visibility: Visibility,
    /// Memory scope for isolation (user/session/agent/global)
    #[serde(default)]
    pub scope: MemoryScope,
    /// Workspace for project-based isolation (normalized: lowercase, [a-z0-9_-], max 64 chars)
    #[serde(default = "default_workspace")]
    pub workspace: String,
    /// Memory tier for tiered storage (permanent vs daily)
    #[serde(default)]
    pub tier: MemoryTier,
    /// Current version number
    #[serde(default = "default_version")]
    pub version: i32,
    /// Whether embedding is computed
    #[serde(default)]
    pub has_embedding: bool,
    /// When the memory expires (None = never for permanent, required for daily)
    pub expires_at: Option<DateTime<Utc>>,
    /// Content hash for deduplication (SHA256 of normalized content)
    pub content_hash: Option<String>,
    // Phase 1 - Cognitive memory fields (ENG-33)
    /// Timestamp when the event occurred (for Episodic memories)
    pub event_time: Option<DateTime<Utc>>,
    /// Duration of the event in seconds (for Episodic memories)
    pub event_duration_seconds: Option<i64>,
    /// Pattern that triggers this procedure (for Procedural memories)
    pub trigger_pattern: Option<String>,
    /// Number of times this procedure succeeded (for Procedural memories)
    #[serde(default)]
    pub procedure_success_count: i32,
    /// Number of times this procedure failed (for Procedural memories)
    #[serde(default)]
    pub procedure_failure_count: i32,
    /// ID of the memory this is a summary of (for Summary memories)
    pub summary_of_id: Option<MemoryId>,
    // Phase 5 - Lifecycle management (ENG-37)
    /// Lifecycle state for memory management (active, stale, archived)
    #[serde(default)]
    pub lifecycle_state: LifecycleState,
    /// Memory stability factor earned through spaced reinforcement [1.0, 4.0]
    #[serde(default = "default_stability")]
    pub stability: f32,
    /// URL or local path to the primary media asset (for Image/Audio/Video memories)
    /// Format: local:///path/to/file or https://... or s3://...
    pub media_url: Option<String>,
}

/// Lifecycle state for memory management (Phase 5 - ENG-37)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleState {
    /// Normal state - included in search/list by default
    #[default]
    Active,
    /// Not accessed recently - included in search/list by default
    Stale,
    /// Compressed/summarized - EXCLUDED from search/list by default
    Archived,
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleState::Active => write!(f, "active"),
            LifecycleState::Stale => write!(f, "stale"),
            LifecycleState::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for LifecycleState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(LifecycleState::Active),
            "stale" => Ok(LifecycleState::Stale),
            "archived" => Ok(LifecycleState::Archived),
            _ => Err(format!("Unknown lifecycle state: {}", s)),
        }
    }
}

pub(crate) fn default_workspace() -> String {
    "default".to_string()
}

/// Reserved workspace names that cannot be used
pub const RESERVED_WORKSPACES: &[&str] = &["_system", "_archive"];

/// Maximum workspace name length
pub const MAX_WORKSPACE_LENGTH: usize = 64;

/// Workspace validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    Empty,
    TooLong,
    InvalidChars,
    Reserved,
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::Empty => write!(f, "Workspace name cannot be empty"),
            WorkspaceError::TooLong => write!(f, "Workspace name exceeds {} characters", MAX_WORKSPACE_LENGTH),
            WorkspaceError::InvalidChars => write!(f, "Workspace name can only contain lowercase letters, numbers, hyphens, and underscores"),
            WorkspaceError::Reserved => write!(f, "Workspace name is reserved"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

/// Normalize and validate a workspace name
///
/// Rules:
/// - Trim whitespace and convert to lowercase
/// - Only allow [a-z0-9_-] characters
/// - Max 64 characters
/// - Cannot start with underscore (reserved for system workspaces)
/// - "default" is allowed (it's the default workspace)
pub fn normalize_workspace(s: &str) -> Result<String, WorkspaceError> {
    let normalized = s.trim().to_lowercase();

    if normalized.is_empty() {
        return Err(WorkspaceError::Empty);
    }

    if normalized.len() > MAX_WORKSPACE_LENGTH {
        return Err(WorkspaceError::TooLong);
    }

    if !normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(WorkspaceError::InvalidChars);
    }

    if normalized.starts_with('_') || RESERVED_WORKSPACES.contains(&normalized.as_str()) {
        return Err(WorkspaceError::Reserved);
    }

    Ok(normalized)
}

pub(crate) fn default_importance() -> f32 {
    0.5
}

pub(crate) fn default_version() -> i32 {
    1
}

pub(crate) fn default_confidence() -> f32 {
    1.0
}

pub(crate) fn default_strength() -> f32 {
    1.0
}

pub(crate) fn default_stability() -> f32 {
    1.0
}
