//! Q4: contract journey on one isolated database, over real `engram-server` processes.
//!
//! A stdio server and an authenticated HTTP server (reached through `/v1/mcp`, the
//! path both SDKs use) are started one after the other against the same
//! caller-owned database. Each phase proves that rejected requests (bad bearer,
//! malformed body, permission denial, invalid params, unknown tool, absent id)
//! leave the persisted rows byte-for-byte unchanged, by reading the SQLite file
//! with an independent connection and then reading the state back through the API.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

use super::support::real_server::{
    initialize_request, tool_call_request, RealServer, RealServerConfig,
};
use super::JOURNEY_LOCK;

const WORKSPACE: &str = "q4-contract";
const PAGE_WORKSPACE: &str = "q4-pages";
const TARGET: &str = "Q4 contract target memory must survive every rejected request";
const MUTATION: &str = "Q4 MUTATION that no rejected request may persist";

/// Error codes of the normalized tool-error contract (RFC 0006).
const NORMALIZED_CODES: [&str; 9] = [
    "invalid_params",
    "missing_argument",
    "not_found",
    "tool_not_found",
    "permission_denied",
    "conflict",
    "version_mismatch",
    "rate_limited",
    "internal_error",
];

// ---------------------------------------------------------------------------
// Wire helpers
// ---------------------------------------------------------------------------

struct Reply {
    is_error: bool,
    payload: Value,
}

impl Reply {
    fn code(&self) -> &str {
        self.payload["error"]["code"].as_str().unwrap_or("")
    }
}

/// One live server process plus the way to talk to it.
enum Wire {
    Stdio(RealServer),
    Http {
        server: RealServer,
        port: u16,
        api_key: String,
    },
}

impl Wire {
    fn stdio(db_path: &Path) -> Self {
        let server =
            RealServer::start_stdio_at(RealServerConfig::from_cargo_bin(), db_path.to_path_buf())
                .expect("start stdio server");
        let mut wire = Self::Stdio(server);
        initialize(&mut wire);
        wire
    }

    fn http(db_path: &Path) -> Self {
        let config = RealServerConfig::from_cargo_bin();
        let api_key = config.api_key().to_string();
        let server =
            RealServer::start_http_at(config, db_path.to_path_buf()).expect("start HTTP server");
        let port = server.port().expect("HTTP server exposes its port");
        let mut wire = Self::Http {
            server,
            port,
            api_key,
        };
        initialize(&mut wire);
        wire
    }

    fn rpc(&mut self, request: Value) -> Value {
        match self {
            Self::Stdio(server) => server.stdio_request(request).expect("stdio response"),
            Self::Http { port, api_key, .. } => {
                let raw = post(
                    *port,
                    "/v1/mcp",
                    &[("Authorization", format!("Bearer {api_key}"))],
                    &request.to_string(),
                );
                assert_eq!(raw.status, 200, "authenticated /v1/mcp: {}", raw.body);
                serde_json::from_str(&raw.body).expect("JSON-RPC body")
            }
        }
    }

    fn tool(&mut self, name: &str, arguments: Value) -> Reply {
        let response = self.rpc(tool_call_request(7, name, arguments));
        assert!(
            response.get("error").is_none(),
            "{name}: tool failures travel in the result: {response}"
        );
        let result = &response["result"];
        let text = result["content"][0]["text"].as_str().expect("text content");
        Reply {
            is_error: result["isError"] == json!(true),
            payload: serde_json::from_str(text).expect("tool text is JSON"),
        }
    }

    fn port(&self) -> Option<u16> {
        match self {
            Self::Stdio(_) => None,
            Self::Http { port, .. } => Some(*port),
        }
    }

    fn api_key(&self) -> Option<&str> {
        match self {
            Self::Stdio(_) => None,
            Self::Http { api_key, .. } => Some(api_key),
        }
    }

    fn shutdown(self) {
        let server = match self {
            Self::Stdio(server) | Self::Http { server, .. } => server,
        };
        let report = server.shutdown_and_verify();
        assert!(
            !report.temp_removed,
            "caller-owned journey state must not be removed by the server"
        );
        assert!(report.port_released.unwrap_or(true), "port not released");
    }
}

fn initialize(wire: &mut Wire) {
    let initialized = wire.rpc(initialize_request(1));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
}

struct RawResponse {
    status: u16,
    body: String,
}

