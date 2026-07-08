//! Historical Memory Update Detection — RML-1213
//!
//! A-Mem-inspired automatic memory update detection. When new information
//! contradicts or supplements existing memories, this module detects the
//! relationship and suggests an appropriate action.
//!
//! ## How it works
//!
//! 1. Fetch recent memories from the target workspace.
//! 2. For each existing memory, compute keyword overlap and entity matching
//!    with the new content.
//! 3. Classify the relationship: Contradiction, Supplement, Correction,
//!    or Obsolescence.
//! 4. Return `UpdateCandidate` structs for every pair whose confidence
//!    exceeds the threshold (0.3).
//! 5. The caller may then call `apply_update` to commit a chosen action and
//!    record it in the `update_log` table.
//!
//! ## Invariants
//!
//! - Detection never panics on any input.
//! - Empty workspace returns an empty candidate list.
//! - `apply_update` always writes one row to `update_log`.
//! - Confidence scores are in the range [0.0, 1.0].

mod apply;
mod detector;
mod helpers;
mod log;
#[cfg(test)]
mod tests;
mod types;

pub use apply::apply_update;
pub use detector::UpdateDetector;
pub use log::{create_update_log, list_update_logs, CREATE_UPDATE_LOG_TABLE};
pub use types::{ConflictType, UpdateAction, UpdateCandidate, UpdateLogEntry, UpdateResult};
