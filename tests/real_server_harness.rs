mod support;

use std::sync::Mutex;

use serde_json::{json, Value};
use tempfile::tempdir;

use support::real_server::{
    bad_executable_path, initialize_request, tool_call_request, tool_result_json,
    tools_list_request, RealServer, RealServerConfig,
};

const STDIO_MEMORY: &str = "real stdio harness durable memory alpha";
const HTTP_MEMORY: &str = "real http harness durable memory beta";
static REAL_SERVER_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn stdio_real_server_initializes_lists_tools_and_roundtrips_memory() {
    let _guard = REAL_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Given: a real engram-server binary running over stdio with isolated storage.
    let mut server = RealServer::start_stdio(RealServerConfig::from_cargo_bin())
        .expect("start real stdio engram-server");
    let temp_path = server.temp_path().to_path_buf();
    let db_path = server.db_path().to_path_buf();

    // When: the harness drives initialize, tools/list, memory_create, and memory_search.
    let initialized = server
        .stdio_request(initialize_request(1))
        .expect("stdio initialize response");
    let tools = server
        .stdio_request(tools_list_request(2))
        .expect("stdio tools/list response");
    let created = server
        .stdio_request(tool_call_request(
            3,
            "memory_create",
            json!({
                "content": STDIO_MEMORY,
                "memory_type": "note",
                "workspace": "real-stdio-harness",
                "tags": ["real-server-harness"]
            }),
        ))
        .expect("stdio memory_create response");
    let searched = server
        .stdio_request(tool_call_request(
            4,
            "memory_search",
            json!({
                "query": "stdio harness durable memory alpha",
                "workspace": "real-stdio-harness",
                "rerank": false
            }),
        ))
        .expect("stdio memory_search response");

    // Then: all observable JSON-RPC contracts succeed against the real process.
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    assert_has_tool(&tools, "memory_create");
    assert_eq!(tool_result_json(&created)["content"], STDIO_MEMORY);
    assert_search_contains(&tool_result_json(&searched), STDIO_MEMORY);
    assert!(db_path.exists(), "real server should create isolated DB");

    let cleanup = server.shutdown_and_verify();
    assert!(
        cleanup.temp_removed,
        "temp dir should be removed: {}",
        cleanup.temp_path.display()
    );
    assert!(
        cleanup.child_id > 0,
        "cleanup should report child process id"
    );
    assert_eq!(
        cleanup.port_released, None,
        "stdio server should not own a port"
    );
    assert!(
        !cleanup
            .redacted_stderr
            .contains(RealServerConfig::from_cargo_bin().api_key()),
        "redacted stderr must not leak harness API key"
    );
    assert!(!temp_path.exists(), "temp residue remained: {temp_path:?}");
}

#[test]
fn http_real_server_initializes_lists_tools_and_roundtrips_memory() {
    let _guard = REAL_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Given: a real engram-server binary running HTTP on a free loopback port.
    let server = RealServer::start_http(RealServerConfig::from_cargo_bin())
        .expect("start real HTTP engram-server");
    let temp_path = server.temp_path().to_path_buf();
    let port = server.port().expect("HTTP server should expose its port");

    // When: the harness drives initialize, tools/list, memory_create, and memory_search over /mcp.
    let initialized = server
        .http_json_rpc(initialize_request(11))
        .expect("HTTP initialize response");
    let tools = server
        .http_json_rpc(tools_list_request(12))
        .expect("HTTP tools/list response");
    let created = server
        .http_json_rpc(tool_call_request(
            13,
            "memory_create",
            json!({
                "content": HTTP_MEMORY,
                "memory_type": "note",
                "workspace": "real-http-harness",
                "tags": ["real-server-harness"]
            }),
        ))
        .expect("HTTP memory_create response");
    let searched = server
        .http_json_rpc(tool_call_request(
            14,
            "memory_search",
            json!({
                "query": "http harness durable memory beta",
                "workspace": "real-http-harness",
                "rerank": false
            }),
        ))
        .expect("HTTP memory_search response");

    // Then: all observable JSON-RPC contracts succeed and cleanup releases process, port, and temp DB.
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    assert_has_tool(&tools, "memory_search");
    assert_eq!(tool_result_json(&created)["content"], HTTP_MEMORY);
    assert_search_contains(&tool_result_json(&searched), HTTP_MEMORY);

    let cleanup = server.shutdown_and_verify();
    assert_eq!(cleanup.port, Some(port));
    assert!(
        cleanup.child_id > 0,
        "cleanup should report child process id"
    );
    assert_eq!(
        cleanup.port_released,
        Some(true),
        "port {port} was not released"
    );
    assert!(
        cleanup.temp_removed,
        "temp dir should be removed: {temp_path:?}"
    );
    assert!(
        !cleanup
            .redacted_stderr
            .contains(RealServerConfig::from_cargo_bin().api_key()),
        "redacted stderr must not leak harness API key"
    );
    assert!(!temp_path.exists(), "temp residue remained: {temp_path:?}");
}

#[test]
fn bad_executable_self_test_returns_bounded_error_and_cleans_tempdir() {
    let _guard = REAL_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Given: an isolated temp root and an executable path that does not exist.
    let temp_dir = tempdir().expect("create bad executable tempdir");
    let temp_path = temp_dir.path().to_path_buf();
    let config = RealServerConfig::from_cargo_bin().with_executable(bad_executable_path());

    // When: starting the real-server harness fails before any mock or external process can run.
    let result = RealServer::start_stdio_in(config, temp_dir);
    let error = match result {
        Ok(_) => panic!("bad executable self-test should fail to spawn"),
        Err(error) => error.to_string(),
    };

    // Then: the failure is explicit and the consumed temp root is cleaned up.
    assert!(
        error.contains("No such file") || error.contains("os error") || error.contains("not found"),
        "unexpected bad executable error: {error}"
    );
    assert!(
        !temp_path.exists(),
        "bad executable self-test left temp residue: {temp_path:?}"
    );
}

fn assert_has_tool(response: &Value, expected: &str) {
    assert!(
        response.get("error").is_none(),
        "tools/list should not return JSON-RPC error: {response}"
    );
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list result should contain tools array");
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"].as_str() == Some(expected)),
        "tools/list should include {expected}: {response}"
    );
}

fn assert_search_contains(search: &Value, expected_content: &str) {
    let results = search
        .as_array()
        .or_else(|| search.get("results").and_then(Value::as_array))
        .or_else(|| search.get("memories").and_then(Value::as_array))
        .expect("memory_search should return result array or wrapper object");
    assert!(
        results.iter().any(|result| {
            result["memory"]["content"].as_str() == Some(expected_content)
                || result["content"].as_str() == Some(expected_content)
        }),
        "memory_search did not return created memory {expected_content:?}: {search}"
    );
}
