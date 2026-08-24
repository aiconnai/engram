//! MCP tool for running questions through a multi-model LLM Council session.

use std::collections::{HashMap, HashSet};
use std::env;
use std::time::Duration;

use serde_json::{json, Value};

use super::HandlerContext;

const DEFAULT_COUNCIL_URL: &str = "http://127.0.0.1:8001";
const DEFAULT_TIMEOUT_SECONDS: u64 = 90;
const MAX_TIMEOUT_SECONDS: u64 = 300;

/// Run a query through a remote llm-council backend and return its final verdict.
pub fn memory_council(ctx: &HandlerContext, params: Value) -> Value {
    let prompt = match params.get("prompt").and_then(Value::as_str) {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => return json!({"error": "prompt is required and must be a non-empty string"}),
    };

    let conversation_id = params
        .get("conversation_id")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string);

    let mut base_url = params
        .get("council_url")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| env::var("LLM_COUNCIL_URL").ok())
        .unwrap_or_else(|| DEFAULT_COUNCIL_URL.to_string());

    if base_url.ends_with('/') {
        while base_url.ends_with('/') {
            base_url.pop();
        }
    }

    let timeout_secs = params
        .get("timeout_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_TIMEOUT_SECONDS)
        .clamp(1, MAX_TIMEOUT_SECONDS);

    let include_raw_stages = params
        .get("include_raw_stages")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let persist = params
        .get("persist")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let workspace = params
        .get("workspace")
        .and_then(Value::as_str)
        .unwrap_or("default");

    let tags = parse_tags(params.get("memory_tags"));

    let request = CouncilRequest {
        prompt: prompt.clone(),
        conversation_id,
        base_url,
        timeout: Duration::from_secs(timeout_secs),
    };

    let result = match tokio::runtime::Handle::try_current() {
        Ok(handle) => match tokio::task::block_in_place(|| handle.block_on(run_council(request))) {
            Ok(v) => v,
            Err(e) => return json!({"error": e}),
        },
        Err(_) => {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    return json!({"error": format!("Failed to create async runtime: {e}")});
                }
            };
            match rt.block_on(run_council(request)) {
                Ok(v) => v,
                Err(e) => return json!({"error": e}),
            }
        }
    };

    let mut output = json!({
        "conversation_id": result.conversation_id,
        "prompt": prompt,
        "final_model": result.final_model.clone(),
        "final_answer": result.final_answer,
        "metadata": result.metadata.clone(),
    });

    if include_raw_stages {
        output["stage1"] = result.stage1.clone();
        output["stage2"] = result.stage2.clone();
        output["stage3"] = result.stage3.clone();
    } else {
        output["stage1_count"] = json!(count_array_like(&result.stage1));
        output["stage2_count"] = json!(count_array_like(&result.stage2));
        output["stage3_present"] = json!(!result.stage3.is_null());
    }

    if !persist {
        return output;
    }

    let mut unique_tags = HashSet::from(["llm-council".to_string(), "consensus".to_string()]);
    for tag in tags {
        unique_tags.insert(tag);
    }

    let metadata = build_persist_metadata(&result);
    let mut mem_tags = unique_tags.into_iter().collect::<Vec<_>>();
    mem_tags.sort_unstable();

    let memory_input = crate::types::CreateMemoryInput {
        content: build_memory_content(&prompt, &result),
        memory_type: crate::types::MemoryType::Checkpoint,
        tags: mem_tags,
        metadata,
        workspace: Some(workspace.to_string()),
        ..Default::default()
    };

    match ctx
        .storage
        .with_transaction(|conn| crate::storage::queries::create_memory(conn, &memory_input))
    {
        Ok(memory) => {
            output["memory_id"] = json!(memory.id);
            output
        }
        Err(e) => {
            output["warning"] = json!(format!(
                "Council result created successfully, but memory persistence failed: {e}"
            ));
            output
        }
    }
}

#[derive(Clone)]
struct CouncilRequest {
    prompt: String,
    conversation_id: Option<String>,
    base_url: String,
    timeout: Duration,
}

#[derive(Debug)]
struct CouncilResult {
    conversation_id: String,
    stage1: Value,
    stage2: Value,
    stage3: Value,
    metadata: Value,
    final_model: Option<String>,
    final_answer: String,
}

async fn run_council(req: CouncilRequest) -> Result<CouncilResult, String> {
    let client = reqwest::Client::builder()
        .timeout(req.timeout)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let conversation_id = if let Some(id) = req.conversation_id {
        id
    } else {
        create_conversation(&client, &req.base_url).await?
    };

    let message_result =
        post_message(&client, &req.base_url, &conversation_id, &req.prompt).await?;
    let stage1 = message_result.get("stage1").cloned().unwrap_or(Value::Null);
    let stage2 = message_result.get("stage2").cloned().unwrap_or(Value::Null);
    let stage3 = message_result.get("stage3").cloned().unwrap_or(Value::Null);
    let metadata = message_result
        .get("metadata")
        .cloned()
        .unwrap_or(Value::Null);

    let final_model = stage3
        .get("model")
        .or_else(|| stage3.get("metadata").and_then(|m| m.get("model")))
        .and_then(Value::as_str)
        .map(str::to_string);

    let final_answer = extract_stage3_text(&stage3);

    Ok(CouncilResult {
        conversation_id,
        stage1,
        stage2,
        stage3,
        metadata,
        final_model,
        final_answer,
    })
}

async fn create_conversation(client: &reqwest::Client, base_url: &str) -> Result<String, String> {
    let response = client
        .post(format!("{base_url}/api/conversations"))
        .json(&json!({}))
        .send()
        .await
        .map_err(|e| format!("LLM Council create conversation request failed: {e}"))?;

    let payload = parse_json_body(response)
        .await
        .map_err(|e| format!("LLM Council create conversation returned invalid response: {e}"))?;

    payload
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "LLM Council create conversation response missing `id`".to_string())
}

