use super::*;

/// Find semantically similar memories using embedding cosine similarity.
/// This is "LLM-powered" dedup — goes beyond hash/n-gram matching to detect
/// memories that convey the same information with different wording.
pub fn find_duplicates_by_embedding(
    conn: &Connection,
    threshold: f32,
    workspace: Option<&str>,
    limit: usize,
) -> Result<Vec<DuplicatePair>> {
    use crate::embedding::{cosine_similarity, get_embedding};

    let now = Utc::now().to_rfc3339();

    // Get all memory IDs with embeddings (scoped to workspace if provided)
    let (sql, params_vec): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ws) = workspace {
        (
            "SELECT id FROM memories
             WHERE has_embedding = 1 AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
               AND COALESCE(lifecycle_state, 'active') = 'active'
               AND workspace = ?
             ORDER BY id",
            vec![Box::new(now), Box::new(ws.to_string())],
        )
    } else {
        (
            "SELECT id FROM memories
             WHERE has_embedding = 1 AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
               AND COALESCE(lifecycle_state, 'active') = 'active'
             ORDER BY id",
            vec![Box::new(now)],
        )
    };

    let mut stmt = conn.prepare(sql)?;
    let ids: Vec<i64> = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
            |row| row.get(0),
        )?
        .filter_map(|r| r.ok())
        .collect();

    // Load all embeddings into memory for pairwise comparison
    let mut embeddings: Vec<(i64, Vec<f32>)> = Vec::with_capacity(ids.len());
    for &id in &ids {
        if let Ok(Some(emb)) = get_embedding(conn, id) {
            embeddings.push((id, emb));
        }
    }

    let mut duplicates = Vec::new();

    // Pairwise comparison (O(n^2) — bounded by limit)
    for i in 0..embeddings.len() {
        if duplicates.len() >= limit {
            break;
        }
        for j in (i + 1)..embeddings.len() {
            if duplicates.len() >= limit {
                break;
            }
            let sim = cosine_similarity(&embeddings[i].1, &embeddings[j].1);
            if sim >= threshold {
                let memory_a = get_memory_internal(conn, embeddings[i].0, false)?;
                let memory_b = get_memory_internal(conn, embeddings[j].0, false)?;
                duplicates.push(DuplicatePair {
                    memory_a,
                    memory_b,
                    similarity_score: sim as f64,
                    match_type: DuplicateMatchType::EmbeddingSimilarity,
                });
            }
        }
    }

    // Sort by similarity descending
    duplicates.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(duplicates)
}
