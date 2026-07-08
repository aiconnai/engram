//! Context Quality Module (Phase 9: ENG-48 to ENG-66)
//!
//! Provides:
//! - Near-duplicate detection (ENG-48)
//! - Semantic deduplication (ENG-49)
//! - Conflict detection (ENG-50)
//! - Contradiction resolution (ENG-51)
//! - Enhanced quality scoring (ENG-52)
//! - Source credibility (ENG-53)
//! - Quality improvement suggestions (ENG-57)

mod conflicts;
mod duplicates;
mod report;
mod scoring;
mod source_trust;
#[cfg(test)]
mod tests;
mod types;

pub use conflicts::{detect_conflicts, get_unresolved_conflicts, resolve_conflict};
pub use duplicates::{
    calculate_text_similarity, find_near_duplicates, find_semantic_duplicates,
    get_pending_duplicates,
};
pub use report::generate_quality_report;
pub use scoring::calculate_quality_score;
pub use source_trust::{get_source_trust, update_source_trust};
pub use types::{
    ConflictSeverity, ConflictType, ContextQualityConfig, DuplicateCandidate, EnhancedQualityScore,
    MemoryConflict, QualityIssue, QualityReport, QualitySuggestion, ResolutionType,
    SourceTrustScore, ValidationStatus,
};

#[cfg(test)]
use duplicates::cosine_similarity;
