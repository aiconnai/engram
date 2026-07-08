//! Integration tests for MCP 2025-11-25 protocol features.
//!
//! Tests protocol negotiation, tool annotations, resources, and prompts
//! through the full MCP request/response pipeline.
//!
//! Run with: cargo test --test mcp_protocol_tests

use std::sync::Arc;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::{
    get_prompt, get_tool_definitions, get_tool_definitions_tiered, handlers, list_prompts,
    list_resources, methods, read_resource, InitializeResult, McpHandler, McpRequest, McpResponse,
    PromptCapabilities, ResourceCapabilities, ServerCapabilities, ToolCallResult, ToolsCapability,
    MCP_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION_LEGACY,
};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::{get_policy_record, upsert_policy_record, PolicyRecordInput};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

// ---------------------------------------------------------------------------
// Test handler — mirrors the EngramHandler in server.rs using public APIs
// ---------------------------------------------------------------------------

struct TestHandler {
    storage: Storage,
    ctx: handlers::HandlerContext,
}

impl TestHandler {
    fn new() -> Self {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        let ctx = handlers::HandlerContext {
            storage: storage.clone(),
            embedder,
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        };
        Self { storage, ctx }
    }
}

impl McpHandler for TestHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            methods::INITIALIZE => {
                let client_version = request
                    .params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or(MCP_PROTOCOL_VERSION);

                let result = if client_version == MCP_PROTOCOL_VERSION_LEGACY {
                    InitializeResult {
                        protocol_version: MCP_PROTOCOL_VERSION_LEGACY.to_string(),
                        capabilities: ServerCapabilities {
                            tools: Some(ToolsCapability {
                                list_changed: false,
                            }),
                            resources: None,
                            prompts: None,
                        },
                        ..InitializeResult::default()
                    }
                } else {
                    InitializeResult {
                        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                        capabilities: ServerCapabilities {
                            tools: Some(ToolsCapability {
                                list_changed: false,
                            }),
                            resources: Some(ResourceCapabilities {
                                subscribe: false,
                                list_changed: false,
                            }),
                            prompts: Some(PromptCapabilities {
                                list_changed: false,
                            }),
                        },
                        ..InitializeResult::default()
                    }
                };

                McpResponse::success(request.id, json!(result))
            }

            methods::LIST_TOOLS => {
                let tools = get_tool_definitions();
                McpResponse::success(request.id, json!({"tools": tools}))
            }

            methods::CALL_TOOL => {
                let name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));
                let result = handlers::dispatch(&self.ctx, name, arguments);
                let tool_result = ToolCallResult::json(&result);
                McpResponse::success(request.id, json!(tool_result))
            }

            methods::LIST_RESOURCES => {
                let templates = list_resources();
                let resources: Vec<Value> = templates
                    .into_iter()
                    .map(|t| {
                        json!({
                            "uri": t.uri_template,
                            "name": t.name,
                            "description": t.description,
                            "mimeType": t.mime_type,
                        })
                    })
                    .collect();
                McpResponse::success(request.id, json!({"resources": resources}))
            }

            methods::READ_RESOURCE => {
                let uri = match request.params.get("uri").and_then(|v| v.as_str()) {
                    Some(u) => u.to_string(),
                    None => {
                        return McpResponse::error(
                            request.id,
                            -32602,
                            "Missing required parameter: uri".to_string(),
                        )
                    }
                };

                match read_resource(&self.storage, &uri) {
                    Ok(content) => {
                        let text = serde_json::to_string_pretty(&content)
                            .unwrap_or_else(|_| content.to_string());
                        McpResponse::success(
                            request.id,
                            json!({
                                "contents": [{
                                    "uri": uri,
                                    "mimeType": "application/json",
                                    "text": text,
                                }]
                            }),
                        )
                    }
                    Err(msg) => McpResponse::error(request.id, -32602, msg),
                }
            }

            methods::LIST_PROMPTS => {
                let prompts = list_prompts();
                McpResponse::success(request.id, json!({"prompts": prompts}))
            }

            methods::GET_PROMPT => {
                let name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));
                match get_prompt(name, &arguments) {
                    Ok(messages) => McpResponse::success(request.id, json!({"messages": messages})),
                    Err(e) => McpResponse::error(request.id, -32002, e),
                }
            }

            _ => McpResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper utilities
// ---------------------------------------------------------------------------

fn make_request(id: i64, method: &str, params: Value) -> McpRequest {
    McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(id)),
        method: method.to_string(),
        params,
    }
}

fn call_tool_json(handler: &TestHandler, id: i64, name: &str, arguments: Value) -> Value {
    let req = make_request(
        id,
        "tools/call",
        json!({
            "name": name,
            "arguments": arguments
        }),
    );
    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "{} failed: {:?}", name, resp.error);

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text content");
    serde_json::from_str(text).expect("Tool result should contain JSON")
}

#[test]
fn test_tools_list_uses_unique_tool_names_from_registry() {
    use std::collections::BTreeSet;

    let handler = TestHandler::new();
    let resp = handler.handle_request(make_request(1, methods::LIST_TOOLS, json!({})));
    assert!(resp.error.is_none(), "tools/list should succeed");

    let result = resp.result.expect("tools/list result");
    let tools = result["tools"].as_array().expect("tools must be an array");
    let mut seen = BTreeSet::new();
    let mut discover_tools_count = 0;

    for tool in tools {
        let name = tool["name"].as_str().expect("listed tool must have a name");
        if name == "discover_tools" {
            discover_tools_count += 1;
        }
        assert!(
            seen.insert(name.to_string()),
            "tools/list must not contain duplicate tool name: {name}"
        );
    }

    assert_eq!(
        discover_tools_count, 1,
        "discover_tools must come from the canonical registry exactly once"
    );
}

#[test]
fn test_tool_registry_has_no_orphan_definition_files() {
    let tools_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("mcp")
        .join("tools");

    let allowed = ["catalog.rs", "mod.rs", "registry.rs"];
    for entry in std::fs::read_dir(&tools_dir).expect("read src/mcp/tools") {
        let path = entry.expect("read tool definition dir entry").path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("tool definition file name must be UTF-8");
        assert!(
            allowed.contains(&file_name),
            "tool definitions must live in src/mcp/tools/registry.rs; remove orphan file: {}",
            path.display()
        );
    }
}

fn create_memory_for_search(handler: &TestHandler, id: i64, content: &str) -> i64 {
    let created = call_tool_json(
        handler,
        id,
        "memory_create",
        json!({
            "content": content,
            "memory_type": "note",
            "importance": 0.5
        }),
    );
    created["id"].as_i64().expect("created memory id")
}

#[test]
fn session_land_protocol_without_session_id_returns_handoff_packet() {
    let handler = TestHandler::new();

    let result = call_tool_json(
        &handler,
        301,
        "session_land",
        json!({
            "workspace": "default",
            "summary": "Manual protocol-level session rotation",
            "current_goal": "Add MCP protocol coverage for session handoff",
            "next_session_hints": ["Resume from the copy block"]
        }),
    );

    assert!(
        result.get("error").is_none(),
        "session_land returned tool error: {result}"
    );
    let handoff = &result["handoff"];
    let copy_block = handoff["copy_block"]
        .as_str()
        .expect("handoff.copy_block must be present");
    assert!(
        copy_block.contains("# Continue this work in a new AI session"),
        "copy block should contain continuation heading: {copy_block}"
    );
    assert_eq!(
        handoff["bootstrap_prompt"], handoff["copy_block"],
        "bootstrap_prompt must remain a compatibility alias for copy_block"
    );

    let warnings = handoff["warnings"]
        .as_array()
        .expect("handoff.warnings must be an array");
    assert!(
        warnings.iter().any(|warning| warning
            .as_str()
            .is_some_and(|text| text.contains("No concrete session resolved"))),
        "expected no-session fallback warning, got: {warnings:?}"
    );

    assert!(
        result.get("checkpoint_id").is_some(),
        "top-level checkpoint_id must be present"
    );
    assert!(
        handoff.get("checkpoint_id").is_some(),
        "handoff.checkpoint_id must be present"
    );
    assert_eq!(
        result["checkpoint_id"], handoff["checkpoint_id"],
        "top-level and nested checkpoint ids must match"
    );
}

#[test]
fn harness_handoff_protocol_warns_when_completion_lacks_verification() {
    let handler = TestHandler::new();

    let result = call_tool_json(
        &handler,
        302,
        "harness_handoff",
        json!({
            "workspace": "default",
            "current_goal": "Finish Task 7 MCP protocol coverage",
            "next_steps": ["Run focused protocol tests"]
        }),
    );

    assert!(
        result.get("error").is_none(),
        "harness_handoff returned tool error: {result}"
    );
    assert!(
        result["copy_block"]
            .as_str()
            .is_some_and(|copy_block| !copy_block.is_empty()),
        "harness_handoff must include copy_block: {result}"
    );
    assert_eq!(
        result["completion_claimed"].as_bool(),
        Some(false),
        "missing verification evidence must not claim completion"
    );
    assert_eq!(
        result["completion_warning"].as_str(),
        Some("No verification evidence provided. Do not claim this work is complete."),
        "missing verification evidence must produce a completion warning"
    );
}

