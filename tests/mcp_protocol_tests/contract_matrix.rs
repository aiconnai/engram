//! Q4: MCP protocol contract matrix, driven through `McpHandler::handle_request_as`.
//!
//! Every call goes through a JSON-RPC `tools/call` request and the same
//! `ToolCallResult::from_tool_output` envelope that `engram-server` builds, so
//! `isError`, the text content block and the normalized error object are the
//! shapes a client observes. The handler here is an in-process mirror of
//! `EngramHandler` (the server binary is not importable); the real-binary
//! counterparts live in `tests/canonical_journey.rs`.
//!
//! Environment-variable permission modes are deliberately NOT exercised here:
//! mutating `ENGRAM_PERMISSION_MODE` would race the other tests in this binary.
//! They are pinned against a real `engram-server` child in `canonical_journey`.

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::mcp::{
    get_tool_definitions, handlers, methods, McpHandler, McpRequest, McpResponse, ToolCallResult,
};

use super::TestHandler;

/// Error codes a client may see in `error.code`: the RFC 0006 set plus the
/// `invalid_permission_mode` code emitted for a malformed environment mode.
const NORMALIZED_CODES: [&str; 10] = [
    "invalid_params",
    "missing_argument",
    "not_found",
    "tool_not_found",
    "permission_denied",
    "conflict",
    "version_mismatch",
    "rate_limited",
    "internal_error",
    "invalid_permission_mode",
];

/// In-process mirror of `EngramHandler::handle_request_as` for tool calls.
struct ProtocolHandler {
    ctx: Mutex<handlers::HandlerContext>,
}

impl ProtocolHandler {
    fn new() -> Self {
        let TestHandler { ctx, .. } = TestHandler::new();
        Self {
            ctx: Mutex::new(ctx),
        }
    }

    fn tool_call(
        &self,
        principal: Option<TransportPrincipal>,
        name: &str,
        arguments: Value,
    ) -> ToolEnvelope {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: methods::CALL_TOOL.to_string(),
            params: json!({"name": name, "arguments": arguments}),
        };
        let response = self.handle_request_as(request, principal);
        assert!(
            response.error.is_none(),
            "{name}: tool failures travel in the result, not the JSON-RPC error: {response:?}"
        );
        ToolEnvelope::from_result(response.result.expect("tools/call result"))
    }

    fn call(&self, name: &str, arguments: Value) -> ToolEnvelope {
        self.tool_call(None, name, arguments)
    }
}

impl McpHandler for ProtocolHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        self.handle_request_as(request, None)
    }

    fn handle_request_as(
        &self,
        request: McpRequest,
        principal: Option<TransportPrincipal>,
    ) -> McpResponse {
        match request.method.as_str() {
            methods::CALL_TOOL => {
                let name = request.params["name"].as_str().unwrap_or("").to_string();
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let mut ctx = self.ctx.lock();
                ctx.principal = principal;
                let output = handlers::dispatch(&ctx, &name, arguments);
                ctx.principal = None;
                let result = ToolCallResult::from_tool_output(&output);
                McpResponse::success(request.id, json!(result))
            }
            _ => McpResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }
}

/// A decoded `tools/call` result: the wire envelope plus its parsed payload.
struct ToolEnvelope {
    raw: Value,
    payload: Value,
}

impl ToolEnvelope {
    fn from_result(raw: Value) -> Self {
        let content = raw["content"].as_array().expect("content array");
        assert_eq!(content.len(), 1, "exactly one content block: {raw}");
        assert_eq!(content[0]["type"], "text", "text content block: {raw}");
        let text = content[0]["text"].as_str().expect("text payload");
        let payload = serde_json::from_str(text).expect("tool text must be JSON");
        Self { raw, payload }
    }

    fn is_error(&self) -> bool {
        self.raw["isError"] == json!(true)
    }