/// POST over a fresh connection; `Connection: close` lets us read to EOF.
fn post(port: u16, path: &str, headers: &[(&str, String)], body: &str) -> RawResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout");
    let mut request = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("read response to EOF");
    let text = String::from_utf8(response).expect("UTF-8 response");
    let status = text
        .strip_prefix("HTTP/1.1 ")
        .and_then(|rest| rest.get(..3))
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| panic!("unparsable status line: {text}"));
    let body = text
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();
    RawResponse { status, body }
}

// ---------------------------------------------------------------------------
// Persisted state, read with an independent connection
// ---------------------------------------------------------------------------

/// Persisted memory rows, read straight from the SQLite file.
///
/// ONLY call this while no `engram-server` is running on `db_path`. The server
/// re-opens its own `-wal`/`-shm` files after every statement, which drops the
/// SQLite locks it holds (POSIX fcntl semantics); a second process that opens and
/// closes the database then deletes the live WAL, so every later server write is
/// invisible to it and lost on restart (see the Q4 report and coverage map, G-1).
/// Reading after `Wire::shutdown` is safe: the WAL left by the killed server is
/// recovered normally.
///
/// Access counters and `stability` are excluded on purpose: API reads reinforce
/// them, so they change on legitimate reads.
fn persisted_state(db_path: &Path) -> String {
    let conn = rusqlite::Connection::open(db_path).expect("open raw db");
    let mut statement = conn
        .prepare(
            "SELECT id, content, content_hash, memory_type, importance, workspace, tier,
                    lifecycle_state, created_at, updated_at, version, valid_from, valid_to,
                    metadata, scope_path
             FROM memories ORDER BY id",
        )
        .expect("prepare state query");
    let rows = statement
        .query_map([], |row| {
            (0..15)
                .map(|index| row.get::<_, rusqlite::types::Value>(index))
                .collect::<Result<Vec<_>, _>>()
        })
        .expect("query state")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect state");
    let tags: Vec<(i64, i64)> = conn
        .prepare("SELECT memory_id, tag_id FROM memory_tags ORDER BY memory_id, tag_id")
        .expect("prepare tag query")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query tags")
        .collect::<Result<_, _>>()
        .expect("collect tags");
    format!("rows={rows:?} tags={tags:?}")
}

// ---------------------------------------------------------------------------
// The rejection matrix, identical for every transport
// ---------------------------------------------------------------------------

struct Case {
    label: &'static str,
    tool: &'static str,
    arguments: Value,
    codes: &'static [&'static str],
}

fn rejection_cases(target: i64) -> Vec<Case> {
    vec![
        Case {
            label: "per-call read_only update",
            tool: "memory_update",
            arguments: json!({"id": target, "content": MUTATION, "_permission_mode": "read_only"}),
            codes: &["permission_denied"],
        },
        Case {
            label: "per-call read_only delete",
            tool: "memory_delete",
            arguments: json!({"id": target, "_permission_mode": "read_only"}),
            codes: &["permission_denied"],
        },
        Case {
            label: "wrong-typed content",
            tool: "memory_update",
            arguments: json!({"id": target, "content": 123}),
            codes: &["invalid_params"],
        },
        Case {
            label: "wrong-typed id",
            tool: "memory_update",
            arguments: json!({"id": "not-a-number", "content": MUTATION}),
            codes: &["missing_argument", "invalid_params"],
        },
        Case {
            label: "missing id",
            tool: "memory_delete",
            arguments: json!({}),
            codes: &["missing_argument"],
        },
        Case {
            label: "absent id update",
            tool: "memory_update",
            arguments: json!({"id": 99_999_999, "content": MUTATION}),
            codes: &["not_found"],
        },
        Case {
            label: "absent id delete",
            tool: "memory_delete",
            arguments: json!({"id": 99_999_999}),
            codes: &["not_found"],
        },
        Case {
            label: "unknown tool",
            tool: "q4_memory_obliterate",
            arguments: json!({"id": target}),
            codes: &["tool_not_found"],
        },
    ]
}

/// Run every case; each must be a normalized `isError` result. Returns the
/// `error` objects so transports can be compared with each other.
fn run_rejections(wire: &mut Wire, target: i64) -> Vec<Value> {
    rejection_cases(target)
        .into_iter()
        .map(|case| {
            let reply = wire.tool(case.tool, case.arguments);
            assert!(reply.is_error, "{}: isError must be true", case.label);
            assert!(
                NORMALIZED_CODES.contains(&reply.code()),
                "{}: code outside the normalized set: {}",
                case.label,
                reply.payload
            );
            assert!(
                case.codes.contains(&reply.code()),
                "{}: expected one of {:?}, got {}",
                case.label,
                case.codes,
                reply.payload
            );
            assert!(
                reply.payload["error"]["message"]
                    .as_str()
                    .is_some_and(|message| !message.is_empty()),
                "{}: empty message: {}",
                case.label,
                reply.payload
            );
            reply.payload["error"].clone()
        })
        .collect()
}

