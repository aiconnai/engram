//! Streaming Protocol Tests — RFC 0001 Tier 1
//!
//! Validates the MCP progress notification protocol implementation:
//! - JSON-RPC progress notification formatting and bounds
//! - ProgressToken extraction from request params
//! - ChannelProgressReporter and NoopProgressReporter behavior
//! - ProgressReporterExt convenience methods
//! - dispatch_with_progress integration for batch operations

use serde_json::json;
use std::sync::Arc;

use engram::mcp::progress::{
    extract_progress_token, ChannelProgressReporter, NoopProgressReporter, ProgressNotification,
    ProgressReporter, ProgressReporterExt, ProgressToken,
};

// ---------------------------------------------------------------------------
// Progress Token Tests
// ---------------------------------------------------------------------------

#[test]
fn test_progress_token_string_roundtrip() {
    let token = ProgressToken::String("req-42".to_string());
    let json = serde_json::to_value(&token).unwrap();
    assert_eq!(json, json!("req-42"));
    let parsed: ProgressToken = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, token);
}

#[test]
fn test_progress_token_integer_roundtrip() {
    let token = ProgressToken::Integer(1234);
    let json = serde_json::to_value(&token).unwrap();
    assert_eq!(json, json!(1234));
    let parsed: ProgressToken = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, token);
}

#[test]
fn test_progress_token_from_str() {
    let token = ProgressToken::from("hello");
    assert_eq!(token, ProgressToken::String("hello".to_string()));
}

#[test]
fn test_progress_token_from_i64() {
    let token = ProgressToken::from(99i64);
    assert_eq!(token, ProgressToken::Integer(99));
}

#[test]
fn test_progress_token_display() {
    assert_eq!(format!("{}", ProgressToken::String("abc".into())), "abc");
    assert_eq!(format!("{}", ProgressToken::Integer(42)), "42");
}

// ---------------------------------------------------------------------------
// Extract Progress Token
// ---------------------------------------------------------------------------

#[test]
fn test_extract_token_from_meta_string() {
    let params = json!({
        "_meta": { "progressToken": "pt-1" },
        "name": "memory_create_batch",
        "arguments": {}
    });
    assert_eq!(
        extract_progress_token(&params),
        Some(ProgressToken::String("pt-1".to_string()))
    );
}

#[test]
fn test_extract_token_from_meta_integer() {
    let params = json!({
        "_meta": { "progressToken": 7 }
    });
    assert_eq!(
        extract_progress_token(&params),
        Some(ProgressToken::Integer(7))
    );
}

#[test]
fn test_extract_token_no_meta() {
    assert!(extract_progress_token(&json!({})).is_none());
    assert!(extract_progress_token(&json!({"name": "test"})).is_none());
}

#[test]
fn test_extract_token_meta_without_progress_token() {
    let params = json!({ "_meta": { "other": "value" } });
    assert!(extract_progress_token(&params).is_none());
}

#[test]
fn test_extract_token_invalid_type() {
    let params = json!({ "_meta": { "progressToken": true } });
    assert!(extract_progress_token(&params).is_none());

    let params = json!({ "_meta": { "progressToken": [1, 2] } });
    assert!(extract_progress_token(&params).is_none());

    let params = json!({ "_meta": { "progressToken": null } });
    assert!(extract_progress_token(&params).is_none());
}

// ---------------------------------------------------------------------------
// Progress Notification Formatting
// ---------------------------------------------------------------------------

#[test]
fn test_notification_has_correct_jsonrpc_structure() {
    let notification = ProgressNotification::new(
        ProgressToken::String("req-1".to_string()),
        5,
        Some(10),
        Some("Processing".to_string()),
    );
    let json = serde_json::to_value(&notification).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["method"], "notifications/progress");
    assert!(json.get("id").is_none(), "notifications must not have id");
    assert_eq!(json["params"]["progressToken"], "req-1");
    assert_eq!(json["params"]["progress"], 5);
    assert_eq!(json["params"]["total"], 10);
    assert_eq!(json["params"]["message"], "Processing");
}

