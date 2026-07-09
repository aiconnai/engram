use serde::{Deserialize, Serialize};

// =============================================================================
// DDL
// =============================================================================

/// SQL that creates the `graph_conflicts` table.
///
/// Safe to run on an existing database — uses `IF NOT EXISTS`.
pub const CREATE_CONFLICTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS graph_conflicts (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    conflict_type       TEXT    NOT NULL,
    edge_ids            TEXT    NOT NULL DEFAULT '[]',
    description         TEXT    NOT NULL,
    severity            TEXT    NOT NULL,
    resolved_at         TEXT,
    resolution_strategy TEXT,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_graph_conflicts_type     ON graph_conflicts(conflict_type);
CREATE INDEX IF NOT EXISTS idx_graph_conflicts_severity ON graph_conflicts(severity);
CREATE INDEX IF NOT EXISTS idx_graph_conflicts_resolved ON graph_conflicts(resolved_at);
"#;

// =============================================================================
// Types
// =============================================================================

/// The category of graph conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Two edges between the same pair of nodes carry contradicting relation
    /// types (e.g. "supports" AND "contradicts" for the same A→B pair).
    DirectContradiction,
    /// Two or more edges for the same entity pair have overlapping validity
    /// periods, indicating a temporal inconsistency.
    TemporalInconsistency,
    /// A cycle exists in the directed edge graph (A→B→C→A).
    CyclicDependency,
    /// An edge references a `from_id` or `to_id` that does not exist in the
    /// `memories` table.
    OrphanedReference,
}

impl ConflictType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ConflictType::DirectContradiction => "direct_contradiction",
            ConflictType::TemporalInconsistency => "temporal_inconsistency",
            ConflictType::CyclicDependency => "cyclic_dependency",
            ConflictType::OrphanedReference => "orphaned_reference",
        }
    }

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "direct_contradiction" => Some(ConflictType::DirectContradiction),
            "temporal_inconsistency" => Some(ConflictType::TemporalInconsistency),
            "cyclic_dependency" => Some(ConflictType::CyclicDependency),
            "orphaned_reference" => Some(ConflictType::OrphanedReference),
            _ => None,
        }
    }
}

/// Severity level of a detected conflict.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Severity::Low),
            "medium" => Some(Severity::Medium),
            "high" => Some(Severity::High),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
}

/// A detected conflict in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Unique row identifier (`0` for unsaved conflicts).
    pub id: i64,
    /// Category of conflict.
    pub conflict_type: ConflictType,
    /// IDs of the edges involved in this conflict.
    pub edge_ids: Vec<i64>,
    /// Human-readable description of the conflict.
    pub description: String,
    /// How severe this conflict is.
    pub severity: Severity,
    /// When the conflict was resolved (`None` = unresolved).
    pub resolved_at: Option<String>,
    /// Which strategy was used to resolve this conflict.
    pub resolution_strategy: Option<ResolutionStrategy>,
}

/// Strategy to apply when resolving a conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStrategy {
    /// Remove all but the most recently created edge.
    KeepNewer,
    /// Remove all but the edge with the highest confidence / importance proxy.
    KeepHigherConfidence,
    /// Merge edge metadata into a single edge.
    Merge,
    /// Mark the conflict resolved without modifying any edges.
    Manual,
}

impl ResolutionStrategy {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ResolutionStrategy::KeepNewer => "keep_newer",
            ResolutionStrategy::KeepHigherConfidence => "keep_higher_confidence",
            ResolutionStrategy::Merge => "merge",
            ResolutionStrategy::Manual => "manual",
        }
    }

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "keep_newer" => Some(ResolutionStrategy::KeepNewer),
            "keep_higher_confidence" => Some(ResolutionStrategy::KeepHigherConfidence),
            "merge" => Some(ResolutionStrategy::Merge),
            "manual" => Some(ResolutionStrategy::Manual),
            _ => None,
        }
    }
}

/// Outcome of resolving a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    /// The conflict that was resolved.
    pub conflict_id: i64,
    /// Strategy that was applied.
    pub strategy: ResolutionStrategy,
    /// Edge IDs that were deleted during resolution.
    pub edges_removed: Vec<i64>,
    /// Edge IDs that were kept during resolution.
    pub edges_kept: Vec<i64>,
}

// Pairs of relation types that are considered direct contradictions.
pub(crate) const CONTRADICTING_PAIRS: &[(&str, &str)] = &[
    ("supports", "contradicts"),
    ("agrees_with", "disagrees_with"),
    ("confirms", "refutes"),
    ("approves", "rejects"),
    ("enables", "prevents"),
    ("causes", "prevents"),
];
