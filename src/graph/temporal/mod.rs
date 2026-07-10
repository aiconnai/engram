//! Temporal knowledge graph — edges with validity periods.
//!
//! Provides bi-temporal edge tracking: each edge carries a `valid_from` /
//! `valid_to` validity interval. Adding a new edge for the same
//! `(from_id, to_id, relation)` triple automatically closes the previous open
//! interval so the graph stays consistent.
//!
//! This module is a thin facade over responsibility submodules:
//! - [`types`] — DDL, edge/diff types, row mapping
//! - [`edges`] — edge mutations (add / invalidate)
//! - [`queries`] — point-in-time queries (snapshot, timeline)
//! - [`analysis`] — contradiction detection and graph diffing

mod analysis;
mod edges;
mod queries;
mod types;

#[cfg(test)]
mod tests;

pub use analysis::{detect_contradictions, diff};
pub use edges::{add_edge, invalidate_edge};
pub use queries::{edges_for_memory_at, relationship_timeline, snapshot_at};
pub use types::{GraphDiff, TemporalEdge, CREATE_TEMPORAL_EDGES_TABLE};