fn page_ids(wire: &mut Wire, limit: i64, offset: i64) -> Vec<i64> {
    let reply = wire.tool(
        "memory_list",
        json!({
            "workspace": PAGE_WORKSPACE,
            "sort_by": "created_at",
            "sort_order": "asc",
            "limit": limit,
            "offset": offset,
        }),
    );
    assert!(!reply.is_error, "memory_list: {}", reply.payload);
    reply
        .payload
        .as_array()
        .expect("array")
        .iter()
        .map(|memory| memory["id"].as_i64().expect("id"))
        .collect()
}

fn assert_pagination(wire: &mut Wire, expected_ids: &[i64]) {
    let pages = [
        page_ids(wire, 2, 0),
        page_ids(wire, 2, 2),
        page_ids(wire, 2, 4),
        page_ids(wire, 2, 6),
    ];
    assert_eq!(
        pages.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 0],
        "pages must drain the workspace then return empty: {pages:?}"
    );
    let flattened: Vec<i64> = pages.into_iter().flatten().collect();
    let mut sorted = flattened.clone();
    sorted.sort_unstable();
    assert_eq!(
        sorted, expected_ids,
        "pages must cover exactly the workspace"
    );
    assert_eq!(flattened, sorted, "ascending creation order across pages");
}

fn assert_target_unchanged(wire: &mut Wire, target: i64) {
    let reread = wire.tool("memory_get", json!({"id": target}));
    assert!(!reread.is_error, "target vanished: {}", reread.payload);
    assert_eq!(reread.payload["content"], TARGET);
    assert_eq!(reread.payload["workspace"], WORKSPACE);
    assert_eq!(reread.payload["version"], 1, "target was versioned");
}

// ---------------------------------------------------------------------------
// Journey
// ---------------------------------------------------------------------------

struct Seed {
    target: i64,
    scratch: i64,
    page_ids: Vec<i64>,
}

/// Phase 1 (stdio): seed rows and check the accepted-path contract.
fn seed_phase(db_path: &Path) -> Seed {
    let mut wire = Wire::stdio(db_path);

    let target = wire.tool(
        "memory_create",
        json!({"content": TARGET, "workspace": WORKSPACE, "tags": ["q4-contract"]}),
    );
    assert!(!target.is_error, "{}", target.payload);
    let target = target.payload["id"].as_i64().expect("target id");
    let page_ids: Vec<i64> = (0..5)
        .map(|index| {
            let page = wire.tool(
                "memory_create",
                json!({"content": format!("Q4 page entry {index}"), "workspace": PAGE_WORKSPACE}),
            );
            page.payload["id"].as_i64().expect("page id")
        })
        .collect();

    // snake_case is the wire contract; a camelCase key is dropped, not aliased.
    let snake = wire.tool(
        "memory_create",
        json!({"content": "Q4 snake", "workspace": WORKSPACE, "memory_type": "issue"}),
    );
    let camel = wire.tool(
        "memory_create",
        json!({"content": "Q4 camel", "workspace": WORKSPACE, "memoryType": "issue"}),
    );
    assert_eq!(snake.payload["type"], "issue");
    assert_eq!(camel.payload["type"], "note", "camelCase must not apply");

    assert_pagination(&mut wire, &page_ids);
    wire.shutdown();
    Seed {
        target,
        scratch: snake.payload["id"].as_i64().expect("scratch id"),
        page_ids,
    }
}

/// Phase 2 (stdio): the rejection matrix; returns the error objects it observed.
fn stdio_rejection_phase(db_path: &Path, seed: &Seed) -> Vec<Value> {
    let mut wire = Wire::stdio(db_path);
    let errors = run_rejections(&mut wire, seed.target);
    assert_target_unchanged(&mut wire, seed.target);
    wire.shutdown();
    errors
}

