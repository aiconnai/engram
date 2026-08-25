//! AAAK compression tool handlers.
//! Implements memory_compress_aaak and memory_decompress_aaak.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::error::EngramError;
use crate::intelligence::aaak::{AaakCompressor, AaakMode};
use crate::storage::queries::get_memory;

/// Handler for `memory_compress_aaak`.
pub fn memory_compress_aaak(ctx: &HandlerContext, params: Value) -> Value {
    let mode_str = params
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("ultradense");
    let mode = mode_str.parse::<AaakMode>().unwrap_or(AaakMode::UltraDense);

    let text_opt = params.get("text").and_then(|v| v.as_str());
    let memory_id_opt = params.get("memory_id").and_then(|v| v.as_i64());

    let content_to_compress = if let Some(text) = text_opt {
        text.to_string()
    } else if let Some(mem_id) = memory_id_opt {
        let fetch_res = ctx.storage.with_connection(|conn| get_memory(conn, mem_id));
        match fetch_res {
            Ok(mem) => mem.content,
            Err(EngramError::NotFound(id)) => {
                return json!({
                    "error": format!("Memory not found: {}", id)
                });
            }
            Err(e) => {
                return json!({
                    "error": format!("Failed to retrieve memory: {}", e)
                });
            }
        }
    } else {
        return json!({
            "error": "Either 'text' or 'memory_id' must be provided."
        });
    };

    let result = AaakCompressor::compress(&content_to_compress, mode);

    json!({
        "compressed": result.compressed,
        "original_bytes": result.original_bytes,
        "compressed_bytes": result.compressed_bytes,
        "original_tokens": result.original_tokens,
        "compressed_tokens": result.compressed_tokens,
        "compression_ratio": format!("{:.2}x", result.compression_ratio),
        "token_savings_pct": format!("{:.1}%", result.token_savings_pct),
        "mode": format!("{:?}", result.mode).to_lowercase()
    })
}

/// Handler for `memory_decompress_aaak`.
pub fn memory_decompress_aaak(_ctx: &HandlerContext, params: Value) -> Value {
    let text = match params.get("text").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return json!({
                "error": "Missing required field: 'text'"
            });
        }
    };

    let decompressed = AaakCompressor::decompress(text);

    json!({
        "decompressed": decompressed,
        "original_length": text.len(),
        "expanded_length": decompressed.len()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::create_memory;
    use parking_lot::{Mutex, RwLock};
    use std::sync::Arc;

    fn test_context() -> HandlerContext {
        HandlerContext {
            storage: crate::Storage::open_in_memory().expect("in-memory storage"),
            embedder: crate::embedding::create_embedder(&crate::types::EmbeddingConfig::default())
                .expect("tfidf embedder"),
            fuzzy_engine: Arc::new(Mutex::new(crate::search::FuzzyEngine::new())),
            search_config: crate::search::SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(crate::search::SearchResultCache::new(
                crate::search::AdaptiveCacheConfig::default(),
            )),
            hnsw_index: Arc::new(RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
            principal: None,
        }
    }

    #[test]
    fn test_memory_compress_aaak_direct_text() {
        let ctx = test_context();
        let params = json!({
            "text": "The database configuration function requires authentication.",
            "mode": "ultradense"
        });
        let res = memory_compress_aaak(&ctx, params);
        assert!(res["compressed"].as_str().unwrap().contains("db"));
        assert!(res["compressed"].as_str().unwrap().contains("cfg"));
        assert!(res["compressed"].as_str().unwrap().contains("fn"));
    }

    #[test]
    fn test_memory_compress_aaak_memory_id() {
        let ctx = test_context();
        let mem = ctx
            .storage
            .with_transaction(|conn| {
                create_memory(
                    conn,
                    &crate::types::CreateMemoryInput {
                        content: "The repository authentication service generates JWT tokens."
                            .to_string(),
                        ..Default::default()
                    },
                )
            })
            .unwrap();

        let params = json!({
            "memory_id": mem.id,
            "mode": "lossless"
        });
        let res = memory_compress_aaak(&ctx, params);
        assert!(res["compressed"].as_str().unwrap().contains("repo"));
        assert!(res["compressed"].as_str().unwrap().contains("auth"));
    }

    #[test]
    fn test_memory_decompress_aaak_handler() {
        let ctx = test_context();
        let params = json!({
            "text": "[AAAK:v1:dense]\n[U] please cfg db fn."
        });
        let res = memory_decompress_aaak(&ctx, params);
        let decomp = res["decompressed"].as_str().unwrap();
        assert!(decomp.contains("User:"));
        assert!(decomp.contains("configuration"));
        assert!(decomp.contains("database"));
        assert!(decomp.contains("function"));
    }
}