#[test]
fn test_notification_without_optional_fields() {
    let notification = ProgressNotification::new(ProgressToken::Integer(99), 3, None, None);
    let json = serde_json::to_value(&notification).unwrap();

    assert_eq!(json["params"]["progress"], 3);
    // Optional fields should be omitted (skip_serializing_if = "Option::is_none")
    assert!(json["params"].get("total").is_none());
    assert!(json["params"].get("message").is_none());
}

#[test]
fn test_notification_with_integer_token() {
    let notification = ProgressNotification::new(ProgressToken::Integer(42), 1, Some(5), None);
    let json = serde_json::to_value(&notification).unwrap();
    assert_eq!(json["params"]["progressToken"], 42);
}

// ---------------------------------------------------------------------------
// Channel Reporter
// ---------------------------------------------------------------------------

#[test]
fn test_channel_reporter_sends_notifications() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("batch-1"));

    reporter.report(1, Some(3), Some("item 1".to_string()));
    reporter.report(2, Some(3), Some("item 2".to_string()));
    reporter.report(3, Some(3), Some("done".to_string()));

    let notifications: Vec<ProgressNotification> = rx.try_iter().collect();
    assert_eq!(notifications.len(), 3);

    // Verify monotonically increasing progress
    for (i, n) in notifications.iter().enumerate() {
        assert_eq!(n.params.progress, (i + 1) as u64);
        assert_eq!(n.params.total, Some(3));
        assert_eq!(
            n.params.progress_token,
            ProgressToken::String("batch-1".to_string())
        );
    }
}

#[test]
fn test_channel_reporter_survives_dropped_receiver() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("drop"));
    drop(rx);
    // Best-effort: no panic on send after drop
    reporter.report(1, Some(10), Some("after drop".to_string()));
}

#[test]
fn test_channel_reporter_from_sender() {
    let (tx, rx) = std::sync::mpsc::channel();
    let reporter =
        ChannelProgressReporter::from_sender(ProgressToken::String("shared".to_string()), tx);

    reporter.report(5, Some(10), Some("halfway".to_string()));

    let notification = rx.try_recv().unwrap();
    assert_eq!(notification.params.progress, 5);
    assert_eq!(notification.params.total, Some(10));
}

// ---------------------------------------------------------------------------
// Noop Reporter
// ---------------------------------------------------------------------------

#[test]
fn test_noop_reporter_discards_silently() {
    let reporter = NoopProgressReporter;
    // Must not panic or produce side effects
    reporter.report(0, None, None);
    reporter.report(100, Some(100), Some("done".to_string()));
}

// ---------------------------------------------------------------------------
// Reporter Extension Methods
// ---------------------------------------------------------------------------

#[test]
fn test_fraction_scales_to_100() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("frac"));

    reporter.fraction(0.0, "start");
    reporter.fraction(0.25, "quarter");
    reporter.fraction(0.5, "half");
    reporter.fraction(0.75, "three-quarters");
    reporter.fraction(1.0, "done");

    let notifications: Vec<_> = rx.try_iter().collect();
    assert_eq!(notifications.len(), 5);
    assert_eq!(notifications[0].params.progress, 0);
    assert_eq!(notifications[1].params.progress, 25);
    assert_eq!(notifications[2].params.progress, 50);
    assert_eq!(notifications[3].params.progress, 75);
    assert_eq!(notifications[4].params.progress, 100);

    // All should have total=100
    for n in &notifications {
        assert_eq!(n.params.total, Some(100));
    }
}

#[test]
fn test_fraction_clamps_to_bounds() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("clamp"));

    reporter.fraction(-0.5, "below zero");
    reporter.fraction(1.5, "above one");

    let notifications: Vec<_> = rx.try_iter().collect();
    assert_eq!(notifications[0].params.progress, 0);
    assert_eq!(notifications[1].params.progress, 100);
}

