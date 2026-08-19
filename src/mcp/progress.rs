//! MCP Progress Notification Protocol (MCP 2025-11-25)
//!
//! Implements the `notifications/progress` JSON-RPC notification as specified by
//! the Model Context Protocol. Provides a transport-agnostic `ProgressReporter`
//! trait that handlers use to emit progress updates without knowing whether the
//! client connected via stdio, HTTP/SSE, or any other transport.
//!
//! # Wire format
//!
//! Request: `{ "_meta": { "progressToken": <string|number> } }`
//!
//! Notification:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "notifications/progress",
//!   "params": {
//!     "progressToken": <string|number>,
//!     "progress": <number>,
//!     "total": <number>,       // optional
//!     "message": "<string>"    // optional
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::mpsc;

// ---------------------------------------------------------------------------
// Progress token
// ---------------------------------------------------------------------------

/// MCP progress token — either a string or an integer, matching the spec.
///
/// The client supplies this in `_meta.progressToken` and the server echoes it
/// back in every `notifications/progress` payload so the client can correlate
/// progress events with the originating request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProgressToken {
    /// A string progress token.
    String(String),
    /// An integer progress token.
    Integer(i64),
}

impl std::fmt::Display for ProgressToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Integer(n) => write!(f, "{n}"),
        }
    }
}

