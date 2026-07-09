use std::collections::HashSet;

use chrono::Utc;
use rusqlite::{params, Connection};

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
    // Get memories that haven't been checked yet
    let mut stmt = conn.prepare(
        r#"
        SELECT id, content FROM memories
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )?;

    let memories: Vec<(i64, String)> = stmt
        .query_map(params![limit * 2], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let mut duplicates = Vec::new();

    // Compare pairs
    for i in 0..memories.len() {
        for j in (i + 1)..memories.len() {
            let (id_a, content_a) = &memories[i];
            let (id_b, content_b) = &memories[j];

            let similarity = calculate_text_similarity(content_a, content_b);

            if similarity >= threshold {
                // Check if already recorded
                let exists: bool = conn.query_row(
                    "SELECT 1 FROM duplicate_candidates WHERE memory_a_id = ? AND memory_b_id = ?",
                    params![id_a, id_b],
                    |_| Ok(true),
                ).unwrap_or(false);

                if !exists {
                    conn.execute(
                        r#"
                        INSERT OR IGNORE INTO duplicate_candidates
                        (memory_a_id, memory_b_id, similarity_score, similarity_type)
                        VALUES (?, ?, ?, 'content')
                        "#,
                        params![id_a, id_b, similarity],
                    )?;

                    duplicates.push(DuplicateCandidate {
                        id: 0,
                        memory_a_id: *id_a,
                        memory_b_id: *id_b,
                        similarity_score: similarity,
                        similarity_type: "content".to_string(),
                        detected_at: Utc::now(),
                        status: "pending".to_string(),
                    });
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
        .filter_map(|r| r.ok())
        .collect();

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
    // Use existing embedding search infrastructure
    let mut stmt = conn.prepare(
        r#"
        SELECT m.id, e.embedding
        FROM memories m
        JOIN embeddings e ON m.id = e.memory_id
        WHERE m.deleted_at IS NULL
        LIMIT ?
        "#,
    )?;

    let memories: Vec<(i64, Vec<f32>)> = stmt
        .query_map(params![limit], |row| {
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
        .filter_map(|r| r.ok())
        .collect();

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

/// Calculate cosine similarity between two vectors
pub(super) fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

// ============================================================================
