//! Process-wide counters for the few critical failure paths (task O1).
//!
//! Labels are fixed enums, never ids, tenants, workspaces, tool names, queries
//! or error text: every series has bounded cardinality by construction. The
//! counters only say *how often* and *which class*; the correlated log line
//! (see [`super::operation`]) says *where*.
//!
//! Counters start at zero on every process start. A zero therefore means "none
//! seen since this process started", never "none happened": the snapshot
//! carries `uptime_seconds` and an explicit `not_instrumented` list so a
//! consumer cannot mistake an absent series for a healthy one.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use serde::Serialize;

/// Why an authorization check refused a call (see
/// `docs/security/workspace-operation-matrix.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDeniedReason {
    /// The permission mode (env, per-call override or principal) is below the tool's requirement.
    ModeInsufficient,
    /// A claimed workspace is outside the principal's allowed workspaces.
    WorkspaceNotAllowed,
    /// A restricted principal asked for `global: true`.
    GlobalScopeRequested,
    /// C1 conservative denial: the tool cannot honor a workspace claim.
    ToolNotWorkspaceScoped,
    /// Anonymous principal without any workspace claim.
    WorkspaceClaimMissing,
    /// Hierarchical scope grant missing for the calling agent.
    ScopeGrantMissing,
    /// A referenced memory id is foreign or missing (indistinguishable by design).
    ForeignOrMissingMemory,
    /// The authorization lookup itself failed; the call was denied (fail closed).
    AuthorizationCheckFailed,
    /// `ENGRAM_PERMISSION_MODE` holds an invalid value.
    InvalidModeConfig,
}

impl PermissionDeniedReason {
    pub const ALL: [Self; 9] = [
        Self::ModeInsufficient,
        Self::WorkspaceNotAllowed,
        Self::GlobalScopeRequested,
        Self::ToolNotWorkspaceScoped,
        Self::WorkspaceClaimMissing,
        Self::ScopeGrantMissing,
        Self::ForeignOrMissingMemory,
        Self::AuthorizationCheckFailed,
        Self::InvalidModeConfig,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ModeInsufficient => "mode_insufficient",
            Self::WorkspaceNotAllowed => "workspace_not_allowed",
            Self::GlobalScopeRequested => "global_scope_requested",
            Self::ToolNotWorkspaceScoped => "tool_not_workspace_scoped",
            Self::WorkspaceClaimMissing => "workspace_claim_missing",
            Self::ScopeGrantMissing => "scope_grant_missing",
            Self::ForeignOrMissingMemory => "foreign_or_missing_memory",
            Self::AuthorizationCheckFailed => "authorization_check_failed",
            Self::InvalidModeConfig => "invalid_mode_config",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|r| *r == self).unwrap_or(0)
    }
}

/// Result of a `replication_recover` call.
///
/// `Rejected` is a caller input error (missing/invalid path, an unsupported
/// option): the recovery engine never ran, so it is neither an attempt nor a
/// failure. Only `Succeeded` and `Failed` are recovery attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryOutcome {
    Succeeded,
    Failed,
    Rejected,
}

impl RecoveryOutcome {
    pub const ALL: [Self; 3] = [Self::Succeeded, Self::Failed, Self::Rejected];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Succeeded => 0,
            Self::Failed => 1,
            Self::Rejected => 2,
        }
    }
}

/// External provider call whose failures are counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCall {
    Embedding,
}

impl ProviderCall {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embedding => "embedding",
        }
    }
}

/// Critical-path counters. Use [`counters`] for the process-wide instance.
pub struct CriticalCounters {
    started: Instant,
    permission_denied: [AtomicU64; PermissionDeniedReason::ALL.len()],
    sqlite_busy: AtomicU64,
    provider_calls: AtomicU64,
    provider_timeout: AtomicU64,
    provider_failure: AtomicU64,
    authorization_checks: AtomicU64,
    storage_operations: AtomicU64,
    recovery: [AtomicU64; RecoveryOutcome::ALL.len()],
}

