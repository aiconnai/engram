//! C1: workspace authorization through the real `engram-server` HTTP transport.
//!
//! The database is seeded before the server starts; the server then runs as a
//! separate process, so every assertion observes the real transport principal
//! flowing from the HTTP layer into dispatch and the storage lookup.

use std::path::Path;

use serde_json::{json, Value};

use super::{http_post_json, pick_loopback_port, ServerProcess, HTTP_SECURITY_TEST_LOCK};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, StorageConfig, StorageMode};

const PRIVATE_SECRET: &str = "zebracorn private payroll secret";
const DEFAULT_CONTENT: &str = "zebracorn default public note";
const API_KEY: &str = "secret-key";

struct Seeded {
    default_id: i64,
    private_id: i64,
}

fn seed(db_path: &Path) -> Seeded {
    let storage = Storage::open(StorageConfig {
        db_path: db_path.to_string_lossy().to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .expect("open seed storage");
    storage
        .with_transaction(|conn| {
            let create = |content: &str, workspace: &str| {
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: content.to_string(),
                        workspace: Some(workspace.to_string()),
                        importance: Some(0.9),
                        ..Default::default()
                    },
                )
                .map(|memory| memory.id)
            };
            Ok(Seeded {
                default_id: create(DEFAULT_CONTENT, "default")?,
                private_id: create(PRIVATE_SECRET, "private")?,
            })
        })
        .expect("seed memories")
}

/// Persisted state of a memory row, read with a separate raw connection.
///
/// The server keeps the database open in WAL mode while this runs, so the
/// raw reader only sees the server's later commits if the server never drops
/// its SQLite locks (G1). `assert_raw_reader_sees_accepted_read` is the
/// positive control that keeps the "unchanged" assertions from being vacuous.
fn row_state(db_path: &Path, id: i64) -> String {
    let conn = rusqlite::Connection::open(db_path).expect("open raw db");
    let row: Vec<rusqlite::types::Value> = conn
        .query_row(
            "SELECT content, workspace, access_count, last_accessed_at, updated_at,
                    stability, version, valid_to
             FROM memories WHERE id = ?1",
            [id],
            |r| (0..8).map(|i| r.get(i)).collect(),
        )
        .expect("memory row");
    let reinforcements: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_reinforcements WHERE memory_id = ?1",
            [id],
            |r| r.get(0),
        )
        .expect("reinforcement count");
    format!("{row:?} reinforcements={reinforcements}")
}

/// Positive control: an accepted `memory_get` records access/reinforcement,
/// and a raw reader opened after earlier raw reads must observe that commit.
fn assert_raw_reader_sees_accepted_read(port: u16, db_path: &Path, id: i64, tool: &str) {
    let before = row_state(db_path, id);
    let read = tool_payload(port, tool, json!({"id": id, "workspace": "default"}), None);
    assert_eq!(read["id"], id, "{tool}: positive control read: {read}");
    assert_ne!(
        before,
        row_state(db_path, id),
        "{tool}: accepted read is not visible to the raw reader; \
         row_state comparisons would be vacuous"
    );
}

fn spawn(args: &[&str], seeded: &mut Option<Seeded>) -> ServerProcess {
    let mut process = ServerProcess::spawn_seeded(args, |db| *seeded = Some(seed(db)))
        .expect("spawn seeded server");
    let port = args
        .iter()
        .position(|arg| *arg == "--http-port")
        .and_then(|i| args.get(i + 1))
        .expect("port arg");
    process
        .wait_for_log(&format!("HTTP transport listening on 127.0.0.1:{port}"))
        .expect("HTTP readiness");
    process
}

fn rpc(port: u16, method: &str, params: Value, bearer: Option<&str>) -> (u16, Value) {
    let body = json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params});
    let response = http_post_json(port, "/v1/mcp", body, bearer).expect("HTTP request");
    let parsed = serde_json::from_str(&response.body).unwrap_or(Value::Null);
    (response.status, parsed)
}