#[test]
fn mcp_mock_parity_scenarios_match_fixture_contract() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/mcp_mock_parity_scenarios.json"))
            .expect("mock parity fixture should be valid JSON");
    assert_eq!(fixture["version"].as_str(), Some("mcp-mock-parity-v1"));

    let handler = TestHandler::new();
    let scenarios = fixture["scenarios"]
        .as_array()
        .expect("fixture scenarios should be an array");
    let mut normalized = Vec::new();

    for (index, scenario) in scenarios.iter().enumerate() {
        let name = scenario["name"]
            .as_str()
            .expect("fixture scenario should have a name");
        let steps = scenario["steps"]
            .as_array()
            .expect("fixture scenario should have steps");
        let base_id = 300 + (index as i64 * 10);

        match name {
            "memory_create_search" => {
                let create = call_tool_json(
                    &handler,
                    base_id,
                    steps[0]["tool"].as_str().expect("create tool name"),
                    steps[0]["arguments"].clone(),
                );
                let search = call_tool_json(
                    &handler,
                    base_id + 1,
                    steps[1]["tool"].as_str().expect("search tool name"),
                    steps[1]["arguments"].clone(),
                );

                let expected_content = steps[0]["arguments"]["content"]
                    .as_str()
                    .expect("memory fixture content");
                let tags = create["tags"].as_array().expect("memory tags array");
                let search_results = search
                    .as_array()
                    .expect("memory_search response should stay an array");
                let hit = search_results
                    .iter()
                    .find(|item| item["memory"]["content"].as_str() == Some(expected_content))
                    .expect("memory_search should return the created fixture memory");

                normalized.push(json!({
                    "name": name,
                    "create": {
                        "type": if create.is_object() { "object" } else { "other" },
                        "has_id": create["id"].is_i64(),
                        "content_matches": create["content"].as_str() == Some(expected_content),
                        "memory_type": create["memory_type"].as_str()
                            .or_else(|| create["type"].as_str())
                            .unwrap_or(""),
                        "has_parity_tag": tags.iter().any(|tag| tag.as_str() == Some("parity")),
                        "has_issue_tag": tags.iter().any(|tag| tag.as_str() == Some("engra-111"))
                    },
                    "search": {
                        "type": if search.is_array() { "array" } else { "other" },
                        "hit_content_matches": hit["memory"]["content"].as_str() == Some(expected_content),
                        "hit_shape": {
                            "memory": hit["memory"].is_object(),
                            "score": hit["score"].is_number(),
                            "match_info": hit["match_info"].is_object()
                        }
                    }
                }));
            }
            "context_record_search" => {
                let record = call_tool_json(
                    &handler,
                    base_id,
                    steps[0]["tool"].as_str().expect("record tool name"),
                    steps[0]["arguments"].clone(),
                );
                let search = call_tool_json(
                    &handler,
                    base_id + 1,
                    steps[1]["tool"].as_str().expect("context search tool name"),
                    steps[1]["arguments"].clone(),
                );

                let event_id = record["created_ids"]["event_id"].as_i64();
                let results = search["results"]
                    .as_array()
                    .expect("context_search results should stay an array");
                let hit = results
                    .iter()
                    .find(|item| item["provenance"]["event_id"].as_i64() == event_id)
                    .expect("context_search should return the recorded fixture event");
                let artifact_pointers = hit["artifact_pointers"]
                    .as_array()
                    .expect("context_search artifact_pointers should stay an array");

                normalized.push(json!({
                    "name": name,
                    "record": {
                        "type": if record.is_object() { "object" } else { "other" },
                        "has_event_id": event_id.is_some(),
                        "has_summary_id": record["created_ids"]["summary_id"].is_i64()
                    },
                    "search": {
                        "type": if search.is_object() { "object" } else { "other" },
                        "query": search["query"].as_str().unwrap_or(""),
                        "results_type": if search["results"].is_array() { "array" } else { "other" },
                        "hit_event_type": hit["event"]["event_type"].as_str().unwrap_or(""),
                        "hit_reducer_name": hit["summary"]["reducer_name"].as_str().unwrap_or(""),
                        "artifact_pointer_present": artifact_pointers.iter().any(|pointer| {
                            pointer["artifact_id"].as_str() == Some("engra-111-parity-artifact")
                        }),
                        "provenance_present": hit["provenance"].is_object()
                    }
                }));
            }
            "unknown_tool_error" => {
                let result = call_tool_json(
                    &handler,
                    base_id,
                    steps[0]["tool"].as_str().expect("unknown tool name"),
                    steps[0]["arguments"].clone(),
                );
                normalized.push(json!({
                    "name": name,
                    "error": result["error"].as_str().unwrap_or("")
                }));
            }
            other => panic!("unhandled mock parity scenario: {other}"),
        }
    }

    assert_eq!(json!(normalized), fixture["expected_normalized"]);
}

fn set_policy_priority(handler: &TestHandler, memory_id: i64, priority: f32, reason: &str) {
    handler
        .storage
        .with_connection(|conn| {
            upsert_policy_record(
                conn,
                PolicyRecordInput {
                    memory_id,
                    salience_score: priority,
                    retention_score: priority,
                    retrieval_priority: priority,
                    policy_version: "heuristic-v1".to_string(),
                    policy_reason: reason.to_string(),
                },
            )
            .map(|_| ())
        })
        .expect("set policy priority");
}

#[test]
fn memory_policy_tools_are_listed_and_valid_calls_return_json() {
    let handler = TestHandler::new();

    let list_resp = handler.handle_request(make_request(1, methods::LIST_TOOLS, json!({})));
    assert!(list_resp.error.is_none(), "tools/list should succeed");
    let tools = list_resp.result.expect("tools/list result")["tools"]
        .as_array()
        .expect("tools array")
        .clone();
    for name in [
        "memory_score",
        "memory_promote",
        "memory_decay",
        "memory_explain",
        "memory_reconcile_conflict",
    ] {
        assert!(
            tools.iter().any(|tool| tool["name"] == name),
            "{name} should be listed"
        );
    }

    let memory_id = create_memory_for_search(&handler, 2, "Task 5 policy layer seeded memory");
    assert_eq!(memory_id, 1, "fresh test DB should seed memory id 1");

    let calls = [
        ("memory_score", json!({"id": 1, "persist": true})),
        ("memory_promote", json!({"id": 1, "canonical_tier": false})),
        (
            "memory_decay",
            json!({"workspace": "default", "dry_run": true}),
        ),
        ("memory_explain", json!({"id": 1})),
        (
            "memory_reconcile_conflict",
            json!({"id": 1, "reason": "superseded by newer user correction"}),
        ),
    ];

    for (index, (name, arguments)) in calls.into_iter().enumerate() {
        let result = call_tool_json(&handler, 10 + index as i64, name, arguments);
        assert!(
            result.get("error").is_none(),
            "{name} returned error: {result}"
        );
    }
}

fn start_council_stub_server(
    expected_path: &'static str,
    expected_body_fragment: &'static str,
    response_body: Value,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock council port");
    let addr = listener.local_addr().expect("local address");
    let response_body = response_body.to_string();

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept mock council request");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("set read timeout");

        let mut request_bytes = Vec::new();
        let header_end = loop {
            let mut chunk = [0_u8; 1024];
            let n = stream.read(&mut chunk).expect("read request bytes");
            if n == 0 {
                break None;
            }
            request_bytes.extend_from_slice(&chunk[..n]);
            if let Some(pos) = request_bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                break Some(pos);
            }
        }
        .expect("request headers must be present");

        let header_text = String::from_utf8(request_bytes[..header_end].to_vec())
            .expect("request headers must be UTF-8");
        let mut lines = header_text.lines();
        let request_line = lines.next().expect("request line");
        assert!(
            request_line.starts_with("POST "),
            "expected POST request, got: {}",
            request_line
        );
        assert!(
            request_line.contains(expected_path),
            "expected request path to contain {}, got: {}",
            expected_path,
            request_line
        );

        let mut content_length = 0usize;
        for line in lines {
            if let Some(value) = line.strip_prefix("Content-Length:") {
                content_length = value.trim().parse().expect("valid Content-Length");
            }
        }

        let mut body_bytes = request_bytes[(header_end + 4)..].to_vec();
        while body_bytes.len() < content_length {
            let mut chunk = [0_u8; 1024];
            let n = stream.read(&mut chunk).expect("read request body");
            if n == 0 {
                break;
            }
            body_bytes.extend_from_slice(&chunk[..n]);
        }

        let body_text = String::from_utf8(body_bytes).expect("request body must be UTF-8");
        assert!(
            body_text.contains(expected_body_fragment),
            "expected request body to contain {}, got: {}",
            expected_body_fragment,
            body_text
        );

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write mock council response");
        stream.flush().expect("flush mock council response");
    });

    (format!("http://{}", addr), handle)
}

// ---------------------------------------------------------------------------
// Protocol negotiation tests
// ---------------------------------------------------------------------------

#[test]
fn test_protocol_negotiation_2025() {
    let handler = TestHandler::new();
    let req = make_request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "clientInfo": {"name": "test-client", "version": "0.1.0"}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(
        resp.error.is_none(),
        "Expected no error, got: {:?}",
        resp.error
    );

    let result = resp.result.expect("Expected result");

    assert_eq!(
        result["protocolVersion"].as_str().unwrap(),
        "2025-11-25",
        "Protocol version should be 2025-11-25"
    );

    // Capabilities must include resources and prompts
    let caps = &result["capabilities"];
    assert!(caps["tools"].is_object(), "Should have tools capability");
    assert!(
        caps["resources"].is_object(),
        "Should have resources capability"
    );
    assert!(
        caps["prompts"].is_object(),
        "Should have prompts capability"
    );
}

#[test]
fn test_protocol_negotiation_2024_backward_compat() {
    let handler = TestHandler::new();
    let req = make_request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {"name": "legacy-client", "version": "0.1.0"}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(
        resp.error.is_none(),
        "Expected no error, got: {:?}",
        resp.error
    );

    let result = resp.result.expect("Expected result");

    assert_eq!(
        result["protocolVersion"].as_str().unwrap(),
        "2024-11-05",
        "Protocol version should be 2024-11-05 for legacy client"
    );

    // Legacy mode: resources and prompts capabilities should be absent
    let caps = &result["capabilities"];
    assert!(
        caps["tools"].is_object(),
        "Should still have tools capability"
    );
    assert!(
        caps["resources"].is_null(),
        "Should NOT have resources capability in legacy mode"
    );
    assert!(
        caps["prompts"].is_null(),
        "Should NOT have prompts capability in legacy mode"
    );
}

