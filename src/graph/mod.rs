//! Knowledge graph visualization (RML-894 improvements)
//!
//! Provides:
//! - Interactive graph visualization with vis.js
//! - Graph clustering and community detection
//! - Graph statistics and metrics
//! - Export to multiple formats (HTML, DOT, JSON)
//! - Filtering and traversal utilities
//! - Temporal knowledge graph with validity periods (RML-1235)

pub mod coactivation;
pub mod conflicts;
#[cfg(feature = "duckdb-graph")]
pub mod duckdb_graph;
pub mod temporal;
pub mod triplets;

mod core;
pub use core::*;
