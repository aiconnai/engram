//! Graph Conflict Detection & Resolution (RML-1217)
//!
//! Inspired by Mem0's approach to managing conflicting knowledge in graphs.
//! Provides:
//! - Detection of four conflict types: direct contradictions, temporal inconsistencies,
//!   cyclic dependencies, and orphaned references.
//! - Resolution strategies: keep newer, keep higher confidence, merge, or manual.
//! - Persistence of detected conflicts in the `graph_conflicts` table.

mod detector;
mod helpers;
mod resolver;
mod types;

pub use detector::ConflictDetector;
pub use resolver::ConflictResolver;
pub use types::{
    Conflict, ConflictType, ResolutionResult, ResolutionStrategy, Severity, CREATE_CONFLICTS_TABLE,
};

#[cfg(test)]
mod tests;
