//! Deterministic operational-context reducers.
//!
//! Reducers in this module summarize noisy command output into a stable
//! Engram-native contract. They intentionally do not depend on external reducer
//! toolkits and avoid raw hunks/log bodies by default.

mod cargo_clippy;
mod cargo_test;
mod generic_error_log;
mod git_diff_stat;
mod output;
mod redaction;
mod rg_matches;
mod util;

pub use cargo_clippy::{
    parse_cargo_clippy_diagnostics, reduce_cargo_clippy, reduce_cargo_clippy_with_redactor,
    ActionableSpan, CargoClippyDiagnostic, CargoClippyDiagnosticGroup,
};
pub use cargo_test::{
    parse_cargo_test_output, reduce_cargo_test, reduce_cargo_test_with_redactor, CargoTestPanic,
    CargoTestReduction,
};
pub use generic_error_log::{reduce_generic_error_log, reduce_generic_error_log_with_redactor};
pub use git_diff_stat::{
    high_risk_file, parse_git_diff_stat, reduce_git_diff_stat, reduce_git_diff_stat_with_redactor,
    DiffStatFile,
};
pub use output::{EvidenceItem, ObservedFact, RedactionRecord, ReducerOutput};
pub use redaction::{NoopRedactor, RedactionResult, Redactor};
pub use rg_matches::{
    parse_rg_match_line, reduce_rg_matches, reduce_rg_matches_with_limits,
    reduce_rg_matches_with_redactor, RepresentativeLine, RgFileMatches, RgLineMatch,
};