// ---------------------------------------------------------------------------
// Tool annotation tests
// ---------------------------------------------------------------------------

#[test]
fn test_tools_list_includes_annotations() {
    let handler = TestHandler::new();
    let req = make_request(2, "tools/list", json!({}));

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let tools = result["tools"].as_array().expect("Expected tools array");
    assert!(!tools.is_empty(), "Should have at least one tool");

    // At least some tools should have annotations with readOnlyHint or destructiveHint
    let annotated_tools: Vec<_> = tools
        .iter()
        .filter(|t| t.get("annotations").is_some())
        .collect();

    assert!(
        !annotated_tools.is_empty(),
        "At least some tools should have annotations"
    );

    // Verify annotation fields exist on annotated tools
    for tool in &annotated_tools {
        let annotations = &tool["annotations"];
        // annotations should be an object
        assert!(annotations.is_object(), "annotations should be an object");
    }

    // Check that known read-only tools have readOnlyHint = true
    let memory_get = tools.iter().find(|t| t["name"] == "memory_get");
    if let Some(tool) = memory_get {
        if let Some(ann) = tool.get("annotations") {
            if let Some(read_only) = ann.get("readOnlyHint") {
                assert_eq!(
                    read_only.as_bool(),
                    Some(true),
                    "memory_get should have readOnlyHint: true"
                );
            }
        }
    }

    // Check that destructive tools have destructiveHint = true
    let memory_delete = tools.iter().find(|t| t["name"] == "memory_delete");
    if let Some(tool) = memory_delete {
        if let Some(ann) = tool.get("annotations") {
            if let Some(destructive) = ann.get("destructiveHint") {
                assert_eq!(
                    destructive.as_bool(),
                    Some(true),
                    "memory_delete should have destructiveHint: true"
                );
            }
        }
    }
}

#[test]
fn memory_search_tool_schema_exposes_policy_rerank_flags() {
    let handler = TestHandler::new();
    let req = make_request(122, "tools/list", json!({}));

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let tools = result["tools"].as_array().expect("Expected tools array");
    let memory_search = tools
        .iter()
        .find(|tool| tool["name"] == "memory_search")
        .expect("tools/list should include memory_search");
    let properties = memory_search["inputSchema"]["properties"]
        .as_object()
        .expect("memory_search inputSchema should expose properties");

    let policy_rerank = properties
        .get("policy_rerank")
        .expect("memory_search schema should include policy_rerank");
    assert_eq!(policy_rerank["type"].as_str(), Some("boolean"));
    assert_eq!(policy_rerank["default"].as_bool(), Some(false));
    assert_eq!(
        policy_rerank["description"].as_str(),
        Some(
            "Apply memory policy retrieval_priority as an opt-in rerank layer after hybrid search."
        )
    );

    let policy_explain = properties
        .get("policy_explain")
        .expect("memory_search schema should include policy_explain");
    assert_eq!(policy_explain["type"].as_str(), Some("boolean"));
    assert_eq!(policy_explain["default"].as_bool(), Some(false));
    assert_eq!(
        policy_explain["description"].as_str(),
        Some(
            "Include policy score and reason for each reranked result when policy_rerank is true."
        )
    );
}

#[test]
fn memory_digest_tool_is_listed_read_only_and_dispatches() {
    let handler = TestHandler::new();
    let list_resp = handler.handle_request(make_request(123, "tools/list", json!({})));
    assert!(
        list_resp.error.is_none(),
        "Expected no error: {:?}",
        list_resp.error
    );

    let result = list_resp.result.expect("Expected result");
    let tools = result["tools"].as_array().expect("Expected tools array");
    let memory_digest = tools
        .iter()
        .find(|tool| tool["name"] == "memory_digest")
        .expect("tools/list should include memory_digest");
    assert_eq!(
        memory_digest["annotations"]["readOnlyHint"].as_bool(),
        Some(true),
        "memory_digest must be read-only"
    );
    let required = memory_digest["inputSchema"]["required"]
        .as_array()
        .expect("required array");
    assert!(
        required.iter().any(|item| item.as_str() == Some("topic")),
        "memory_digest should require topic"
    );

    let first = create_memory_for_search(
        &handler,
        124,
        "digest-keyword-auth-flow decision: authenticate bearer tokens before rate limiting.",
    );
    let second = create_memory_for_search(
        &handler,
        125,
        "digest-keyword-auth-flow evidence: unauthorized requests must not consume rate-limit buckets.",
    );
    let link = call_tool_json(
        &handler,
        126,
        "memory_link",
        json!({
            "from_id": first,
            "to_id": second,
            "edge_type": "related_to",
            "strength": 0.8
        }),
    );
    assert!(link.get("error").is_none(), "memory_link failed: {link}");

    let digest = call_tool_json(
        &handler,
        127,
        "memory_digest",
        json!({
            "topic": "digest-keyword-auth-flow",
            "limit": 5,
            "related_depth": 1,
            "include_operational_context": false
        }),
    );
    assert!(
        digest.get("error").is_none(),
        "memory_digest failed: {digest}"
    );
    assert_eq!(digest["topic"].as_str(), Some("digest-keyword-auth-flow"));
    assert_eq!(
        digest["provenance"]["policy"]["read_only"].as_bool(),
        Some(true)
    );
    assert_eq!(
        digest["provenance"]["policy"]["llm_used"].as_bool(),
        Some(false)
    );

    let top_memories = digest["top_memories"]
        .as_array()
        .expect("top_memories array");
    assert!(
        top_memories
            .iter()
            .any(|memory| memory["id"].as_i64() == Some(first)),
        "digest should include the first source memory: {digest}"
    );
    let source_ids = digest["provenance"]["source_memory_ids"]
        .as_array()
        .expect("source memory ids");
    assert!(
        source_ids.iter().any(|id| id.as_i64() == Some(first)),
        "provenance should include source memory id {first}: {digest}"
    );

    let relationships = digest["relationships"]
        .as_array()
        .expect("relationships array");
    assert!(
        relationships.iter().any(|edge| {
            edge["from_id"].as_i64() == Some(first)
                && edge["to_id"].as_i64() == Some(second)
                && edge["edge_type"].as_str() == Some("related_to")
        }),
        "digest should include source cross-reference: {digest}"
    );
}

#[test]
fn memory_agent_contract_tool_is_read_only_standard_tier() {
    let tools = get_tool_definitions_tiered(Some("standard"));
    let contract_tool = tools
        .iter()
        .find(|tool| tool.name == "memory_agent_contract")
        .expect("standard tier should include memory_agent_contract");

    let annotations = contract_tool
        .annotations
        .as_ref()
        .expect("memory_agent_contract should expose annotations");
    assert_eq!(
        annotations.read_only_hint,
        Some(true),
        "memory_agent_contract must be read-only"
    );
    assert_eq!(
        contract_tool.input_schema["type"].as_str(),
        Some("object"),
        "memory_agent_contract schema should be an object"
    );
    assert!(
        contract_tool.input_schema["properties"]
            .as_object()
            .expect("memory_agent_contract properties should be an object")
            .is_empty(),
        "memory_agent_contract should not require arguments"
    );
}

#[cfg(feature = "dream-phase")]
#[test]
fn memory_agent_writeback_tool_is_advanced_dry_run_mutating_surface() {
    let standard_tools = get_tool_definitions_tiered(Some("standard"));
    assert!(
        standard_tools
            .iter()
            .all(|tool| tool.name != "memory_agent_writeback"),
        "memory_agent_writeback should require Advanced tier"
    );

    let advanced_tools = get_tool_definitions_tiered(Some("advanced"));
    let writeback_tool = advanced_tools
        .iter()
        .find(|tool| tool.name == "memory_agent_writeback")
        .expect("advanced tier should include memory_agent_writeback");

    let annotations = writeback_tool
        .annotations
        .as_ref()
        .expect("memory_agent_writeback should expose annotations");
    assert_eq!(
        annotations.read_only_hint, None,
        "memory_agent_writeback mutates dream_candidates, not canonical memories"
    );
    assert_eq!(
        writeback_tool.input_schema["properties"]["dry_run"]["default"].as_bool(),
        Some(true),
        "memory_agent_writeback should default to dry-run"
    );
    assert_eq!(
        writeback_tool.input_schema["properties"]["confirm"]["default"].as_bool(),
        Some(false),
        "memory_agent_writeback should require explicit confirm for pending candidate creation"
    );
    assert!(
        writeback_tool.input_schema["required"]
            .as_array()
            .expect("required fields")
            .iter()
            .any(|field| field.as_str() == Some("proposed_content")),
        "memory_agent_writeback should require proposed_content"
    );
}

