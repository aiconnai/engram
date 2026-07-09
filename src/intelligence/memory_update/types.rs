use serde::{Deserialize, Serialize};

use crate::error::{EngramError, Result};

// Public types
// =============================================================================

/// Classifies the relationship between new content and an existing memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// New content directly contradicts the existing memory
    /// (e.g., negation keywords + shared entities).
    Contradiction,
    /// New content adds new predicates about the same entities without
    /// contradicting them.
    Supplement,
    /// New content explicitly corrects the existing memory
    /// (e.g., "actually", "correction", "update").
    Correction,
    /// The existing memory references old dates while new content uses
    /// temporal markers like "now" or "currently".
    Obsolescence,
}

impl ConflictType {
    pub fn as_str(self) -> &'static str {
        match self {
            ConflictType::Contradiction => "contradiction",
            ConflictType::Supplement => "supplement",
            ConflictType::Correction => "correction",
            ConflictType::Obsolescence => "obsolescence",
        }
    }
}

impl std::str::FromStr for ConflictType {
    type Err = EngramError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "contradiction" => Ok(ConflictType::Contradiction),
            "supplement" => Ok(ConflictType::Supplement),
            "correction" => Ok(ConflictType::Correction),
            "obsolescence" => Ok(ConflictType::Obsolescence),
            _ => Err(EngramError::InvalidInput(format!(
                "Unknown conflict type: {}",
                s
            ))),
        }
    }
}

/// The action to take when an update is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateAction {
    /// Overwrite the existing memory content with the new content.
    Replace,
    /// Append the new content to the existing memory.
    Merge,
    /// Change the memory type to `archived` so it is preserved but deprioritised.
    Archive,
    /// Add a `needs-review` tag so a human can inspect the conflict.
    Flag,
}

impl UpdateAction {
    pub fn as_str(self) -> &'static str {
        match self {
            UpdateAction::Replace => "replace",
            UpdateAction::Merge => "merge",
            UpdateAction::Archive => "archive",
            UpdateAction::Flag => "flag",
        }
    }
}

impl std::str::FromStr for UpdateAction {
    type Err = EngramError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "replace" => Ok(UpdateAction::Replace),
            "merge" => Ok(UpdateAction::Merge),
            "archive" => Ok(UpdateAction::Archive),
            "flag" => Ok(UpdateAction::Flag),
            _ => Err(EngramError::InvalidInput(format!(
                "Unknown update action: {}",
                s
            ))),
        }
    }
}

/// A candidate memory that may need to be updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCandidate {
    /// ID of the existing memory that may need updating.
    pub existing_id: i64,
    /// How the new content relates to the existing memory.
    pub conflict_type: ConflictType,
    /// Confidence score in the range [0.0, 1.0].
    pub confidence: f32,
    /// Suggested action to resolve the detected conflict.
    pub suggested_action: UpdateAction,
    /// Human-readable explanation for the suggestion.
    pub reason: String,
}

/// Result of applying an update to an existing memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// ID of the memory that was updated.
    pub memory_id: i64,
    /// The action that was applied.
    pub action_taken: UpdateAction,
    /// SHA-256 hex digest of the content *before* the update.
    pub old_content_hash: String,
    /// SHA-256 hex digest of the content *after* the update.
    pub new_content_hash: String,
}

/// A stored entry in the `update_log` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogEntry {
    /// Database-assigned id.
    pub id: i64,
    /// Memory that was updated.
    pub memory_id: i64,
    /// Action that was applied.
    pub action: UpdateAction,
    /// Content hash before the update.
    pub old_hash: String,
    /// Content hash after the update.
    pub new_hash: String,
    /// Human-readable reason for the update.
    pub reason: String,
    /// RFC3339 UTC timestamp.
    pub timestamp: String,
}

// =============================================================================
