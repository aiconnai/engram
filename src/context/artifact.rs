//! Policy-controlled operational context artifacts.
//!
//! Artifacts are not ordinary memories. The public metadata type deliberately
//! excludes raw bytes so search/list surfaces cannot expose captured command
//! output, logs, traces, or other high-entropy operational material by default.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::{EngramError, Result};

/// Redaction state for an artifact.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRedactionStatus {
    #[default]
    Unknown,
    Passed,
    Redacted,
    NotRequired,
    Rejected,
}

impl ArtifactRedactionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Passed => "passed",
            Self::Redacted => "redacted",
            Self::NotRequired => "not_required",
            Self::Rejected => "rejected",
        }
    }

    /// Whether raw artifact bytes may be persisted under this redaction state.
    pub fn allows_raw_storage(self) -> bool {
        matches!(self, Self::Passed | Self::Redacted | Self::NotRequired)
    }
}

impl FromStr for ArtifactRedactionStatus {
    type Err = EngramError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "unknown" => Ok(Self::Unknown),
            "passed" => Ok(Self::Passed),
            "redacted" => Ok(Self::Redacted),
            "not_required" => Ok(Self::NotRequired),
            "rejected" => Ok(Self::Rejected),
            other => Err(EngramError::InvalidInput(format!(
                "Unknown artifact redaction status: {other}"
            ))),
        }
    }
}

/// Access boundary enforced before raw artifact retrieval.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactAccessPolicy {
    #[default]
    SameSession,
    SameTask,
    SameAgent,
    Repo,
    Public,
}

impl ArtifactAccessPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SameSession => "same_session",
            Self::SameTask => "same_task",
            Self::SameAgent => "same_agent",
            Self::Repo => "repo",
            Self::Public => "public",
        }
    }
}

impl FromStr for ArtifactAccessPolicy {
    type Err = EngramError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "same_session" => Ok(Self::SameSession),
            "same_task" => Ok(Self::SameTask),
            "same_agent" => Ok(Self::SameAgent),
            "repo" => Ok(Self::Repo),
            "public" => Ok(Self::Public),
            other => Err(EngramError::InvalidInput(format!(
                "Unknown artifact access policy: {other}"
            ))),
        }
    }
}

/// Retention policy supplied by callers when creating artifacts.
///
/// The default is pointer-only: raw bytes are discarded even if the caller
/// provides them. Persisting raw bytes requires `retain_raw = true` and a
/// redaction status that allows storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRetentionPolicy {
    pub policy_name: String,
    pub retain_raw: bool,
    pub redaction_status: ArtifactRedactionStatus,
    pub ttl_seconds: Option<i64>,
    pub stale_after_seconds: Option<i64>,
    pub access_policy: ArtifactAccessPolicy,
}

impl ArtifactRetentionPolicy {
    pub fn pointer_only(policy_name: impl Into<String>) -> Self {
        Self {
            policy_name: policy_name.into(),
            retain_raw: false,
            redaction_status: ArtifactRedactionStatus::NotRequired,
            ttl_seconds: None,
            stale_after_seconds: None,
            access_policy: ArtifactAccessPolicy::SameSession,
        }
    }

    pub fn validate_raw_storage(&self, has_raw_content: bool) -> Result<()> {
        if self.retain_raw && has_raw_content && !self.redaction_status.allows_raw_storage() {
            return Err(EngramError::InvalidInput(format!(
                "Artifact raw retention requires redaction to pass first; status={}",
                self.redaction_status.as_str()
            )));
        }
        Ok(())
    }
}

impl Default for ArtifactRetentionPolicy {
    fn default() -> Self {
        Self::pointer_only("pointer_only")
    }
}

/// Public artifact metadata. Raw content is intentionally absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextArtifact {
    pub id: String,
    pub source_event_id: Option<i64>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub kind: String,
    pub label: Option<String>,
    pub uri: Option<String>,
    pub media_type: Option<String>,
    pub content_sha256: Option<String>,
    pub byte_len: Option<i64>,
    pub redaction_status: ArtifactRedactionStatus,
    pub retention_policy: String,
    pub access_policy: ArtifactAccessPolicy,
    pub retain_raw: bool,
    pub stale_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl ContextArtifact {
    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at
            .as_ref()
            .is_some_and(|expires_at| expires_at <= &now)
    }

    pub fn is_stale_at(&self, now: DateTime<Utc>) -> bool {
        self.stale_at
            .as_ref()
            .is_some_and(|stale_at| stale_at <= &now)
    }

    pub fn access_allowed(&self, request: &ArtifactRetrievalRequest) -> bool {
        match self.access_policy {
            ArtifactAccessPolicy::Public => true,
            ArtifactAccessPolicy::SameAgent => same_opt(
                self.agent_id.as_deref(),
                request.requester_agent_id.as_deref(),
            ),
            ArtifactAccessPolicy::SameSession => {
                same_opt(self.session_id.as_deref(), request.session_id.as_deref())
            }
            ArtifactAccessPolicy::SameTask => {
                same_opt(self.task_id.as_deref(), request.task_id.as_deref())
            }
            ArtifactAccessPolicy::Repo => {
                same_opt(self.repo_id.as_deref(), request.repo_id.as_deref())
                    || same_opt(
                        self.workspace_path_hash.as_deref(),
                        request.workspace_path_hash.as_deref(),
                    )
            }
        }
    }
}

fn same_opt(left: Option<&str>, right: Option<&str>) -> bool {
    matches!((left, right), (Some(l), Some(r)) if !l.is_empty() && l == r)
}

/// Artifact creation request. Raw content is accepted here, but storage policy
/// decides whether it is discarded or retained.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewContextArtifact {
    pub id: Option<String>,
    pub source_event_id: Option<i64>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub kind: String,
    pub label: Option<String>,
    pub uri: Option<String>,
    pub media_type: Option<String>,
    pub content_sha256: Option<String>,
    pub byte_len: Option<i64>,
    pub raw_content: Option<Vec<u8>>,
    pub retention: ArtifactRetentionPolicy,
    pub metadata: serde_json::Value,
}

/// Explicit raw artifact retrieval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRetrievalRequest {
    pub artifact_id: String,
    pub requester_agent_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub max_bytes: Option<usize>,
    pub allow_stale: bool,
    pub reason: Option<String>,
}

/// Raw artifact retrieval response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedContextArtifact {
    pub artifact: ContextArtifact,
    pub content: Vec<u8>,
    pub returned_bytes: usize,
    pub original_bytes: usize,
    pub truncated: bool,
}