impl From<&str> for ProgressToken {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ProgressToken {
    fn from(n: i64) -> Self {
        Self::Integer(n)
    }
}

/// Try to extract a `ProgressToken` from a `serde_json::Value`.
///
/// Returns `None` if the value is neither a string nor an integer.
impl TryFrom<&Value> for ProgressToken {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(Self::String(s.clone())),
            Value::Number(n) => n.as_i64().map(Self::Integer).ok_or(()),
            _ => Err(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Progress notification payload
// ---------------------------------------------------------------------------

/// The `params` object of a `notifications/progress` JSON-RPC notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressParams {
    /// The token echoed from the original request's `_meta.progressToken`.
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,

    /// A monotonically increasing number representing how far along the
    /// operation has progressed. Must not exceed `total` when `total` is set.
    pub progress: u64,

    /// The expected total number of work units (optional). When absent, the
    /// client should treat the progress bar as indeterminate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,

    /// A human-readable description of the current step (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A complete `notifications/progress` JSON-RPC notification ready for
/// serialization and writing to the transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: ProgressParams,
}

impl ProgressNotification {
    /// Create a new progress notification.
    pub fn new(
        token: ProgressToken,
        progress: u64,
        total: Option<u64>,
        message: Option<String>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "notifications/progress".to_string(),
            params: ProgressParams {
                progress_token: token,
                progress,
                total,
                message,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Request _meta extraction
// ---------------------------------------------------------------------------

/// Extract the `progressToken` from an MCP request's `params._meta` field.
///
/// Returns `None` if `_meta` is absent, `progressToken` is absent, or the
/// value is neither a string nor an integer.
pub fn extract_progress_token(params: &Value) -> Option<ProgressToken> {
    let meta = params.get("_meta")?;
    let token_value = meta.get("progressToken")?;
    ProgressToken::try_from(token_value).ok()
}

// ---------------------------------------------------------------------------
// ProgressReporter trait
// ---------------------------------------------------------------------------

/// Transport-agnostic interface for emitting progress notifications.
///
/// Handlers call `report()` or `fraction()` to emit progress. The concrete
/// implementation decides how to deliver the notification (stdio, SSE, etc.).
///
/// # Thread safety
///
/// Implementations must be `Send + Sync` so handlers can share a reporter
/// across spawned tasks.
pub trait ProgressReporter: Send + Sync {
    /// Emit a progress notification.
    ///
    /// - `progress`: current step (monotonically increasing).
    /// - `total`: expected total steps (optional for indeterminate progress).
    /// - `message`: human-readable description of the current step (optional).
    fn report(&self, progress: u64, total: Option<u64>, message: Option<String>);
}

/// Extension methods for `ProgressReporter`.
pub trait ProgressReporterExt: ProgressReporter {
    /// Emit progress as a fraction (0.0–1.0), scaled to a total of 100.
    fn fraction(&self, frac: f64, message: impl Into<String>) {
        let clamped = frac.clamp(0.0, 1.0);
        let progress = (clamped * 100.0).round() as u64;
        self.report(progress, Some(100), Some(message.into()));
    }

    /// Emit progress for step `i` out of `n` items.
    fn step(&self, i: u64, n: u64, message: impl Into<String>) {
        self.report(i, Some(n), Some(message.into()));
    }

    /// Emit progress for stage `stage_idx` out of `total_stages` execution phases.
    fn stage(&self, stage_idx: u64, total_stages: u64, stage_name: impl Into<String>) {
        self.report(stage_idx, Some(total_stages), Some(stage_name.into()));
    }

    /// Emit progress for a partial chunk assembly.
    fn chunk(&self, chunk_idx: u64, total_chunks: Option<u64>, message: impl Into<String>) {
        self.report(chunk_idx, total_chunks, Some(message.into()));
    }

    /// Emit a completion notification (progress == total).
    fn complete(&self, total: u64, message: impl Into<String>) {
        self.report(total, Some(total), Some(message.into()));
    }
}

// Blanket implementation of `ProgressReporterExt` for all `ProgressReporter`.
impl<T: ProgressReporter + ?Sized> ProgressReporterExt for T {}

// ---------------------------------------------------------------------------
// Channel-based reporter
// ---------------------------------------------------------------------------

/// A `ProgressReporter` that sends `ProgressNotification`s through an
/// `mpsc::Sender`. The receiving end is owned by the transport layer, which
/// serializes and writes each notification.
///
/// This is the primary reporter used in both stdio and HTTP transports.
pub struct ChannelProgressReporter {
    token: ProgressToken,
    tx: mpsc::Sender<ProgressNotification>,
}

impl ChannelProgressReporter {
    /// Create a new channel reporter and its receiving end.
    ///
    /// The caller should drain `rx` and write each `ProgressNotification` to
    /// the appropriate transport.
    pub fn new(token: ProgressToken) -> (Self, mpsc::Receiver<ProgressNotification>) {
        let (tx, rx) = mpsc::channel();
        (Self { token, tx }, rx)
    }

    /// Create a reporter from an existing sender and token.
    pub fn from_sender(token: ProgressToken, tx: mpsc::Sender<ProgressNotification>) -> Self {
        Self { token, tx }
    }
}

impl ProgressReporter for ChannelProgressReporter {
    fn report(&self, progress: u64, total: Option<u64>, message: Option<String>) {
        let notification = ProgressNotification::new(self.token.clone(), progress, total, message);
        // Best-effort: if the receiver is dropped, silently discard.
        let _ = self.tx.send(notification);
    }
}

// ---------------------------------------------------------------------------
// Callback-based reporter
// ---------------------------------------------------------------------------

/// A `ProgressReporter` that invokes a callback function on each notification.
pub struct CallbackProgressReporter<F>
where
    F: Fn(ProgressNotification) + Send + Sync,
{
    token: ProgressToken,
    callback: F,
}

impl<F> CallbackProgressReporter<F>
where
    F: Fn(ProgressNotification) + Send + Sync,
{
    /// Create a new callback reporter.
    pub fn new(token: ProgressToken, callback: F) -> Self {
        Self { token, callback }
    }
}

impl<F> ProgressReporter for CallbackProgressReporter<F>
where
    F: Fn(ProgressNotification) + Send + Sync,
{
    fn report(&self, progress: u64, total: Option<u64>, message: Option<String>) {
        let notification = ProgressNotification::new(self.token.clone(), progress, total, message);
        (self.callback)(notification);
    }
}

// ---------------------------------------------------------------------------
// No-op reporter
// ---------------------------------------------------------------------------

/// A `ProgressReporter` that discards all progress events.
///
/// Used when the client did not supply a `progressToken` in `_meta`, so there
/// is no one listening for progress notifications.
pub struct NoopProgressReporter;

impl ProgressReporter for NoopProgressReporter {
    fn report(&self, _progress: u64, _total: Option<u64>, _message: Option<String>) {
        // Intentionally empty.
    }
}

// ---------------------------------------------------------------------------
// Arc<dyn ProgressReporter> convenience
// ---------------------------------------------------------------------------

impl ProgressReporter for std::sync::Arc<dyn ProgressReporter> {
    fn report(&self, progress: u64, total: Option<u64>, message: Option<String>) {
        (**self).report(progress, total, message);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_token_string_serde() {
        let token = ProgressToken::String("abc-123".to_string());
        let json = serde_json::to_string(&token).expect("serialize");
        assert_eq!(json, "\"abc-123\"");
        let parsed: ProgressToken = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, token);
    }

    #[test]
    fn test_progress_token_integer_serde() {
        let token = ProgressToken::Integer(42);
        let json = serde_json::to_string(&token).expect("serialize");
        assert_eq!(json, "42");
        let parsed: ProgressToken = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, token);
    }

    #[test]
    fn test_extract_progress_token_string() {
        let params = serde_json::json!({
            "_meta": { "progressToken": "req-1" },
            "name": "memory_create_batch"
        });
        let token = extract_progress_token(&params);
        assert_eq!(token, Some(ProgressToken::String("req-1".to_string())));
    }

    #[test]
    fn test_extract_progress_token_integer() {
        let params = serde_json::json!({
            "_meta": { "progressToken": 42 }
        });
        let token = extract_progress_token(&params);
        assert_eq!(token, Some(ProgressToken::Integer(42)));
    }

    #[test]
    fn test_extract_progress_token_missing_meta() {
        let params = serde_json::json!({ "name": "test" });
        assert!(extract_progress_token(&params).is_none());
    }

    #[test]
    fn test_extract_progress_token_missing_token() {
        let params = serde_json::json!({ "_meta": {} });
        assert!(extract_progress_token(&params).is_none());
    }

    #[test]
    fn test_extract_progress_token_invalid_type() {
        let params = serde_json::json!({
            "_meta": { "progressToken": true }
        });
        assert!(extract_progress_token(&params).is_none());
    }

    #[test]
    fn test_progress_notification_serialization() {
        let notification = ProgressNotification::new(
            ProgressToken::String("req-1".to_string()),
            5,
            Some(10),
            Some("Processing item 5 of 10".to_string()),
        );
        let json = serde_json::to_value(&notification).expect("serialize");
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "notifications/progress");
        assert_eq!(json["params"]["progressToken"], "req-1");
        assert_eq!(json["params"]["progress"], 5);
        assert_eq!(json["params"]["total"], 10);
        assert_eq!(json["params"]["message"], "Processing item 5 of 10");
    }

    #[test]
    fn test_progress_notification_without_optional_fields() {
        let notification = ProgressNotification::new(ProgressToken::Integer(99), 3, None, None);
        let json = serde_json::to_value(&notification).expect("serialize");
        assert_eq!(json["params"]["progress"], 3);
        assert!(json["params"].get("total").is_none());
        assert!(json["params"].get("message").is_none());
    }

    #[test]
    fn test_channel_reporter() {
        let token = ProgressToken::from("batch-1");
        let (reporter, rx) = ChannelProgressReporter::new(token);

        reporter.report(1, Some(5), Some("step 1".to_string()));
        reporter.report(2, Some(5), None);
        reporter.report(3, Some(5), Some("step 3".to_string()));

        let msgs: Vec<ProgressNotification> = rx.try_iter().collect();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].params.progress, 1);
        assert_eq!(msgs[1].params.progress, 2);
        assert_eq!(msgs[2].params.progress, 3);
        assert_eq!(msgs[0].params.message.as_deref(), Some("step 1"));
        assert!(msgs[1].params.message.is_none());
    }