    fn code(&self) -> &str {
        self.payload["error"]["code"].as_str().unwrap_or("")
    }
}

fn restricted_token(namespace: &str) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("q4-agent"),
        key_id: "q4-key".to_string(),
        permissions: PermissionSet::standard_user(),
        namespace: Some(namespace.to_string()),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

fn process_bearer() -> TransportPrincipal {
    TransportPrincipal::from_process_bearer(Some("Bearer q4-secret"), "q4-secret")
        .expect("matching process bearer")
}

fn assert_normalized_error(envelope: &ToolEnvelope, context: &str) {
    assert!(envelope.is_error(), "{context}: isError must be true");
    let error = &envelope.payload["error"];
    let code = error["code"].as_str().unwrap_or("");
    assert!(
        NORMALIZED_CODES.contains(&code),
        "{context}: error.code {code:?} is outside the normalized set: {error}"
    );
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty()),
        "{context}: error.message must be a non-empty string: {error}"
    );
}

// ---------------------------------------------------------------------------
// Unknown tool: both documented behaviors, with and without a principal.
// ---------------------------------------------------------------------------

#[test]
fn unknown_tool_without_principal_is_tool_not_found() {
    let handler = ProtocolHandler::new();
    let envelope = handler.call("q4_no_such_tool", json!({}));

    assert_normalized_error(&envelope, "unknown tool, no principal");
    assert_eq!(envelope.code(), "tool_not_found");
    assert_eq!(envelope.payload["error"]["tool"], "q4_no_such_tool");
    assert!(envelope.payload["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("q4_no_such_tool")));
}

#[test]
fn unknown_tool_for_unrestricted_admin_principal_is_tool_not_found() {
    let handler = ProtocolHandler::new();
    let envelope = handler.tool_call(Some(process_bearer()), "q4_no_such_tool", json!({}));

    assert_normalized_error(&envelope, "unknown tool, process bearer (admin)");
    assert_eq!(envelope.code(), "tool_not_found");
}

#[test]
fn unknown_tool_for_restricted_principals_is_permission_denied() {
    let handler = ProtocolHandler::new();
    let principals: [(&str, TransportPrincipal); 3] = [
        (
            "anonymous loopback",
            TransportPrincipal::anonymous_loopback(),
        ),
        ("namespaced stored token", restricted_token("q4-own")),
        (
            "unscoped standard token",
            TransportPrincipal::from_token_claims(TokenClaims {
                user_id: UserId::from_string("q4-operator"),
                key_id: "q4-operator-key".to_string(),
                permissions: PermissionSet::standard_user(),
                namespace: None,
                issued_at: Utc::now(),
                expires_at: None,
            })
            .expect("valid token claims"),
        ),
    ];
    let argument_shapes = [json!({}), json!({"workspace": "q4-own"})];

    for (label, principal) in principals {
        for arguments in &argument_shapes {
            let envelope = handler.tool_call(
                Some(principal.clone()),
                "q4_no_such_tool",
                arguments.clone(),
            );
            let context = format!("unknown tool, {label}, args {arguments}");
            assert_normalized_error(&envelope, &context);
            assert_eq!(
                envelope.code(),
                "permission_denied",
                "{context}: unknown names require admin under a principal: {}",
                envelope.payload
            );
            assert_ne!(
                envelope.code(),
                "tool_not_found",
                "{context}: a restricted caller must not learn which names exist"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Invalid params and normalized envelopes.
// ---------------------------------------------------------------------------

#[test]
fn invalid_params_return_normalized_envelopes_with_is_error() {
    let handler = ProtocolHandler::new();
    let cases: [(&str, Value, &[&str]); 8] = [
        (
            "memory_create",
            json!({"content": 123}),
            &["invalid_params"],
        ),
        (
            "memory_create",
            json!({"content": "x", "tags": "not-a-list"}),
            &["invalid_params"],
        ),
        (
            "memory_create",
            json!({"content": "x", "importance": "high"}),
            &["invalid_params"],
        ),
        (
            "memory_create",
            json!({"content": "x", "memory_type": "not-a-type"}),
            &["invalid_params"],
        ),
        ("memory_get", json!({}), &["missing_argument"]),
        ("memory_delete", json!({}), &["missing_argument"]),
        // Known gap: a wrong-typed `id` is reported as a missing argument, not
        // `invalid_params`. Either is a rejection; neither may succeed.
        (
            "memory_get",
            json!({"id": "not-a-number"}),
            &["missing_argument", "invalid_params"],
        ),
        (
            "memory_update",
            json!({"id": true, "content": "x"}),
            &["missing_argument", "invalid_params"],
        ),
    ];

    for (tool, arguments, accepted) in cases {
        let envelope = handler.call(tool, arguments.clone());
        let context = format!("{tool} {arguments}");
        assert_normalized_error(&envelope, &context);
        assert!(
            accepted.contains(&envelope.code()),
            "{context}: code {:?} not in {accepted:?}",
            envelope.code()
        );
    }
}

#[test]
fn missing_and_absent_entities_carry_structured_details() {
    let handler = ProtocolHandler::new();

    let missing = handler.call("memory_get", json!({}));
    assert_eq!(missing.payload["error"]["details"]["argument"], "id");

    let absent = handler.call("memory_get", json!({"id": 987_654}));
    assert_normalized_error(&absent, "absent memory");
    assert_eq!(absent.code(), "not_found");
    assert_eq!(absent.payload["error"]["details"]["entity"], "memory");
    assert_eq!(absent.payload["error"]["details"]["id"], "987654");
}

#[test]
fn per_call_permission_override_returns_normalized_denial() {
    let handler = ProtocolHandler::new();
    let created = handler.call("memory_create", json!({"content": "q4 denial target"}));
    let id = created.payload["id"].as_i64().expect("created id");

    let denied = handler.call(
        "memory_delete",
        json!({"id": id, "_permission_mode": "read_only"}),
    );
    assert_normalized_error(&denied, "read_only delete");
    assert_eq!(denied.code(), "permission_denied");
    assert_eq!(denied.payload["error"]["tool"], "memory_delete");
    assert_eq!(denied.payload["error"]["current_mode"], "read_only");
    assert_eq!(denied.payload["error"]["required_mode"], "admin");

    let still_there = handler.call("memory_get", json!({"id": id}));
    assert!(!still_there.is_error(), "denied delete must not mutate");
    assert_eq!(still_there.payload["content"], "q4 denial target");
}

#[test]
fn success_envelope_has_no_error_flag_and_text_json() {
    let handler = ProtocolHandler::new();
    let created = handler.call("memory_create", json!({"content": "q4 success shape"}));

    assert!(!created.is_error());
    assert!(
        created.raw.get("isError").is_none(),
        "successful calls omit isError: {}",
        created.raw
    );
    assert_eq!(created.payload["content"], "q4 success shape");
    assert!(created.payload.get("error").is_none());
}

#[test]
fn legacy_string_errors_are_still_flagged_is_error() {
    // Several handlers predate the normalized contract and return
    // `{"error": "<string>"}`. Clients rely on `isError`, which both shapes set.
    let handler = ProtocolHandler::new();
    let envelope = handler.call("memory_block_create", json!({}));

    assert!(envelope.is_error(), "{}", envelope.raw);
    assert!(
        envelope.payload.get("error").is_some(),
        "legacy error payload must still carry `error`: {}",
        envelope.payload
    );
}

#[test]
fn unknown_jsonrpc_method_is_a_protocol_error_not_a_tool_error() {
    let handler = ProtocolHandler::new();
    let response = handler.handle_request(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(9)),
        method: "tools/does_not_exist".to_string(),
        params: json!({}),
    });

    let error = response.error.expect("JSON-RPC error");
    assert_eq!(error.code, -32601);
    assert!(response.result.is_none());
}

// ---------------------------------------------------------------------------
// snake_case on the wire; camelCase is not an alias.
// ---------------------------------------------------------------------------

fn is_snake_case(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[test]
fn registry_tool_and_argument_names_are_snake_case() {
    let mut offenders = Vec::new();
    for tool in get_tool_definitions() {
        let tool = serde_json::to_value(&tool).expect("serialize tool definition");
        let name = tool["name"].as_str().expect("tool name");
        if !is_snake_case(name) {
            offenders.push(format!("tool {name}"));
        }
        if let Some(properties) = tool["inputSchema"]["properties"].as_object() {
            for property in properties.keys().filter(|key| !is_snake_case(key)) {
                offenders.push(format!("{name}.{property}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "non-snake_case names in the public registry: {offenders:?}"
    );
}

#[test]
fn camel_case_argument_keys_are_ignored_not_aliased() {
    // Characterization: unknown argument keys are dropped without an error, so a
    // camelCase caller silently loses the value. SDKs must send snake_case. If an
    // alias is ever added deliberately, update this test and docs/MCP_TOOLS.md.
    let handler = ProtocolHandler::new();

    let camel = handler.call(
        "memory_create",
        json!({"content": "q4 camel", "memoryType": "issue"}),
    );
    assert!(
        !camel.is_error(),
        "unknown keys do not error: {}",
        camel.raw
    );
    assert_eq!(
        camel.payload["type"], "note",
        "camelCase key must not apply"
    );

    let snake = handler.call(
        "memory_create",
        json!({"content": "q4 snake", "memory_type": "issue"}),
    );
    assert_eq!(snake.payload["type"], "issue", "snake_case key applies");
}

// ---------------------------------------------------------------------------
// Pagination.
// ---------------------------------------------------------------------------

fn page_ids(handler: &ProtocolHandler, limit: i64, offset: i64) -> Vec<i64> {
    let page = handler.call(
        "memory_list",
        json!({
            "workspace": "q4-pages",
            "sort_by": "created_at",
            "sort_order": "asc",
            "limit": limit,
            "offset": offset,
        }),
    );
    assert!(!page.is_error(), "memory_list failed: {}", page.raw);
    page.payload
        .as_array()
        .expect("memory_list returns an array")
        .iter()
        .map(|memory| memory["id"].as_i64().expect("memory id"))
        .collect()
}

#[test]
fn memory_list_pages_are_disjoint_ordered_and_exhaustive() {
    let handler = ProtocolHandler::new();
    let mut created = Vec::new();
    for index in 0..5 {
        let memory = handler.call(
            "memory_create",
            json!({"content": format!("q4 page entry {index}"), "workspace": "q4-pages"}),
        );
        created.push(memory.payload["id"].as_i64().expect("created id"));
    }
    handler.call(
        "memory_create",
        json!({"content": "q4 other workspace entry", "workspace": "q4-other"}),
    );

    let pages = [
        page_ids(&handler, 2, 0),
        page_ids(&handler, 2, 2),
        page_ids(&handler, 2, 4),
        page_ids(&handler, 2, 6),
    ];

    assert_eq!(
        pages.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 0],
        "page sizes must drain the workspace and then return empty: {pages:?}"
    );
    let mut seen: Vec<i64> = pages.iter().flatten().copied().collect();
    let in_order = seen.clone();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 5, "pages overlap: {pages:?}");
    let mut expected = created;
    expected.sort_unstable();
    assert_eq!(
        seen, expected,
        "pages must cover exactly the workspace rows"
    );
    assert_eq!(
        in_order,
        {
            let mut ordered = in_order.clone();
            ordered.sort_unstable();
            ordered
        },
        "ascending creation order must hold across pages: {pages:?}"
    );
}

#[test]
fn memory_search_limit_bounds_the_result_count() {
    let handler = ProtocolHandler::new();
    for index in 0..3 {
        handler.call(
            "memory_create",
            json!({"content": format!("q4 limit probe entry {index}"), "workspace": "q4-limit"}),
        );
    }
    let search = |limit: i64| {
        let result = handler.call(
            "memory_search",
            json!({
                "query": "limit probe",
                "workspace": "q4-limit",
                "rerank": false,
                "limit": limit,
                // Bypass the result cache so this exercises the limit itself; the
                // cache-key behavior is covered by the tests below.
                "skip_cache": true,
            }),
        );
        assert!(!result.is_error(), "memory_search failed: {}", result.raw);
        result.payload.as_array().expect("array of hits").len()
    };

    assert_eq!(search(1), 1);
    assert_eq!(search(2), 2);
    assert_eq!(search(3), 3);
}

/// Executable record of G-3: the result cache key omits `limit`, so a repeated
/// query returns the first answer whatever limit is asked. This asserts the
/// CORRECT behavior (fixed by including `limit` and the other result-shaping
/// options in the cache key).
#[test]
fn memory_search_limit_survives_the_result_cache() {
    let handler = ProtocolHandler::new();
    for index in 0..3 {
        handler.call(
            "memory_create",
            json!({"content": format!("q4 cache probe entry {index}"), "workspace": "q4-cache"}),
        );
    }
    let search = |limit: i64| {
        let result = handler.call(
            "memory_search",
            json!({"query": "cache probe", "workspace": "q4-cache", "rerank": false, "limit": limit}),
        );
        assert!(!result.is_error(), "memory_search failed: {}", result.raw);
        result.payload.as_array().expect("array of hits").len()
    };

    assert_eq!(search(3), 3);
    assert_eq!(
        search(1),
        1,
        "a cached answer must still honor a smaller limit"
    );
    assert_eq!(search(2), 2);
}

/// G-3 follow-up: every result-shaping option is part of the cache key, not only
/// `limit` (multi-workspace filter, minimum score).
#[test]
fn memory_search_cache_distinguishes_workspaces_filter_and_min_score() {
    let handler = ProtocolHandler::new();
    for ws in ["q4-ckey-a", "q4-ckey-b"] {
        handler.call(
            "memory_create",
            json!({"content": format!("ckey probe entry {ws}"), "workspace": ws}),
        );
    }
    let ids_for = |workspaces: &str| -> Vec<String> {
        let result = handler.call(
            "memory_search",
            json!({"query": "ckey probe", "workspaces": [workspaces], "rerank": false}),
        );
        assert!(!result.is_error(), "memory_search failed: {}", result.raw);
        result
            .payload
            .as_array()
            .expect("array of hits")
            .iter()
            .map(|hit| hit["memory"]["workspace"].as_str().unwrap().to_string())
            .collect()
    };

    assert_eq!(ids_for("q4-ckey-a"), vec!["q4-ckey-a"]);
    assert_eq!(
        ids_for("q4-ckey-b"),
        vec!["q4-ckey-b"],
        "a cached answer for another workspaces filter must not be served"
    );

    let with_min_score = |min_score: f64| -> usize {
        let result = handler.call(
            "memory_search",
            json!({
                "query": "ckey probe",
                "workspaces": ["q4-ckey-a", "q4-ckey-b"],
                "rerank": false,
                "min_score": min_score,
            }),
        );
        assert!(!result.is_error(), "memory_search failed: {}", result.raw);
        result.payload.as_array().expect("array of hits").len()
    };
    assert_eq!(with_min_score(0.0), 2);
    assert_eq!(
        with_min_score(10.0),
        0,
        "a cached answer for another min_score must not be served"
    );
}

/// Q2-B10: a stored row that cannot be decoded must surface as a normalized
/// error from `memory_list`, never as a shorter list that looks complete.
#[test]
fn memory_list_reports_an_undecodable_row_instead_of_dropping_it() {
    let handler = ProtocolHandler::new();
    let mut ids = Vec::new();
    for index in 0..2 {
        let created = handler.call(
            "memory_create",
            json!({"content": format!("q2b10 row {index}"), "workspace": "q2b10"}),
        );
        assert!(!created.is_error(), "memory_create: {}", created.raw);
        ids.push(created.payload["id"].as_i64().expect("id"));
    }
    let healthy = handler.call("memory_list", json!({"workspace": "q2b10"}));
    assert_eq!(healthy.payload.as_array().expect("rows").len(), 2);

    handler
        .ctx
        .lock()
        .storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET importance = 'not-a-number' WHERE id = ?",
                [ids[0]],
            )?;
            Ok(())
        })
        .expect("corrupt one row");

    let listed = handler.call("memory_list", json!({"workspace": "q2b10"}));
    assert_normalized_error(&listed, "memory_list over a corrupt row");
    assert_eq!(listed.code(), "internal_error");
    let message = listed.payload["error"]["message"].as_str().unwrap_or("");
    assert!(
        message.contains(&format!("id {}", ids[0])),
        "the error must name the corrupt row: {message}"
    );
}

/// C3 follow-up: huge numeric offsets used to reach `chrono::Duration::seconds`,
/// `Duration::days` or `DateTime + Duration`, which panic on overflow (process
/// abort in release). Covered here, through `tools/call`: `memory_create`,
/// `memory_create_daily`, `memory_update`, `memory_set_expiration`
/// (`ttl_seconds`), `context_record_artifact` (`ttl_seconds`,
/// `stale_after_seconds`), `memory_boost` (`duration_seconds`),
/// `memory_archive_old` (`max_age_days`) and `memory_get_working_memory`
/// (`since_minutes`). Other chrono call sites are bounded in the storage layer
/// but not exercised through MCP here.
#[test]
fn huge_time_offsets_are_rejected_with_normalized_errors_on_covered_tools() {
    // `memory_create`, `memory_boost` and `context_record_artifact` still answer with the legacy
    // `{"error": "<message>"}` string (isError set, not the normalized object);
    // that predates this change. Every other tool here must be normalized.
    const LEGACY_STRING_ERROR_TOOLS: [&str; 3] =
        ["memory_create", "memory_boost", "context_record_artifact"];
    let handler = ProtocolHandler::new();
    let seed = handler.call(
        "memory_create",
        json!({"content": "ttl overflow seed", "workspace": "q2f-ttl", "tier": "daily", "ttl_seconds": 3600}),
    );
    assert!(!seed.is_error(), "seed: {}", seed.raw);
    let id = seed.payload["id"].as_i64().expect("id");

    // Non-positive TTLs mean "default"/"remove" on create and update, so only
    // positive overflow is an error there; `memory_set_expiration` bounds both signs.
    for ttl in [i64::MAX, i64::MAX / 1000, 3_155_760_001] {
        let calls = [
            (
                "memory_create",
                json!({"content": "ttl create", "workspace": "q2f-ttl", "tier": "daily", "ttl_seconds": ttl}),
            ),
            (
                "memory_create_daily",
                json!({"content": "ttl daily", "workspace": "q2f-ttl", "ttl_seconds": ttl}),
            ),
            ("memory_update", json!({"id": id, "ttl_seconds": ttl})),
            (
                "memory_set_expiration",
                json!({"id": id, "ttl_seconds": ttl}),
            ),
            (
                "context_record_artifact",
                json!({"kind": "tool_output", "repo_id": "q2f-repo", "ttl_seconds": ttl}),
            ),
            (
                "context_record_artifact",
                json!({"kind": "tool_output", "repo_id": "q2f-repo", "stale_after_seconds": ttl}),
            ),
            ("memory_boost", json!({"id": id, "duration_seconds": ttl})),
        ];
        for (tool, args) in calls {
            let result = handler.call(tool, args);
            let context = format!("{tool} with offset {ttl}");
            if LEGACY_STRING_ERROR_TOOLS.contains(&tool) {
                // Pre-existing handler shape: `{"error": "<message>"}` with isError.
                assert!(result.is_error(), "{context}: isError must be true");
                let message = result.payload["error"].as_str().unwrap_or("");
                assert!(message.contains("out of range"), "{context}: {message}");
            } else {
                assert_normalized_error(&result, &context);
            }
        }
    }
    for (tool, args) in [
        (
            "memory_set_expiration",
            json!({"id": id, "ttl_seconds": i64::MIN}),
        ),
        (
            "memory_archive_old",
            json!({"max_age_days": i64::MAX, "workspace": "q2f-ttl"}),
        ),
        (
            "memory_get_working_memory",
            json!({"session_id": "q2f-session", "since_minutes": u64::MAX}),
        ),
    ] {
        let result = handler.call(tool, args);
        assert_normalized_error(&result, &format!("{tool} with an extreme offset"));
    }
    let invalid = handler.call(
        "memory_set_expiration",
        json!({"id": id, "ttl_seconds": i64::MAX}),
    );
    assert_eq!(invalid.code(), "invalid_params", "{}", invalid.raw);
    let missing = handler.call("memory_set_expiration", json!({"id": id}));
    assert_eq!(missing.code(), "missing_argument", "{}", missing.raw);

    // The memory survives the rejected calls and a sane TTL still works.
    let ok = handler.call(
        "memory_set_expiration",
        json!({"id": id, "ttl_seconds": 7200}),
    );
    assert_eq!(ok.payload["success"], true, "{}", ok.raw);
}

// ---------------------------------------------------------------------------
// Async lifecycle: a job moves through persisted states.
// ---------------------------------------------------------------------------

#[cfg(feature = "dream-phase")]
#[test]
fn dream_job_lifecycle_moves_pending_canceled_archived_and_is_idempotent() {
    let handler = ProtocolHandler::new();
    let created = handler.call(
        "dream_create",
        json!({"workspace": "q4-lifecycle", "job_id": "q4-job", "run": false}),
    );
    assert!(!created.is_error(), "dream_create: {}", created.raw);
    assert_eq!(created.payload["job"]["id"], "q4-job");
    assert_eq!(created.payload["job"]["status"], "pending");

    let status_of = |id: &str| {
        let fetched = handler.call("dream_get", json!({"id": id}));
        assert!(!fetched.is_error(), "dream_get: {}", fetched.raw);
        fetched.payload["job"]["status"]
            .as_str()
            .expect("job status")
            .to_string()
    };
    assert_eq!(status_of("q4-job"), "pending");

    let canceled = handler.call("dream_cancel", json!({"id": "q4-job"}));
    assert_eq!(canceled.payload["job"]["status"], "canceled");
    let canceled_again = handler.call("dream_cancel", json!({"id": "q4-job"}));
    assert_eq!(
        canceled_again.payload["job"]["status"], "canceled",
        "cancel is idempotent: {}",
        canceled_again.payload
    );
    assert_eq!(status_of("q4-job"), "canceled");

    let listed = handler.call(
        "dream_list",
        json!({"workspace": "q4-lifecycle", "status": "canceled"}),
    );
    assert_eq!(listed.payload["count"], 1);

    let archived = handler.call("dream_archive", json!({"id": "q4-job"}));
    assert_eq!(archived.payload["job"]["status"], "archived");
    assert_eq!(status_of("q4-job"), "archived");

    // Characterization: an unknown job id is not an error; `job` is null.
    let absent = handler.call("dream_get", json!({"id": "q4-missing-job"}));
    assert!(!absent.is_error(), "{}", absent.raw);
    assert!(absent.payload["job"].is_null(), "{}", absent.payload);
}
