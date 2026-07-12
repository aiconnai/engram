mod support;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value};
use support::real_server::{
    bad_executable_path, initialize_request, tool_call_request, tool_result_json,
    tools_list_request, RealServer, RealServerConfig,
};

const WORKSPACE: &str = "canonical-journey";
const OTHER_WORKSPACE: &str = "canonical-private";
const ORIGINAL: &str = "Canonical journey remembers the cobalt launch checklist";
const UPDATED: &str = "Canonical journey remembers the updated cobalt launch checklist";
const PRIVATE: &str = "private workspace sentinel must never cross the boundary";
static JOURNEY_LOCK: Mutex<()> = Mutex::new(());

enum Transport {
    Stdio,
    Http,
}

#[test]
fn canonical_real_binary_journey_over_stdio_and_authenticated_http() {
    let _guard = JOURNEY_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    run_journey(Transport::Stdio);
    run_journey(Transport::Http);
}

fn run_journey(transport: Transport) {
    let config = RealServerConfig::from_cargo_bin();
    let api_key = config.api_key().to_string();
    // Keep every public Todo 10 harness contract exercised by this integration target so
    // target-scoped Clippy sees the shared support module as fully used.
    let _bad_config_contract =
        RealServerConfig::from_cargo_bin().with_executable(bad_executable_path());
    let mut server = match transport {
        Transport::Stdio => RealServer::start_stdio(config).expect("start canonical stdio server"),
        Transport::Http => RealServer::start_http(config).expect("start canonical HTTP server"),
    };
    let temp_path = server.temp_path().to_path_buf();
    assert!(server.db_path().starts_with(&temp_path));

    let initialized = request(&mut server, &transport, initialize_request(1));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");

    let tools = request(&mut server, &transport, tools_list_request(2));
    for name in fixture_tools() {
        assert_has_tool(&tools, &name);
    }

    let created = call(
        &mut server,
        &transport,
        3,
        "memory_create",
        json!({"content": ORIGINAL, "workspace": WORKSPACE, "tags": ["canonical"]}),
    );
    let id = created["id"]
        .as_i64()
        .expect("memory_create returns numeric id");
    assert_eq!(created["content"], ORIGINAL);

    let private = call(
        &mut server,
        &transport,
        4,
        "memory_create",
        json!({"content": PRIVATE, "workspace": OTHER_WORKSPACE}),
    );
    assert_ne!(private["id"], id);

    let fetched = call(&mut server, &transport, 5, "memory_get", json!({"id": id}));
    assert_eq!(fetched["content"], ORIGINAL);

    let listed = call(
        &mut server,
        &transport,
        6,
        "memory_list",
        json!({"workspace": WORKSPACE}),
    );
    assert_contains_id(&listed, id);
    assert_not_contains(&listed, PRIVATE);

    let searched = call(
        &mut server,
        &transport,
        7,
        "memory_search",
        json!({"query": "cobalt launch checklist", "workspace": WORKSPACE, "rerank": false}),
    );
    assert_contains_id(&searched, id);
    assert_not_contains(&searched, PRIVATE);

    let updated = call(
        &mut server,
        &transport,
        8,
        "memory_update",
        json!({"id": id, "content": UPDATED}),
    );
    assert_eq!(updated["content"], UPDATED);

    let exported = call(
        &mut server,
        &transport,
        9,
        "memory_export",
        json!({"workspace": WORKSPACE}),
    );
    assert_contains_id(&exported, id);
    assert_not_contains(&exported, PRIVATE);

    let export_dir = temp_path.join("markdown-readback");
    let markdown = call(
        &mut server,
        &transport,
        10,
        "memory_export_markdown",
        json!({"workspace": WORKSPACE, "output_dir": export_dir, "include_links": false}),
    );
    assert!(markdown["files_written"].as_u64().unwrap_or_default() >= 2);
    assert_markdown_readback(&export_dir, UPDATED);

    let isolated = call(
        &mut server,
        &transport,
        11,
        "memory_list",
        json!({"workspace": "canonical-empty"}),
    );
    assert_not_contains(&isolated, ORIGINAL);
    assert_not_contains(&isolated, UPDATED);
    assert_not_contains(&isolated, PRIVATE);

    if let Some(port) = server.port() {
        assert_wrong_bearer_rejected(port, &api_key);
    }

    let cleanup = server.shutdown_and_verify();
    assert!(cleanup.child_id > 0);
    assert_eq!(cleanup.port.is_some(), matches!(transport, Transport::Http));
    assert!(
        cleanup.temp_removed,
        "temp state remained at {}",
        cleanup.temp_path.display()
    );
    assert!(
        cleanup.port_released.unwrap_or(true),
        "HTTP port was not released"
    );
    assert!(!cleanup.redacted_stderr.contains(&api_key));
    assert!(!temp_path.exists());
}

fn request(server: &mut RealServer, transport: &Transport, value: Value) -> Value {
    match transport {
        Transport::Stdio => server
            .stdio_request(value)
            .expect("stdio JSON-RPC response"),
        Transport::Http => server.http_json_rpc(value).expect("HTTP JSON-RPC response"),
    }
}

fn call(
    server: &mut RealServer,
    transport: &Transport,
    id: i64,
    name: &str,
    arguments: Value,
) -> Value {
    let response = request(server, transport, tool_call_request(id, name, arguments));
    tool_result_json(&response)
}

fn fixture_tools() -> Vec<String> {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/canonical_journey/contract.json"))
            .expect("canonical journey contract fixture is valid JSON");
    fixture["required_tools"]
        .as_array()
        .expect("required_tools array")
        .iter()
        .map(|value| value.as_str().expect("tool name string").to_string())
        .collect()
}

fn assert_has_tool(response: &Value, expected: &str) {
    let tools = response["result"]["tools"].as_array().expect("tools array");
    assert!(tools.iter().any(|tool| tool["name"] == expected));
}

fn assert_contains_id(value: &Value, id: i64) {
    assert!(
        contains_id(value, id),
        "response does not contain id {id}: {value}"
    );
}

fn contains_id(value: &Value, id: i64) -> bool {
    match value {
        Value::Object(map) => {
            map.get("id").and_then(Value::as_i64) == Some(id)
                || map.values().any(|child| contains_id(child, id))
        }
        Value::Array(values) => values.iter().any(|child| contains_id(child, id)),
        _ => false,
    }
}

fn assert_not_contains(value: &Value, forbidden: &str) {
    assert!(
        !value.to_string().contains(forbidden),
        "workspace data leaked: {value}"
    );
}

fn assert_markdown_readback(root: &Path, expected: &str) {
    let mut found = false;
    for entry in fs::read_dir(root).expect("read markdown export directory") {
        let path = entry.expect("read markdown entry").path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            let text = fs::read_to_string(path).expect("read exported markdown");
            found |= text.contains(expected);
            assert!(!text.contains(PRIVATE));
        }
    }
    assert!(found, "updated content missing from markdown export");
}

fn assert_wrong_bearer_rejected(port: u16, valid_key: &str) {
    let body = initialize_request(99).to_string();
    let wrong = format!("{valid_key}-wrong");
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect with wrong bearer");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set timeout");
    write!(
        stream,
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {wrong}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("write wrong bearer request");
    stream.flush().expect("flush wrong bearer request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read rejection");
    assert!(
        response.starts_with("HTTP/1.1 401"),
        "wrong bearer was not rejected: {response}"
    );
    assert!(!response.contains(ORIGINAL));
    assert!(!response.contains(UPDATED));
    assert!(!response.contains(PRIVATE));
}
