//! Embedding generation and async queue management (RML-873)
//!
//! Supports multiple embedding backends:
//! - OpenAI API (text-embedding-3-small) - requires `openai` feature
//! - TF-IDF fallback (no external dependencies)
//!
//! Features:
//! - LRU embedding cache with zero-copy Arc<[f32]> sharing
//! - Async queue processing for batch operations
//!
//! # Feature Flags
//!
//! - `openai`: Enables OpenAI embedding backend (requires API key)

mod cache;
mod provider;
mod queue;
mod tfidf;

#[cfg(feature = "clip-embeddings")]
pub mod clip;
#[cfg(feature = "cohere")]
pub mod cohere;
#[cfg(feature = "ollama")]
pub mod ollama;
#[cfg(feature = "onnx-embed")]
pub mod onnx;
#[cfg(feature = "onnx-embed")]
pub mod onnx_registry;
#[cfg(feature = "voyage")]
pub mod voyage;

pub use cache::{EmbeddingCache, EmbeddingCacheStats};
#[cfg(feature = "clip-embeddings")]
pub use clip::{ClipEmbedder, MultimodalEmbedder, CLIP_PROVIDER_NAME};
pub use provider::{EmbeddingProvider, EmbeddingProviderInfo, EmbeddingRegistry};
pub use queue::{
    drain_pending_embeddings, get_embedding, get_embedding_queue_health, get_embedding_status,
    requeue_stale_processing_embeddings, run_embedding_queue_hygiene, EmbeddingQueue,
    EmbeddingQueueHealth, EmbeddingQueueHygieneConfig, EmbeddingQueueHygieneReport,
    EmbeddingWorker, DEFAULT_COMPLETE_RETENTION, DEFAULT_MAX_EMBEDDING_RETRIES,
    DEFAULT_STALE_PROCESSING_AFTER,
};
pub use tfidf::TfIdfEmbedder;

use std::sync::Arc;

use crate::error::{EngramError, Result};
use crate::types::EmbeddingConfig;

/// Trait for embedding generators
pub trait Embedder: Send + Sync {
    /// Generate embedding for a single text
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Get embedding dimensions
    fn dimensions(&self) -> usize;

    /// Get model name
    fn model_name(&self) -> &str;
}

/// OpenAI embedding client
///
/// Requires the `openai` feature to be enabled.
/// Supports OpenAI, OpenRouter, Azure OpenAI, and other OpenAI-compatible APIs.
#[cfg(feature = "openai")]
pub struct OpenAIEmbedder {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    dimensions: usize,
    /// Owned single-threaded runtime used by the sync `embed()` / `embed_batch()`
    /// path when no tokio runtime is in scope (e.g. MCP stdio transport, which
    /// runs the JSON-RPC loop synchronously). When the embedder is called from
    /// inside an existing runtime (HTTP transport), we detect that with
    /// `Handle::try_current()` and use `block_in_place` instead of this owned
    /// runtime to avoid the cost of a nested runtime. Fixes #9.
    runtime: tokio::runtime::Runtime,
}

