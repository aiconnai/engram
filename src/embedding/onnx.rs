//! Local ONNX sentence-transformer embedding provider.
//!
//! This backend is opt-in via the `local-embeddings` feature. It loads an
//! ONNX model and HuggingFace `tokenizer.json` from disk; model downloads are
//! handled explicitly by the CLI.

#[cfg(feature = "onnx-embed")]
mod inner {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use ndarray::{Array2, Array3};
    use ort::session::Session;
    use ort::value::Tensor;
    use tokenizers::Tokenizer;

    use crate::embedding::onnx_registry::{default_model_dir, DEFAULT_MODEL_NAME, REGISTRY};
    use crate::embedding::Embedder;
    use crate::error::{EngramError, Result};

    /// Configuration for the local ONNX embedding provider.
    #[derive(Debug, Clone)]
    pub struct OnnxConfig {
        /// Directory containing `model.onnx` and `tokenizer.json`.
        pub model_dir: PathBuf,
        /// Number of output dimensions.
        pub dimensions: usize,
        /// Maximum token sequence length.
        pub max_length: usize,
        /// Human-readable model name returned by [`Embedder::model_name`].
        pub model_name: String,
    }

    impl Default for OnnxConfig {
        fn default() -> Self {
            let entry = &REGISTRY[0];
            Self {
                model_dir: default_model_dir(),
                dimensions: entry.dimensions,
                max_length: entry.max_seq_len,
                model_name: DEFAULT_MODEL_NAME.to_string(),
            }
        }
    }

    /// ONNX Runtime backed sentence-transformer provider.
    pub struct OnnxEmbedder {
        config: OnnxConfig,
        session: Mutex<Session>,
        tokenizer: Tokenizer,
    }

    /// Resolve model directory using config > env > platform data dir.
    pub fn resolve_model_dir(config_path: Option<&str>) -> PathBuf {
        if let Some(path) = config_path.filter(|p| !p.trim().is_empty()) {
            return PathBuf::from(path);
        }

        if let Ok(path) = std::env::var("ENGRAM_ONNX_MODEL_DIR") {
            if !path.trim().is_empty() {
                return PathBuf::from(path);
            }
        }

        default_model_dir()
    }

    impl OnnxEmbedder {
        /// Load an ONNX model and HuggingFace tokenizer from a model directory.
        pub fn from_dir(model_dir: &Path) -> Result<Self> {
            let entry = &REGISTRY[0];
            let config = OnnxConfig {
                model_dir: model_dir.to_path_buf(),
                dimensions: entry.dimensions,
                max_length: entry.max_seq_len,
                model_name: entry.name.to_string(),
            };
            Self::new(config)
        }

        /// Load an ONNX model and tokenizer from an explicit config.
        pub fn new(config: OnnxConfig) -> Result<Self> {
            let model_path = config.model_dir.join("model.onnx");
            let tokenizer_path = config.model_dir.join("tokenizer.json");

            if !model_path.is_file() {
                return Err(EngramError::Config(format!(
                    "Local embedding model file not found at {}. Run: engram-cli model download {}",
                    model_path.display(),
                    DEFAULT_MODEL_NAME
                )));
            }
            if !tokenizer_path.is_file() {
                return Err(EngramError::Config(format!(
                    "Local embedding tokenizer not found at {}. Run: engram-cli model download {}",
                    tokenizer_path.display(),
                    DEFAULT_MODEL_NAME
                )));
            }

            let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| {
                EngramError::Config(format!(
                    "Failed to load tokenizer from {}: {e}",
                    tokenizer_path.display()
                ))
            })?;

            let session = Session::builder()
                .map_err(|e| {
                    EngramError::Embedding(format!("Failed to create ONNX session builder: {e}"))
                })?
                .commit_from_file(&model_path)
                .map_err(|e| {
                    EngramError::Embedding(format!(
                        "Failed to load ONNX model from {}: {e}",
                        model_path.display()
                    ))
                })?;

            Ok(Self {
                config,
                session: Mutex::new(session),
                tokenizer,
            })
        }