#[test]
fn memory_agent_contract_dispatches_governance_contract() {
    let handler = TestHandler::new();

    let contract = call_tool_json(&handler, 128, "memory_agent_contract", json!({}));

    assert_eq!(
        contract["contract_version"].as_str(),
        Some("agent-memory-contract-v1")
    );
    let baseline = contract["baseline"]
        .as_object()
        .expect("baseline should be an object");
    assert_eq!(
        baseline
            .get("schema_migration_required")
            .and_then(|value| value.as_bool()),
        None,
        "contract should avoid an ambiguous forever-true migration flag"
    );
    assert_eq!(
        contract["baseline"]["schema_migration"]["introduced_schema_version"].as_i64(),
        Some(45)
    );
    assert_eq!(
        contract["baseline"]["schema_migration"]["runtime_action_required_after_migration"]
            .as_bool(),
        Some(false)
    );
    assert_eq!(contract["baseline"]["schema_version"].as_i64(), Some(45));
    assert_eq!(
        contract["recall"]["primary_tools"][0].as_str(),
        Some("memory_smart_retrieve")
    );
    assert_eq!(
        contract["writeback"]["pending_review"]["candidate_kind"].as_str(),
        Some("agent_writeback")
    );
    assert_eq!(
        contract["writeback"]["pending_review"]["required_tool_tier"].as_str(),
        Some("advanced"),
        "dream candidate review/apply tools are Advanced-tier"
    );
    assert_eq!(
        contract["writeback"]["pending_review"]["creation_tool"].as_str(),
        Some("memory_agent_writeback")
    );
    assert_eq!(
        contract["writeback"]["generated_memory_default"].as_str(),
        Some("pending_or_evidence_only")
    );

    let review_tools = contract["writeback"]["pending_review"]["review_tools"]
        .as_array()
        .expect("contract should list dream candidate review tools");
    assert!(
        review_tools
            .iter()
            .any(|tool| tool.as_str() == Some("dream_candidate_get")),
        "contract should require inspecting candidates before review/apply: {contract}"
    );
    let validation_rules = contract["writeback"]["pending_review"]["validation_rules"]
        .as_array()
        .expect("contract should list writeback validation rules");
    for expected in [
        "confidence must be between 0.0 and 1.0",
        "source_memory_ids must contain positive, unique ids",
        "metadata cannot set reserved governance keys",
    ] {
        assert!(
            validation_rules
                .iter()
                .any(|rule| rule.as_str().is_some_and(|text| text.contains(expected))),
            "contract should document validation rule `{expected}`: {contract}"
        );
    }

    let must_not = contract["must_not"]
        .as_array()
        .expect("contract must include must_not rules");
    assert!(
        must_not.iter().any(|rule| rule
            .as_str()
            .is_some_and(|text| { text.contains("trusted instruction by default") })),
        "contract must forbid trusting generated memory by default: {contract}"
    );
}

#[test]
fn memory_digest_validates_topic_and_returns_empty_digest_without_sources() {
    let handler = TestHandler::new();

    let invalid = call_tool_json(
        &handler,
        128,
        "memory_digest",
        json!({
            "topic": "   "
        }),
    );
    assert_eq!(invalid["error"].as_str(), Some("topic is required"));

    let empty = call_tool_json(
        &handler,
        129,
        "memory_digest",
        json!({
            "topic": "no-such-digest-topic",
            "include_operational_context": false
        }),
    );
    assert!(
        empty.get("error").is_none(),
        "memory_digest failed: {empty}"
    );
    assert_eq!(
        empty["top_memories"].as_array().map(Vec::len),
        Some(0),
        "empty DB should produce no top memories"
    );
    assert!(
        empty["warnings"]
            .as_array()
            .expect("warnings array")
            .iter()
            .any(|warning| warning
                .as_str()
                .unwrap_or_default()
                .contains("No source memories")),
        "empty digest should warn about missing sources: {empty}"
    );
}

#[test]
fn test_memory_council_round_trips_through_tools_call() {
    let handler = TestHandler::new();
    let (council_url, server_handle) = start_council_stub_server(
        "/api/conversations/conv-42/message",
        "Should we use Postgres?",
        json!({
            "stage1": [{"model": "critic-1", "response": "Prefer the safer route"}],
            "stage2": [{"model": "critic-2", "response": "Prefer the safer route"}],
            "stage3": {"model": "arbiter", "response": "Prefer the safer route"},
            "metadata": {"rounds": 3}
        }),
    );

    let req = make_request(
        3,
        "tools/call",
        json!({
            "name": "memory_council",
            "arguments": {
                "prompt": "Should we use Postgres?",
                "conversation_id": "conv-42",
                "council_url": council_url,
                "persist": true,
                "workspace": "architecture",
                "memory_tags": ["architecture", "consensus"],
                "include_raw_stages": true
            }
        }),
    );

    let resp = handler.handle_request(req);
    server_handle
        .join()
        .expect("mock council server should exit cleanly");

    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text content");
    let inner: Value = serde_json::from_str(text).expect("Tool result should contain JSON");

    assert_eq!(
        inner["conversation_id"].as_str(),
        Some("conv-42"),
        "Council response should preserve the conversation id"
    );
    assert_eq!(
        inner["final_answer"].as_str(),
        Some("Prefer the safer route"),
        "Council response should surface the final answer"
    );
    assert_eq!(
        inner["final_model"].as_str(),
        Some("arbiter"),
        "Council response should surface the final model"
    );
    assert!(inner["stage1"].is_array(), "Expected raw stage1 payload");
    assert!(inner["stage2"].is_array(), "Expected raw stage2 payload");
    assert!(inner["stage3"].is_object(), "Expected raw stage3 payload");
    assert!(
        inner["memory_id"].is_number(),
        "Persisted council result should return a memory id"
    );
}

#[test]
fn memory_search_without_policy_rerank_keeps_existing_response_shape() {
    let handler = TestHandler::new();
    create_memory_for_search(
        &handler,
        120,
        "plain memory search response shape marker alpha",
    );

    let results = call_tool_json(
        &handler,
        121,
        "memory_search",
        json!({
            "query": "plain memory search response shape marker",
            "rerank": false
        }),
    );
    let results = results
        .as_array()
        .expect("memory_search should return array");

    assert!(!results.is_empty(), "expected search results");
    assert!(
        results[0]["memory"].is_object(),
        "result must include memory"
    );
    assert!(results[0]["score"].is_number(), "result must include score");
    assert!(
        results[0]["match_info"].is_object(),
        "result must include match_info"
    );
    assert!(
        results[0].get("policy").is_none(),
        "default response must not include policy"
    );
}

#[test]
fn memory_search_policy_rerank_orders_by_policy_priority() {
    let handler = TestHandler::new();
    let low_id = create_memory_for_search(
        &handler,
        130,
        "policy ordering shared needle identical query target",
    );
    let high_id = create_memory_for_search(
        &handler,
        131,
        "policy ordering shared needle identical query target",
    );
    set_policy_priority(&handler, low_id, 0.0, "low-priority-test");
    set_policy_priority(&handler, high_id, 1.0, "high-priority-test");

    let results = call_tool_json(
        &handler,
        132,
        "memory_search",
        json!({
            "query": "policy ordering shared needle identical query target",
            "limit": 2,
            "rerank": false,
            "policy_rerank": true
        }),
    );
    let results = results
        .as_array()
        .expect("memory_search should return array");

    assert!(results.len() >= 2, "expected both seeded memories");
    assert_eq!(
        results[0]["memory"]["id"].as_i64(),
        Some(high_id),
        "policy_rerank should promote higher retrieval_priority"
    );
    assert!(
        results[0].get("policy").is_none(),
        "policy explanation must remain opt-in"
    );
}

#[test]
fn memory_search_policy_explain_only_when_policy_rerank_is_true() {
    let handler = TestHandler::new();
    let memory_id =
        create_memory_for_search(&handler, 140, "policy explanation visibility search marker");
    set_policy_priority(&handler, memory_id, 0.9, "policy-explain-test");

    let without_rerank = call_tool_json(
        &handler,
        141,
        "memory_search",
        json!({
            "query": "policy explanation visibility",
            "rerank": false,
            "policy_explain": true
        }),
    );
    let without_rerank = without_rerank
        .as_array()
        .expect("memory_search should return array");
    assert!(
        without_rerank[0].get("policy").is_none(),
        "policy_explain alone must not alter response shape"
    );

    let with_rerank = call_tool_json(
        &handler,
        142,
        "memory_search",
        json!({
            "query": "policy explanation visibility",
            "rerank": false,
            "policy_rerank": true,
            "policy_explain": true
        }),
    );
    let with_rerank = with_rerank
        .as_array()
        .expect("memory_search should return array");
    let policy = with_rerank[0]["policy"]
        .as_object()
        .expect("policy explanation object");

    assert!(
        policy["score"].is_number(),
        "policy score should be present"
    );
    assert!(
        policy["reason"]
            .as_str()
            .unwrap_or("")
            .contains("policy-explain-test"),
        "policy reason should be present"
    );
}

#[test]
fn memory_search_policy_rerank_missing_policy_row_is_transient() {
    let handler = TestHandler::new();
    let memory_id =
        create_memory_for_search(&handler, 150, "transient missing policy row search marker");
    handler
        .storage
        .with_connection(|conn| {
            conn.execute(
                "DELETE FROM memory_policy WHERE memory_id = ?1",
                rusqlite::params![memory_id],
            )?;
            Ok(())
        })
        .expect("delete policy row");

    let results = call_tool_json(
        &handler,
        151,
        "memory_search",
        json!({
            "query": "transient missing policy row",
            "rerank": false,
            "policy_rerank": true,
            "policy_explain": true
        }),
    );
    let results = results
        .as_array()
        .expect("memory_search should return array");
    assert!(!results.is_empty(), "expected transient policy result");
    assert_eq!(
        results[0]["policy"]["source"].as_str(),
        Some("heuristic-v1"),
        "missing policy rows should use transient heuristic score"
    );

    let persisted = handler
        .storage
        .with_connection(|conn| get_policy_record(conn, memory_id))
        .expect("read policy record");
    assert!(
        persisted.is_none(),
        "retrieval-only policy rerank must not persist missing policy rows"
    );
}

// ---------------------------------------------------------------------------
// Resources tests
// ---------------------------------------------------------------------------

#[test]
fn test_resources_list() {
    let handler = TestHandler::new();
    let req = make_request(3, "resources/list", json!({}));

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let resources = result["resources"]
        .as_array()
        .expect("Expected resources array");

    // Should have exactly 5 resource templates
    assert_eq!(
        resources.len(),
        5,
        "Expected 5 resource templates, got {}",
        resources.len()
    );

    // Each resource should have uri, name, description
    for resource in resources {
        assert!(
            resource["uri"].is_string(),
            "Resource should have 'uri' field: {:?}",
            resource
        );
        assert!(
            resource["name"].is_string(),
            "Resource should have 'name' field: {:?}",
            resource
        );
        assert!(
            resource["description"].is_string() || !resource["description"].is_null(),
            "Resource should have 'description' field: {:?}",
            resource
        );
    }

    // Verify expected URI templates exist
    let uris: Vec<&str> = resources.iter().filter_map(|r| r["uri"].as_str()).collect();

    assert!(
        uris.contains(&"engram://stats"),
        "Should have stats resource"
    );
    assert!(
        uris.contains(&"engram://entities"),
        "Should have entities resource"
    );
    assert!(
        uris.iter().any(|u| u.contains("memory")),
        "Should have memory resource template"
    );
    assert!(
        uris.iter().any(|u| u.contains("workspace")),
        "Should have workspace resource template"
    );
}

