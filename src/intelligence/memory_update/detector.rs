use rusqlite::Connection;

use crate::error::Result;

use super::helpers::{
    contains_old_year, extract_keywords, fetch_workspace_memories, keyword_overlap,
    shared_entity_count,
};
use super::{ConflictType, UpdateAction, UpdateCandidate};

// Detection engine
// =============================================================================

/// Confidence threshold below which candidates are discarded.
pub(super) const MIN_CONFIDENCE: f32 = 0.3;

/// Negation / contradiction signal words.
static NEGATION_WORDS: &[&str] = &[
    "not",
    "no longer",
    "never",
    "incorrect",
    "wrong",
    "false",
    "untrue",
    "doesn't",
    "don't",
    "isn't",
    "aren't",
    "wasn't",
    "weren't",
];

/// Explicit correction signal words.
static CORRECTION_WORDS: &[&str] = &[
    "actually",
    "correction",
    "update",
    "correcting",
    "in fact",
    "to clarify",
    "clarification",
    "erratum",
    "revised",
];

/// Temporal "now" markers that suggest the new content supersedes older info.
static NOW_WORDS: &[&str] = &[
    "now",
    "currently",
    "today",
    "as of",
    "at present",
    "present",
    "latest",
    "recent",
];

/// Core update-detection engine.
pub struct UpdateDetector;

impl UpdateDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect update candidates for `new_content` against memories in `workspace`.
    ///
    /// Fetches at most `MAX_RECENT_MEMORIES` memories from the workspace and
    /// computes a confidence score for each one. Returns candidates whose
    /// confidence exceeds `MIN_CONFIDENCE`, sorted descending.
    pub fn detect_updates(
        &self,
        conn: &Connection,
        new_content: &str,
        workspace: &str,
    ) -> Result<Vec<UpdateCandidate>> {
        if new_content.trim().is_empty() || workspace.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Fetch recent memories from the workspace.
        let memories = fetch_workspace_memories(conn, workspace)?;
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        let new_lower = new_content.to_lowercase();
        let new_keywords = extract_keywords(&new_lower);

        let mut candidates: Vec<UpdateCandidate> = Vec::new();

        for (id, content, memory_type, tags) in &memories {
            let existing_lower = content.to_lowercase();
            let existing_keywords = extract_keywords(&existing_lower);

            let overlap = keyword_overlap(&new_keywords, &existing_keywords);
            if overlap == 0.0 {
                // No shared vocabulary — skip entirely.
                continue;
            }

            // Try each conflict class in priority order.
            // The first one that fires wins.
            if let Some(cand) = detect_correction(&new_lower, &existing_lower, *id, overlap) {
                candidates.push(cand);
            } else if let Some(cand) =
                detect_contradiction(&new_lower, &existing_lower, *id, overlap)
            {
                candidates.push(cand);
            } else if let Some(cand) =
                detect_obsolescence(&new_lower, &existing_lower, *id, overlap)
            {
                candidates.push(cand);
            } else if let Some(cand) =
                detect_supplement(&new_lower, &existing_lower, *id, overlap, memory_type, tags)
            {
                candidates.push(cand);
            }
        }

        // Sort by confidence descending, then by id ascending for determinism.
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.existing_id.cmp(&b.existing_id))
        });

        Ok(candidates)
    }
}

impl Default for UpdateDetector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Conflict classifiers
// =============================================================================

fn detect_contradiction(
    new_lower: &str,
    existing_lower: &str,
    id: i64,
    overlap: f32,
) -> Option<UpdateCandidate> {
    if overlap < 0.15 {
        return None;
    }

    let has_negation = NEGATION_WORDS.iter().any(|w| new_lower.contains(w));

    if !has_negation {
        return None;
    }

    // Both texts must share some entity-like tokens.
    let shared = shared_entity_count(new_lower, existing_lower);
    if shared == 0 {
        return None;
    }

    let confidence = (overlap * 0.5 + 0.3).min(1.0);
    if confidence < MIN_CONFIDENCE {
        return None;
    }

    Some(UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Contradiction,
        confidence,
        suggested_action: UpdateAction::Flag,
        reason: format!(
            "New content contains negation signals ('not', 'no longer', etc.) \
             and shares {} entity tokens with the existing memory (keyword overlap {:.0}%).",
            shared,
            overlap * 100.0
        ),
    })
}

fn detect_correction(
    new_lower: &str,
    existing_lower: &str,
    id: i64,
    overlap: f32,
) -> Option<UpdateCandidate> {
    if overlap < 0.10 {
        return None;
    }

    let has_correction = CORRECTION_WORDS.iter().any(|w| new_lower.contains(w));

    if !has_correction {
        return None;
    }

    let _ = existing_lower; // kept for API symmetry

    let confidence = (overlap * 0.6 + 0.35).min(1.0);
    if confidence < MIN_CONFIDENCE {
        return None;
    }

    Some(UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Correction,
        confidence,
        suggested_action: UpdateAction::Replace,
        reason: format!(
            "New content starts with an explicit correction signal ('actually', \
             'correction', etc.) and overlaps with the existing memory at {:.0}%.",
            overlap * 100.0
        ),
    })
}

fn detect_obsolescence(
    new_lower: &str,
    existing_lower: &str,
    id: i64,
    overlap: f32,
) -> Option<UpdateCandidate> {
    if overlap < 0.10 {
        return None;
    }

    let existing_has_old_date = contains_old_year(existing_lower);
    let new_has_now = NOW_WORDS.iter().any(|w| new_lower.contains(w));

    if !(existing_has_old_date && new_has_now) {
        return None;
    }

    let confidence = (overlap * 0.5 + 0.25).min(1.0);
    if confidence < MIN_CONFIDENCE {
        return None;
    }

    Some(UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Obsolescence,
        confidence,
        suggested_action: UpdateAction::Archive,
        reason: format!(
            "Existing memory references old dates while the new content uses \
             temporal markers ('now', 'currently', etc.) at {:.0}% keyword overlap.",
            overlap * 100.0
        ),
    })
}

fn detect_supplement(
    new_lower: &str,
    existing_lower: &str,
    id: i64,
    overlap: f32,
    _memory_type: &str,
    _tags: &[String],
) -> Option<UpdateCandidate> {
    if overlap < 0.20 {
        return None;
    }

    // No negation or correction signals — pure additive information.
    let has_negation = NEGATION_WORDS.iter().any(|w| new_lower.contains(w));
    let has_correction = CORRECTION_WORDS.iter().any(|w| new_lower.contains(w));
    if has_negation || has_correction {
        return None;
    }

    // New content should have tokens not present in existing content.
    let new_keywords = extract_keywords(new_lower);
    let existing_keywords = extract_keywords(existing_lower);
    let new_unique: usize = new_keywords
        .iter()
        .filter(|k| !existing_keywords.contains(*k))
        .count();

    if new_unique == 0 {
        return None;
    }

    // Supplement confidence: base 0.15 so even moderate overlap (0.25+) clears the 0.3 threshold.
    let confidence = (overlap * 0.6 + 0.15).min(1.0);
    if confidence < MIN_CONFIDENCE {
        return None;
    }

    Some(UpdateCandidate {
        existing_id: id,
        conflict_type: ConflictType::Supplement,
        confidence,
        suggested_action: UpdateAction::Merge,
        reason: format!(
            "New content shares {:.0}% keywords with the existing memory and adds \
             {} new unique tokens — supplementary information detected.",
            overlap * 100.0,
            new_unique
        ),
    })
}

// =============================================================================
