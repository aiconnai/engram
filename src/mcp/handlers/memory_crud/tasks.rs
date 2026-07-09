//! Task-shaped creation tools (todo, issue).
use serde_json::{json, Value};

use super::super::HandlerContext;
use super::create::memory_create;
use crate::types::*;

pub fn create_todo(ctx: &HandlerContext, params: Value) -> Value {
    let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let priority = params
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("priority".to_string(), json!(priority));
    if let Some(due) = params.get("due_date") {
        metadata.insert("due_date".to_string(), due.clone());
    }

    let importance: f32 = match priority {
        "critical" => 1.0,
        "high" => 0.8,
        "medium" => 0.5,
        "low" => 0.3,
        _ => 0.5,
    };

    let input = CreateMemoryInput {
        content: content.to_string(),
        memory_type: MemoryType::Todo,
        tags,
        metadata,
        importance: Some(importance),
        scope: Default::default(),
        workspace: None,
        tier: Default::default(),
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    memory_create(ctx, serde_json::to_value(input).unwrap_or_default())
}
// end of create_issue

pub fn create_issue(ctx: &HandlerContext, params: Value) -> Value {
    let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let description = params
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let severity = params
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let content = if description.is_empty() {
        title.to_string()
    } else {
        format!("{}\n\n{}", title, description)
    };

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("severity".to_string(), json!(severity));
    metadata.insert("title".to_string(), json!(title));

    let importance: f32 = match severity {
        "critical" => 1.0,
        "high" => 0.8,
        "medium" => 0.5,
        "low" => 0.3,
        _ => 0.5,
    };

    let input = CreateMemoryInput {
        content,
        memory_type: MemoryType::Issue,
        tags,
        metadata,
        importance: Some(importance),
        scope: Default::default(),
        workspace: None,
        tier: Default::default(),
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    memory_create(ctx, serde_json::to_value(input).unwrap_or_default())
}