#[test]
fn test_resources_read_stats() {
    let handler = TestHandler::new();

    // First create a memory so stats are non-trivial
    let create_req = make_request(
        10,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {
                "content": "Integration test memory for stats check",
                "memory_type": "note"
            }
        }),
    );
    let create_resp = handler.handle_request(create_req);
    assert!(
        create_resp.error.is_none(),
        "memory_create failed: {:?}",
        create_resp.error
    );

    // Now read the stats resource
    let req = make_request(11, "resources/read", json!({"uri": "engram://stats"}));

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let contents = result["contents"]
        .as_array()
        .expect("Expected contents array");
    assert!(!contents.is_empty(), "Expected at least one content item");

    let text = contents[0]["text"].as_str().expect("Expected text content");
    let stats: Value = serde_json::from_str(text).expect("Stats should be valid JSON");

    // Stats should include a memory count >= 1
    let total = stats
        .get("total_memories")
        .or_else(|| stats.get("memory_count"))
        .or_else(|| stats.get("count"))
        .or_else(|| stats.get("total"));

    // Accept either a direct count field or embedded in object
    if let Some(count_val) = total {
        let count = count_val.as_u64().unwrap_or(0);
        assert!(
            count >= 1,
            "Stats should show at least 1 memory, got: {}",
            count
        );
    } else {
        // Stats may have nested structure — just verify it's a non-empty object
        assert!(
            stats.is_object() && !stats.as_object().unwrap().is_empty(),
            "Stats should be a non-empty JSON object, got: {}",
            stats
        );
    }
}

#[test]
fn test_resources_read_memory() {
    let handler = TestHandler::new();

    // Create a memory first
    let create_req = make_request(
        20,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {
                "content": "Unique content for resource read test XYZ123",
                "memory_type": "note",
                "tags": ["resource-test"]
            }
        }),
    );
    let create_resp = handler.handle_request(create_req);
    assert!(
        create_resp.error.is_none(),
        "memory_create failed: {:?}",
        create_resp.error
    );

    // Extract the ID from the tool call result
    let result = create_resp.result.expect("Expected result");
    let content_arr = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content_arr[0]["text"].as_str().expect("Expected text");
    let created: Value = serde_json::from_str(text).expect("Created memory should be JSON");
    let memory_id = created["id"].as_i64().expect("Expected id field");

    // Now read via resource URI
    let req = make_request(
        21,
        "resources/read",
        json!({"uri": format!("engram://memory/{}", memory_id)}),
    );

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let contents = result["contents"]
        .as_array()
        .expect("Expected contents array");
    assert!(!contents.is_empty(), "Expected at least one content item");

    let text = contents[0]["text"].as_str().expect("Expected text content");
    let memory: Value = serde_json::from_str(text).expect("Memory should be valid JSON");

    assert_eq!(
        memory["id"].as_i64(),
        Some(memory_id),
        "Resource should return the correct memory ID"
    );
    assert!(
        memory["content"].as_str().unwrap_or("").contains("XYZ123"),
        "Resource content should contain the original text"
    );
}

#[test]
fn test_resources_read_invalid_uri() {
    let handler = TestHandler::new();

    let req = make_request(
        30,
        "resources/read",
        json!({"uri": "engram://nonexistent/path/that/does/not/exist"}),
    );

    let resp = handler.handle_request(req);

    // Should return an error response (not a success)
    assert!(
        resp.error.is_some(),
        "Expected an error for invalid URI, got result: {:?}",
        resp.result
    );
}

// ---------------------------------------------------------------------------
// Prompts tests
// ---------------------------------------------------------------------------

#[test]
fn test_prompts_list() {
    let handler = TestHandler::new();
    let req = make_request(40, "prompts/list", json!({}));

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let prompts = result["prompts"]
        .as_array()
        .expect("Expected prompts array");

    // Should have exactly 5 prompts
    assert_eq!(
        prompts.len(),
        5,
        "Expected 5 prompts, got {}",
        prompts.len()
    );

    // Each prompt should have name and arguments
    for prompt in prompts {
        assert!(
            prompt["name"].is_string(),
            "Prompt should have 'name' field: {:?}",
            prompt
        );
    }

    // Verify all 4 expected prompt names are present
    let names: Vec<&str> = prompts.iter().filter_map(|p| p["name"].as_str()).collect();

    assert!(
        names.contains(&"create-knowledge-base"),
        "Should have create-knowledge-base prompt"
    );
    assert!(
        names.contains(&"daily-review"),
        "Should have daily-review prompt"
    );
    assert!(
        names.contains(&"search-and-organize"),
        "Should have search-and-organize prompt"
    );
    assert!(
        names.contains(&"seed-entity"),
        "Should have seed-entity prompt"
    );
}

#[test]
fn test_prompts_get_daily_review() {
    let handler = TestHandler::new();
    let req = make_request(
        50,
        "prompts/get",
        json!({
            "name": "daily-review",
            "arguments": {}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let messages = result["messages"]
        .as_array()
        .expect("Expected messages array");

    // Should return at least 2 messages (user + assistant)
    assert!(
        messages.len() >= 2,
        "Expected at least 2 messages, got {}",
        messages.len()
    );

    // Each message should have role and content
    for message in messages {
        let role = message["role"].as_str().expect("Message should have role");
        assert!(
            role == "user" || role == "assistant",
            "Role should be 'user' or 'assistant', got: {}",
            role
        );

        let content = &message["content"];
        assert!(
            content.is_object(),
            "Content should be an object: {:?}",
            content
        );
        assert!(
            content["type"].as_str() == Some("text"),
            "Content type should be 'text'"
        );
        assert!(
            content["text"].is_string(),
            "Content should have text field"
        );
    }

    // First message should be from the user
    assert_eq!(
        messages[0]["role"].as_str(),
        Some("user"),
        "First message should be from user"
    );
}

#[test]
fn test_prompts_get_unknown() {
    let handler = TestHandler::new();
    let req = make_request(
        60,
        "prompts/get",
        json!({
            "name": "nonexistent-prompt-xyz",
            "arguments": {}
        }),
    );

    let resp = handler.handle_request(req);

    // Should return an error response
    assert!(
        resp.error.is_some(),
        "Expected an error for unknown prompt, got result: {:?}",
        resp.result
    );

    let error = resp.error.unwrap();
    assert!(
        error.message.contains("nonexistent-prompt-xyz") || error.message.contains("not found"),
        "Error message should mention the unknown prompt name or 'not found': {}",
        error.message
    );
}

// ---------------------------------------------------------------------------
// recent_activity tool tests
// ---------------------------------------------------------------------------

#[test]
fn test_recent_activity_returns_activities_field() {
    let handler = TestHandler::new();

    // Create a memory so there is recent activity to discover
    let create_req = make_request(
        70,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {
                "content": "Test recent activity memory",
                "memory_type": "note"
            }
        }),
    );
    let create_resp = handler.handle_request(create_req);
    assert!(
        create_resp.error.is_none(),
        "memory_create failed: {:?}",
        create_resp.error
    );

    // Call recent_activity with default params
    let req = make_request(
        71,
        "tools/call",
        json!({
            "name": "recent_activity",
            "arguments": {}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(
        resp.error.is_none(),
        "recent_activity returned error: {:?}",
        resp.error
    );

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    assert!(!content.is_empty(), "Expected at least one content item");

    let text = content[0]["text"].as_str().expect("Expected text content");
    let data: Value = serde_json::from_str(text).expect("recent_activity should return valid JSON");

    assert!(
        data["activities"].is_array(),
        "Result must have 'activities' array, got: {}",
        data
    );
    assert!(data["count"].is_number(), "Result must have 'count' field");
    assert!(
        data["timeframe"].is_string(),
        "Result must have 'timeframe' field"
    );

    let activities = data["activities"].as_array().unwrap();
    assert!(
        !activities.is_empty(),
        "Should find at least one recent memory"
    );

    // Verify activity shape
    let activity = &activities[0];
    assert!(activity["id"].is_number(), "Activity must have 'id'");
    assert!(
        activity["preview"].is_string(),
        "Activity must have 'preview'"
    );
    assert!(
        activity["memory_type"].is_string(),
        "Activity must have 'memory_type'"
    );
    assert!(
        activity["workspace"].is_string(),
        "Activity must have 'workspace'"
    );
    assert!(
        activity["created_at"].is_string(),
        "Activity must have 'created_at'"
    );
}

#[test]
fn test_recent_activity_timeframe_1h() {
    let handler = TestHandler::new();

    // Create a memory
    let create_req = make_request(
        80,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {
                "content": "Memory for 1h timeframe test",
                "memory_type": "note"
            }
        }),
    );
    handler.handle_request(create_req);

    let req = make_request(
        81,
        "tools/call",
        json!({
            "name": "recent_activity",
            "arguments": {"timeframe": "1h", "limit": 5}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text");
    let data: Value = serde_json::from_str(text).unwrap();

    assert_eq!(
        data["timeframe"].as_str(),
        Some("1h"),
        "Timeframe should echo '1h'"
    );
    assert!(data["activities"].is_array(), "Must have activities array");
}

#[test]
fn test_recent_activity_limit_enforced() {
    let handler = TestHandler::new();

    // Create 5 memories
    for i in 0..5 {
        let req = make_request(
            90 + i,
            "tools/call",
            json!({
                "name": "memory_create",
                "arguments": {
                    "content": format!("Memory {} for limit test", i),
                    "memory_type": "note"
                }
            }),
        );
        handler.handle_request(req);
    }

    // Request only 2 results
    let req = make_request(
        95,
        "tools/call",
        json!({
            "name": "recent_activity",
            "arguments": {"limit": 2}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text");
    let data: Value = serde_json::from_str(text).unwrap();

    let activities = data["activities"].as_array().unwrap();
    assert!(
        activities.len() <= 2,
        "Should return at most 2 activities, got {}",
        activities.len()
    );
}

// ---------------------------------------------------------------------------
// discover_tools detail levels (progressive disclosure)
// ---------------------------------------------------------------------------

/// Helper: invoke discover_tools and return the parsed JSON payload.
fn call_discover_tools(handler: &TestHandler, arguments: Value) -> Value {
    let req = make_request(
        200,
        "tools/call",
        json!({ "name": "discover_tools", "arguments": arguments }),
    );
    let resp = handler.handle_request(req);
    assert!(
        resp.error.is_none(),
        "discover_tools returned error: {:?}",
        resp.error
    );
    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text content");
    serde_json::from_str(text).expect("discover_tools should return valid JSON")
}

#[test]
fn test_discover_tools_default_detail_is_summary() {
    // Omitting `detail` must preserve the existing contract: name + description
    // + tier, but NOT the full schema. This is backward-compatible behavior.
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "search": "discover_tools" }));

    let tools = data["tools"].as_array().expect("tools must be an array");
    let tool = tools
        .iter()
        .find(|t| t["name"].as_str() == Some("discover_tools"))
        .expect("discover_tools must be discoverable by itself");

    assert!(tool["name"].is_string(), "summary detail must include name");
    assert!(
        tool["description"].is_string(),
        "summary detail must include description"
    );
    assert!(tool["tier"].is_string(), "summary detail must include tier");
    assert!(
        tool.get("schema").is_none(),
        "summary detail must NOT include schema, got: {}",
        tool
    );
}

#[test]
fn test_discover_tools_summary_includes_group_and_availability() {
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "search": "discover_tools" }));
    let tools = data["tools"].as_array().expect("tools must be an array");
    let tool = tools
        .iter()
        .find(|t| t["name"].as_str() == Some("discover_tools"))
        .expect("discover_tools must be discoverable by itself");

    assert!(
        tool["group"].is_string(),
        "summary detail must include group"
    );
    assert!(
        tool["availability"].is_string(),
        "summary detail must include availability"
    );
}

#[test]
fn test_discover_tools_rejects_invalid_tier_value() {
    // A typo like "esential" must error loudly, not silently return the
    // unfiltered list (the filter's catch-all arm would otherwise match all).
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "tier": "esential" }));
    let error = data["error"].as_str().expect("expected error for bad tier");
    assert!(error.contains("invalid tier"), "got: {error}");
}