    #[test]
    fn test_noop_reporter_does_not_panic() {
        let reporter = NoopProgressReporter;
        reporter.report(1, Some(10), Some("test".to_string()));
        reporter.report(2, None, None);
    }

    #[test]
    fn test_fraction_helper() {
        let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("f"));
        reporter.fraction(0.75, "75% done");
        let msg = rx.try_recv().expect("should have one message");
        assert_eq!(msg.params.progress, 75);
        assert_eq!(msg.params.total, Some(100));
    }

    #[test]
    fn test_fraction_clamps_above_one() {
        let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("f"));
        reporter.fraction(1.5, "clamped");
        let msg = rx.try_recv().expect("should have one message");
        assert_eq!(msg.params.progress, 100);
    }

    #[test]
    fn test_fraction_clamps_below_zero() {
        let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("f"));
        reporter.fraction(-0.5, "clamped");
        let msg = rx.try_recv().expect("should have one message");
        assert_eq!(msg.params.progress, 0);
    }

    #[test]
    fn test_step_helper() {
        let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("s"));
        reporter.step(3, 10, "processing item 3");
        let msg = rx.try_recv().expect("should have one message");
        assert_eq!(msg.params.progress, 3);
        assert_eq!(msg.params.total, Some(10));
    }

    #[test]
    fn test_complete_helper() {
        let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("c"));
        reporter.complete(10, "done");
        let msg = rx.try_recv().expect("should have one message");
        assert_eq!(msg.params.progress, 10);
        assert_eq!(msg.params.total, Some(10));
    }

    #[test]
    fn test_channel_reporter_dropped_receiver() {
        let (reporter, rx) = ChannelProgressReporter::new(ProgressToken::from("drop"));
        drop(rx);
        // Should not panic — sends are best-effort.
        reporter.report(1, Some(10), Some("after drop".to_string()));
    }

    #[test]
    fn test_progress_token_display() {
        assert_eq!(ProgressToken::String("abc".to_string()).to_string(), "abc");
        assert_eq!(ProgressToken::Integer(42).to_string(), "42");
    }
}
