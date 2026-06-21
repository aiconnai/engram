// PostToolUse hook handler.
// Triggered after a tool use completes to apply best-effort policy reinforcement.

use super::{HookContext, HookResult};
use crate::storage::{queries::record_reinforcement, Storage};
use crate::Result;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

const POST_TOOL_REINFORCEMENT_BOOST: f32 = 0.05;

/// Handler for the PostToolUse hook.
///
/// The handler reinforces memory policy from explicit memory IDs in hook
/// metadata. It does not synthesize or create memories from tool output.
#[derive(Default)]
pub struct PostToolUseHandler {
    /// Optional storage handle for best-effort policy reinforcement.
    pub storage: Option<Storage>,
}

impl PostToolUseHandler {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Some(storage),
        }
    }

    pub fn handle(&self, _hook: super::LifecycleHook, context: &HookContext) -> Result<HookResult> {
        eprintln!(
            "[Hook] PostToolUse: tool={:?}",
            context.metadata.get("tool_name")
        );

        self.reinforce_policy_from_metadata(context);

        Ok(HookResult::Continue)
    }

    fn reinforce_policy_from_metadata(&self, context: &HookContext) {
        let Some(storage) = self.storage.as_ref() else {
            return;
        };
        let Some(tool_name) = context.metadata.get("tool_name").and_then(Value::as_str) else {
            return;
        };

        if !is_policy_reinforcement_tool(tool_name) || metadata_indicates_failure(&context.metadata)
        {
            return;
        }

        let memory_ids = collect_memory_ids_from_metadata(&context.metadata);
        if memory_ids.is_empty() {
            return;
        }

        let triggered_by = format!("post_tool_use:{}", tool_name);
        let result = storage.with_connection(|conn| {
            for memory_id in &memory_ids {
                if let Err(e) = record_reinforcement(
                    conn,
                    *memory_id,
                    POST_TOOL_REINFORCEMENT_BOOST,
                    &triggered_by,
                ) {
                    tracing::warn!(
                        target = "engram::hooks::post_tool_use",
                        memory_id = *memory_id,
                        tool_name,
                        error = %e,
                        "failed to record post-tool-use policy reinforcement; continuing"
                    );
                }
            }
            Ok(())
        });

        if let Err(e) = result {
            tracing::warn!(
                target = "engram::hooks::post_tool_use",
                tool_name,
                error = %e,
                "failed to access storage for post-tool-use policy reinforcement; continuing"
            );
        }
    }
}

pub fn create_handler(
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = PostToolUseHandler::default();
        handler.handle(hook, context)
    }
}

pub fn create_handler_with_storage(
    storage: Storage,
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = PostToolUseHandler::new(storage.clone());
        handler.handle(hook, context)
    }
}

fn is_policy_reinforcement_tool(tool_name: &str) -> bool {
    matches!(tool_name, "memory_search" | "memory_get" | "memory_expand")
        || tool_name.starts_with("memory_policy")
        || tool_name.starts_with("retention_policy")
        || tool_name.contains("_policy_")
}

fn metadata_indicates_failure(metadata: &HashMap<String, Value>) -> bool {
    if metadata
        .get("success")
        .or_else(|| metadata.get("ok"))
        .and_then(Value::as_bool)
        == Some(false)
    {
        return true;
    }

    for key in ["error", "tool_error"] {
        if metadata.get(key).is_some_and(|value| !value.is_null()) {
            return true;
        }
    }

    if metadata.get("status").is_some_and(is_error_status) {
        return true;
    }

    if let Some(output) = metadata.get("tool_output").and_then(Value::as_object) {
        if output.get("isError").and_then(Value::as_bool) == Some(true) {
            return true;
        }
        if output.get("error").is_some_and(|value| !value.is_null()) {
            return true;
        }
        if output.get("status").is_some_and(is_error_status) {
            return true;
        }
    }

    false
}

