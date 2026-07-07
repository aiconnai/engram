use super::*;

/// Compute the normalized SHA-256 hash used for deduplication and persistence checks.
///
/// Normalization lowercases and collapses whitespace, so content equivalent under
/// dedup rules maps to the same hash.
pub fn compute_content_hash(content: &str) -> String {
    let normalized = content
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Compute the raw SHA-256 hash of content bytes without any normalization.
///
/// Used for sync detection where case and whitespace differences must be preserved
/// (e.g. detecting case-only edits in markdown import/export).
pub fn compute_content_hash_raw(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Compute a dedupe hash.
///
/// Kept as an explicit semantic alias for existing callers; currently it delegates
/// to [`compute_content_hash`].
pub fn compute_dedup_hash(content: &str) -> String {
    compute_content_hash(content)
}
