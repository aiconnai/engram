use std::collections::HashSet;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Result;

use super::DuplicateCandidate;

// Near-Duplicate Detection (ENG-48)
// ============================================================================

/// Calculate similarity between two strings using character n-grams
pub fn calculate_text_similarity(text_a: &str, text_b: &str) -> f32 {
    let ngram_size = 3;

    fn get_ngrams(text: &str, n: usize) -> HashSet<String> {
        let normalized: String = text
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        if normalized.len() < n {
            return HashSet::new();
        }
        normalized
            .chars()
            .collect::<Vec<_>>()
            .windows(n)
            .map(|w| w.iter().collect::<String>())
            .collect()
    }

    let ngrams_a = get_ngrams(text_a, ngram_size);
    let ngrams_b = get_ngrams(text_b, ngram_size);

    if ngrams_a.is_empty() && ngrams_b.is_empty() {
        return 1.0;
    }
    if ngrams_a.is_empty() || ngrams_b.is_empty() {
        return 0.0;
    }

    let intersection = ngrams_a.intersection(&ngrams_b).count() as f32;
    let union = ngrams_a.union(&ngrams_b).count() as f32;

    intersection / union
}

/// Find near-duplicate memories using text similarity
pub fn find_near_duplicates(
    conn: &Connection,
    threshold: f32,
    limit: i64,
) -> Result<Vec<DuplicateCandidate>> {
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare(
        r#"
        SELECT id, workspace, content FROM memories
        WHERE valid_to IS NULL
          AND (expires_at IS NULL OR expires_at > ?1)
          AND COALESCE(lifecycle_state, 'active') != 'archived'
        ORDER BY created_at DESC, id DESC
        LIMIT ?2
        "#,
    )?;

    let memories: Vec<(i64, String, String)> = stmt
        .query_map(params![now, limit.saturating_mul(2)], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut duplicates = Vec::new();

    // Compare pairs within the same workspace only
    for i in 0..memories.len() {
        for j in (i + 1)..memories.len() {
            let (id_a, ws_a, content_a) = &memories[i];
            let (id_b, ws_b, content_b) = &memories[j];
            if ws_a != ws_b {
                continue;
            }

            let similarity = calculate_text_similarity(content_a, content_b);

            if similarity >= threshold {
                let existing: Option<(i64, String)> = conn
                    .query_row(
                        r#"
                        SELECT id, status
                        FROM duplicate_candidates
                        WHERE ((memory_a_id = ?1 AND memory_b_id = ?2)
                            OR (memory_a_id = ?2 AND memory_b_id = ?1))
                          AND similarity_type = 'content'
                        LIMIT 1
                        "#,
                        params![id_a, id_b],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?;

                match existing {
                    None => {
                        conn.execute(
                            r#"
                            INSERT OR IGNORE INTO duplicate_candidates
                            (memory_a_id, memory_b_id, similarity_score, similarity_type)
                            VALUES (?1, ?2, ?3, 'content')
                            "#,
                            params![id_a, id_b, similarity],
                        )?;
                        let candidate_id: i64 = conn.query_row(
                            r#"
                            SELECT id FROM duplicate_candidates
                            WHERE memory_a_id = ?1 AND memory_b_id = ?2 AND similarity_type = 'content'
                            "#,
                            params![id_a, id_b],
                            |row| row.get(0),
                        )?;

                        duplicates.push(DuplicateCandidate {
                            id: candidate_id,
                            memory_a_id: *id_a,
                            memory_b_id: *id_b,
                            similarity_score: similarity,
                            similarity_type: "content".to_string(),
                            detected_at: Utc::now(),
                            status: "pending".to_string(),
                        });
                    }
                    Some((candidate_id, status)) if status == "pending" => {
                        duplicates.push(DuplicateCandidate {
                            id: candidate_id,
                            memory_a_id: *id_a,
                            memory_b_id: *id_b,
                            similarity_score: similarity,
                            similarity_type: "content".to_string(),
                            detected_at: Utc::now(),
                            status,
                        });
                    }
                    Some(_) => {}
                }
            }
        }
    }

    Ok(duplicates)
}

/// Get pending duplicate candidates
pub fn get_pending_duplicates(conn: &Connection, limit: i64) -> Result<Vec<DuplicateCandidate>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, memory_a_id, memory_b_id, similarity_score, similarity_type, detected_at, status
        FROM duplicate_candidates
        WHERE status = 'pending'
        ORDER BY similarity_score DESC
        LIMIT ?
        "#,
    )?;

    let duplicates = stmt
        .query_map(params![limit], |row| {
            Ok(DuplicateCandidate {
                id: row.get(0)?,
                memory_a_id: row.get(1)?,
                memory_b_id: row.get(2)?,
                similarity_score: row.get(3)?,
                similarity_type: row.get(4)?,
                detected_at: row
                    .get::<_, String>(5)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
                status: row.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(duplicates)
}

// ============================================================================
// Semantic Deduplication (ENG-49)
// ============================================================================

/// Find semantic duplicates using embedding similarity
pub fn find_semantic_duplicates(
    conn: &Connection,
    query_embedding: &[f32],
    threshold: f32,
    limit: i64,
) -> Result<Vec<DuplicateCandidate>> {
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare(
        r#"
        SELECT m.id, e.embedding
        FROM memories m
        JOIN embeddings e ON m.id = e.memory_id
        WHERE m.valid_to IS NULL
          AND (m.expires_at IS NULL OR m.expires_at > ?1)
          AND COALESCE(m.lifecycle_state, 'active') != 'archived'
        LIMIT ?2
        "#,
    )?;

    let memories: Vec<(i64, Vec<f32>)> = stmt
        .query_map(params![now, limit], |row| {
            let id: i64 = row.get(0)?;
            let embedding_blob: Vec<u8> = row.get(1)?;
            let embedding: Vec<f32> = embedding_blob
                .chunks(4)
                .map(|chunk| {
                    let bytes: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                    f32::from_le_bytes(bytes)
                })
                .collect();
            Ok((id, embedding))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut duplicates = Vec::new();

    for (id, embedding) in &memories {
        let similarity = cosine_similarity(query_embedding, embedding);
        if similarity >= threshold {
            duplicates.push(DuplicateCandidate {
                id: 0,
                memory_a_id: 0, // Query memory
                memory_b_id: *id,
                similarity_score: similarity,
                similarity_type: "semantic".to_string(),
                detected_at: Utc::now(),
                status: "pending".to_string(),
            });
        }
    }

    Ok(duplicates)
}

pub(super) use crate::search::vector::cosine_similarity;

// ============================================================================
