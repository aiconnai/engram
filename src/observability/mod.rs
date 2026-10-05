//! Observability without proprietary content (task O1).
//!
//! - [`redact`]: what may reach a log line (error class and bounded labels,
//!   never memory text, queries, credentials, paths or provider bodies).
//! - [`operation`]: the per-operation record (operation, correlation id,
//!   outcome, duration, retry) emitted on target `engram::ops`.
//! - [`mod@counters`]: bounded-cardinality counters for the critical failure
//!   paths (permission denied, SQLITE_BUSY, provider timeout, recovery).
//! - [`alerts`]: the local alert rules that tie a counter or HTTP signal to an
//!   owner, a runbook entry and a rollback, evaluated over an export snapshot.
//!
//! Operator documentation: `docs/OPERATIONS.md` ("Observability and local
//! incident exercise"). Rollback of the extra export: set
//! `ENGRAM_OBSERVABILITY_EXPORT=off`; structured error messages are untouched.

pub mod alerts;
pub mod counters;
pub mod operation;
pub mod redact;
#[cfg(test)]
pub(crate) mod test_capture;

use std::time::Instant;

use serde_json::Value;

pub use counters::{
    counters, record_authorization_check, record_permission_denied, record_provider_call,
    record_provider_failure, record_recovery, record_sqlite_busy, record_storage_operation,
    CriticalCounters, CriticalCountersSnapshot, PermissionDeniedReason, ProviderCall,
    RecoveryOutcome,
};
pub use operation::{CorrelationId, OperationEvent};

/// Environment switch that disables the extra export (`/health` observability
/// block). Counters keep counting; logs and error envelopes are unaffected.
pub const EXPORT_ENV: &str = "ENGRAM_OBSERVABILITY_EXPORT";

/// Whether the extra export is on (default on).
pub fn export_enabled() -> bool {
    export_enabled_from(std::env::var(EXPORT_ENV).ok().as_deref())
}

pub(crate) fn export_enabled_from(raw: Option<&str>) -> bool {
    !matches!(
        raw.map(|v| v.trim().to_ascii_lowercase()).as_deref(),
        Some("0" | "off" | "false" | "no" | "disabled")
    )
}

/// Count and log a finished `replication_recover` call without changing its
/// result. The caller classifies the call: `Rejected` is an input error (the
/// recovery engine never ran) and is not an attempt; only `Succeeded` and
/// `Failed` count as attempts for the alert rule.
pub fn observe_recovery(started: Instant, outcome: RecoveryOutcome, result: Value) -> Value {
    record_recovery(outcome);
    let id = CorrelationId::generate();
    let event = OperationEvent::new(
        "replication_recover",
        id.as_str(),
        outcome.as_str(),
        started.elapsed(),
    );
    match outcome {
        RecoveryOutcome::Succeeded => event.emit(),
        RecoveryOutcome::Failed => event.failed(true).error_class("recovery_failed").emit(),
        RecoveryOutcome::Rejected => event.error_class("invalid_input").emit(),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn export_switch_defaults_on_and_accepts_off_spellings() {
        assert!(export_enabled_from(None));
        assert!(export_enabled_from(Some("on")));
        assert!(export_enabled_from(Some("garbage")));
        for off in ["off", "0", "FALSE", " disabled "] {
            assert!(!export_enabled_from(Some(off)), "{off}");
        }
    }

    #[test]
    fn recovery_observation_counts_without_changing_the_result() {
        let before = counters().snapshot().recovery_total;
        let ok = observe_recovery(
            Instant::now(),
            RecoveryOutcome::Succeeded,
            json!({"frames": 3}),
        );
        let err = observe_recovery(
            Instant::now(),
            RecoveryOutcome::Failed,
            json!({"error": "target exists: /x/y"}),
        );
        let rejected = observe_recovery(
            Instant::now(),
            RecoveryOutcome::Rejected,
            json!({"error": "target_db_path is required"}),
        );
        assert_eq!(ok, json!({"frames": 3}));
        assert_eq!(err["error"], "target exists: /x/y");
        assert_eq!(rejected["error"], "target_db_path is required");
        let after = counters().snapshot().recovery_total;
        assert!(after["succeeded"] > before["succeeded"]);
        assert!(after["failed"] > before["failed"]);
        assert!(after["rejected"] > before["rejected"]);
    }
}