fn assert_unauthenticated(port: u16, path: &str, headers: &[(&str, String)], target: i64) {
    // Dispatched with a valid bearer this body would delete the target.
    let body = tool_call_request(41, "memory_delete", json!({"id": target})).to_string();
    let response = post(port, path, headers, &body);
    assert_eq!(
        response.status, 401,
        "{path} with {headers:?} must be rejected: {}",
        response.body
    );
    let parsed: Value = serde_json::from_str(&response.body).expect("JSON-RPC 401 body");
    assert_eq!(parsed["error"]["code"], -32001);
    assert!(!response.body.contains(TARGET), "401 leaked content");
}

/// Phase 3 (HTTP): transport rejections, the same matrix, SDK path and header contract.
fn http_rejection_phase(db_path: &Path, seed: &Seed, stdio_errors: &[Value]) {
    let mut wire = Wire::http(db_path);
    let port = wire.port().expect("port");
    let api_key = wire.api_key().expect("api key").to_string();

    // Authentication failures: wrong, missing and malformed bearers, on both
    // routes. The body would delete the target if it were ever dispatched.
    for path in ["/mcp", "/v1/mcp"] {
        assert_unauthenticated(
            port,
            path,
            &[("Authorization", format!("Bearer {api_key}-wrong"))],
            seed.target,
        );
        assert_unauthenticated(port, path, &[], seed.target);
        assert_unauthenticated(
            port,
            path,
            &[("Authorization", format!("Basic {api_key}"))],
            seed.target,
        );
    }

    // A malformed JSON body is rejected by the transport before dispatch.
    let malformed = post(
        port,
        "/v1/mcp",
        &[("Authorization", format!("Bearer {api_key}"))],
        "{not json",
    );
    assert_eq!(malformed.status, 400, "{}", malformed.body);

    // The same rejection matrix over HTTP yields the same normalized errors as stdio.
    let http_errors = run_rejections(&mut wire, seed.target);
    assert_eq!(
        http_errors, stdio_errors,
        "stdio and HTTP must produce identical error objects"
    );

    // `X-Tenant-Slug` is sent by both SDKs but is not an authorization boundary:
    // a bearer-authenticated caller reads the target whatever slug it names.
    let slug_read = post(
        port,
        "/v1/mcp",
        &[
            ("Authorization", format!("Bearer {api_key}")),
            ("X-Tenant-Slug", "some-other-tenant".to_string()),
        ],
        &tool_call_request(42, "memory_get", json!({"id": seed.target})).to_string(),
    );
    assert_eq!(slug_read.status, 200, "{}", slug_read.body);
    assert!(slug_read.body.contains("Q4 contract target"));

    assert_pagination(&mut wire, &seed.page_ids);
    assert_target_unchanged(&mut wire, seed.target);
    wire.shutdown();
}

/// Positive control: an ACCEPTED update must change `persisted_state`, otherwise
/// the equality checks around it could pass vacuously.
fn accepted_update_phase(db_path: &Path, seed: &Seed) {
    let mut wire = Wire::stdio(db_path);
    let accepted = wire.tool(
        "memory_update",
        json!({"id": seed.scratch, "content": "Q4 control update"}),
    );
    assert!(!accepted.is_error, "{}", accepted.payload);
    wire.shutdown();
}

#[test]
fn rejected_requests_never_mutate_shared_state_across_stdio_and_http() {
    let _guard = JOURNEY_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _mode = ModeEnv::set(None); // an inherited permission mode would deny the seed writes
    let state = tempfile::tempdir().expect("shared contract journey state");
    let db_path = state.path().join("q4-contract.db");

    let seed = seed_phase(&db_path);
    let seeded = persisted_state(&db_path);
    assert!(seeded.contains(TARGET), "seed not persisted: {seeded}");

    let stdio_errors = stdio_rejection_phase(&db_path, &seed);
    assert_eq!(
        persisted_state(&db_path),
        seeded,
        "stdio rejections (and the API re-read) mutated persisted rows"
    );

    http_rejection_phase(&db_path, &seed, &stdio_errors);
    assert_eq!(
        persisted_state(&db_path),
        seeded,
        "HTTP rejections (and the API re-read) mutated persisted rows"
    );

    accepted_update_phase(&db_path, &seed);
    assert_ne!(
        persisted_state(&db_path),
        seeded,
        "persisted_state failed to observe an accepted update"
    );

    let state_path = state.path().to_path_buf();
    drop(state);
    assert!(!state_path.exists(), "shared state was not removed");
}

// ---------------------------------------------------------------------------
// Permission-mode environment: unknown-tool behavior on a real stdio server
// ---------------------------------------------------------------------------

