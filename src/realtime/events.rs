//! Real-time event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::MemoryId;

/// Types of real-time events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    MemoryCreated,
    MemoryUpdated,
    MemoryDeleted,
    CrossrefCreated,
    CrossrefDeleted,
    SyncStarted,
    SyncCompleted,
    SyncFailed,
    Progress,
}

/// A real-time event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeEvent {
    /// Sequential event ID, stamped by `RealtimeManager::broadcast`.
    /// `None` for events that have not yet been processed by the manager.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_id: Option<u64>,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Related memory ID (if applicable)
    pub memory_id: Option<MemoryId>,
    /// Preview of content (for created/updated)
    pub preview: Option<String>,
    /// List of changed fields (for updates)
    pub changes: Option<Vec<String>>,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

impl RealtimeEvent {
    /// Create a memory created event with an authoritative workspace.
    pub fn memory_created(id: MemoryId, preview: String, workspace: impl Into<String>) -> Self {
        Self {
            seq_id: None,
            event_type: EventType::MemoryCreated,
            timestamp: Utc::now(),
            memory_id: Some(id),
            preview: Some(truncate(&preview, 100)),
            changes: None,
            data: Some(serde_json::json!({ "workspace": workspace.into() })),
        }
    }

    /// Create a memory updated event with an authoritative workspace.
    pub fn memory_updated(
        id: MemoryId,
        changes: Vec<String>,
        workspace: impl Into<String>,
    ) -> Self {
        Self {
            seq_id: None,
            event_type: EventType::MemoryUpdated,
            timestamp: Utc::now(),
            memory_id: Some(id),
            preview: None,
            changes: Some(changes),
            data: Some(serde_json::json!({ "workspace": workspace.into() })),
        }
    }

    /// Create a memory deleted event with an authoritative workspace.
    pub fn memory_deleted(id: MemoryId, workspace: impl Into<String>) -> Self {
        Self {
            seq_id: None,
            event_type: EventType::MemoryDeleted,
            timestamp: Utc::now(),
            memory_id: Some(id),
            preview: None,
            changes: None,
            data: Some(serde_json::json!({ "workspace": workspace.into() })),
        }
    }

    /// Create a sync completed event
    pub fn sync_completed(direction: &str, changes: i64) -> Self {
        Self {
            seq_id: None,
            event_type: EventType::SyncCompleted,
            timestamp: Utc::now(),
            memory_id: None,
            preview: None,
            changes: None,
            data: Some(serde_json::json!({
                "direction": direction,
                "changes": changes,
            })),
        }
    }

    /// Create a sync failed event
    pub fn sync_failed(error: &str) -> Self {
        Self {
            seq_id: None,
            event_type: EventType::SyncFailed,
            timestamp: Utc::now(),
            memory_id: None,
            preview: None,
            changes: None,
            data: Some(serde_json::json!({
                "error": error,
            })),
        }
    }

    /// Create a progress event correlated with a progress token.
    pub fn progress(
        token: impl Into<String>,
        progress: u64,
        total: Option<u64>,
        message: Option<String>,
        workspace: Option<String>,
    ) -> Self {
        let mut data = serde_json::json!({
            "progress_token": token.into(),
            "progress": progress,
        });
        if let Some(t) = total {
            data["total"] = serde_json::json!(t);
        }
        if let Some(m) = &message {
            data["message"] = serde_json::json!(m);
        }
        if let Some(ws) = workspace {
            data["workspace"] = serde_json::json!(ws);
        }
        Self {
            seq_id: None,
            event_type: EventType::Progress,
            timestamp: Utc::now(),
            memory_id: None,
            preview: message,
            changes: None,
            data: Some(data),
        }
    }

    /// Return the authoritative event workspace, when the producer supplied one.
    pub fn workspace(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|data| data.get("workspace"))
            .and_then(serde_json::Value::as_str)
    }
}

/// Truncate string for preview (UTF-8 safe)
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        // Take max - 3 chars safely, then append "..."
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Subscription filter for events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    /// Only events for specific memory IDs
    pub memory_ids: Option<Vec<MemoryId>>,
    /// Only events with specific tags
    pub tags: Option<Vec<String>>,
    /// Only specific event types
    pub event_types: Option<Vec<EventType>>,
}

impl SubscriptionFilter {
    /// Check if an event matches this filter
    pub fn matches(&self, event: &RealtimeEvent) -> bool {
        // Check event type filter
        if let Some(ref types) = self.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        // Check memory ID filter
        if let Some(ref ids) = self.memory_ids {
            if let Some(event_id) = event.memory_id {
                if !ids.contains(&event_id) {
                    return false;
                }
            }
        }

        // Tags filter would require additional context
        // (memory tags aren't included in events by default)

        true
    }
}
