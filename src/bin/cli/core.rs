use engram::embedding::create_embedder;
use engram::error::Result;
use engram::search::{hybrid_search, SearchConfig};
use engram::storage::queries::*;
use engram::storage::Storage;
use engram::types::*;

use crate::util::truncate;

pub(crate) fn create(
    storage: &Storage,
    content: String,
    r#type: String,
    tags: Option<String>,
    importance: Option<f32>,
) -> Result<()> {
    let memory_type: MemoryType = r#type.parse().unwrap_or(MemoryType::Note);
    let tags: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let input = CreateMemoryInput {
        content,
        memory_type,
        tags,
        metadata: Default::default(),
        importance,
        scope: Default::default(),
        workspace: None,
        tier: Default::default(),
        defer_embedding: true,
        ttl_seconds: None,
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    let memory = storage.with_transaction(|conn| create_memory(conn, &input))?;
    println!("Created memory #{}", memory.id);
    println!("{}", serde_json::to_string_pretty(&memory)?);
    Ok(())
}

pub(crate) fn get(storage: &Storage, id: i64) -> Result<()> {
    let memory = storage.with_connection(|conn| get_memory(conn, id))?;
    println!("{}", serde_json::to_string_pretty(&memory)?);
    Ok(())
}

pub(crate) fn list(
    storage: &Storage,
    limit: i64,
    tags: Option<String>,
    r#type: Option<String>,
) -> Result<()> {
    let tags: Option<Vec<String>> =
        tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let memory_type = r#type.and_then(|t| t.parse().ok());

    let options = ListOptions {
        limit: Some(limit),
        tags,
        memory_type,
        ..Default::default()
    };

    let memories = storage.with_connection(|conn| list_memories(conn, &options))?;
    for memory in memories {
        println!(
            "#{} [{}] {} - {}",
            memory.id,
            memory.memory_type.as_str(),
            memory.tags.join(", "),
            truncate(&memory.content, 60)
        );
    }
    Ok(())
}

pub(crate) fn search(storage: &Storage, query: String, limit: i64, explain: bool) -> Result<()> {
    let embedding_config = EmbeddingConfig::default();
    let embedder = create_embedder(&embedding_config)?;
    let query_embedding = embedder.embed(&query).ok();

    let options = SearchOptions {
        limit: Some(limit),
        explain,
        ..Default::default()
    };

    let config = SearchConfig::default();
    let results = storage.with_connection(|conn| {
        hybrid_search(conn, &query, query_embedding.as_deref(), &options, &config)
    })?;

    for result in results {
        println!(
            "#{} (score: {:.3}) - {}",
            result.memory.id,
            result.score,
            truncate(&result.memory.content, 60)
        );
        if explain {
            println!(
                "  Strategy: {:?}, Matched: {:?}",
                result.match_info.strategy, result.match_info.matched_terms
            );
        }
    }
    Ok(())
}

pub(crate) fn delete(storage: &Storage, id: i64) -> Result<()> {
    storage.with_transaction(|conn| delete_memory(conn, id))?;
    println!("Deleted memory #{}", id);
    Ok(())
}

pub(crate) fn stats(storage: &Storage) -> Result<()> {
    let stats = storage.with_connection(get_stats)?;
    println!("{}", serde_json::to_string_pretty(&stats)?);
    Ok(())
}

pub(crate) fn link(storage: &Storage, from: i64, to: i64, edge_type: String) -> Result<()> {
    let edge_type: EdgeType = edge_type.parse().unwrap_or(EdgeType::RelatedTo);
    let input = CreateCrossRefInput {
        from_id: from,
        to_id: to,
        edge_type,
        strength: None,
        source_context: None,
        pinned: false,
    };

    storage.with_transaction(|conn| create_crossref(conn, &input))?;
    println!("Linked #{} -> #{} ({})", from, to, edge_type.as_str());
    Ok(())
}

pub(crate) fn versions(storage: &Storage, id: i64) -> Result<()> {
    let versions = storage.with_connection(|conn| get_memory_versions(conn, id))?;
    for version in versions {
        println!(
            "v{} ({}) - {}",
            version.version,
            version.created_at.format("%Y-%m-%d %H:%M"),
            truncate(&version.content, 50)
        );
    }
    Ok(())
}
