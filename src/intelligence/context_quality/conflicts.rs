use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::error::Result;
use crate::storage::queries::get_memory;
use crate::types::MemoryId;

use super::{
    calculate_text_similarity, ConflictSeverity, ConflictType, ContextQualityConfig,
    MemoryConflict, ResolutionType,
};

// Conflict Detection (ENG-50)
// ============================================================================

/// Detect conflicts for a memory against existing memories
pub fn detect_conflicts(
    conn: &Connection,
    memory_id: MemoryId,
    config: &ContextQualityConfig,
) -> Result<Vec<MemoryConflict>> {
    let memory = get_memory(conn, memory_id)?;
    let mut conflicts = Vec::new();

    // Find memories with similar tags or content that might conflict
    let mut stmt = conn.prepare(
        r#"
        SELECT id, content, tags, updated_at
        FROM memories
        WHERE id != ? AND deleted_at IS NULL
        AND (
            -- Same workspace
            workspace = (SELECT workspace FROM memories WHERE id = ?)
            -- Or overlapping tags
            OR EXISTS (
                SELECT 1 FROM json_each(tags) t1
                WHERE t1.value IN (SELECT value FROM json_each((SELECT tags FROM memories WHERE id = ?)))
            )
        )
        LIMIT 100
        "#,
    )?;

    let candidates: Vec<(i64, String, String, String)> = stmt
        .query_map(params![memory_id, memory_id, memory_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (other_id, other_content, _other_tags, other_updated) in candidates {
        // Check for staleness conflict
        let memory_date: DateTime<Utc> = memory.updated_at;
        let other_date: DateTime<Utc> = other_updated.parse().unwrap_or(memory_date);
        let days_diff = (memory_date - other_date).num_days().abs();

        if days_diff > config.staleness_days {
            // Check content similarity to see if they're about the same topic
            let similarity = calculate_text_similarity(&memory.content, &other_content);
            if similarity > 0.3 {
                let conflict = create_conflict(
                    conn,
                    memory_id,
                    other_id,
                    ConflictType::Staleness,
                    ConflictSeverity::Medium,
                    Some(format!(
                        "Memories differ by {} days and have {:.0}% content similarity",
                        days_diff,
                        similarity * 100.0
                    )),
                )?;
                conflicts.push(conflict);
            }
        }

        // Check for duplicate/overlap
        let similarity = calculate_text_similarity(&memory.content, &other_content);
        if similarity >= config.duplicate_threshold {
            let conflict = create_conflict(
                conn,
                memory_id,
                other_id,
                ConflictType::Duplicate,
                ConflictSeverity::High,
                Some(format!("Content similarity: {:.0}%", similarity * 100.0)),
            )?;
            conflicts.push(conflict);
        } else if similarity >= config.semantic_threshold {
            let conflict = create_conflict(
                conn,
                memory_id,
                other_id,
                ConflictType::SemanticOverlap,
                ConflictSeverity::Low,
                Some(format!("Semantic overlap: {:.0}%", similarity * 100.0)),
            )?;
            conflicts.push(conflict);
        }
    }

    Ok(conflicts)
}

/// Create a conflict record
fn create_conflict(
    conn: &Connection,
    memory_a_id: MemoryId,
    memory_b_id: MemoryId,
    conflict_type: ConflictType,
    severity: ConflictSeverity,
    description: Option<String>,
) -> Result<MemoryConflict> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    conn.execute(
        r#"
        INSERT OR IGNORE INTO memory_conflicts
        (memory_a_id, memory_b_id, conflict_type, severity, description, detected_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        params![
            memory_a_id,
            memory_b_id,
            conflict_type.as_str(),
            severity.as_str(),
            description,
            now_str
        ],
    )?;

    let id = conn.last_insert_rowid();

    Ok(MemoryConflict {
        id,
        memory_a_id,
        memory_b_id,
        conflict_type,
        severity,
        description,
        detected_at: now,
        resolved_at: None,
        resolution_type: None,
        resolution_notes: None,
        auto_detected: true,
    })
}

/// Get unresolved conflicts
pub fn get_unresolved_conflicts(conn: &Connection, limit: i64) -> Result<Vec<MemoryConflict>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, memory_a_id, memory_b_id, conflict_type, severity, description,
               detected_at, resolved_at, resolution_type, resolution_notes, auto_detected
        FROM memory_conflicts
        WHERE resolved_at IS NULL
        ORDER BY
            CASE severity
                WHEN 'critical' THEN 1
                WHEN 'high' THEN 2
                WHEN 'medium' THEN 3
                ELSE 4
            END,
            detected_at DESC
        LIMIT ?
        "#,
    )?;

    let conflicts = stmt
        .query_map(params![limit], |row| {
            Ok(MemoryConflict {
                id: row.get(0)?,
                memory_a_id: row.get(1)?,
                memory_b_id: row.get(2)?,
                conflict_type: row
                    .get::<_, String>(3)?
                    .parse()
                    .unwrap_or(ConflictType::Contradiction),
                severity: row
                    .get::<_, String>(4)?
                    .parse()
                    .unwrap_or(ConflictSeverity::Medium),
                description: row.get(5)?,
                detected_at: row
                    .get::<_, String>(6)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
                resolved_at: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| s.parse().ok()),
                resolution_type: None,
                resolution_notes: row.get(9)?,
                auto_detected: row.get::<_, i32>(10)? == 1,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(conflicts)
}

// ============================================================================
// Contradiction Resolution (ENG-51)
// ============================================================================

/// Resolve a conflict between memories
pub fn resolve_conflict(
    conn: &Connection,
    conflict_id: i64,
    resolution_type: ResolutionType,
    notes: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        r#"
        UPDATE memory_conflicts
        SET resolved_at = ?, resolution_type = ?, resolution_notes = ?
        WHERE id = ?
        "#,
        params![now, resolution_type.as_str(), notes, conflict_id],
    )?;

    // Apply resolution
    let (memory_a_id, memory_b_id): (i64, i64) = conn.query_row(
        "SELECT memory_a_id, memory_b_id FROM memory_conflicts WHERE id = ?",
        params![conflict_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    match resolution_type {
        ResolutionType::KeepA => {
            // Archive memory B
            conn.execute(
                "UPDATE memories SET lifecycle_state = 'archived' WHERE id = ?",
                params![memory_b_id],
            )?;
        }
        ResolutionType::KeepB => {
            // Archive memory A
            conn.execute(
                "UPDATE memories SET lifecycle_state = 'archived' WHERE id = ?",
                params![memory_a_id],
            )?;
        }
        ResolutionType::DeleteBoth => {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE memories SET deleted_at = ? WHERE id IN (?, ?)",
                params![now, memory_a_id, memory_b_id],
            )?;
        }
        _ => {
            // KeepBoth, Merge, FalsePositive - no automatic action
        }
    }

    Ok(())
}

// ============================================================================
