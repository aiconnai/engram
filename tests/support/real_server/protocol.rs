use serde_json::{json, Value};

pub fn initialize_request(id: i64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "real-server-harness", "version": "1.0.0"}
        }
    })
}

pub fn tools_list_request(id: i64) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/list", "params": {}})
}

pub fn tool_call_request(id: i64, name: &str, arguments: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    })
}

pub fn tool_result_json(response: &Value) -> Value {
    assert!(
        response.get("error").is_none(),
        "JSON-RPC response should not contain error: {response}"
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result should contain text content");
    serde_json::from_str(text).expect("tool result text should be JSON")
}
