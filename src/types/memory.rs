use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::core::{default_confidence, default_strength, MemoryId};

/// Memory type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    #[default]
    Note,
    Todo,
    Issue,
    Decision,
    Preference,
    Learning,
    Context,
    Credential,
    Custom,
    /// Session transcript chunk (for conversation indexing)
    /// Default tier: Daily with 7-day TTL
    TranscriptChunk,
    // Cognitive memory types (Phase 1 - ENG-33)
    /// Events with temporal context (e.g., "User deployed v2.0 on Jan 15")
    /// Tracks when things happened and how long they took
    Episodic,
    /// Learned patterns and workflows (e.g., "When user asks about auth, check JWT first")
    /// Tracks success/failure counts for pattern effectiveness
    Procedural,
    /// Compressed summaries of other memories
    /// References the original via summary_of_id
    Summary,
    /// Conversation state snapshots for session resumption
    /// Replaces Context type for checkpoint-specific use
    Checkpoint,
    // Multimodal memory types
    /// Image memory with optional media_url pointing to the asset
    Image,
    /// Audio memory with optional media_url pointing to the asset
    Audio,
    /// Video memory with optional media_url pointing to the asset
    Video,
    /// Lightweight append-only fact for high-frequency session/watcher ingest
    Fact,
}

/// Memory tier for tiered storage (permanent vs ephemeral)
///
/// Tiers control memory lifetime:
/// - `Permanent`: Never expires, for important knowledge and decisions
/// - `Daily`: Auto-expires after TTL, for session context and scratch notes
///
/// Invariants enforced at write-time:
/// - Permanent tier: expires_at MUST be NULL
/// - Daily tier: expires_at MUST be set (defaults to created_at + 24h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemoryTier {
    /// Never expires (default)
    #[default]
    Permanent,
    /// Auto-expires after configurable TTL (default: 24 hours)
    Daily,
}

impl MemoryTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryTier::Permanent => "permanent",
            MemoryTier::Daily => "daily",
        }
    }

    /// Default TTL in seconds for daily tier
    pub fn default_ttl_seconds(&self) -> Option<i64> {
        match self {
            MemoryTier::Permanent => None,
            MemoryTier::Daily => Some(24 * 60 * 60), // 24 hours
        }
    }
}

impl std::str::FromStr for MemoryTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "permanent" => Ok(MemoryTier::Permanent),
            "daily" => Ok(MemoryTier::Daily),
            _ => Err(format!("Unknown memory tier: {}", s)),
        }
    }
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::Note => "note",
            MemoryType::Todo => "todo",
            MemoryType::Issue => "issue",
            MemoryType::Decision => "decision",
            MemoryType::Preference => "preference",
            MemoryType::Learning => "learning",
            MemoryType::Context => "context",
            MemoryType::Credential => "credential",
            MemoryType::Custom => "custom",
            MemoryType::TranscriptChunk => "transcript_chunk",
            MemoryType::Episodic => "episodic",
            MemoryType::Procedural => "procedural",
            MemoryType::Summary => "summary",
            MemoryType::Checkpoint => "checkpoint",
            MemoryType::Image => "image",
            MemoryType::Audio => "audio",
            MemoryType::Video => "video",
            MemoryType::Fact => "fact",
        }
    }

    /// Returns true if this type should be excluded from default search
    pub fn excluded_from_default_search(&self) -> bool {
        matches!(self, MemoryType::TranscriptChunk)
    }

    /// Returns true if this type is a multimodal (media) memory
    pub fn is_multimodal(&self) -> bool {
        matches!(
            self,
            MemoryType::Image | MemoryType::Audio | MemoryType::Video
        )
    }
}

impl std::str::FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "note" => Ok(MemoryType::Note),
            "todo" => Ok(MemoryType::Todo),
            "issue" => Ok(MemoryType::Issue),
            "decision" => Ok(MemoryType::Decision),
            "preference" => Ok(MemoryType::Preference),
            "learning" => Ok(MemoryType::Learning),
            "context" => Ok(MemoryType::Context),
            "credential" => Ok(MemoryType::Credential),
            "custom" => Ok(MemoryType::Custom),
            "transcript_chunk" => Ok(MemoryType::TranscriptChunk),
            "episodic" => Ok(MemoryType::Episodic),
            "procedural" => Ok(MemoryType::Procedural),
            "summary" => Ok(MemoryType::Summary),
            "checkpoint" => Ok(MemoryType::Checkpoint),
            "image" => Ok(MemoryType::Image),
            "audio" => Ok(MemoryType::Audio),
            "video" => Ok(MemoryType::Video),
            "fact" => Ok(MemoryType::Fact),
            _ => Err(format!("Unknown memory type: {}", s)),
        }
    }
}

/// Visibility levels for memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Private,
    Shared,
    Public,
}

/// Memory scope for isolating memories by user, session, agent, or global
///
/// This enables multi-tenant memory management where:
/// - `User`: Memories belong to a specific user across all sessions
/// - `Session`: Memories are temporary and bound to a conversation session
/// - `Agent`: Memories belong to a specific AI agent instance
/// - `Global`: Memories are shared across all scopes (system-wide)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemoryScope {
    /// User-scoped memory, persists across sessions
    User { user_id: String },
    /// Session-scoped memory, temporary for one conversation
    Session { session_id: String },
    /// Agent-scoped memory, belongs to a specific agent instance
    Agent { agent_id: String },
    /// Global scope, accessible by all (default for backward compatibility)
    #[default]
    Global,
}

