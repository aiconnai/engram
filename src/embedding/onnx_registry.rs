//! Registry for downloadable local embedding models.

use std::path::PathBuf;

/// Metadata for a downloadable ONNX sentence-transformer model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelEntry {
    pub name: &'static str,
    pub model_url: &'static str,
    pub model_sha256: &'static str,
    pub tokenizer_url: &'static str,
    pub tokenizer_sha256: &'static str,
    pub dimensions: usize,
    pub max_seq_len: usize,
}

pub const DEFAULT_MODEL_NAME: &str = "minilm-l6-v2";

/// Built-in local embedding models.
pub const REGISTRY: &[ModelEntry] = &[ModelEntry {
    name: DEFAULT_MODEL_NAME,
    model_url:
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
    model_sha256: "6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452",
    tokenizer_url:
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json",
    tokenizer_sha256: "ed6e5972a684127fafc9d77ef37a4d2a051ea9e01aabfcf9979c41d344e000d7",
    dimensions: 384,
    max_seq_len: 256,
}];

/// Find a model entry by registry name.
pub fn find_model(name: &str) -> Option<&'static ModelEntry> {
    REGISTRY.iter().find(|entry| entry.name == name)
}

/// Return the default model cache directory.
pub fn default_model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("engram")
        .join("models")
        .join(DEFAULT_MODEL_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_contains_default_minilm_model() {
        let model = find_model(DEFAULT_MODEL_NAME).expect("default model should be registered");
        assert_eq!(model.dimensions, 384);
        assert_eq!(model.max_seq_len, 256);
        assert!(model.model_url.contains("all-MiniLM-L6-v2"));
        assert!(model.tokenizer_url.ends_with("/tokenizer.json"));
        assert_eq!(model.model_sha256.len(), 64);
        assert_eq!(model.tokenizer_sha256.len(), 64);
    }

    #[test]
    fn test_default_model_dir_ends_with_registry_name() {
        let dir = default_model_dir();
        assert_eq!(
            dir.file_name().and_then(|s| s.to_str()),
            Some(DEFAULT_MODEL_NAME)
        );
    }
}
