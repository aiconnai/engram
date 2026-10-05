//! The operation record: one structured log event per finished operation.
//!
//! Contract (task O1): `operation` (what ran), `correlation_id` (ties the log
//! line to the HTTP response header and to nearby events), `outcome` (a stable
//! class), `duration_ms`, and `retry_after_secs` when the caller is told to
//! retry. Optional bounded fields: `method`, `tool` (only names from the
//! catalog), `status`, `error_class`. No field ever holds free-form content.

use std::time::Duration;

/// Maximum accepted length of a caller-supplied correlation id.
const MAX_CORRELATION_ID_LEN: usize = 64;

/// Correlation id: server generated, or an accepted caller-supplied
/// `x-request-id` (restricted charset, so it cannot inject log lines).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().simple().to_string())
    }

    /// Accept `raw` only if it is 1..=64 chars of `[A-Za-z0-9._-]`.
    pub fn from_untrusted(raw: &str) -> Option<Self> {
        let ok = !raw.is_empty()
            && raw.len() <= MAX_CORRELATION_ID_LEN
            && raw
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
        ok.then(|| Self(raw.to_string()))
    }

    /// The caller's id when acceptable, otherwise a fresh one.
    pub fn from_header_or_generate(raw: Option<&str>) -> Self {
        raw.and_then(Self::from_untrusted)
            .unwrap_or_else(Self::generate)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One finished operation, ready to log.
#[derive(Debug, Clone)]
pub struct OperationEvent<'a> {
    pub operation: &'static str,
    pub correlation_id: &'a str,
    pub outcome: &'static str,
    pub duration: Duration,
    pub retry_after_secs: Option<u64>,
    pub error_class: Option<&'static str>,
    pub method: Option<&'static str>,
    pub tool: Option<&'static str>,
    pub status: Option<u16>,
    pub route: Option<&'static str>,
    pub notification: Option<bool>,
    /// `true` logs at WARN, `false` at INFO.
    pub is_failure: bool,
}

impl<'a> OperationEvent<'a> {
    pub fn new(
        operation: &'static str,
        correlation_id: &'a str,
        outcome: &'static str,
        duration: Duration,
    ) -> Self {
        Self {
            operation,
            correlation_id,
            outcome,
            duration,
            retry_after_secs: None,
            error_class: None,
            method: None,
            tool: None,
            status: None,
            route: None,
            notification: None,
            is_failure: false,
        }
    }

    pub fn failed(self, is_failure: bool) -> Self {
        Self { is_failure, ..self }
    }

    pub fn retry_after(self, secs: u64) -> Self {
        Self {
            retry_after_secs: Some(secs),
            ..self
        }
    }

    pub fn error_class(self, class: &'static str) -> Self {
        Self {
            error_class: Some(class),
            ..self
        }
    }

    pub fn route(self, route: &'static str) -> Self {
        Self {
            route: Some(route),
            ..self
        }
    }

    pub fn notification(self, notification: bool) -> Self {
        Self {
            notification: Some(notification),
            ..self
        }
    }

    pub fn http(
        self,
        method: Option<&'static str>,
        tool: Option<&'static str>,
        status: u16,
    ) -> Self {
        Self {
            method,
            tool,
            status: Some(status),
            ..self
        }
    }

    /// Emit the event on target `engram::ops`.
    pub fn emit(&self) {
        let duration_ms = u64::try_from(self.duration.as_millis()).unwrap_or(u64::MAX);
        if self.is_failure {
            tracing::warn!(
                target: "engram::ops",
                operation = self.operation,
                correlation_id = self.correlation_id,
                outcome = self.outcome,
                duration_ms,
                retry_after_secs = self.retry_after_secs,
                error_class = self.error_class,
                method = self.method,
                tool = self.tool,
                status = self.status,
                route = self.route,
                notification = self.notification,
                "operation"
            );
        } else {
            tracing::info!(
                target: "engram::ops",
                operation = self.operation,
                correlation_id = self.correlation_id,
                outcome = self.outcome,
                duration_ms,
                retry_after_secs = self.retry_after_secs,
                error_class = self.error_class,
                method = self.method,
                tool = self.tool,
                status = self.status,
                route = self.route,
                notification = self.notification,
                "operation"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::test_capture::{captured, install};

    #[test]
    fn untrusted_correlation_ids_are_validated() {
        assert!(CorrelationId::from_untrusted("req-1.a_B").is_some());
        for bad in [
            "",
            "has space",
            "line\nbreak",
            "semi;colon",
            &"a".repeat(65),
        ] {
            assert!(CorrelationId::from_untrusted(bad).is_none(), "{bad:?}");
        }
        let generated = CorrelationId::from_header_or_generate(Some("bad id"));
        assert_eq!(generated.as_str().len(), 32);
        assert_eq!(
            CorrelationId::from_header_or_generate(Some("abc-1")).as_str(),
            "abc-1"
        );
    }

    #[test]
    fn event_carries_the_contract_fields_and_nothing_else() {
        install();
        let id = CorrelationId::from_untrusted("op-contract-test-1").expect("valid id");
        OperationEvent::new(
            "test.operation",
            id.as_str(),
            "rate_limited",
            Duration::from_millis(7),
        )
        .failed(true)
        .retry_after(1)
        .http(Some("tools/call"), Some("memory_search"), 429)
        .emit();

        let line = captured()
            .lines()
            .find(|l| l.contains("op-contract-test-1"))
            .map(str::to_owned)
            .expect("event line");
        for field in [
            "operation=\"test.operation\"",
            "outcome=\"rate_limited\"",
            "duration_ms=7",
            "retry_after_secs=1",
            "method=\"tools/call\"",
            "tool=\"memory_search\"",
            "status=429",
        ] {
            assert!(line.contains(field), "missing {field} in {line}");
        }
        assert!(line.contains("WARN"));
    }
}