pub(super) struct ModeEnv {
    previous: Option<std::ffi::OsString>,
}

impl ModeEnv {
    const NAME: &'static str = "ENGRAM_PERMISSION_MODE";

    /// Set (or clear) the mode for children spawned while the guard lives.
    /// Callers hold `JOURNEY_LOCK`, so no other test spawns a server meanwhile.
    pub(super) fn set(mode: Option<&str>) -> Self {
        let previous = std::env::var_os(Self::NAME);
        match mode {
            Some(value) => std::env::set_var(Self::NAME, value),
            None => std::env::remove_var(Self::NAME),
        }
        Self { previous }
    }
}

impl Drop for ModeEnv {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => std::env::set_var(Self::NAME, value),
            None => std::env::remove_var(Self::NAME),
        }
    }
}

fn unknown_tool_code_under(mode: Option<&str>) -> (String, String) {
    let _env = ModeEnv::set(mode);
    let state = tempfile::tempdir().expect("mode journey state");
    let mut wire = Wire::stdio(&state.path().join("q4-mode.db"));
    let unknown = wire.tool("q4_no_such_tool", json!({}));
    let known = wire.tool("memory_create", json!({"content": "mode control"}));
    assert!(unknown.is_error, "{}", unknown.payload);
    let known_code = if known.is_error {
        known.code().to_string()
    } else {
        "ok".to_string()
    };
    wire.shutdown();
    (unknown.code().to_string(), known_code)
}

#[test]
fn unknown_tool_behavior_follows_the_environment_permission_mode() {
    let _guard = JOURNEY_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // No mode: the local owner learns the name is unknown.
    assert_eq!(
        unknown_tool_code_under(None),
        ("tool_not_found".to_string(), "ok".to_string())
    );
    // Admin mode permits everything, so the name is still reported as unknown.
    assert_eq!(
        unknown_tool_code_under(Some("admin")),
        ("tool_not_found".to_string(), "ok".to_string())
    );
    // Below admin, an unknown name is denied (admin required), never revealed.
    assert_eq!(
        unknown_tool_code_under(Some("read_only")),
        (
            "permission_denied".to_string(),
            "permission_denied".to_string()
        )
    );
    assert_eq!(
        unknown_tool_code_under(Some("scoped_write")),
        ("permission_denied".to_string(), "ok".to_string())
    );
}

// ---------------------------------------------------------------------------
// Async lifecycle: a persisted job outlives the process that created it
// ---------------------------------------------------------------------------

#[cfg(feature = "dream-phase")]
#[test]
fn dream_job_lifecycle_persists_across_processes_and_transports() {
    let _guard = JOURNEY_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _mode = ModeEnv::set(None);
    let state = tempfile::tempdir().expect("lifecycle journey state");
    let db_path = state.path().join("q4-lifecycle.db");
    let status_of = |wire: &mut Wire| {
        let reply = wire.tool("dream_get", json!({"id": "q4-journey-job"}));
        assert!(!reply.is_error, "dream_get: {}", reply.payload);
        reply.payload["job"]["status"]
            .as_str()
            .expect("job status")
            .to_string()
    };

    let mut stdio = Wire::stdio(&db_path);
    let created = stdio.tool(
        "dream_create",
        json!({"workspace": "q4-lifecycle", "job_id": "q4-journey-job", "run": false}),
    );
    assert!(!created.is_error, "dream_create: {}", created.payload);
    assert_eq!(status_of(&mut stdio), "pending");
    stdio.shutdown();

    // A different process over a different transport sees the persisted state
    // and drives the next transition.
    let mut http = Wire::http(&db_path);
    assert_eq!(status_of(&mut http), "pending");
    let canceled = http.tool("dream_cancel", json!({"id": "q4-journey-job"}));
    assert_eq!(canceled.payload["job"]["status"], "canceled");
    http.shutdown();

    let mut stdio_again = Wire::stdio(&db_path);
    assert_eq!(status_of(&mut stdio_again), "canceled");
    let repeat = stdio_again.tool("dream_cancel", json!({"id": "q4-journey-job"}));
    assert_eq!(
        repeat.payload["job"]["status"], "canceled",
        "idempotent cancel"
    );
    let archived = stdio_again.tool("dream_archive", json!({"id": "q4-journey-job"}));
    assert_eq!(archived.payload["job"]["status"], "archived");
    assert_eq!(status_of(&mut stdio_again), "archived");
    stdio_again.shutdown();
}