        fn encode(&self, text: &str) -> Result<(Vec<i64>, Vec<i64>, Vec<i64>)> {
            let encoding = self.tokenizer.encode(text, true).map_err(|e| {
                EngramError::Embedding(format!(
                    "Failed to tokenize text '{}': {e}",
                    truncate_for_error(text, 80)
                ))
            })?;

            let mut input_ids: Vec<i64> =
                encoding.get_ids().iter().map(|&id| i64::from(id)).collect();
            let mut attention_mask: Vec<i64> = encoding
                .get_attention_mask()
                .iter()
                .map(|&mask| i64::from(mask))
                .collect();
            let mut token_type_ids: Vec<i64> = encoding
                .get_type_ids()
                .iter()
                .map(|&id| i64::from(id))
                .collect();

            input_ids.truncate(self.config.max_length);
            attention_mask.truncate(self.config.max_length);
            token_type_ids.truncate(self.config.max_length);

            let len = input_ids.len();
            if attention_mask.len() != len || token_type_ids.len() != len {
                return Err(EngramError::Embedding(
                    "Tokenizer returned mismatched input lengths".to_string(),
                ));
            }

            Ok((input_ids, attention_mask, token_type_ids))
        }

        fn run_inference(
            &self,
            input_ids: &[i64],
            attention_mask: &[i64],
            token_type_ids: &[i64],
        ) -> Result<Vec<f32>> {
            let seq_len = input_ids.len();
            if seq_len == 0 {
                return Err(EngramError::Embedding(
                    "Tokenizer returned an empty sequence".to_string(),
                ));
            }

            let ids_tensor =
                Tensor::from_array(([1, seq_len], input_ids.to_vec())).map_err(|e| {
                    EngramError::Embedding(format!("Failed to build input_ids tensor: {e}"))
                })?;
            let mask_tensor =
                Tensor::from_array(([1, seq_len], attention_mask.to_vec())).map_err(|e| {
                    EngramError::Embedding(format!("Failed to build attention_mask tensor: {e}"))
                })?;
            let type_ids_tensor = Tensor::from_array(([1, seq_len], token_type_ids.to_vec()))
                .map_err(|e| {
                    EngramError::Embedding(format!("Failed to build token_type_ids tensor: {e}"))
                })?;

            let mut session = self
                .session
                .lock()
                .map_err(|e| EngramError::Embedding(format!("Failed to lock ONNX session: {e}")))?;

            let outputs = session
                .run(ort::inputs![
                    "input_ids" => ids_tensor,
                    "attention_mask" => mask_tensor,
                    "token_type_ids" => type_ids_tensor
                ])
                .map_err(|e| EngramError::Embedding(format!("ONNX inference error: {e}")))?;

            let (shape, data) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                EngramError::Embedding(format!("Failed to extract ONNX output tensor: {e}"))
            })?;

            match shape.len() {
                2 if shape[0] == 1 => {
                    let embedding = data.to_vec();
                    self.validate_dimensions(embedding.len())?;
                    Ok(l2_normalized(embedding))
                }
                3 if shape[0] == 1 => {
                    let actual_seq_len = shape[1] as usize;
                    let hidden_size = shape[2] as usize;
                    self.validate_dimensions(hidden_size)?;

                    let hidden = Array3::from_shape_vec(
                        (1, actual_seq_len, hidden_size),
                        data.to_vec(),
                    )
                    .map_err(|e| {
                        EngramError::Embedding(format!("Failed to reshape ONNX output: {e}"))
                    })?;

                    let token_embeddings = Array2::from_shape_vec(
                        (actual_seq_len, hidden_size),
                        hidden.index_axis(ndarray::Axis(0), 0).iter().copied().collect(),
                    )
                    .map_err(|e| {
                        EngramError::Embedding(format!("Failed to squeeze ONNX output: {e}"))
                    })?;

                    Ok(mean_pool_normalized(&token_embeddings, attention_mask))
                }
                _ => Err(EngramError::Embedding(format!(
                    "Expected ONNX output shape [1, dimensions] or [1, seq_len, dimensions], got {:?}",
                    shape
                ))),
            }
        }

        fn validate_dimensions(&self, actual: usize) -> Result<()> {
            if actual != self.config.dimensions {
                return Err(EngramError::Embedding(format!(
                    "Model output dimensions {} do not match configured dimensions {}",
                    actual, self.config.dimensions
                )));
            }
            Ok(())
        }

        /// Mean-pool token embeddings using an attention mask.
        pub fn mean_pool(token_embeddings: &Array2<f32>, attention_mask: &[i64]) -> Vec<f32> {
            let hidden_size = token_embeddings.ncols();
            let mut sum = vec![0.0_f32; hidden_size];
            let mut count = 0_f32;

            for (row_idx, mask_val) in attention_mask
                .iter()
                .take(token_embeddings.nrows())
                .enumerate()
            {
                if *mask_val == 1 {
                    let row = token_embeddings.row(row_idx);
                    for (s, &v) in sum.iter_mut().zip(row.iter()) {
                        *s += v;
                    }
                    count += 1.0;
                }
            }

            if count > 0.0 {
                for s in &mut sum {
                    *s /= count;
                }
            }

            sum
        }

        /// L2-normalize a vector in place.
        pub fn l2_normalize(v: &mut [f32]) {
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in v.iter_mut() {
                    *x /= norm;
                }
            }
        }
    }

    impl Embedder for OnnxEmbedder {
        fn embed(&self, text: &str) -> Result<Vec<f32>> {
            let (input_ids, attention_mask, token_type_ids) = self.encode(text)?;
            self.run_inference(&input_ids, &attention_mask, &token_type_ids)
        }

        fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            texts.iter().map(|text| self.embed(text)).collect()
        }

        fn dimensions(&self) -> usize {
            self.config.dimensions
        }

        fn model_name(&self) -> &str {
            &self.config.model_name
        }
    }

    fn mean_pool_normalized(token_embeddings: &Array2<f32>, attention_mask: &[i64]) -> Vec<f32> {
        let mut pooled = OnnxEmbedder::mean_pool(token_embeddings, attention_mask);
        OnnxEmbedder::l2_normalize(&mut pooled);
        pooled
    }

    fn l2_normalized(mut embedding: Vec<f32>) -> Vec<f32> {
        OnnxEmbedder::l2_normalize(&mut embedding);
        embedding
    }

    fn truncate_for_error(text: &str, max_chars: usize) -> String {
        let mut chars = text.chars();
        let truncated: String = chars.by_ref().take(max_chars).collect();
        if chars.next().is_some() {
            format!("{truncated}...")
        } else {
            truncated
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::{Mutex as TestMutex, OnceLock};

        fn env_lock() -> std::sync::MutexGuard<'static, ()> {
            static LOCK: OnceLock<TestMutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| TestMutex::new(())).lock().unwrap()
        }

        #[test]
        fn test_resolve_model_dir_prefers_config_path() {
            let _guard = env_lock();
            std::env::set_var("ENGRAM_ONNX_MODEL_DIR", "/env/model");
            let resolved = resolve_model_dir(Some("/config/model"));
            std::env::remove_var("ENGRAM_ONNX_MODEL_DIR");
            assert_eq!(resolved, PathBuf::from("/config/model"));
        }

        #[test]
        fn test_resolve_model_dir_uses_env_when_config_missing() {
            let _guard = env_lock();
            std::env::set_var("ENGRAM_ONNX_MODEL_DIR", "/env/model");
            let resolved = resolve_model_dir(None);
            std::env::remove_var("ENGRAM_ONNX_MODEL_DIR");
            assert_eq!(resolved, PathBuf::from("/env/model"));
        }

        #[test]
        fn test_resolve_model_dir_uses_default_when_unset() {
            let _guard = env_lock();
            std::env::remove_var("ENGRAM_ONNX_MODEL_DIR");
            let resolved = resolve_model_dir(None);
            assert_eq!(
                resolved.file_name().and_then(|s| s.to_str()),
                Some(DEFAULT_MODEL_NAME)
            );
        }

        #[test]
        fn test_from_dir_errors_when_model_missing() {
            let missing =
                std::env::temp_dir().join(format!("engram-missing-model-{}", std::process::id()));
            let err = match OnnxEmbedder::from_dir(&missing) {
                Ok(_) => panic!("missing model should error"),
                Err(err) => err,
            };
            let msg = err.to_string();
            assert!(msg.contains("model.onnx"), "{msg}");
            assert!(msg.contains("engram-cli model download"), "{msg}");
        }

        #[test]
        fn test_mean_pool_basic() {
            let embeddings = ndarray::array![[1.0_f32, 2.0], [3.0, 4.0], [0.0, 0.0],];
            let mask = vec![1i64, 1, 0];
            let pooled = OnnxEmbedder::mean_pool(&embeddings, &mask);
            assert!((pooled[0] - 2.0).abs() < 1e-6);
            assert!((pooled[1] - 3.0).abs() < 1e-6);
        }

        #[test]
        fn test_l2_normalize_unit_vector() {
            let mut v = vec![3.0_f32, 4.0];
            OnnxEmbedder::l2_normalize(&mut v);
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-6);
        }

        #[test]
        fn test_l2_normalize_zero_vector() {
            let mut v = vec![0.0_f32, 0.0, 0.0];
            OnnxEmbedder::l2_normalize(&mut v);
            assert!(v.iter().all(|&x| x == 0.0));
        }
    }
}

#[cfg(feature = "onnx-embed")]
pub use inner::{resolve_model_dir, OnnxConfig, OnnxEmbedder};
