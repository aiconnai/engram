//! Types for operational-context records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct ContextRecordRequest {
    pub source: String,
    #[serde(default)]
    pub source_version: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub workspace_path_hash: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub git_branch: Option<String>,
    #[serde(default)]
    pub worktree_name: Option<String>,
    #[serde(default)]
    pub commit_hash: Option<String>,
    pub session_id: String,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub event_type: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub command_name: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i64>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub key_errors: Vec<String>,
    #[serde(default)]
    pub touched_files: Vec<String>,
    #[serde(default)]
    pub reducer: Option<ContextReducerInput>,
    #[serde(default)]
    pub external_reducer: Option<String>,
    #[serde(default)]
    pub raw_pointer: Option<String>,
    #[serde(default)]
    pub external_unverified: Option<bool>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub retention_policy: Option<String>,
    #[serde(default)]
    pub raw_artifact_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextReducerInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub external_reducer: Option<String>,
    #[serde(default)]
    pub raw_pointer: Option<String>,
    #[serde(default)]
    pub lossy: Option<bool>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub structured_facts: Option<Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub tokens_raw_est: Option<i64>,
    #[serde(default)]
    pub tokens_compact_est: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordCreatedIds {
    pub event_id: i64,
    pub summary_id: Option<i64>,
    pub raw_artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceMetadata {
    pub source: String,
    pub source_version: Option<String>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordMetrics {
    pub estimated: bool,
    pub method: String,
    pub observed_input_tokens_est: Option<i64>,
    pub summary_tokens_est: Option<i64>,
    pub stored_artifact_tokens_est: Option<i64>,
    pub estimated_savings_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordResponse {
    pub created_ids: ContextRecordCreatedIds,
    pub redaction_status: String,
    pub retention_policy: String,
    pub provenance: ProvenanceMetadata,
    pub metrics: ContextRecordMetrics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextRecordArtifactRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub source_event_id: Option<i64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_version: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub workspace_path_hash: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub raw_pointer: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub raw_content: Option<String>,
    #[serde(default)]
    pub content_sha256: Option<String>,
    #[serde(default)]
    pub byte_len: Option<i64>,
    #[serde(default)]
    pub retention_policy: Option<String>,
    #[serde(default)]
    pub access_policy: Option<String>,
    #[serde(default)]
    pub retain_raw: Option<bool>,
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
    #[serde(default)]
    pub stale_after_seconds: Option<i64>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordArtifactResponse {
    pub artifact_id: String,
    pub storage_kind: String,
    pub redaction_status: String,
    pub retention_policy: String,
    pub provenance: ProvenanceMetadata,
    pub metrics: ContextRecordMetrics,
}