impl CriticalCounters {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            permission_denied: std::array::from_fn(|_| AtomicU64::new(0)),
            sqlite_busy: AtomicU64::new(0),
            provider_calls: AtomicU64::new(0),
            provider_timeout: AtomicU64::new(0),
            provider_failure: AtomicU64::new(0),
            authorization_checks: AtomicU64::new(0),
            storage_operations: AtomicU64::new(0),
            recovery: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    pub fn record_permission_denied(&self, reason: PermissionDeniedReason) {
        self.permission_denied[reason.index()].fetch_add(1, Ordering::Relaxed);
    }

    /// Denominator for the permission-denied rule: every authorization check.
    pub fn record_authorization_check(&self) {
        self.authorization_checks.fetch_add(1, Ordering::Relaxed);
    }

    /// Denominator for the SQLITE_BUSY rule: every `Storage` connection/transaction use.
    pub fn record_storage_operation(&self) {
        self.storage_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Denominator for the provider rules: one per provider call attempted.
    pub fn record_provider_call(&self, _call: ProviderCall) {
        self.provider_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_sqlite_busy(&self) {
        self.sqlite_busy.fetch_add(1, Ordering::Relaxed);
    }

    /// Count a failed provider call; `timed_out` selects the timeout series.
    pub fn record_provider_failure(&self, _call: ProviderCall, timed_out: bool) {
        let series = if timed_out {
            &self.provider_timeout
        } else {
            &self.provider_failure
        };
        series.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_recovery(&self, outcome: RecoveryOutcome) {
        self.recovery[outcome.index()].fetch_add(1, Ordering::Relaxed);
    }

    pub fn permission_denied_count(&self, reason: PermissionDeniedReason) -> u64 {
        self.permission_denied[reason.index()].load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> CriticalCountersSnapshot {
        let load = |a: &AtomicU64| a.load(Ordering::Relaxed);
        CriticalCountersSnapshot {
            uptime_seconds: self.started.elapsed().as_secs(),
            permission_denied_total: PermissionDeniedReason::ALL
                .iter()
                .map(|r| (r.as_str(), load(&self.permission_denied[r.index()])))
                .collect(),
            authorization_checks_total: load(&self.authorization_checks),
            storage_operations_total: load(&self.storage_operations),
            sqlite_busy_total: load(&self.sqlite_busy),
            provider_calls_total: BTreeMap::from([(
                ProviderCall::Embedding.as_str(),
                load(&self.provider_calls),
            )]),
            provider_timeout_total: BTreeMap::from([(
                ProviderCall::Embedding.as_str(),
                load(&self.provider_timeout),
            )]),
            provider_failure_total: BTreeMap::from([(
                ProviderCall::Embedding.as_str(),
                load(&self.provider_failure),
            )]),
            recovery_total: RecoveryOutcome::ALL
                .iter()
                .map(|o| (o.as_str(), load(&self.recovery[o.index()])))
                .collect(),
            not_instrumented: NOT_INSTRUMENTED,
        }
    }
}

impl Default for CriticalCounters {
    fn default() -> Self {
        Self::new()
    }
}

/// Signals a consumer might expect but this process does not export. Listed
/// so that an absent series is never read as "zero errors".
pub const NOT_INSTRUMENTED: &[&str] = &[
    "gate_mismatch (harness gates are Python and outside this process)",
    "sqlite_busy_handled_internally (busy retries inside storage/connection.rs are not counted)",
    "provider_timeout_non_embedding (vision/LLM/multimodal providers)",
    "embedding_failures_on_read_paths (query-embedding errors are swallowed by the search handlers)",
    "http_unknown_routes (404/405 never reach a handler)",
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CriticalCountersSnapshot {
    pub uptime_seconds: u64,
    pub permission_denied_total: BTreeMap<&'static str, u64>,
    /// Denominator of `permission_denied_total`.
    pub authorization_checks_total: u64,
    /// Denominator of `sqlite_busy_total`.
    pub storage_operations_total: u64,
    pub sqlite_busy_total: u64,
    /// Denominator of the provider timeout/failure series.
    pub provider_calls_total: BTreeMap<&'static str, u64>,
    pub provider_timeout_total: BTreeMap<&'static str, u64>,
    pub provider_failure_total: BTreeMap<&'static str, u64>,
    pub recovery_total: BTreeMap<&'static str, u64>,
    pub not_instrumented: &'static [&'static str],
}

static GLOBAL: OnceLock<CriticalCounters> = OnceLock::new();

/// The process-wide counters.
pub fn counters() -> &'static CriticalCounters {
    GLOBAL.get_or_init(CriticalCounters::new)
}

pub fn record_permission_denied(reason: PermissionDeniedReason) {
    counters().record_permission_denied(reason);
}

pub fn record_sqlite_busy() {
    counters().record_sqlite_busy();
}

pub fn record_provider_failure(call: ProviderCall, err: &crate::error::EngramError) {
    counters().record_provider_failure(call, super::redact::is_timeout(err));
}

pub fn record_authorization_check() {
    counters().record_authorization_check();
}

pub fn record_storage_operation() {
    counters().record_storage_operation();
}

pub fn record_provider_call(call: ProviderCall) {
    counters().record_provider_call(call);
}

pub fn record_recovery(outcome: RecoveryOutcome) {
    counters().record_recovery(outcome);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_have_bounded_labels_and_start_at_zero() {
        let c = CriticalCounters::new();
        let zero = c.snapshot();
        assert!(zero.permission_denied_total.values().all(|v| *v == 0));
        assert_eq!(
            zero.permission_denied_total.len(),
            PermissionDeniedReason::ALL.len()
        );
        assert_eq!(zero.sqlite_busy_total, 0);

        c.record_permission_denied(PermissionDeniedReason::ToolNotWorkspaceScoped);
        c.record_permission_denied(PermissionDeniedReason::ToolNotWorkspaceScoped);
        c.record_sqlite_busy();
        c.record_provider_failure(ProviderCall::Embedding, true);
        c.record_provider_failure(ProviderCall::Embedding, false);
        c.record_recovery(RecoveryOutcome::Failed);

        let snap = c.snapshot();
        assert_eq!(snap.permission_denied_total["tool_not_workspace_scoped"], 2);
        assert_eq!(snap.sqlite_busy_total, 1);
        assert_eq!(snap.provider_timeout_total["embedding"], 1);
        assert_eq!(snap.provider_failure_total["embedding"], 1);
        assert_eq!(snap.recovery_total["failed"], 1);
        assert_eq!(snap.recovery_total["succeeded"], 0);
        assert_eq!(snap.recovery_total["rejected"], 0);
    }

    #[test]
    fn denominators_are_exported_and_start_at_zero() {
        let c = CriticalCounters::new();
        let zero = c.snapshot();
        assert_eq!(zero.authorization_checks_total, 0);
        assert_eq!(zero.storage_operations_total, 0);
        assert_eq!(zero.provider_calls_total["embedding"], 0);
        c.record_authorization_check();
        c.record_storage_operation();
        c.record_provider_call(ProviderCall::Embedding);
        let snap = c.snapshot();
        assert_eq!(snap.authorization_checks_total, 1);
        assert_eq!(snap.storage_operations_total, 1);
        assert_eq!(snap.provider_calls_total["embedding"], 1);
    }

    #[test]
    fn snapshot_says_what_is_not_instrumented() {
        let snap = CriticalCounters::new().snapshot();
        assert!(snap
            .not_instrumented
            .iter()
            .any(|entry| entry.starts_with("gate_mismatch")));
    }
}