fn is_error_status(value: &Value) -> bool {
    value
        .as_str()
        .map(|status| {
            matches!(
                status.to_ascii_lowercase().as_str(),
                "error" | "failed" | "failure"
            )
        })
        .unwrap_or(false)
}

fn collect_memory_ids_from_metadata(metadata: &HashMap<String, Value>) -> Vec<i64> {
    let mut ids = BTreeSet::new();
    for key in [
        "memory_id",
        "memory_ids",
        "returned_memory_id",
        "returned_memory_ids",
        "returned_memories",
        "result_memory_id",
        "result_memory_ids",
    ] {
        if let Some(value) = metadata.get(key) {
            collect_memory_ids_from_value(value, &mut ids);
        }
    }
    if let Some(value) = metadata.get("tool_output") {
        collect_memory_ids_from_tool_output(value, &mut ids);
    }
    ids.into_iter().collect()
}

fn collect_memory_ids_from_value(value: &Value, ids: &mut BTreeSet<i64>) {
    match value {
        Value::Number(number) => {
            if let Some(id) = number.as_i64().filter(|id| *id > 0) {
                ids.insert(id);
            }
        }
        Value::String(raw) => {
            if let Ok(id) = raw.parse::<i64>() {
                if id > 0 {
                    ids.insert(id);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_memory_ids_from_value(value, ids);
            }
        }
        Value::Object(object) => {
            for key in ["id", "memory_id"] {
                if let Some(value) = object.get(key) {
                    collect_memory_ids_from_value(value, ids);
                }
            }
        }
        Value::Bool(_) | Value::Null => {}
    }
}

fn collect_memory_ids_from_tool_output(value: &Value, ids: &mut BTreeSet<i64>) {
    let Value::Object(object) = value else {
        return;
    };

    for key in [
        "memory_id",
        "memory_ids",
        "returned_memory_id",
        "returned_memory_ids",
        "result_memory_id",
        "result_memory_ids",
    ] {
        if let Some(value) = object.get(key) {
            collect_memory_ids_from_value(value, ids);
        }
    }

    for key in ["memory", "result"] {
        if let Some(Value::Object(item)) = object.get(key) {
            collect_memory_id_from_result_object(item, ids);
        }
    }

    for key in ["memories", "results", "items"] {
        if let Some(Value::Array(values)) = object.get(key) {
            for value in values {
                if let Value::Object(item) = value {
                    collect_memory_id_from_result_object(item, ids);
                }
            }
        }
    }
}

fn collect_memory_id_from_result_object(
    object: &serde_json::Map<String, Value>,
    ids: &mut BTreeSet<i64>,
) {
    if object.contains_key("content")
        || object.contains_key("memory_type")
        || object.contains_key("score")
    {
        for key in ["memory_id", "id"] {
            if let Some(value) = object.get(key) {
                collect_memory_ids_from_value(value, ids);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::{create_memory, get_policy_record, list_memories};
    use crate::types::{CreateMemoryInput, ListOptions, MemoryType};
    use serde_json::json;

    fn test_memory_input(content: &str) -> CreateMemoryInput {
        CreateMemoryInput {
            content: content.to_string(),
            memory_type: MemoryType::Note,
            tags: vec![],
            metadata: HashMap::new(),
            importance: None,
            scope: Default::default(),
            workspace: None,
            tier: Default::default(),
            defer_embedding: true,
            ttl_seconds: None,
            dedup_mode: Default::default(),
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        }
    }

    fn stored_memory_count(storage: &Storage) -> usize {
        storage
            .with_connection(|conn| {
                list_memories(conn, &ListOptions::default()).map(|memories| memories.len())
            })
            .unwrap()
    }

    #[test]
    fn test_post_tool_use_handler() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = PostToolUseHandler::new(storage.clone());
        let mut context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        context
            .metadata
            .insert("tool_name".to_string(), json!("memory_create"));
        context
            .metadata
            .insert("tool_output".to_string(), json!({"content": "fake memory"}));

        let result = handler.handle(crate::hooks::LifecycleHook::PostToolUse, &context);
        assert!(matches!(result, Ok(HookResult::Continue)));
        assert_eq!(
            stored_memory_count(&storage),
            0,
            "PostToolUse must not create synthetic memories from tool output"
        );
    }

    #[test]
    fn post_tool_use_memory_id_metadata_reinforces_policy() {
        let storage = Storage::open_in_memory().unwrap();
        let memory = storage
            .with_connection(|conn| create_memory(conn, &test_memory_input("post tool policy")))
            .unwrap();
        assert_eq!(stored_memory_count(&storage), 1);
        let handler = PostToolUseHandler::new(storage.clone());
        let mut context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        context
            .metadata
            .insert("tool_name".to_string(), json!("memory_search"));
        context.metadata.insert("success".to_string(), json!(true));
        context
            .metadata
            .insert("returned_memory_ids".to_string(), json!([memory.id]));

        let result = handler.handle(crate::hooks::LifecycleHook::PostToolUse, &context);
        assert!(matches!(result, Ok(HookResult::Continue)));

        let policy = storage
            .with_connection(|conn| get_policy_record(conn, memory.id))
            .unwrap()
            .expect("policy record");
        assert_eq!(policy.reinforcement_count, 1);
        assert_eq!(
            stored_memory_count(&storage),
            1,
            "policy reinforcement must not create additional memories"
        );
    }

    #[test]
    fn post_tool_use_tool_output_result_reinforces_policy() {
        let storage = Storage::open_in_memory().unwrap();
        let memory = storage
            .with_connection(|conn| create_memory(conn, &test_memory_input("tool output policy")))
            .unwrap();
        let handler = PostToolUseHandler::new(storage.clone());
        let mut context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        context
            .metadata
            .insert("tool_name".to_string(), json!("memory_search"));
        context.metadata.insert(
            "tool_output".to_string(),
            json!({
                "results": [
                    {
                        "id": memory.id,
                        "content": "tool output policy",
                        "score": 0.91
                    }
                ]
            }),
        );

        let result = handler.handle(crate::hooks::LifecycleHook::PostToolUse, &context);
        assert!(matches!(result, Ok(HookResult::Continue)));

        let policy = storage
            .with_connection(|conn| get_policy_record(conn, memory.id))
            .unwrap()
            .expect("policy record");
        assert_eq!(policy.reinforcement_count, 1);
    }

    #[test]
    fn post_tool_use_tool_output_ignores_unrelated_numeric_ids() {
        let storage = Storage::open_in_memory().unwrap();
        let memory = storage
            .with_connection(|conn| create_memory(conn, &test_memory_input("unrelated id policy")))
            .unwrap();
        let handler = PostToolUseHandler::new(storage.clone());
        let mut context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        context
            .metadata
            .insert("tool_name".to_string(), json!("memory_search"));
        context.metadata.insert(
            "tool_output".to_string(),
            json!({
                "request_id": memory.id,
                "result": {
                    "id": memory.id,
                    "status": "ok"
                }
            }),
        );

        let result = handler.handle(crate::hooks::LifecycleHook::PostToolUse, &context);
        assert!(matches!(result, Ok(HookResult::Continue)));

        let policy = storage
            .with_connection(|conn| get_policy_record(conn, memory.id))
            .unwrap()
            .expect("policy record");
        assert_eq!(policy.reinforcement_count, 0);
    }

    #[test]
    fn malformed_post_tool_use_metadata_continues_without_abort() {
        let storage = Storage::open_in_memory().unwrap();
        let handler = PostToolUseHandler::new(storage);
        let mut context = HookContext {
            session_id: Some("test-session".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        context
            .metadata
            .insert("tool_name".to_string(), json!("memory_get"));
        context.metadata.insert(
            "returned_memory_ids".to_string(),
            json!({"bad": ["not-an-id"]}),
        );

        let result = handler.handle(crate::hooks::LifecycleHook::PostToolUse, &context);
        assert!(matches!(result, Ok(HookResult::Continue)));
    }
}
