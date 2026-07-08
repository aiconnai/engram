use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::Result;
use crate::storage::queries::get_memory;
use crate::types::{Memory, MemoryId};

use super::{ContextQualityConfig, EnhancedQualityScore, QualitySuggestion};

// Enhanced Quality Scoring (ENG-52)
// ============================================================================

/// Calculate enhanced quality score for a memory
pub fn calculate_quality_score(
    conn: &Connection,
    memory_id: MemoryId,
    config: &ContextQualityConfig,
) -> Result<EnhancedQualityScore> {
    let memory = get_memory(conn, memory_id)?;

    let clarity = score_clarity(&memory);
    let completeness = score_completeness(&memory, config);
    let freshness = score_freshness(&memory, config);
    let consistency = score_consistency(conn, memory_id)?;
    let source_trust = get_source_trust_for_memory(conn, &memory)?;

    let overall = clarity * config.clarity_weight
        + completeness * config.completeness_weight
        + freshness * config.freshness_weight
        + consistency * config.consistency_weight
        + source_trust * config.source_trust_weight;

    let grade = match overall {
        s if s >= 0.9 => 'A',
        s if s >= 0.8 => 'B',
        s if s >= 0.7 => 'C',
        s if s >= 0.6 => 'D',
        _ => 'F',
    };

    let suggestions = generate_quality_suggestions(
        &memory,
        clarity,
        completeness,
        freshness,
        consistency,
        source_trust,
    );

    // Record in history
    let now = Utc::now();
    conn.execute(
        r#"
        INSERT INTO quality_history
        (memory_id, quality_score, clarity_score, completeness_score, freshness_score, consistency_score, source_trust_score)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        params![memory_id, overall, clarity, completeness, freshness, consistency, source_trust],
    )?;

    // Update memory quality score
    conn.execute(
        "UPDATE memories SET quality_score = ? WHERE id = ?",
        params![overall, memory_id],
    )?;

    Ok(EnhancedQualityScore {
        overall,
        grade,
        clarity,
        completeness,
        freshness,
        consistency,
        source_trust,
        suggestions,
        calculated_at: now,
    })
}

fn score_clarity(memory: &Memory) -> f32 {
    let content = &memory.content;
    let mut score: f32 = 0.5;

    // Sentence structure
    let sentence_count =
        content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
    if sentence_count > 0 {
        score += 0.15;
    }

    // Word clarity
    let word_count = content.split_whitespace().count();
    if word_count > 0 {
        let avg_word_len: f32 = content
            .split_whitespace()
            .map(|w| w.len() as f32)
            .sum::<f32>()
            / word_count as f32;

        if (3.0..=10.0).contains(&avg_word_len) {
            score += 0.2;
        }
    }

    // Has organization (tags)
    if !memory.tags.is_empty() {
        score += 0.15;
    }

    score.min(1.0)
}

fn score_completeness(memory: &Memory, config: &ContextQualityConfig) -> f32 {
    let len = memory.content.len();

    if len < config.min_content_length {
        return 0.3;
    }

    if len >= config.ideal_content_length {
        return 1.0;
    }

    let range = (config.ideal_content_length - config.min_content_length) as f32;
    let progress = (len - config.min_content_length) as f32;
    0.3 + 0.7 * (progress / range)
}

fn score_freshness(memory: &Memory, config: &ContextQualityConfig) -> f32 {
    let age_days = (Utc::now() - memory.updated_at).num_days() as f32;
    let staleness = config.staleness_days as f32;

    if age_days <= 0.0 {
        1.0
    } else if age_days >= staleness {
        0.2
    } else {
        1.0 - 0.8 * (age_days / staleness)
    }
}

fn score_consistency(conn: &Connection, memory_id: MemoryId) -> Result<f32> {
    // Check for unresolved conflicts
    let conflict_count: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM memory_conflicts
        WHERE (memory_a_id = ? OR memory_b_id = ?) AND resolved_at IS NULL
        "#,
        params![memory_id, memory_id],
        |row| row.get(0),
    )?;

    Ok(match conflict_count {
        0 => 1.0,
        1 => 0.7,
        2 => 0.5,
        _ => 0.3,
    })
}

fn get_source_trust_for_memory(conn: &Connection, memory: &Memory) -> Result<f32> {
    // Determine source type from metadata
    let source_type = memory
        .metadata
        .get("origin")
        .and_then(|v| v.as_str())
        .unwrap_or("user");

    let trust_score: f32 = conn
        .query_row(
            "SELECT trust_score FROM source_trust_scores WHERE source_type = ?",
            params![source_type],
            |row| row.get(0),
        )
        .unwrap_or(0.7);

    Ok(trust_score)
}

fn generate_quality_suggestions(
    memory: &Memory,
    clarity: f32,
    completeness: f32,
    freshness: f32,
    consistency: f32,
    _source_trust: f32,
) -> Vec<QualitySuggestion> {
    let mut suggestions = Vec::new();

    if completeness < 0.5 {
        suggestions.push(QualitySuggestion {
            category: "completeness".to_string(),
            priority: "high".to_string(),
            message: "Add more detail to make this memory more useful".to_string(),
            action: Some("expand".to_string()),
        });
    }

    if clarity < 0.5 {
        suggestions.push(QualitySuggestion {
            category: "clarity".to_string(),
            priority: "medium".to_string(),
            message: "Consider adding structure with clear sentences".to_string(),
            action: Some("restructure".to_string()),
        });
    }

    if memory.tags.is_empty() {
        suggestions.push(QualitySuggestion {
            category: "organization".to_string(),
            priority: "low".to_string(),
            message: "Add tags to improve organization and searchability".to_string(),
            action: Some("add_tags".to_string()),
        });
    }

    if freshness < 0.3 {
        suggestions.push(QualitySuggestion {
            category: "freshness".to_string(),
            priority: "medium".to_string(),
            message: "This memory may be outdated - consider reviewing".to_string(),
            action: Some("review".to_string()),
        });
    }

    if consistency < 0.5 {
        suggestions.push(QualitySuggestion {
            category: "consistency".to_string(),
            priority: "high".to_string(),
            message: "This memory has unresolved conflicts - review and resolve".to_string(),
            action: Some("resolve_conflicts".to_string()),
        });
    }

    suggestions
}

// ============================================================================
