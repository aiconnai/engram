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
pub mod concept_clustering;
pub mod conflicts;
#[cfg(feature = "duckdb-graph")]
pub mod duckdb_graph;
pub mod link_prediction;
pub mod temporal;
pub mod triplets;

mod communities;
mod export;
mod filter;
mod render;
mod stats;
mod types;

pub use communities::GraphCluster;
pub use concept_clustering::{cluster_concepts, ConceptCluster, ConceptClusterOptions};
pub use filter::GraphFilter;
pub use link_prediction::{predict_links, PredictLinksOptions, PredictLinksResult, PredictedLink};
pub use stats::{CentralityScores, GraphStats};
pub use types::{GraphEdge, GraphNode, KnowledgeGraph};

#[cfg(test)]
mod tests;
