use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{EngramError, Result};
use crate::types::MemoryId;

// Types and Enums
// ============================================================================

/// Type of conflict between memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Direct contradiction in facts
    Contradiction,
    /// Outdated information
    Staleness,
    /// Duplicate content
    Duplicate,
    /// Semantic overlap
    SemanticOverlap,
    /// Inconsistent metadata
    MetadataInconsistency,
}

impl ConflictType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConflictType::Contradiction => "contradiction",
            ConflictType::Staleness => "staleness",
            ConflictType::Duplicate => "duplicate",
            ConflictType::SemanticOverlap => "semantic_overlap",
            ConflictType::MetadataInconsistency => "metadata_inconsistency",
        }
    }
}

impl std::str::FromStr for ConflictType {
    type Err = EngramError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "contradiction" => Ok(ConflictType::Contradiction),
            "staleness" => Ok(ConflictType::Staleness),
            "duplicate" => Ok(ConflictType::Duplicate),
            "semantic_overlap" => Ok(ConflictType::SemanticOverlap),
            "metadata_inconsistency" => Ok(ConflictType::MetadataInconsistency),
            _ => Err(EngramError::InvalidInput(format!(
                "Unknown conflict type: {}",
                s
            ))),
        }
    }
}

/// Severity of a conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ConflictSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConflictSeverity::Low => "low",
            ConflictSeverity::Medium => "medium",
            ConflictSeverity::High => "high",
            ConflictSeverity::Critical => "critical",
        }
    }
}

impl std::str::FromStr for ConflictSeverity {
    type Err = EngramError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" => Ok(ConflictSeverity::Low),
            "medium" => Ok(ConflictSeverity::Medium),
            "high" => Ok(ConflictSeverity::High),
            "critical" => Ok(ConflictSeverity::Critical),
            _ => Ok(ConflictSeverity::Medium),
        }
    }
}

/// Resolution type for conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    /// Keep memory A, archive B
    KeepA,
    /// Keep memory B, archive A
    KeepB,
    /// Merge both into new memory
    Merge,
    /// Keep both as-is (mark as reviewed)
    KeepBoth,
    /// Delete both
    DeleteBoth,
    /// Mark as false positive
    FalsePositive,
}

impl ResolutionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResolutionType::KeepA => "keep_a",
            ResolutionType::KeepB => "keep_b",
            ResolutionType::Merge => "merge",
            ResolutionType::KeepBoth => "keep_both",
            ResolutionType::DeleteBoth => "delete_both",
            ResolutionType::FalsePositive => "false_positive",
        }
    }
}

/// Validation status for memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Unverified,
    Verified,
    Disputed,
    Stale,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationStatus::Unverified => "unverified",
            ValidationStatus::Verified => "verified",
            ValidationStatus::Disputed => "disputed",
            ValidationStatus::Stale => "stale",
        }
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// A detected conflict between two memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConflict {
    pub id: i64,
    pub memory_a_id: MemoryId,
    pub memory_b_id: MemoryId,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub description: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_type: Option<ResolutionType>,
    pub resolution_notes: Option<String>,
    pub auto_detected: bool,
}

/// A duplicate candidate pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    pub id: i64,
    pub memory_a_id: MemoryId,
    pub memory_b_id: MemoryId,
    pub similarity_score: f32,
    pub similarity_type: String,
    pub detected_at: DateTime<Utc>,
    pub status: String,
}

/// Enhanced quality score with all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQualityScore {
    pub overall: f32,
    pub grade: char,
    pub clarity: f32,
    pub completeness: f32,
    pub freshness: f32,
    pub consistency: f32,
    pub source_trust: f32,
    pub suggestions: Vec<QualitySuggestion>,
    pub calculated_at: DateTime<Utc>,
}

/// A quality improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySuggestion {
    pub category: String,
    pub priority: String,
    pub message: String,
    pub action: Option<String>,
}

/// Source trust score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTrustScore {
    pub source_type: String,
    pub source_identifier: Option<String>,
    pub trust_score: f32,
    pub verification_count: i32,
    pub notes: Option<String>,
}

/// Quality report for a workspace or set of memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub total_memories: i64,
    pub average_quality: f32,
    pub quality_distribution: HashMap<char, i64>,
    pub top_issues: Vec<QualityIssue>,
    pub conflicts_count: i64,
    pub duplicates_count: i64,
    pub suggestions_summary: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// A quality issue in the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_type: String,
    pub count: i64,
    pub severity: String,
    pub description: String,
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for context quality analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQualityConfig {
    /// Weight for clarity in quality score
    pub clarity_weight: f32,
    /// Weight for completeness
    pub completeness_weight: f32,
    /// Weight for freshness
    pub freshness_weight: f32,
    /// Weight for consistency
    pub consistency_weight: f32,
    /// Weight for source trust
    pub source_trust_weight: f32,
    /// Threshold for near-duplicate detection (0-1)
    pub duplicate_threshold: f32,
    /// Threshold for semantic similarity (0-1)
    pub semantic_threshold: f32,
    /// Days until memory is considered stale
    pub staleness_days: i64,
    /// Minimum content length for quality
    pub min_content_length: usize,
    /// Ideal content length
    pub ideal_content_length: usize,
}

impl Default for ContextQualityConfig {
    fn default() -> Self {
        Self {
            clarity_weight: 0.25,
            completeness_weight: 0.20,
            freshness_weight: 0.20,
            consistency_weight: 0.20,
            source_trust_weight: 0.15,
            duplicate_threshold: 0.85,
            semantic_threshold: 0.80,
            staleness_days: 90,
            min_content_length: 20,
            ideal_content_length: 200,
        }
    }
}

// ============================================================================