/// The tool payload that the dispatcher returned, decoded from the MCP result.
fn tool_payload(port: u16, tool: &str, arguments: Value, bearer: Option<&str>) -> Value {
    let (status, body) = rpc(
        port,
        "tools/call",
        json!({"name": tool, "arguments": arguments}),
        bearer,
    );
    assert_eq!(status, 200, "{tool}: unexpected status, body {body}");
    let text = body["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("{tool}: missing tool text in {body}"));
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_string()))
}

fn normalized(value: &Value, id: i64) -> String {
    // Replace only whole-token occurrences of the caller-supplied id.
    let id = id.to_string();
    value
        .to_string()
        .replace(&format!("'{id}'"), "'<ID>'")
        .replace(&format!("\\\"{id}\\\""), "\\\"<ID>\\\"")
        .replace(&format!("\"{id}\""), "\"<ID>\"")
        .replace(&format!(": {id}\""), ": <ID>\"")
        .replace(&format!(": {id}\\\""), ": <ID>\\\"")
}

#[test]
fn test_claimed_workspace_does_not_authorize_foreign_id() {
    let _guard = HTTP_SECURITY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let port = pick_loopback_port().to_string();
    let mut seeded = None;
    let process = spawn(
        &[
            "--transport",
            "http",
            "--http-bind-address",
            "127.0.0.1",
            "--http-port",
            &port,
        ],
        &mut seeded,
    );
    let Seeded {
        default_id,
        private_id,
    } = seeded.expect("seeded ids");
    let port: u16 = port.parse().expect("port");
    let db = process.db_path();
    let missing_id = private_id + 10_000;

    for tool in ["memory_get", "memory_get_public"] {
        let before = row_state(&db, private_id);
        let foreign = tool_payload(
            port,
            tool,
            json!({"id": private_id, "workspace": "default"}),
            None,
        );
        assert_eq!(foreign["error"]["code"], "not_found", "{tool}: {foreign}");
        assert!(
            !foreign.to_string().contains("payroll"),
            "{tool}: private content leaked over HTTP: {foreign}"
        );
        assert_eq!(
            before,
            row_state(&db, private_id),
            "{tool}: denied read changed access tracking or reinforcement"
        );

        let missing = tool_payload(
            port,
            tool,
            json!({"id": missing_id, "workspace": "default"}),
            None,
        );
        assert_eq!(
            normalized(&foreign, private_id),
            normalized(&missing, missing_id),
            "{tool}: foreign and nonexistent IDs must be indistinguishable"
        );

        let legit = tool_payload(
            port,
            tool,
            json!({"id": default_id, "workspace": "default"}),
            None,
        );
        assert_eq!(
            legit["id"], default_id,
            "{tool}: legit default read: {legit}"
        );
        assert_eq!(legit["content"], DEFAULT_CONTENT, "{tool}: {legit}");

        assert_raw_reader_sees_accepted_read(port, &db, default_id, tool);
    }
}

