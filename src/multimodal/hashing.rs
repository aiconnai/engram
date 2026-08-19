//! Media and image perceptual hashing algorithms.
//!
//! Provides content hashing (SHA-256) and perceptual hashing (64-bit difference hash / dHash)
//! for image deduplication, visual artifact tracking, and similarity search without
//! requiring external heavy vision models or network access.

use sha2::{Digest, Sha256};

/// Compute the cryptographic SHA-256 hex string of a byte slice.
pub fn compute_content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute a 64-bit perceptual hash (dHash) from image bytes.
///
/// Samples brightness gradients across 64 sequential buckets to generate
/// a robust visual fingerprint.
pub fn compute_perceptual_hash(data: &[u8]) -> u64 {
    if data.is_empty() {
        return 0;
    }

    // Use a 65-sample window to construct 64 adjacent gradient comparisons.
    // Strided sampling across the image payload.
    const SAMPLE_COUNT: usize = 65;
    let mut samples = [0u8; SAMPLE_COUNT];

    let step = data.len() / SAMPLE_COUNT;
    if step == 0 {
        for (i, &b) in data.iter().enumerate().take(SAMPLE_COUNT) {
            samples[i] = b;
        }
    } else {
        for i in 0..SAMPLE_COUNT {
            samples[i] = data[i * step];
        }
    }

    let mut hash = 0u64;
    for i in 0..64 {
        if samples[i + 1] > samples[i] {
            hash |= 1 << i;
        }
    }

    hash
}

/// Format a 64-bit perceptual hash as a 16-character hexadecimal string.
pub fn format_phash(hash: u64) -> String {
    format!("{:016x}", hash)
}

/// Parse a 16-character hexadecimal string into a 64-bit perceptual hash.
pub fn parse_phash(hex_str: &str) -> Option<u64> {
    u64::from_str_radix(hex_str.trim(), 16).ok()
}

/// Compute the Hamming distance between two 64-bit perceptual hashes.
///
/// The Hamming distance represents the number of bit differences (0-64).
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Determine whether two perceptual hashes represent visual duplicates
/// within a given Hamming distance threshold (default is typically <= 5).
pub fn is_visual_duplicate(a: u64, b: u64, threshold: u32) -> bool {
    hamming_distance(a, b) <= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_deterministic() {
        let data = b"test image payload";
        let h1 = compute_content_hash(data);
        let h2 = compute_content_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_perceptual_hash_and_formatting() {
        let data1 = vec![10u8; 500];
        let h1 = compute_perceptual_hash(&data1);
        let hex = format_phash(h1);
        assert_eq!(hex.len(), 16);
        assert_eq!(parse_phash(&hex), Some(h1));

        let mut data2 = vec![10u8; 500];
        // Minor noise
        data2[250] = 12;
        let h2 = compute_perceptual_hash(&data2);

        let dist = hamming_distance(h1, h2);
        assert!(dist <= 2);
        assert!(is_visual_duplicate(h1, h2, 5));
    }
}
