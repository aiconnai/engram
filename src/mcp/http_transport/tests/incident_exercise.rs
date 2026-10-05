//! O1 local incident exercise: trigger real failures against a local router,
//! read the exported `/health`, and show which alert fires with its owner,
//! runbook and rollback. Notifications go to an in-memory sink only; nothing is
//! sent anywhere. Availability here is the availability of this process since
//! it started, not a deployment SLO.

use std::sync::Arc;
use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use super::support::{json_rpc_request, test_app_with_limits};
use crate::error::EngramError;
use crate::mcp::http_transport::security_config::HttpSecurityConfig;
use crate::mcp::protocol::{McpHandler, McpRequest, McpResponse};
use crate::observability::alerts::{evaluate_all, AlertEvaluation, AlertStatus};
use crate::observability::{record_recovery, CriticalCounters, RecoveryOutcome};

const KEY: &str = "incident-exercise-key";

/// Panics for `boom`, sleeps past the transport timeout for `slow`.
struct IncidentHandler;

impl McpHandler for IncidentHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.params["name"].as_str() {
            Some("boom") => panic!("simulated handler bug"),
            Some("slow") => std::thread::sleep(Duration::from_millis(300)),
            _ => {}
        }
        McpResponse::success(request.id, json!({"content": []}))
    }
}

fn call(name: &str, bearer: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {bearer}"))
        .body(Body::from(
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":{}}})
                .to_string(),
        ))
        .expect("request")
}

async fn health(app: &axum::Router) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health");
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json")
}

/// The local "pager": formats what an on-call would read and keeps it in memory.
fn notify(sink: &mut Vec<String>, alert: &AlertEvaluation) {
    sink.push(format!(
        "ALERT {} | owner: {} | runbook: docs/OPERATIONS.md#{} | rollback: {} | why: {}",
        alert.id,
        alert.owner,
        alert
            .runbook
            .to_lowercase()
            .replace([' ', ':'], "-")
            .replace("--", "-"),
        alert.rollback,
        alert.detail
    ));
}

#[tokio::test]
async fn local_incident_exercise_fires_alerts_with_owner_runbook_and_rollback() {
    let mut security = HttpSecurityConfig::default();
    security.request_timeout = Duration::from_millis(100);
    let app = test_app_with_limits(Arc::new(IncidentHandler), Some(KEY), None, security, 0, 0);

    // 1. Before any traffic nothing may look healthy: EVERY rule is no_data. The
    //    process-wide critical counters are shared by the whole lib test binary,
    //    so a fresh process' export is simulated with fresh counters; the HTTP
    //    half is this router's own (fresh) metrics.
    let mut fresh = health(&app).await;
    fresh["observability"] = json!(CriticalCounters::new().snapshot());
    let quiet = evaluate_all(&fresh);
    assert_eq!(quiet.len(), crate::observability::alerts::RULES.len());
    for rule in &quiet {
        assert_eq!(
            rule.status,
            AlertStatus::NoData,
            "{} must not report ok or firing without samples: {rule:?}",
            rule.id
        );
    }
    // And with the observability export off the same rules stay no_data.
    let mut export_off = fresh.clone();
    export_off
        .as_object_mut()
        .expect("object")
        .remove("observability");
    assert!(evaluate_all(&export_off)
        .iter()
        .all(|e| e.status == AlertStatus::NoData));

    // 2. Inject real failures.
    for _ in 0..3 {
        let panicked = app
            .clone()
            .oneshot(call("boom", KEY))
            .await
            .expect("response");
        assert_eq!(
            panicked.status(),
            StatusCode::OK,
            "panic is answered, not dropped"
        );
    }
    let timed_out = app
        .clone()
        .oneshot(call("slow", KEY))
        .await
        .expect("response");
    assert_eq!(timed_out.status(), StatusCode::REQUEST_TIMEOUT);
    for _ in 0..60 {
        app.clone()
            .oneshot(call("fine", "wrong-key"))
            .await
            .expect("response");
    }
    for _ in 0..25 {
        app.clone()
            .oneshot(json_rpc_request("/mcp", Some(KEY)))
            .await
            .expect("response");
    }
    // Counters outside the HTTP path: a SQLITE_BUSY conversion and a failed recovery.
    let busy =
        rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY), None);
    let _ = EngramError::from(busy);
    record_recovery(RecoveryOutcome::Failed);

    // 3. Evaluate the export and "page" locally.
    let evals = evaluate_all(&health(&app).await);
    let mut sink = Vec::new();
    for alert in evals.iter().filter(|e| e.status == AlertStatus::Firing) {
        notify(&mut sink, alert);
    }
    println!("--- local alert sink (in memory, nothing sent) ---");
    for line in &sink {
        println!("{line}");
    }

    for id in [
        "mcp_handler_panic",
        "mcp_timeouts",
        "mcp_auth_failure_rate",
        "sqlite_busy",
        "replication_recover_failed",
    ] {
        let line = sink
            .iter()
            .find(|l| l.starts_with(&format!("ALERT {id} ")))
            .unwrap_or_else(|| panic!("{id} did not fire; sink: {sink:#?}"));
        assert!(line.contains("owner: ") && line.contains("runbook: docs/OPERATIONS.md#"));
        assert!(line.contains("rollback: "));
    }
    // The wire contract stayed intact while failing: panic is a JSON-RPC error.
    let sample = app
        .clone()
        .oneshot(call("boom", KEY))
        .await
        .expect("response");
    let body: Value = serde_json::from_slice(
        &to_bytes(sample.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    assert_eq!(body["error"]["code"], -32603);
}