#[cfg(feature = "openai")]
impl OpenAIEmbedder {
    /// Build the owned fallback runtime. Single-threaded with all drivers
    /// enabled is enough for the rare case where there's no ambient runtime.
    fn build_fallback_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("OpenAIEmbedder: failed to build fallback tokio runtime")
    }

    /// Create a new OpenAI embedder with default settings
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "text-embedding-3-small".to_string(),
            dimensions: 1536,
            runtime: Self::build_fallback_runtime(),
        }
    }

    /// Create a new OpenAI embedder with custom settings
    ///
    /// # Arguments
    /// * `api_key` - API key for authentication
    /// * `base_url` - API base URL (e.g., `<https://openrouter.ai/api/v1>` for OpenRouter)
    /// * `model` - Model name (e.g., "openai/text-embedding-3-small" for OpenRouter)
    /// * `dimensions` - Expected embedding dimensions (must match model output)
    pub fn with_config(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        dimensions: Option<usize>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
            dimensions: dimensions.unwrap_or(1536),
            runtime: Self::build_fallback_runtime(),
        }
    }

    /// Legacy constructor for backwards compatibility
    pub fn with_model(api_key: String, model: String, dimensions: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model,
            dimensions,
            runtime: Self::build_fallback_runtime(),
        }
    }

    /// Async embedding call to OpenAI-compatible API
    pub async fn embed_async(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            // OpenRouter requires HTTP-Referer header
            .header("HTTP-Referer", "https://github.com/engram")
            // Optional: helps OpenRouter track usage
            .header("X-Title", "Engram Memory")
            .json(&serde_json::json!({
                "input": text,
                "model": self.model,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EngramError::Embedding(format!(
                "Embedding API error {}: {}",
                status, text
            )));
        }

        let data: serde_json::Value = response.json().await?;
        let embedding: Vec<f32> = data["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| EngramError::Embedding("Invalid response format".to_string()))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        // Validate dimensions match configuration
        if embedding.len() != self.dimensions {
            return Err(EngramError::Embedding(format!(
                "Embedding dimensions mismatch: expected {}, got {}. Set OPENAI_EMBEDDING_DIMENSIONS={} to match your model.",
                self.dimensions, embedding.len(), embedding.len()
            )));
        }

        Ok(embedding)
    }

    /// Async batch embedding (up to 2048 inputs per call)
    pub async fn embed_batch_async(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let url = format!("{}/embeddings", self.base_url);

        // OpenAI allows up to 2048 inputs per batch
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(2048) {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                // OpenRouter requires HTTP-Referer header
                .header("HTTP-Referer", "https://github.com/engram")
                .header("X-Title", "Engram Memory")
                .json(&serde_json::json!({
                    "input": chunk,
                    "model": self.model,
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(EngramError::Embedding(format!(
                    "Embedding API error {}: {}",
                    status, text
                )));
            }

            let data: serde_json::Value = response.json().await?;
            let embeddings: Vec<Vec<f32>> = data["data"]
                .as_array()
                .ok_or_else(|| EngramError::Embedding("Invalid response format".to_string()))?
                .iter()
                .map(|item| {
                    item["embedding"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect();

            // Validate dimensions on first batch
            if !embeddings.is_empty() && embeddings[0].len() != self.dimensions {
                return Err(EngramError::Embedding(format!(
                    "Embedding dimensions mismatch: expected {}, got {}. Set OPENAI_EMBEDDING_DIMENSIONS={} to match your model.",
                    self.dimensions, embeddings[0].len(), embeddings[0].len()
                )));
            }

            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }
}

#[cfg(feature = "openai")]
impl Embedder for OpenAIEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // If we are inside an existing tokio runtime (e.g. HTTP transport),
        // use block_in_place so we don't create a nested runtime. Otherwise
        // (e.g. MCP stdio), drive the async call from our owned fallback
        // runtime — without this, Handle::current() panics with "there is
        // no reactor running". Fixes #9.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(self.embed_async(text))),
            Err(_) => self.runtime.block_on(self.embed_async(text)),
        }
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                tokio::task::block_in_place(|| handle.block_on(self.embed_batch_async(texts)))
            }
            Err(_) => self.runtime.block_on(self.embed_batch_async(texts)),
        }
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Create an embedder from configuration
///
/// Available models depend on enabled features:
/// - `"tfidf"`: Always available, no external dependencies
/// - `"openai"`: Requires `openai` feature and API key
/// - `"local"` / `"onnx"`: Requires `local-embeddings` feature and a downloaded ONNX model
///
/// For OpenAI-compatible APIs (OpenRouter, Azure, etc.), set:
/// - `base_url`: API endpoint (e.g., `<https://openrouter.ai/api/v1>`)
/// - `embedding_model`: Model name (e.g., "openai/text-embedding-3-small")
/// - `dimensions`: Expected output dimensions
pub fn create_embedder(config: &EmbeddingConfig) -> Result<Arc<dyn Embedder>> {
    match config.model.as_str() {
        #[cfg(feature = "clip-embeddings")]
        "clip" => {
            clip::create_clip_embedder()
                .map(|e| e as Arc<dyn Embedder>)
        }
        #[cfg(feature = "openai")]
        "openai" => {
            let api_key = config
                .api_key
                .clone()
                .ok_or_else(|| EngramError::Config(
                    "OPENAI_API_KEY required when ENGRAM_EMBEDDING_MODEL=openai".to_string()
                ))?;
            Ok(Arc::new(OpenAIEmbedder::with_config(
                api_key,
                config.base_url.clone(),
                config.embedding_model.clone(),
                Some(config.dimensions),
            )))
        }
        #[cfg(not(feature = "openai"))]
        "openai" => Err(EngramError::Config(
            "OpenAI embeddings require the 'openai' feature to be enabled. Build with: cargo build --features openai".to_string(),
        )),
        #[cfg(feature = "onnx-embed")]
        "local" | "onnx" => {
            let model_dir = onnx::resolve_model_dir(config.model_path.as_deref());
            Ok(Arc::new(onnx::OnnxEmbedder::from_dir(&model_dir)?))
        }
        #[cfg(not(feature = "onnx-embed"))]
        "local" | "onnx" => Err(EngramError::Config(
            "Local sentence-transformer embeddings require the 'local-embeddings' feature. Build with: cargo build --features local-embeddings, then run: engram-cli model download minilm-l6-v2 and set ENGRAM_EMBEDDING_MODEL=local".to_string(),
        )),
        "tfidf" => Ok(Arc::new(TfIdfEmbedder::new(config.dimensions))),
        _ => Err(EngramError::Config(format!(
            "Unknown embedding model: '{}'. Use 'tfidf', 'local', or 'openai'",
            config.model
        ))),
    }
}

pub use crate::search::vector::cosine_similarity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tfidf_embedder() {
        let embedder = TfIdfEmbedder::new(384);
        let embedding = embedder.embed("Hello world").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[cfg(not(feature = "onnx-embed"))]
    #[test]
    fn test_local_embedder_requires_feature_when_disabled() {
        let config = EmbeddingConfig {
            model: "local".to_string(),
            ..EmbeddingConfig::default()
        };

        let err = match create_embedder(&config) {
            Ok(_) => panic!("local backend should require opt-in feature"),
            Err(err) => err,
        };
        let msg = err.to_string();

        assert!(msg.contains("local-embeddings"), "{msg}");
        assert!(msg.contains("ENGRAM_EMBEDDING_MODEL=local"), "{msg}");
    }

    #[cfg(not(feature = "onnx-embed"))]
    #[test]
    fn test_onnx_alias_requires_feature_when_disabled() {
        let config = EmbeddingConfig {
            model: "onnx".to_string(),
            ..EmbeddingConfig::default()
        };

        let err = match create_embedder(&config) {
            Ok(_) => panic!("onnx alias should require opt-in feature"),
            Err(err) => err,
        };
        let msg = err.to_string();

        assert!(msg.contains("local-embeddings"), "{msg}");
        assert!(msg.contains("ENGRAM_EMBEDDING_MODEL=local"), "{msg}");
    }

    #[cfg(feature = "onnx-embed")]
    #[test]
    fn test_local_embedder_errors_with_download_instructions_when_model_missing() {
        let config = EmbeddingConfig {
            model: "local".to_string(),
            model_path: Some("/tmp/nonexistent-engram-test-model-path".to_string()),
            ..EmbeddingConfig::default()
        };

        let err = match create_embedder(&config) {
            Ok(_) => panic!("missing model should return config error"),
            Err(err) => err,
        };
        let msg = err.to_string();
        assert!(msg.contains("model.onnx"), "{msg}");
        assert!(msg.contains("engram-cli model download"), "{msg}");
    }
}
