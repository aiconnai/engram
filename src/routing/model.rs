//! Unified Model Routing Contract (RFC 0011).
//!
//! Inspects active providers, feature flags, dimensions, and operational status
//! across embeddings, reranking, vision, and audio pipelines.

use serde::{Deserialize, Serialize};

use crate::types::EmbeddingConfig;

/// Capability category for model providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Embeddings,
    Reranking,
    Vision,
    Audio,
    Synthesis,
}

/// Status and health report for an individual model provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub capability: ModelCapability,
    pub name: String,
    pub model_id: String,
    pub dimensions: Option<usize>,
    pub is_local: bool,
    pub is_available: bool,
    pub is_degraded: bool,
    pub status_message: String,
}

/// Comprehensive model routing status across all subsystems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoutingReport {
    pub primary_embedding_provider: String,
    pub primary_embedding_model: String,
    pub primary_dimensions: usize,
    pub is_fully_local: bool,
    pub providers: Vec<ProviderStatus>,
}

/// Inspect current system configuration and produce an aggregate routing report.
pub fn inspect_model_routing(config: &EmbeddingConfig) -> ModelRoutingReport {
    let mut providers = Vec::new();

    // 1. TF-IDF Provider (Always present)
    providers.push(ProviderStatus {
        capability: ModelCapability::Embeddings,
        name: "tfidf".to_string(),
        model_id: "built-in-sparse-tfidf".to_string(),
        dimensions: Some(config.dimensions),
        is_local: true,
        is_available: true,
        is_degraded: false,
        status_message: "Built-in zero-dependency lexical/sparse provider".to_string(),
    });

    // 2. Local ONNX Provider
    let onnx_available = cfg!(feature = "onnx-embed");
    providers.push(ProviderStatus {
        capability: ModelCapability::Embeddings,
        name: "onnx".to_string(),
        model_id: "all-MiniLM-L6-v2".to_string(),
        dimensions: Some(384),
        is_local: true,
        is_available: onnx_available,
        is_degraded: false,
        status_message: if onnx_available {
            "ONNX Runtime local sentence-transformers operational"
        } else {
            "Build with --features local-embeddings to enable local ONNX embeddings"
        }
        .to_string(),
    });

    // 3. OpenAI Embeddings
    let openai_available = cfg!(feature = "openai");
    let has_openai_key = config
        .api_key
        .as_ref()
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    providers.push(ProviderStatus {
        capability: ModelCapability::Embeddings,
        name: "openai".to_string(),
        model_id: config
            .embedding_model
            .clone()
            .unwrap_or_else(|| "text-embedding-3-small".to_string()),
        dimensions: Some(config.dimensions),
        is_local: false,
        is_available: openai_available && has_openai_key,
        is_degraded: openai_available && !has_openai_key,
        status_message: if !openai_available {
            "Build with --features openai to enable OpenAI backend"
        } else if !has_openai_key {
            "OpenAI feature enabled but OPENAI_API_KEY is not configured"
        } else {
            "OpenAI API embedding backend active"
        }
        .to_string(),
    });

    // 4. Multimodal / CLIP Provider
    let clip_available = cfg!(feature = "clip-embeddings");
    providers.push(ProviderStatus {
        capability: ModelCapability::Vision,
        name: "clip".to_string(),
        model_id: "clip-vit-base-patch32".to_string(),
        dimensions: Some(512),
        is_local: false,
        is_available: clip_available,
        is_degraded: false,
        status_message: if clip_available {
            "Multimodal CLIP vision & text embedder available"
        } else {
            "Build with --features clip-embeddings to enable multimodal embeddings"
        }
        .to_string(),
    });

    // 5. Neural Reranker
    let rerank_available = cfg!(feature = "neural-rerank");
    providers.push(ProviderStatus {
        capability: ModelCapability::Reranking,
        name: "neural_rerank".to_string(),
        model_id: "ms-marco-MiniLM-L-6-v2".to_string(),
        dimensions: None,
        is_local: true,
        is_available: rerank_available,
        is_degraded: false,
        status_message: if rerank_available {
            "Cross-encoder neural reranking operational"
        } else {
            "Build with --features neural-rerank to enable neural reranker"
        }
        .to_string(),
    });

    let is_fully_local = matches!(config.model.as_str(), "tfidf" | "local" | "onnx");

    ModelRoutingReport {
        primary_embedding_provider: config.model.clone(),
        primary_embedding_model: config
            .embedding_model
            .clone()
            .unwrap_or_else(|| config.model.clone()),
        primary_dimensions: config.dimensions,
        is_fully_local,
        providers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_model_routing_default() {
        let config = EmbeddingConfig::default();
        let report = inspect_model_routing(&config);

        assert_eq!(report.primary_embedding_provider, "tfidf");
        assert!(report.is_fully_local);
        assert!(!report.providers.is_empty());
        assert!(report
            .providers
            .iter()
            .any(|p| p.name == "tfidf" && p.is_available));
    }
}