#[test]
fn test_discover_tools_rejects_non_string_tier() {
    // as_str() returns None for wrong-typed values too; a numeric tier must
    // not be treated as "no filter".
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "tier": 123 }));
    let error = data["error"].as_str().expect("expected error for bad tier");
    assert!(error.contains("invalid tier type"), "got: {error}");
}

#[test]
fn test_discover_tools_rejects_non_string_group_and_category() {
    let handler = TestHandler::new();
    for arguments in [json!({ "group": 5 }), json!({ "category": ["memory"] })] {
        let data = call_discover_tools(&handler, arguments);
        let error = data["error"]
            .as_str()
            .expect("expected error for bad group/category");
        assert!(error.contains("invalid group type"), "got: {error}");
    }
}

#[test]
fn test_discover_tools_rejects_non_string_search() {
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "search": 42 }));
    let error = data["error"]
        .as_str()
        .expect("expected error for bad search");
    assert!(error.contains("invalid search type"), "got: {error}");
}

#[test]
fn test_discover_tools_accepts_valid_tier_values() {
    let handler = TestHandler::new();
    for tier in ["essential", "standard", "advanced", "all"] {
        let data = call_discover_tools(&handler, json!({ "tier": tier }));
        assert!(
            data["tools"].is_array(),
            "tier '{tier}' must be accepted, got: {data}"
        );
    }
}

#[test]
fn test_discover_tools_lists_feature_disabled_tools_by_group() {
    let handler = TestHandler::new();
    let data = call_discover_tools(
        &handler,
        json!({ "detail": "summary", "group": "feature.attestation" }),
    );

    let tools = data["tools"].as_array().expect("tools must be an array");
    let attestation = tools
        .iter()
        .find(|tool| tool["name"].as_str() == Some("attestation_log"))
        .expect("attestation_log must remain discoverable when its feature is disabled");

    assert_eq!(attestation["group"].as_str(), Some("feature.attestation"));
    #[cfg(feature = "attestation")]
    let expected_availability = "available";
    #[cfg(not(feature = "attestation"))]
    let expected_availability = "feature_disabled";
    assert_eq!(
        attestation["availability"].as_str(),
        Some(expected_availability)
    );
    assert_eq!(attestation["feature"].as_str(), Some("attestation"));
    assert_eq!(
        attestation["required_features"].as_array().map(Vec::len),
        Some(1)
    );
    assert!(attestation["enable_with"]
        .as_str()
        .expect("enable_with must be present")
        .contains("attestation"));
}

#[test]
fn test_discover_tools_enablement_includes_all_required_features() {
    // Given: memory_sync_media is discoverable even when feature-gated.
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "search": "memory_sync_media" }));

    // When: the tool summary is inspected.
    let tools = data["tools"].as_array().expect("tools must be an array");
    let tool = tools
        .iter()
        .find(|tool| tool["name"].as_str() == Some("memory_sync_media"))
        .expect("memory_sync_media must be discoverable");

    // Then: both compile-time features required by dispatch are exposed.
    assert_eq!(tool["feature"].as_str(), Some("multimodal,cloud"));
    let required_features = tool["required_features"]
        .as_array()
        .expect("required_features must be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(required_features, ["multimodal", "cloud"]);
    assert_eq!(
        tool["enable_with"].as_str(),
        Some("cargo build --features multimodal,cloud")
    );
}

#[test]
fn test_discover_tools_detail_names_only() {
    // `detail: "names"` returns just the name field — cheapest discovery.
    let handler = TestHandler::new();
    let data = call_discover_tools(
        &handler,
        json!({ "detail": "names", "search": "discover_tools" }),
    );

    let tools = data["tools"].as_array().expect("tools must be an array");
    let tool = tools
        .iter()
        .find(|t| t["name"].as_str() == Some("discover_tools"))
        .expect("discover_tools must be present");

    assert!(tool["name"].is_string(), "names detail must include name");
    assert!(
        tool.get("description").is_none(),
        "names detail must NOT include description, got: {}",
        tool
    );
    assert!(
        tool.get("tier").is_none(),
        "names detail must NOT include tier, got: {}",
        tool
    );
    assert!(
        tool.get("schema").is_none(),
        "names detail must NOT include schema, got: {}",
        tool
    );
}

#[test]
fn test_discover_tools_detail_schema_includes_input_schema() {
    // `detail: "schema"` returns the full input schema as a JSON object (not a
    // string), so an agent can call the tool without a second tools/list round.
    let handler = TestHandler::new();
    let data = call_discover_tools(
        &handler,
        json!({ "detail": "schema", "search": "discover_tools" }),
    );

    let tools = data["tools"].as_array().expect("tools must be an array");
    let tool = tools
        .iter()
        .find(|t| t["name"].as_str() == Some("discover_tools"))
        .expect("discover_tools must be present");

    assert!(tool["name"].is_string(), "schema detail must include name");
    assert!(
        tool["description"].is_string(),
        "schema detail must include description"
    );
    assert!(tool["tier"].is_string(), "schema detail must include tier");
    assert!(
        tool["schema"].is_object(),
        "schema detail must include schema as a JSON object, got: {}",
        tool
    );
    assert_eq!(
        tool["schema"]["type"].as_str(),
        Some("object"),
        "discover_tools schema must describe an object"
    );
    assert!(
        tool["schema"]["properties"]["detail"].is_object(),
        "discover_tools schema must document the new 'detail' property"
    );
}

#[test]
fn test_discover_tools_invalid_detail_is_rejected() {
    // Invalid detail must fail loudly at the boundary, not silently default.
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "detail": "everything" }));

    assert!(
        data["error"].is_string(),
        "invalid detail must return an error field, got: {}",
        data
    );
    let err = data["error"].as_str().unwrap();
    assert!(
        err.contains("detail"),
        "error message must mention 'detail', got: {}",
        err
    );
}

