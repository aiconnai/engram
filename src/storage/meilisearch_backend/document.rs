use crate::error::EngramError;
use crate::storage::queries::compute_dedup_hash;
use crate::types::{
    normalize_workspace, CreateMemoryInput, LifecycleState, Memory, MemoryScope, MemoryTier,
    Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct MeilisearchMemory {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
    pub memory_type: String,
    // Add missing fields to support full reconstruction
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub importance: f32,
    pub access_count: i32,
    pub last_accessed_at: Option<i64>,
    pub owner_id: Option<String>,
    pub visibility: String,
    pub scope: String,
    pub scope_id: Option<String>,
    pub workspace: String,
    pub tier: String,
    pub version: i32,
    pub has_embedding: bool,
    pub expires_at: Option<i64>,
    pub content_hash: Option<String>,
    // Phase 1 - Cognitive fields
    pub event_time: Option<i64>,
    pub event_duration_seconds: Option<i64>,
    pub trigger_pattern: Option<String>,
    pub procedure_success_count: i32,
    pub procedure_failure_count: i32,
    pub summary_of_id: Option<i64>,
    pub lifecycle_state: String,
}

impl From<&Memory> for MeilisearchMemory {
    fn from(m: &Memory) -> Self {
        Self {
            id: m.id,
            content: m.content.clone(),
            created_at: m.created_at.timestamp(),
            updated_at: m.updated_at.timestamp(),
            tags: m.tags.clone(),
            memory_type: m.memory_type.as_str().to_string(),
            metadata: Some(m.metadata.clone()),
            importance: m.importance,
            access_count: m.access_count,
            last_accessed_at: m.last_accessed_at.map(|t| t.timestamp()),
            owner_id: m.owner_id.clone(),
            visibility: visibility_to_str(m.visibility).to_string(),
            scope: m.scope.scope_type().to_string(),
            scope_id: m.scope.scope_id().map(|s| s.to_string()),
            workspace: m.workspace.clone(),
            tier: m.tier.as_str().to_string(),
            version: m.version,
            has_embedding: m.has_embedding,
            expires_at: m.expires_at.map(|t| t.timestamp()),
            content_hash: m.content_hash.clone(),
            event_time: m.event_time.map(|t| t.timestamp()),
            event_duration_seconds: m.event_duration_seconds,
            trigger_pattern: m.trigger_pattern.clone(),
            procedure_success_count: m.procedure_success_count,
            procedure_failure_count: m.procedure_failure_count,
            summary_of_id: m.summary_of_id,
            lifecycle_state: m.lifecycle_state.to_string(),
        }
    }
}

pub(super) fn timestamp_to_datetime(timestamp: i64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(chrono::Utc::now)
}

pub(super) fn opt_timestamp_to_datetime(
    timestamp: Option<i64>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    timestamp.and_then(|t| chrono::DateTime::from_timestamp(t, 0))
}

pub(super) fn scope_from_parts(scope: &str, scope_id: Option<String>) -> MemoryScope {
    match (scope, scope_id) {
        ("user", Some(id)) => MemoryScope::User { user_id: id },
        ("session", Some(id)) => MemoryScope::Session { session_id: id },
        ("agent", Some(id)) => MemoryScope::Agent { agent_id: id },
        _ => MemoryScope::Global,
    }
}

pub(super) fn visibility_from_str(value: &str) -> Visibility {
    match value {
        "shared" => Visibility::Shared,
        "public" => Visibility::Public,
        _ => Visibility::Private,
    }
}

pub(super) fn build_memory_from_doc(doc: MeilisearchMemory) -> Memory {
    Memory {
        id: doc.id,
        content: doc.content,
        memory_type: doc.memory_type.parse().unwrap_or_default(),
        tags: doc.tags,
        metadata: doc.metadata.unwrap_or_default(),
        created_at: timestamp_to_datetime(doc.created_at),
        updated_at: timestamp_to_datetime(doc.updated_at),
        last_accessed_at: opt_timestamp_to_datetime(doc.last_accessed_at),
        importance: doc.importance,
        access_count: doc.access_count,
        owner_id: doc.owner_id,
        visibility: visibility_from_str(&doc.visibility),
        scope: scope_from_parts(&doc.scope, doc.scope_id),
        workspace: doc.workspace,
        tier: doc.tier.parse().unwrap_or_default(),
        version: doc.version,
        has_embedding: doc.has_embedding,
        expires_at: opt_timestamp_to_datetime(doc.expires_at),
        content_hash: doc.content_hash,
        event_time: opt_timestamp_to_datetime(doc.event_time),
        event_duration_seconds: doc.event_duration_seconds,
        trigger_pattern: doc.trigger_pattern,
        procedure_success_count: doc.procedure_success_count,
        procedure_failure_count: doc.procedure_failure_count,
        summary_of_id: doc.summary_of_id,
        lifecycle_state: doc.lifecycle_state.parse().unwrap_or_default(),
        media_url: None,
    }
}

pub(super) fn visibility_to_str(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Shared => "shared",
        Visibility::Public => "public",
    }
}

pub(super) fn generate_memory_id() -> i64 {
    (rand::random::<u64>() & i64::MAX as u64) as i64
}

pub(super) fn build_memory_from_input(
    id: i64,
    input: CreateMemoryInput,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Memory, EngramError> {
    let workspace = normalize_workspace(input.workspace.as_deref().unwrap_or("default"))
        .map_err(|e| EngramError::InvalidInput(e.to_string()))?;

    let expires_at = match input.tier {
        MemoryTier::Permanent => {
            if input.ttl_seconds.is_some() && input.ttl_seconds != Some(0) {
                return Err(EngramError::InvalidInput(
                    "Permanent tier memories cannot have a TTL. Use Daily tier for expiring memories.".to_string(),
                ));
            }
            None
        }
        MemoryTier::Daily => {
            let ttl = input.ttl_seconds.filter(|&t| t > 0).unwrap_or(86400);
            Some(now + chrono::Duration::seconds(ttl))
        }
    };

    let content_hash = Some(compute_dedup_hash(&input.content));

    Ok(Memory {
        id,
        content: input.content,
        memory_type: input.memory_type,
        tags: input.tags,
        metadata: input.metadata,
        created_at: now,
        updated_at: now,
        last_accessed_at: None,
        importance: input.importance.unwrap_or(0.5),
        access_count: 0,
        owner_id: None,
        visibility: Visibility::Private,
        scope: input.scope,
        workspace,
        tier: input.tier,
        version: 1,
        has_embedding: false,
        expires_at,
        content_hash,
        event_time: input.event_time,
        event_duration_seconds: input.event_duration_seconds,
        trigger_pattern: input.trigger_pattern,
        procedure_success_count: 0,
        procedure_failure_count: 0,
        summary_of_id: input.summary_of_id,
        lifecycle_state: LifecycleState::Active,
        media_url: None,
    })
}
