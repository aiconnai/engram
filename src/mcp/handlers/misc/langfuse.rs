//! Langfuse integration handlers (feature-gated behind `langfuse`).

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

#[cfg(feature = "langfuse")]
pub fn langfuse_connect(ctx: &HandlerContext, params: Value) -> Value {
    use crate::integrations::langfuse::{LangfuseClient, LangfuseConfig};

    let public_key = params
        .get("public_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("LANGFUSE_PUBLIC_KEY").ok());

    let secret_key = params
        .get("secret_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("LANGFUSE_SECRET_KEY").ok());

    let base_url = params
        .get("base_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://cloud.langfuse.com")
        .to_string();

    let (public_key, secret_key) = match (public_key, secret_key) {
        (Some(pk), Some(sk)) => (pk, sk),
        _ => {
            return json!({
                "error": "Missing credentials. Provide public_key and secret_key or set LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY environment variables."
            });
        }
    };

    let config = LangfuseConfig {
        public_key: public_key.clone(),
        secret_key,
        base_url: base_url.clone(),
    };

    let client = LangfuseClient::new(config);

    let connected = ctx
        .langfuse_runtime
        .block_on(async { client.test_connection().await });

    match connected {
        Ok(true) => json!({
            "status": "connected",
            "base_url": base_url,
            "public_key_prefix": &public_key[..8.min(public_key.len())]
        }),
        Ok(false) => json!({
            "status": "failed",
            "error": "Connection test failed"
        }),
        Err(e) => json!({
            "status": "error",
            "error": e.to_string()
        }),
    }
}

#[cfg(feature = "langfuse")]
pub fn langfuse_sync(ctx: &HandlerContext, params: Value) -> Value {
    use crate::integrations::langfuse::{LangfuseClient, LangfuseConfig};
    use crate::storage::queries::{upsert_sync_task, SyncTask};
    use chrono::{Duration, Utc};

    let config = match LangfuseConfig::from_env() {
        Some(c) => c,
        None => {
            return json!({
                "error": "Langfuse not configured. Call langfuse_connect first or set environment variables."
            });
        }
    };

    let since = params
        .get("since")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - Duration::hours(24));

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let task_id = uuid::Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();

    let initial_task = SyncTask {
        task_id: task_id.clone(),
        task_type: "langfuse_sync".to_string(),
        status: "running".to_string(),
        progress_percent: 0,
        traces_processed: 0,
        memories_created: 0,
        error_message: None,
        started_at: started_at.clone(),
        completed_at: None,
    };

    if let Err(e) = ctx
        .storage
        .with_connection(|conn| upsert_sync_task(conn, &initial_task))
    {
        return json!({"error": format!("Failed to create sync task: {}", e)});
    }

    let client = LangfuseClient::new(config);

    let result = ctx
        .langfuse_runtime
        .block_on(async { client.fetch_traces(since, limit).await });

    match result {
        Ok(traces) => {
            if dry_run {
                let trace_summaries: Vec<_> = traces
                    .iter()
                    .map(|t| {
                        json!({
                            "id": t.id,
                            "name": t.name,
                            "timestamp": t.timestamp.to_rfc3339(),
                            "user_id": t.user_id,
                            "tags": t.tags
                        })
                    })
                    .collect();

                let final_task = SyncTask {
                    task_id: task_id.clone(),
                    task_type: "langfuse_sync".to_string(),
                    status: "completed".to_string(),
                    progress_percent: 100,
                    traces_processed: traces.len() as i64,
                    memories_created: 0,
                    error_message: None,
                    started_at,
                    completed_at: Some(Utc::now().to_rfc3339()),
                };
                let _ = ctx
                    .storage
                    .with_connection(|conn| upsert_sync_task(conn, &final_task));

                return json!({
                    "task_id": task_id,
                    "dry_run": true,
                    "traces_found": traces.len(),
                    "traces": trace_summaries
                });
            }

            use crate::integrations::langfuse::trace_to_memory_content;
            use crate::storage::queries::create_memory;
            use crate::types::{CreateMemoryInput, MemoryType};

            let mut memories_created = 0i64;
            let mut errors: Vec<String> = Vec::new();

            for trace in &traces {
                let content = trace_to_memory_content(trace, &[]);

                let input = CreateMemoryInput {
                    content,
                    memory_type: MemoryType::Episodic,
                    importance: Some(0.5),
                    tags: {
                        let mut tags = trace.tags.clone();
                        tags.push("langfuse".to_string());
                        tags
                    },
                    workspace: workspace.clone(),
                    event_time: Some(trace.timestamp),
                    ..Default::default()
                };

                match ctx
                    .storage
                    .with_connection(|conn| create_memory(conn, &input))
                {
                    Ok(_) => memories_created += 1,
                    Err(e) => errors.push(format!("Trace {}: {}", trace.id, e)),
                }
            }

            let final_task = SyncTask {
                task_id: task_id.clone(),
                task_type: "langfuse_sync".to_string(),
                status: if errors.is_empty() {
                    "completed".to_string()
                } else {
                    "completed_with_errors".to_string()
                },
                progress_percent: 100,
                traces_processed: traces.len() as i64,
                memories_created,
                error_message: if errors.is_empty() {
                    None
                } else {
                    Some(errors.join("; "))
                },
                started_at,
                completed_at: Some(Utc::now().to_rfc3339()),
            };
            let _ = ctx
                .storage
                .with_connection(|conn| upsert_sync_task(conn, &final_task));

            json!({
                "task_id": task_id,
                "status": final_task.status,
                "traces_processed": traces.len(),
                "memories_created": memories_created,
                "errors": errors
            })
        }
        Err(e) => {
            let final_task = SyncTask {
                task_id: task_id.clone(),
                task_type: "langfuse_sync".to_string(),
                status: "failed".to_string(),
                progress_percent: 0,
                traces_processed: 0,
                memories_created: 0,
                error_message: Some(e.to_string()),
                started_at,
                completed_at: Some(Utc::now().to_rfc3339()),
            };
            let _ = ctx
                .storage
                .with_connection(|conn| upsert_sync_task(conn, &final_task));

            json!({
                "task_id": task_id,
                "status": "failed",
                "error": e.to_string()
            })
        }
    }
}

#[cfg(feature = "langfuse")]
pub fn langfuse_sync_status(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_sync_task;

    let task_id = match params.get("task_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return json!({"error": "task_id is required"}),
    };

    ctx.storage
        .with_connection(|conn| match get_sync_task(conn, task_id)? {
            Some(task) => Ok(json!(task)),
            None => Ok(json!({"error": "Task not found", "task_id": task_id})),
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

#[cfg(feature = "langfuse")]
pub fn langfuse_extract_patterns(ctx: &HandlerContext, params: Value) -> Value {
    use crate::integrations::langfuse::{extract_patterns, LangfuseClient, LangfuseConfig};
    use chrono::{Duration, Utc};

    let config = match LangfuseConfig::from_env() {
        Some(c) => c,
        None => {
            return json!({
                "error": "Langfuse not configured. Set LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY environment variables."
            });
        }
    };

    let since = params
        .get("since")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - Duration::days(7));

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let min_confidence = params
        .get("min_confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7);

    let client = LangfuseClient::new(config);

    let result = ctx
        .langfuse_runtime
        .block_on(async { client.fetch_traces(since, limit).await });

    match result {
        Ok(traces) => {
            let patterns = extract_patterns(&traces);
            let filtered: Vec<_> = patterns
                .into_iter()
                .filter(|p| p.confidence >= min_confidence)
                .collect();

            json!({
                "traces_analyzed": traces.len(),
                "patterns_found": filtered.len(),
                "patterns": filtered
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

#[cfg(feature = "langfuse")]
pub fn memory_from_trace(ctx: &HandlerContext, params: Value) -> Value {
    use crate::integrations::langfuse::{trace_to_memory_content, LangfuseClient, LangfuseConfig};
    use crate::storage::queries::create_memory;
    use crate::types::{CreateMemoryInput, MemoryType};

    let trace_id = match params.get("trace_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return json!({"error": "trace_id is required"}),
    };

    let memory_type_str = params
        .get("memory_type")
        .and_then(|v| v.as_str())
        .unwrap_or("episodic");

    let memory_type = match memory_type_str {
        "note" => MemoryType::Note,
        "episodic" => MemoryType::Episodic,
        "procedural" => MemoryType::Procedural,
        "learning" => MemoryType::Learning,
        _ => MemoryType::Episodic,
    };

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let extra_tags: Vec<String> = params
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let config = match LangfuseConfig::from_env() {
        Some(c) => c,
        None => {
            return json!({
                "error": "Langfuse not configured. Set environment variables."
            });
        }
    };

    let client = LangfuseClient::new(config);

    let trace_result = ctx
        .langfuse_runtime
        .block_on(async { client.fetch_trace(trace_id).await });

    match trace_result {
        Ok(Some(trace)) => {
            let content = trace_to_memory_content(&trace, &[]);

            let mut tags = trace.tags.clone();
            tags.push("langfuse".to_string());
            tags.push(format!("trace:{}", trace_id));
            tags.extend(extra_tags);

            let input = CreateMemoryInput {
                content,
                memory_type,
                importance: Some(0.6),
                tags,
                workspace,
                event_time: Some(trace.timestamp),
                ..Default::default()
            };

            ctx.storage
                .with_connection(|conn| {
                    let memory = create_memory(conn, &input)?;
                    Ok(json!({
                        "id": memory.id,
                        "trace_id": trace_id,
                        "memory_type": memory_type_str,
                        "content_length": memory.content.len()
                    }))
                })
                .unwrap_or_else(|e| json!({"error": e.to_string()}))
        }
        Ok(None) => json!({"error": format!("Trace {} not found", trace_id)}),
        Err(e) => json!({"error": e.to_string()}),
    }
}
