//! DuckDB-backed TemporalGraph for OLAP read operations (Phase M).
//!
//! Implements the CQRS read side: SQLite handles all writes, DuckDB attaches
//! the same file read-only and provides fast analytical queries. When the
//! optional `duckpgq` extension is available a full property-graph (`MATCH`)
//! query surface is also created; otherwise the module degrades gracefully to
//! plain SQL.

mod lifecycle;
mod queries;
#[cfg(test)]
mod tests;
mod traversal;
mod types;

pub use types::{DuckDbGraphDiff, DuckDbTemporalEdge, PathStep, TemporalGraph};