async fn post_message(
    client: &reqwest::Client,
    base_url: &str,
    conversation_id: &str,
    prompt: &str,
) -> Result<Value, String> {
    let response = client
        .post(format!(
            "{base_url}/api/conversations/{conversation_id}/message"
        ))
        .json(&json!({"content": prompt}))
        .send()
        .await
        .map_err(|e| format!("LLM Council message request failed: {e}"))?;

    parse_json_body(response)
        .await
        .map_err(|e| format!("LLM Council message request returned invalid response: {e}"))
}

async fn parse_json_body(response: reqwest::Response) -> Result<Value, String> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("failed to read LLM Council response body: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "LLM Council returned HTTP {} with body: {}",
            status,
            truncate_for_error(&text, 1_000),
        ));
    }

    serde_json::from_str(&text).map_err(|e| {
        format!(
            "failed to parse JSON response (`{}`): {}",
            truncate_for_error(&text, 300),
            e
        )
    })
}

fn extract_stage3_text(stage3: &Value) -> String {
    if let Some(text) = stage3.get("response").and_then(Value::as_str) {
        return text.to_string();
    }

    match serde_json::to_string_pretty(stage3) {
        Ok(serialized) => serialized,
        Err(_) => stage3.to_string(),
    }
}

fn parse_tags(tags: Option<&Value>) -> Vec<String> {
    tags.and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|value| value.as_str())
                .filter(|tag| !tag.trim().is_empty())
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(|| vec!["llm-council".to_string(), "consensus".to_string()])
}

fn build_memory_content(prompt: &str, result: &CouncilResult) -> String {
    let mut lines = vec![
        "LLM Council consensus result".to_string(),
        "".to_string(),
        format!("Prompt: {prompt}"),
        "".to_string(),
        format!("Final answer: {}", result.final_answer),
    ];

    if let Some(model) = &result.final_model {
        lines.push(String::new());
        lines.push(format!("Final model: {model}"));
    }

    lines.join("\n")
}

fn build_persist_metadata(result: &CouncilResult) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert("tool".to_string(), json!("memory_council"));
    metadata.insert("conversation_id".to_string(), json!(result.conversation_id));
    metadata.insert(
        "stage1_count".to_string(),
        json!(count_array_like(&result.stage1)),
    );
    metadata.insert(
        "stage2_count".to_string(),
        json!(count_array_like(&result.stage2)),
    );
    metadata.insert(
        "stage3_present".to_string(),
        json!(!result.stage3.is_null()),
    );

    if let Some(model) = &result.final_model {
        metadata.insert("final_model".to_string(), json!(model));
    }

    metadata
}

fn count_array_like(value: &Value) -> usize {
    value.as_array().map_or(0, std::vec::Vec::len)
}

fn truncate_for_error(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        input.to_string()
    } else {
        format!("{}...", input.chars().take(limit).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::handlers::HandlerContext;
    use crate::storage::Storage;
    use std::sync::Arc;

    fn test_ctx() -> HandlerContext {
        let storage = Storage::open_in_memory().expect("open in-memory storage");
        HandlerContext {
            storage,
            embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(128)),
            fuzzy_engine: Arc::new(parking_lot::Mutex::new(crate::search::FuzzyEngine::new())),
            search_config: crate::search::SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(crate::search::SearchResultCache::new(
                crate::search::AdaptiveCacheConfig::default(),
            )),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 300,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
            principal: None,
        }
    }

    #[test]
    fn test_extract_stage3_text_prefers_response_field() {
        let input = json!({"response":"ok"});
        assert_eq!(extract_stage3_text(&input), "ok");
    }

    #[test]
    fn test_extract_stage3_text_falls_back_to_pretty_json() {
        let input = json!({"model":"x","text":"ok"});
        assert!(extract_stage3_text(&input).contains("\"model\""));
    }

    #[test]
    fn test_parse_tags_filters_empty_and_defaults() {
        assert_eq!(
            parse_tags(Some(&json!(["", "analysis", "analysis", "decisions"]))).len(),
            3
        );
    }

    #[test]
    fn test_truncate_for_error() {
        assert_eq!(truncate_for_error("hello", 10), "hello");
        assert_eq!(truncate_for_error("hello world", 5), "hello...");
        assert_eq!(truncate_for_error("ação necessária", 2), "aç...");
    }

    #[test]
    fn test_build_memory_content_includes_prompt_and_answer() {
        let result = CouncilResult {
            conversation_id: "abc".to_string(),
            stage1: json!([]),
            stage2: json!([]),
            stage3: json!({"response":"ok"}),
            metadata: json!({}),
            final_model: Some("m".to_string()),
            final_answer: "yes".to_string(),
        };

        let content = build_memory_content("what is x?", &result);
        assert!(content.contains("what is x?"));
        assert!(content.contains("Final answer: yes"));
        assert!(content.contains("Final model: m"));
    }

    #[test]
    fn test_count_array_like() {
        assert_eq!(count_array_like(&json!([1, 2, 3])), 3);
        assert_eq!(count_array_like(&json!({"a":1})), 0);
    }

    #[test]
    fn test_parse_tags_uses_defaults_when_missing() {
        assert_eq!(
            parse_tags(None),
            vec!["llm-council".to_string(), "consensus".to_string()],
        );
    }

    #[test]
    fn test_memory_council_rejects_empty_prompt() {
        let ctx = test_ctx();
        let value = memory_council(&ctx, json!({"prompt": ""}));
        assert!(value.get("error").is_some());
    }
}