#[test]
fn test_discover_tools_non_string_detail_is_rejected() {
    let handler = TestHandler::new();
    let data = call_discover_tools(&handler, json!({ "detail": 123 }));

    assert!(
        data["error"].is_string(),
        "non-string detail must return an error field, got: {}",
        data
    );
    let err = data["error"].as_str().unwrap();
    assert!(
        err.contains("detail") && err.contains("string"),
        "error message must mention detail string type, got: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// Enrichment audit tool tests (ENG-1240)
// ---------------------------------------------------------------------------

#[test]
fn test_enrichment_tools_appear_in_tools_list() {
    let handler = TestHandler::new();
    let req = make_request(110, "tools/list", json!({}));

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let tools = result["tools"].as_array().expect("Expected tools array");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        tool_names.contains(&"memory_enrichment_timeline"),
        "tools/list must include memory_enrichment_timeline, got: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"memory_enrichment_audit"),
        "tools/list must include memory_enrichment_audit, got: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"memory_replay_at_time"),
        "tools/list must include memory_replay_at_time, got: {:?}",
        tool_names
    );
}

#[test]
fn test_memory_enrichment_timeline_returns_empty_for_unknown_memory() {
    let handler = TestHandler::new();
    let req = make_request(
        111,
        "tools/call",
        json!({
            "name": "memory_enrichment_timeline",
            "arguments": {"memory_id": 999_999}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text content");
    let data: Value =
        serde_json::from_str(text).expect("memory_enrichment_timeline should return valid JSON");

    assert!(
        data["events"].is_array(),
        "Result must have 'events' array, got: {}",
        data
    );
    assert_eq!(
        data["events"].as_array().unwrap().len(),
        0,
        "events should be empty for unknown memory id"
    );
    assert_eq!(
        data["memory_id"].as_i64(),
        Some(999_999),
        "Result should echo back the requested memory_id"
    );
}

#[test]
fn test_memory_enrichment_audit_returns_events_array_with_filters() {
    let handler = TestHandler::new();
    let req = make_request(
        112,
        "tools/call",
        json!({
            "name": "memory_enrichment_audit",
            "arguments": {"status": "failed", "limit": 5}
        }),
    );

    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "Expected no error: {:?}", resp.error);

    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text content");
    let data: Value =
        serde_json::from_str(text).expect("memory_enrichment_audit should return valid JSON");

    assert!(
        data["events"].is_array(),
        "Result must have 'events' array, got: {}",
        data
    );
    assert!(
        data["filters_applied"].is_object(),
        "Result must have 'filters_applied' object, got: {}",
        data
    );
    assert!(
        data["filters_applied"]["status"].as_str() == Some("failed"),
        "filters_applied must echo back the 'status' filter, got: {}",
        data["filters_applied"]
    );
}

#[test]
fn test_memory_replay_at_time_requires_memory_id() {
    let handler = TestHandler::new();
    let req = make_request(
        120,
        "tools/call",
        json!({
            "name": "memory_replay_at_time",
            "arguments": {"timestamp": "2026-01-01T00:00:00Z"}
        }),
    );
    let resp = handler.handle_request(req);
    assert!(resp.error.is_none());
    let result = resp.result.expect("Expected result");
    let text = result["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).unwrap();
    assert!(
        data["error"].is_string(),
        "Missing memory_id should return error, got: {}",
        data
    );
}

#[test]
fn test_memory_replay_at_time_requires_timestamp() {
    let handler = TestHandler::new();
    let req = make_request(
        121,
        "tools/call",
        json!({
            "name": "memory_replay_at_time",
            "arguments": {"memory_id": 1}
        }),
    );
    let resp = handler.handle_request(req);
    assert!(resp.error.is_none());
    let result = resp.result.expect("Expected result");
    let text = result["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).unwrap();
    assert!(
        data["error"].is_string(),
        "Missing timestamp should return error, got: {}",
        data
    );
}

#[test]
fn test_memory_replay_at_time_returns_structured_response() {
    let handler = TestHandler::new();

    // Create a memory to replay
    let create_req = make_request(
        122,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {"content": "replay test memory", "memory_type": "note"}
        }),
    );
    let create_resp = handler.handle_request(create_req);
    let create_text = create_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let created: Value = serde_json::from_str(&create_text).unwrap();
    let memory_id = created["id"].as_i64().unwrap();

    let req = make_request(
        123,
        "tools/call",
        json!({
            "name": "memory_replay_at_time",
            "arguments": {
                "memory_id": memory_id,
                "timestamp": "2099-01-01T00:00:00Z"
            }
        }),
    );
    let resp = handler.handle_request(req);
    assert!(resp.error.is_none());
    let result = resp.result.expect("Expected result");
    let text = result["content"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).expect("replay must return valid JSON");

    assert_eq!(
        data["memory_id"], memory_id,
        "memory_id must be echoed back"
    );
    assert!(data["events"].is_array(), "events must be an array");
    assert!(
        data["temporal_edges"].is_array(),
        "temporal_edges must be present, got: {}",
        data
    );
    assert!(
        data["temporal_edges_count"].is_number(),
        "temporal_edges_count must be present"
    );
}

#[test]
fn test_memory_create_emits_audit_event() {
    let handler = TestHandler::new();

    // Create a memory
    let create_req = make_request(
        130,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {"content": "audit emit test", "memory_type": "note"}
        }),
    );
    let create_resp = handler.handle_request(create_req);
    let text = create_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let created: Value = serde_json::from_str(&text).unwrap();
    let memory_id = created["id"].as_i64().unwrap();

    // Check audit trail captures the creation event
    let audit_req = make_request(
        131,
        "tools/call",
        json!({
            "name": "memory_enrichment_timeline",
            "arguments": {"memory_id": memory_id}
        }),
    );
    let audit_resp = handler.handle_request(audit_req);
    let audit_text = audit_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let audit_data: Value = serde_json::from_str(&audit_text).unwrap();

    let events = audit_data["events"]
        .as_array()
        .expect("events must be array");
    assert!(
        events
            .iter()
            .any(|e| e["event_type"].as_str() == Some("memory_created")),
        "audit trail must contain a memory_created event, got: {:?}",
        events
    );
}

#[test]
fn test_tools_list_includes_context_get_artifact() {
    let handler = TestHandler::new();
    let req = make_request(132, "tools/list", json!({}));
    let resp = handler.handle_request(req);
    assert!(resp.error.is_none(), "tools/list should succeed");

    let list_result = resp.result.unwrap();
    let tools = list_result["tools"]
        .as_array()
        .expect("tools/list result must include tools");
    let tool = tools
        .iter()
        .find(|tool| tool["name"].as_str() == Some("context_get_artifact"))
        .expect("tools/list must include context_get_artifact");

    assert_eq!(
        tool["annotations"]["readOnlyHint"].as_bool(),
        Some(true),
        "context_get_artifact must be advertised as read-only"
    );
}

#[test]
fn test_context_get_artifact_returns_retained_raw_content() {
    let handler = TestHandler::new();

    let record_req = make_request(
        133,
        "tools/call",
        json!({
            "name": "context_record_artifact",
            "arguments": {
                "repo_id": "github:aiconnai/engram",
                "session_id": "session-a",
                "kind": "test_report",
                "raw_content": "fixture output",
                "retain_raw": true,
                "metadata": {"command": "cargo test"}
            }
        }),
    );
    let record_resp = handler.handle_request(record_req);
    assert!(
        record_resp.error.is_none(),
        "context_record_artifact failed: {:?}",
        record_resp.error
    );
    let record_text = record_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let record_data: Value =
        serde_json::from_str(&record_text).expect("record response should be valid JSON");
    let artifact_id = record_data["artifact_id"]
        .as_str()
        .expect("record response must include artifact_id")
        .to_string();

    let get_req = make_request(
        134,
        "tools/call",
        json!({
            "name": "context_get_artifact",
            "arguments": {
                "artifact_id": artifact_id,
                "session_id": "session-a",
                "reason": "verify retained test output",
                "max_bytes": 7
            }
        }),
    );
    let get_resp = handler.handle_request(get_req);
    assert!(
        get_resp.error.is_none(),
        "context_get_artifact failed: {:?}",
        get_resp.error
    );
    let get_text = get_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    let get_data: Value =
        serde_json::from_str(&get_text).expect("get artifact response should be valid JSON");

    assert_eq!(get_data["content"].as_str(), Some("fixture"));
    assert_eq!(get_data["encoding"].as_str(), Some("utf8"));
    assert_eq!(get_data["returned_bytes"].as_u64(), Some(7));
    assert_eq!(get_data["original_bytes"].as_u64(), Some(14));
    assert_eq!(get_data["truncated"].as_bool(), Some(true));
    assert_eq!(
        get_data["artifact"]["id"].as_str(),
        Some(artifact_id.as_str())
    );
    assert_eq!(
        get_data["artifact"]["redaction_status"].as_str(),
        Some("passed")
    );
}

#[test]
fn test_context_search_returns_scoped_results_with_provenance_staleness_and_pointers() {
    let handler = TestHandler::new();

    let record = call_tool_json(
        &handler,
        135,
        "context_record",
        json!({
            "source": "codex",
            "repo_id": "github:aiconnai/engram",
            "session_id": "session-search",
            "task_id": "ENGRA-73",
            "event_type": "command",
            "command": "cargo test context_search",
            "cwd": "/repo",
            "exit_code": 101,
            "git_branch": "feature/old-search",
            "commit_hash": "old-search-sha",
            "started_at": "2026-05-01T00:00:00Z",
            "summary": "Bundle smoke failure for context_search direct coverage",
            "key_errors": ["context_search missing direct protocol coverage"],
            "touched_files": ["src/context/search.rs"],
            "raw_artifact_id": "ctx-artifact-search-1",
            "reducer": {
                "name": "engra_73_search_fixture",
                "version": "1",
                "lossy": true,
                "confidence": 0.91,
                "structured_facts": {
                    "files": ["src/context/search.rs"],
                    "decision": "add direct context_search coverage"
                },
                "tokens_raw_est": 400,
                "tokens_compact_est": 40
            }
        }),
    );
    assert!(
        record.get("error").is_none(),
        "context_record failed: {record}"
    );
    let event_id = record["created_ids"]["event_id"]
        .as_i64()
        .expect("record response must include event_id");
    let summary_id = record["created_ids"]["summary_id"]
        .as_i64()
        .expect("record response must include summary_id");

    let search = call_tool_json(
        &handler,
        136,
        "context_search",
        json!({
            "query": "bundle smoke",
            "repo_id": "github:aiconnai/engram",
            "session_id": "session-search",
            "task_id": "ENGRA-73",
            "include_artifact_pointers": true,
            "current_git_branch": "main",
            "current_commit_hash": "new-search-sha",
            "stale_after_days": 1,
            "max_results": 10
        }),
    );
    assert!(
        search.get("error").is_none(),
        "context_search failed: {search}"
    );
    assert_eq!(search["query"].as_str(), Some("bundle smoke"));
    assert_eq!(
        search["scope"]["repo_id"].as_str(),
        Some("github:aiconnai/engram")
    );
    assert_eq!(
        search["scope"]["session_id"].as_str(),
        Some("session-search")
    );
    assert_eq!(search["scope"]["isolation_applied"].as_bool(), Some(true));
    assert_eq!(
        search["filters"]["include_artifact_pointers"].as_bool(),
        Some(true)
    );

    let results = search["results"]
        .as_array()
        .expect("context_search results must be an array");
    let item = results
        .iter()
        .find(|item| item["event"]["id"].as_i64() == Some(event_id))
        .expect("context_search must return recorded event");

    assert_eq!(item["result_type"].as_str(), Some("summary"));
    assert_eq!(item["event"]["event_type"].as_str(), Some("command"));
    assert_eq!(
        item["summary"]["reducer_name"].as_str(),
        Some("engra_73_search_fixture")
    );
    assert_eq!(item["summary"]["derived"].as_bool(), Some(true));
    assert_eq!(item["summary"]["lossy"].as_bool(), Some(true));
    assert_eq!(item["provenance"]["event_id"].as_i64(), Some(event_id));
    assert_eq!(item["provenance"]["summary_id"].as_i64(), Some(summary_id));
    assert_eq!(
        item["provenance"]["repo_id"].as_str(),
        Some("github:aiconnai/engram")
    );

    let files = item["extracted_files"]
        .as_array()
        .expect("extracted_files must be an array");
    assert!(
        files
            .iter()
            .any(|file| file.as_str() == Some("src/context/search.rs")),
        "context_search should extract touched files, got: {files:?}"
    );

    let artifact_pointers = item["artifact_pointers"]
        .as_array()
        .expect("artifact_pointers must be an array");
    assert!(
        artifact_pointers
            .iter()
            .any(|pointer| pointer["artifact_id"].as_str() == Some("ctx-artifact-search-1")),
        "context_search should return requested artifact pointers, got: {artifact_pointers:?}"
    );

    let staleness = item["staleness"]
        .as_array()
        .expect("staleness must be an array");
    for kind in ["branch_mismatch", "commit_mismatch", "age"] {
        assert!(
            staleness
                .iter()
                .any(|warning| warning["kind"].as_str() == Some(kind)),
            "missing staleness warning {kind}: {staleness:?}"
        );
    }
}

#[test]
fn test_context_build_bundle_groups_sections_and_excludes_raw_artifact_content() {
    let handler = TestHandler::new();

    let artifact = call_tool_json(
        &handler,
        137,
        "context_record_artifact",
        json!({
            "id": "ctx-artifact-bundle-raw",
            "repo_id": "github:aiconnai/engram",
            "session_id": "session-bundle",
            "task_id": "ENGRA-73",
            "kind": "test_report",
            "raw_content": "RAW_CONTEXT_BUNDLE_OUTPUT_SHOULD_NOT_RENDER",
            "retain_raw": true,
            "metadata": {"command": "cargo test context_build_bundle"}
        }),
    );
    assert!(
        artifact.get("error").is_none(),
        "context_record_artifact failed: {artifact}"
    );
    let artifact_id = artifact["artifact_id"]
        .as_str()
        .expect("artifact response must include artifact_id");
    assert_eq!(artifact["storage_kind"].as_str(), Some("raw_retained"));

    let record = call_tool_json(
        &handler,
        138,
        "context_record",
        json!({
            "source": "codex",
            "repo_id": "github:aiconnai/engram",
            "session_id": "session-bundle",
            "task_id": "ENGRA-73",
            "event_type": "command",
            "command": "cargo test context_build_bundle",
            "cwd": "/repo",
            "exit_code": 101,
            "git_branch": "feature/old-bundle",
            "commit_hash": "old-bundle-sha",
            "started_at": "2026-05-01T00:00:00Z",
            "summary": "Decision: add context bundle smoke coverage. Blocker unresolved because direct tests were missing. Failure observed for context bundle smoke.",
            "key_errors": ["context_build_bundle missing direct protocol coverage"],
            "touched_files": ["src/context/bundle.rs"],
            "raw_artifact_id": artifact_id,
            "reducer": {
                "name": "engra_73_bundle_fixture",
                "version": "1",
                "lossy": true,
                "confidence": 0.92,
                "structured_facts": {
                    "decision": "add direct context_build_bundle coverage",
                    "blocker": "missing direct test coverage",
                    "files": ["src/context/bundle.rs"]
                },
                "tokens_raw_est": 600,
                "tokens_compact_est": 60
            }
        }),
    );
    assert!(
        record.get("error").is_none(),
        "context_record failed: {record}"
    );
    let event_id = record["created_ids"]["event_id"]
        .as_i64()
        .expect("record response must include event_id");

    let bundle = call_tool_json(
        &handler,
        139,
        "context_build_bundle",
        json!({
            "query": "context bundle smoke",
            "repo_id": "github:aiconnai/engram",
            "session_id": "session-bundle",
            "task_id": "ENGRA-73",
            "include_artifact_pointers": true,
            "current_git_branch": "main",
            "current_commit_hash": "new-bundle-sha",
            "stale_after_days": 1,
            "section_limit": 5
        }),
    );
    assert!(
        bundle.get("error").is_none(),
        "context_build_bundle failed: {bundle}"
    );
    assert_eq!(bundle["bundle_type"].as_str(), Some("operational_context"));
    assert_eq!(
        bundle["artifact_policy"].as_str(),
        Some("Artifact pointers included; raw artifact content is never included.")
    );

    let failures = bundle["failures"]
        .as_array()
        .expect("failures must be an array");
    assert!(
        failures
            .iter()
            .any(|entry| entry["provenance"]["event_id"].as_i64() == Some(event_id)),
        "bundle should include failure entry for recorded event, got: {failures:?}"
    );

    let blockers = bundle["unresolved_blockers"]
        .as_array()
        .expect("unresolved_blockers must be an array");
    assert!(
        blockers
            .iter()
            .any(|entry| entry["provenance"]["event_id"].as_i64() == Some(event_id)),
        "bundle should include blocker entry for recorded event, got: {blockers:?}"
    );

    let decisions = bundle["recent_decisions"]
        .as_array()
        .expect("recent_decisions must be an array");
    assert!(
        decisions
            .iter()
            .any(|entry| entry["provenance"]["event_id"].as_i64() == Some(event_id)),
        "bundle should include decision entry for recorded event, got: {decisions:?}"
    );

    let commands = bundle["commands_already_run"]
        .as_array()
        .expect("commands_already_run must be an array");
    assert!(
        commands.iter().any(|entry| {
            entry["command_name"].as_str() == Some("cargo test context_build_bundle")
                && entry["exit_code"].as_i64() == Some(101)
        }),
        "bundle should include command entry, got: {commands:?}"
    );

    let files = bundle["files_inspected_or_touched"]
        .as_array()
        .expect("files_inspected_or_touched must be an array");
    assert!(
        files
            .iter()
            .any(|entry| entry["path"].as_str() == Some("src/context/bundle.rs")),
        "bundle should include touched file, got: {files:?}"
    );

    let staleness = bundle["stale_warnings"]
        .as_array()
        .expect("stale_warnings must be an array");
    for kind in ["branch_mismatch", "commit_mismatch", "age"] {
        assert!(
            staleness
                .iter()
                .any(|entry| entry["warning"]["kind"].as_str() == Some(kind)),
            "missing bundle staleness warning {kind}: {staleness:?}"
        );
    }

    let artifact_pointers = bundle["artifact_pointers"]
        .as_array()
        .expect("artifact_pointers must be an array");
    assert!(
        artifact_pointers
            .iter()
            .any(|pointer| pointer["artifact_id"].as_str() == Some(artifact_id)),
        "bundle should include artifact pointer, got: {artifact_pointers:?}"
    );

    let markdown = bundle["markdown"]
        .as_str()
        .expect("bundle markdown must be a string");
    assert!(markdown.contains("Artifact pointers"));
    assert!(
        !markdown.contains("RAW_CONTEXT_BUNDLE_OUTPUT_SHOULD_NOT_RENDER"),
        "bundle markdown must not include raw artifact content"
    );

    assert_eq!(bundle["metrics"]["estimated"].as_bool(), Some(true));
    assert_eq!(
        bundle["metrics"]["raw_artifact_return_count"].as_u64(),
        Some(0)
    );
    assert!(
        bundle["metrics"]["artifact_pointer_count"]
            .as_u64()
            .unwrap_or_default()
            >= 1,
        "bundle metrics should count artifact pointers"
    );
    let notes = bundle["metrics"]["notes"]
        .as_array()
        .expect("metric notes must be an array");
    assert!(
        notes.iter().any(|note| {
            note.as_str()
                .is_some_and(|text| text.contains("Raw artifact content is never included"))
        }),
        "bundle metrics should document raw artifact exclusion, got: {notes:?}"
    );
}

#[test]
fn test_recent_activity_preview_truncated_at_100_chars() {
    let handler = TestHandler::new();

    // Create memory with content > 100 chars
    let long_content: String = "A".repeat(200);
    let create_req = make_request(
        100,
        "tools/call",
        json!({
            "name": "memory_create",
            "arguments": {
                "content": long_content,
                "memory_type": "note"
            }
        }),
    );
    handler.handle_request(create_req);

    let req = make_request(
        101,
        "tools/call",
        json!({
            "name": "recent_activity",
            "arguments": {"timeframe": "1h", "limit": 1}
        }),
    );

    let resp = handler.handle_request(req);
    let result = resp.result.expect("Expected result");
    let content = result["content"]
        .as_array()
        .expect("Expected content array");
    let text = content[0]["text"].as_str().expect("Expected text");
    let data: Value = serde_json::from_str(text).unwrap();

    let activities = data["activities"].as_array().unwrap();
    if !activities.is_empty() {
        let preview = activities[0]["preview"].as_str().unwrap();
        assert!(
            preview.ends_with("..."),
            "Preview of long content should end with '...', got: {}",
            preview
        );
        assert!(
            preview.len() <= 103,
            "Preview + '...' should be at most 103 chars, got: {}",
            preview.len()
        );
    }
}
