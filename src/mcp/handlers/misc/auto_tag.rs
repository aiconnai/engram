//! Auto-tagging handlers: suggest tags and apply them to memories.

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};

pub fn memory_suggest_tags(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::{AutoTagConfig, AutoTagger};
    use crate::storage::queries::get_memory;

    let (content, memory_type, existing_tags) = if let Some(id) = params
        .get("id")
        .or_else(|| params.get("memory_id"))
        .and_then(|v| v.as_i64())
    {
        match ctx.storage.with_connection(|conn| get_memory(conn, id)) {
            Ok(memory) => (memory.content, Some(memory.memory_type), memory.tags),
            Err(e) => return json!({"error": e.to_string()}),
        }
    } else if let Some(content) = params.get("content").and_then(|v| v.as_str()) {
        let memory_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok());
        let existing: Vec<String> = params
            .get("existing_tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        (content.to_string(), memory_type, existing)
    } else {
        return json!({"error": "Either 'id'/'memory_id' or 'content' is required"});
    };

    let mut config = AutoTagConfig::default();

    if let Some(min_conf) = params.get("min_confidence").and_then(|v| v.as_f64()) {
        config.min_confidence = min_conf as f32;
    }
    if let Some(max) = params.get("max_tags").and_then(|v| v.as_u64()) {
        config.max_tags = max as usize;
    }
    if let Some(v) = params.get("enable_patterns").and_then(|v| v.as_bool()) {
        config.enable_patterns = v;
    }
    if let Some(v) = params.get("enable_keywords").and_then(|v| v.as_bool()) {
        config.enable_keywords = v;
    }
    if let Some(v) = params.get("enable_entities").and_then(|v| v.as_bool()) {
        config.enable_entities = v;
    }
    if let Some(v) = params.get("enable_type_tags").and_then(|v| v.as_bool()) {
        config.enable_type_tags = v;
    }

    if let Some(mappings) = params.get("keyword_mappings").and_then(|v| v.as_object()) {
        for (keyword, tag) in mappings {
            if let Some(tag_str) = tag.as_str() {
                config
                    .keyword_mappings
                    .insert(keyword.clone(), tag_str.to_string());
            }
        }
    }

    let tagger = AutoTagger::new(config);
    let result = tagger.suggest_tags(&content, memory_type, &existing_tags);

    json!({
        "suggestions": result.suggestions,
        "analysis_count": result.analysis_count
    })
}

pub fn memory_auto_tag(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::{AutoTagConfig, AutoTagger};
    use crate::storage::queries::{get_memory, update_memory};
    use crate::types::UpdateMemoryInput;

    let id = match params
        .get("id")
        .or_else(|| params.get("memory_id"))
        .and_then(|v| v.as_i64())
    {
        Some(id) => id,
        None => return json!({"error": "id or memory_id is required"}),
    };

    let apply = params
        .get("apply")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let merge = params
        .get("merge")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut config = AutoTagConfig::default();

    if let Some(min_conf) = params.get("min_confidence").and_then(|v| v.as_f64()) {
        config.min_confidence = min_conf as f32;
    }
    if let Some(max) = params.get("max_tags").and_then(|v| v.as_u64()) {
        config.max_tags = max as usize;
    }

    if let Some(mappings) = params.get("keyword_mappings").and_then(|v| v.as_object()) {
        for (keyword, tag) in mappings {
            if let Some(tag_str) = tag.as_str() {
                config
                    .keyword_mappings
                    .insert(keyword.clone(), tag_str.to_string());
            }
        }
    }

    let (memory, suggestions) = match ctx.storage.with_connection(|conn| {
        let memory = get_memory(conn, id)?;
        let tagger = AutoTagger::new(config);
        let result = tagger.suggest_for_memory(&memory);
        Ok((memory, result))
    }) {
        Ok(r) => r,
        Err(e) => return json!({"error": e.to_string()}),
    };

    if !apply {
        return json!({
            "memory_id": id,
            "suggestions": suggestions.suggestions,
            "applied": false,
            "message": "Tags suggested but not applied. Set apply=true to apply them."
        });
    }

    let suggested_tags: Vec<String> = suggestions
        .suggestions
        .iter()
        .map(|s| s.tag.clone())
        .collect();

    let new_tags = if merge {
        let mut tags = memory.tags.clone();
        for tag in suggested_tags.iter() {
            if !tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()) {
                tags.push(tag.clone());
            }
        }
        tags
    } else {
        suggested_tags.clone()
    };

    let update_input = UpdateMemoryInput {
        content: None,
        memory_type: None,
        tags: Some(new_tags.clone()),
        metadata: None,
        importance: None,
        scope: None,
        ttl_seconds: None,
        event_time: None,
        trigger_pattern: None,
        media_url: None,
    };

    match ctx
        .storage
        .with_transaction(|conn| update_memory(conn, id, &update_input))
    {
        Ok(updated_memory) => {
            ctx.storage
                .with_connection(|conn| {
                    let op_id = uuid::Uuid::new_v4().to_string();
                    emit_best_effort(
                        conn,
                        &EnrichmentEvent {
                            operation_id: &op_id,
                            event_type: "auto_tag",
                            memory_id: Some(id),
                            version_id: None,
                            triggered_by: "memory_auto_tag",
                            agent_id: None,
                            workspace: None,
                            params: json!({"apply": true, "merge": merge}),
                            outcome: json!({"applied_tags": &suggested_tags}),
                            status: "completed",
                            dry_run: false,
                        },
                    );
                    Ok::<_, crate::error::EngramError>(())
                })
                .ok();

            json!({
                "memory_id": id,
                "suggestions": suggestions.suggestions,
                "applied": true,
                "applied_tags": suggested_tags,
                "final_tags": updated_memory.tags,
                "merged": merge
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}