impl MemoryScope {
    /// Create a user-scoped memory scope
    pub fn user(user_id: impl Into<String>) -> Self {
        MemoryScope::User {
            user_id: user_id.into(),
        }
    }

    /// Create a session-scoped memory scope
    pub fn session(session_id: impl Into<String>) -> Self {
        MemoryScope::Session {
            session_id: session_id.into(),
        }
    }

    /// Create an agent-scoped memory scope
    pub fn agent(agent_id: impl Into<String>) -> Self {
        MemoryScope::Agent {
            agent_id: agent_id.into(),
        }
    }

    /// Get the scope type as a string
    pub fn scope_type(&self) -> &'static str {
        match self {
            MemoryScope::User { .. } => "user",
            MemoryScope::Session { .. } => "session",
            MemoryScope::Agent { .. } => "agent",
            MemoryScope::Global => "global",
        }
    }

    /// Get the scope ID (user_id, session_id, agent_id, or None for global)
    pub fn scope_id(&self) -> Option<&str> {
        match self {
            MemoryScope::User { user_id } => Some(user_id.as_str()),
            MemoryScope::Session { session_id } => Some(session_id.as_str()),
            MemoryScope::Agent { agent_id } => Some(agent_id.as_str()),
            MemoryScope::Global => None,
        }
    }

    /// Check if this scope matches or is accessible from another scope
    /// Global scope can access everything, specific scopes can only access their own
    pub fn can_access(&self, other: &MemoryScope) -> bool {
        match (self, other) {
            (MemoryScope::Global, _) => true,
            (MemoryScope::User { user_id: a }, MemoryScope::User { user_id: b }) => a == b,
            (MemoryScope::Session { session_id: a }, MemoryScope::Session { session_id: b }) => {
                a == b
            }
            (MemoryScope::Agent { agent_id: a }, MemoryScope::Agent { agent_id: b }) => a == b,
            (_, MemoryScope::Global) => true,
            _ => false,
        }
    }
}

/// Cross-reference (relation) between memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Source memory ID
    pub from_id: MemoryId,
    /// Target memory ID
    pub to_id: MemoryId,
    /// Type of relationship
    pub edge_type: EdgeType,
    /// Similarity/relevance score (0.0 - 1.0)
    pub score: f32,
    /// Confidence level (decays over time)
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    /// User-adjustable importance
    #[serde(default = "default_strength")]
    pub strength: f32,
    /// How the relation was created
    #[serde(default)]
    pub source: RelationSource,
    /// Context explaining why the relation exists
    pub source_context: Option<String>,
    /// When the relation was created
    pub created_at: DateTime<Utc>,
    /// When the relation became valid
    pub valid_from: DateTime<Utc>,
    /// When the relation stopped being valid (None = still valid)
    pub valid_to: Option<DateTime<Utc>>,
    /// Exempt from confidence decay
    #[serde(default)]
    pub pinned: bool,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of edges/relationships between memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    #[default]
    RelatedTo,
    Supersedes,
    Contradicts,
    Implements,
    Extends,
    References,
    DerivedFrom,
    DependsOn,
    Blocks,
    FollowsUp,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::RelatedTo => "related_to",
            EdgeType::Supersedes => "supersedes",
            EdgeType::Contradicts => "contradicts",
            EdgeType::Implements => "implements",
            EdgeType::Extends => "extends",
            EdgeType::References => "references",
            EdgeType::DerivedFrom => "derived_from",
            EdgeType::DependsOn => "depends_on",
            EdgeType::Blocks => "blocks",
            EdgeType::FollowsUp => "follows_up",
        }
    }

    pub fn all() -> &'static [EdgeType] {
        &[
            EdgeType::RelatedTo,
            EdgeType::Supersedes,
            EdgeType::Contradicts,
            EdgeType::Implements,
            EdgeType::Extends,
            EdgeType::References,
            EdgeType::DerivedFrom,
            EdgeType::DependsOn,
            EdgeType::Blocks,
            EdgeType::FollowsUp,
        ]
    }
}

impl std::str::FromStr for EdgeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "related_to" | "related" => Ok(EdgeType::RelatedTo),
            "supersedes" => Ok(EdgeType::Supersedes),
            "contradicts" => Ok(EdgeType::Contradicts),
            "implements" => Ok(EdgeType::Implements),
            "extends" => Ok(EdgeType::Extends),
            "references" => Ok(EdgeType::References),
            "derived_from" | "derived" => Ok(EdgeType::DerivedFrom),
            "depends_on" => Ok(EdgeType::DependsOn),
            "blocks" => Ok(EdgeType::Blocks),
            "follows_up" => Ok(EdgeType::FollowsUp),
            _ => Err(format!("Unknown edge type: {}", s)),
        }
    }
}

/// How a relation was created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RelationSource {
    #[default]
    Auto,
    Manual,
    Llm,
}
