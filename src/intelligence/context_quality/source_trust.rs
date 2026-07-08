use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::{EngramError, Result};

use super::SourceTrustScore;

// Source Trust (ENG-53)
// ============================================================================

/// Get or set source trust score
pub fn get_source_trust(
    conn: &Connection,
    source_type: &str,
    source_identifier: Option<&str>,
) -> Result<SourceTrustScore> {
    let identifier = source_identifier.unwrap_or("default");

    let result = conn.query_row(
        r#"
        SELECT source_type, source_identifier, trust_score, verification_count, notes
        FROM source_trust_scores
        WHERE source_type = ? AND (source_identifier = ? OR source_identifier IS NULL)
        ORDER BY source_identifier DESC
        LIMIT 1
        "#,
        params![source_type, identifier],
        |row| {
            Ok(SourceTrustScore {
                source_type: row.get(0)?,
                source_identifier: row.get(1)?,
                trust_score: row.get(2)?,
                verification_count: row.get(3)?,
                notes: row.get(4)?,
            })
        },
    );

    result.map_err(|_| EngramError::NotFound(0))
}

/// Update source trust score
pub fn update_source_trust(
    conn: &Connection,
    source_type: &str,
    source_identifier: Option<&str>,
    trust_score: f32,
    notes: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        r#"
        INSERT INTO source_trust_scores (source_type, source_identifier, trust_score, notes, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(source_type, source_identifier)
        DO UPDATE SET trust_score = ?, notes = ?, updated_at = ?
        "#,
        params![
            source_type,
            source_identifier,
            trust_score,
            notes,
            now,
            trust_score,
            notes,
            now
        ],
    )?;

    Ok(())
}

// ============================================================================