#[test]
fn test_step_helper() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("step"));

    reporter.step(3, 10, "processing item 3");

    let n = rx.try_recv().unwrap();
    assert_eq!(n.params.progress, 3);
    assert_eq!(n.params.total, Some(10));
    assert_eq!(n.params.message.as_deref(), Some("processing item 3"));
}

#[test]
fn test_complete_helper() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("complete"));

    reporter.complete(10, "batch finished");

    let n = rx.try_recv().unwrap();
    assert_eq!(n.params.progress, 10);
    assert_eq!(n.params.total, Some(10));
    assert_eq!(n.params.message.as_deref(), Some("batch finished"));
}

// ---------------------------------------------------------------------------
// Arc<dyn ProgressReporter> usability
// ---------------------------------------------------------------------------

#[test]
fn test_arc_reporter_impl() {
    let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("arc"));
    let arc_reporter: Arc<dyn ProgressReporter> = Arc::new(reporter);

    // Can call report through Arc
    arc_reporter.report(1, Some(5), Some("through arc".to_string()));

    // Can also use extension methods
    arc_reporter.step(2, 5, "step via arc");

    let notifications: Vec<_> = rx.try_iter().collect();
    assert_eq!(notifications.len(), 2);
    assert_eq!(notifications[0].params.progress, 1);
    assert_eq!(notifications[1].params.progress, 2);
}

// ---------------------------------------------------------------------------
// Wire format serialization: full round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_notification_json_wire_format() {
    let notification = ProgressNotification::new(
        ProgressToken::String("test-token".to_string()),
        42,
        Some(100),
        Some("Indexing memories".to_string()),
    );

    let wire = serde_json::to_string(&notification).unwrap();

    // Deserialize back and verify
    let parsed: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "notifications/progress");
    assert_eq!(parsed["params"]["progressToken"], "test-token");
    assert_eq!(parsed["params"]["progress"], 42);
    assert_eq!(parsed["params"]["total"], 100);
    assert_eq!(parsed["params"]["message"], "Indexing memories");

    // Should parse as a valid ProgressNotification
    let reparsed: ProgressNotification = serde_json::from_str(&wire).unwrap();
    assert_eq!(reparsed.params.progress, 42);
    assert_eq!(
        reparsed.params.progress_token,
        ProgressToken::String("test-token".to_string())
    );
}

#[test]
fn test_notification_parses_as_valid_jsonrpc() {
    // Verify it's a valid JSON-RPC 2.0 notification (no id field)
    let notification = ProgressNotification::new(ProgressToken::Integer(1), 0, Some(10), None);
    let json = serde_json::to_value(&notification).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert!(
        json.get("id").is_none(),
        "JSON-RPC notifications must not have an id"
    );
    assert!(json.get("method").is_some(), "must have method");
    assert!(json.get("params").is_some(), "must have params");
}

// ---------------------------------------------------------------------------
// Boundary / Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn test_progress_zero_total() {
    let notification = ProgressNotification::new(ProgressToken::from("z"), 0, Some(0), None);
    let json = serde_json::to_value(&notification).unwrap();
    assert_eq!(json["params"]["progress"], 0);
    assert_eq!(json["params"]["total"], 0);
}

#[test]
fn test_progress_large_values() {
    let notification = ProgressNotification::new(
        ProgressToken::from("big"),
        u64::MAX,
        Some(u64::MAX),
        Some("huge batch".to_string()),
    );
    let json = serde_json::to_value(&notification).unwrap();
    assert_eq!(json["params"]["progress"], u64::MAX);
    assert_eq!(json["params"]["total"], u64::MAX);
}

#[test]
fn test_empty_message_string() {
    let notification =
        ProgressNotification::new(ProgressToken::from("e"), 1, Some(1), Some(String::new()));
    let json = serde_json::to_value(&notification).unwrap();
    assert_eq!(json["params"]["message"], "");
}

#[test]
fn test_progress_token_negative_integer() {
    let token = ProgressToken::Integer(-1);
    let json = serde_json::to_value(&token).unwrap();
    assert_eq!(json, json!(-1));
    let parsed: ProgressToken = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, token);
}