#[test]
fn anonymous_resources_read_is_bound_to_persisted_workspace() {
    let _guard = HTTP_SECURITY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let port = pick_loopback_port().to_string();
    let mut seeded = None;
    let process = spawn(
        &[
            "--transport",
            "http",
            "--http-bind-address",
            "127.0.0.1",
            "--http-port",
            &port,
        ],
        &mut seeded,
    );
    let Seeded {
        default_id,
        private_id,
    } = seeded.expect("seeded ids");
    let port: u16 = port.parse().expect("port");
    let db = process.db_path();
    let missing_id = private_id + 10_000;

    let before = row_state(&db, private_id);
    let read = |uri: String| rpc(port, "resources/read", json!({"uri": uri}), None).1;

    let foreign = read(format!("engram://memory/{private_id}"));
    let missing = read(format!("engram://memory/{missing_id}"));
    assert!(foreign["error"].is_object(), "foreign resource: {foreign}");
    assert!(!foreign.to_string().contains("payroll"), "leak: {foreign}");
    assert_eq!(
        normalized(&foreign["error"], private_id),
        normalized(&missing["error"], missing_id),
        "foreign and nonexistent resource IDs must be indistinguishable"
    );
    assert_eq!(
        before,
        row_state(&db, private_id),
        "resource read touched row"
    );

    for uri in [
        "engram://workspace/private/memories",
        "engram://workspace/private",
        "engram://stats",
        "engram://entities",
    ] {
        let denied = read(uri.to_string());
        assert!(denied["error"].is_object(), "{uri}: {denied}");
        assert!(
            !denied.to_string().contains("payroll"),
            "{uri} leaked: {denied}"
        );
    }

    let own = read(format!("engram://memory/{default_id}"));
    assert!(
        own["result"]["contents"][0]["text"]
            .as_str()
            .is_some_and(|text| text.contains(DEFAULT_CONTENT)),
        "own resource read: {own}"
    );
    let listed = read("engram://workspace/default/memories".to_string());
    let listed_text = listed["result"]["contents"][0]["text"]
        .as_str()
        .unwrap_or("");
    assert!(
        listed_text.contains(DEFAULT_CONTENT),
        "default list: {listed}"
    );
    assert!(
        !listed_text.contains("payroll"),
        "default list leaked: {listed}"
    );

    assert_raw_reader_sees_accepted_read(port, &db, default_id, "memory_get");
}

#[test]
fn anonymous_claim_cannot_scope_tool_without_workspace_parameter() {
    let _guard = HTTP_SECURITY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let port = pick_loopback_port().to_string();
    let mut seeded = None;
    let _process = spawn(
        &[
            "--transport",
            "http",
            "--http-bind-address",
            "127.0.0.1",
            "--http-port",
            &port,
        ],
        &mut seeded,
    );
    let port: u16 = port.parse().expect("port");

    let (status, body) = rpc(
        port,
        "tools/call",
        json!({
            "name": "memory_export_graph",
            "arguments": {"format": "json", "workspace": "default"}
        }),
        None,
    );
    assert_eq!(status, 403, "export_graph must be denied: {body}");
    assert!(
        !body.to_string().contains("payroll"),
        "export leaked: {body}"
    );

    // `workspace` is an alias for `workspace_path_hash` here: not a claim.
    let (status, body) = rpc(
        port,
        "tools/call",
        json!({
            "name": "context_get_artifact",
            "arguments": {"artifact_id": "a", "reason": "r", "workspace": "default"}
        }),
        None,
    );
    assert_eq!(status, 403, "context_get_artifact must be denied: {body}");

    // Catalog metadata stays available without any workspace claim.
    let tools = tool_payload(port, "discover_tools", json!({"detail": "names"}), None);
    assert!(tools.get("error").is_none(), "discover_tools: {tools}");

    let listed = tool_payload(port, "memory_list", json!({"workspace": "default"}), None);
    assert!(
        listed.to_string().contains(DEFAULT_CONTENT),
        "list: {listed}"
    );
    assert!(
        !listed.to_string().contains("payroll"),
        "list leaked: {listed}"
    );
}

#[test]
fn keyed_principal_reads_any_workspace_by_id() {
    let _guard = HTTP_SECURITY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let port = pick_loopback_port().to_string();
    let mut seeded = None;
    let _process = spawn(
        &[
            "--transport",
            "http",
            "--http-bind-address",
            "127.0.0.1",
            "--http-port",
            &port,
            "--http-api-key",
            API_KEY,
        ],
        &mut seeded,
    );
    let Seeded { private_id, .. } = seeded.expect("seeded ids");
    let port: u16 = port.parse().expect("port");

    let read = tool_payload(port, "memory_get", json!({"id": private_id}), Some(API_KEY));
    assert_eq!(read["id"], private_id, "process bearer is unscoped: {read}");
    assert_eq!(read["content"], PRIVATE_SECRET);
}
