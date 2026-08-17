//! Accelerated Vector Mathematics & SIMD-friendly Kernels (RFC 0005)
//!
//! Provides high-throughput, autovectorization-friendly dot product, cosine similarity,
//! and Euclidean distance calculations with 8-way loop unrolling for AVX2 / ARM NEON.

use serde::{Deserialize, Serialize};

/// Distance/similarity metric for vector search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VectorMetric {
    /// Cosine distance: `1.0 - cosine_similarity(a, b)`. Range [0.0, 2.0].
    #[default]
    Cosine,
    /// Inverted dot product for normalized vectors: `-dot_product(a, b)`.
    DotProduct,
    /// Euclidean distance ($L_2$ norm): `||a - b||_2`. Range [0.0, inf).
    Euclidean,
}

impl VectorMetric {
    /// Compute distance between two vectors according to this metric (smaller = more similar).
    #[inline]
    pub fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        match self {
            Self::Cosine => cosine_distance(a, b),
            Self::DotProduct => -dot_product(a, b),
            Self::Euclidean => euclidean_distance(a, b),
        }
    }

    /// Compute similarity score between two vectors according to this metric (larger = more similar).
    #[inline]
    pub fn similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        match self {
            Self::Cosine => cosine_similarity(a, b),
            Self::DotProduct => dot_product(a, b),
            Self::Euclidean => {
                let d = euclidean_distance(a, b);
                1.0 / (1.0 + d)
            }
        }
    }
}

/// Compute the dot product between two slices `a` and `b`.
///
/// Uses an 8-lane unrolled accumulator to enable compiler autovectorization
/// via FMA (Fused Multiply-Add) on AVX2 / AVX-512 and ARM NEON.
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    if len != b.len() || len == 0 {
        return 0.0;
    }

    let mut sum0 = 0.0f32;
    let mut sum1 = 0.0f32;
    let mut sum2 = 0.0f32;
    let mut sum3 = 0.0f32;
    let mut sum4 = 0.0f32;
    let mut sum5 = 0.0f32;
    let mut sum6 = 0.0f32;
    let mut sum7 = 0.0f32;

    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let remainder_a = chunks_a.remainder();
    let remainder_b = chunks_b.remainder();

    for (ca, cb) in chunks_a.zip(chunks_b) {
        sum0 += ca[0] * cb[0];
        sum1 += ca[1] * cb[1];
        sum2 += ca[2] * cb[2];
        sum3 += ca[3] * cb[3];
        sum4 += ca[4] * cb[4];
        sum5 += ca[5] * cb[5];
        sum6 += ca[6] * cb[6];
        sum7 += ca[7] * cb[7];
    }

    let mut total = (sum0 + sum1) + (sum2 + sum3) + (sum4 + sum5) + (sum6 + sum7);

    for (x, y) in remainder_a.iter().zip(remainder_b.iter()) {
        total += x * y;
    }

    total
}

/// Compute the $L_2$ norm (magnitude) squared of a vector: $\sum x_i^2$.
#[inline]
pub fn l2_norm_squared(v: &[f32]) -> f32 {
    dot_product(v, v)
}

/// Compute the $L_2$ norm (magnitude) of a vector: $\sqrt{\sum x_i^2}$.
#[inline]
pub fn l2_norm(v: &[f32]) -> f32 {
    l2_norm_squared(v).sqrt()
}

/// Compute cosine similarity between two vectors `a` and `b` in a single pass.
///
/// Returns a value in `[-1.0, 1.0]`. If lengths mismatch, slices are empty,
/// or either vector has zero magnitude, returns `0.0`.
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    if len != b.len() || len == 0 {
        return 0.0;
    }

    // 8-lane parallel accumulators for dot, norm_a_sq, norm_b_sq
    let mut dot0 = 0.0f32;
    let mut dot1 = 0.0f32;
    let mut dot2 = 0.0f32;
    let mut dot3 = 0.0f32;
    let mut dot4 = 0.0f32;
    let mut dot5 = 0.0f32;
    let mut dot6 = 0.0f32;
    let mut dot7 = 0.0f32;

    let mut na0 = 0.0f32;
    let mut na1 = 0.0f32;
    let mut na2 = 0.0f32;
    let mut na3 = 0.0f32;
    let mut na4 = 0.0f32;
    let mut na5 = 0.0f32;
    let mut na6 = 0.0f32;
    let mut na7 = 0.0f32;

    let mut nb0 = 0.0f32;
    let mut nb1 = 0.0f32;
    let mut nb2 = 0.0f32;
    let mut nb3 = 0.0f32;
    let mut nb4 = 0.0f32;
    let mut nb5 = 0.0f32;
    let mut nb6 = 0.0f32;
    let mut nb7 = 0.0f32;

    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let remainder_a = chunks_a.remainder();
    let remainder_b = chunks_b.remainder();

    for (ca, cb) in chunks_a.zip(chunks_b) {
        dot0 += ca[0] * cb[0];
        dot1 += ca[1] * cb[1];
        dot2 += ca[2] * cb[2];
        dot3 += ca[3] * cb[3];
        dot4 += ca[4] * cb[4];
        dot5 += ca[5] * cb[5];
        dot6 += ca[6] * cb[6];
        dot7 += ca[7] * cb[7];

        na0 += ca[0] * ca[0];
        na1 += ca[1] * ca[1];
        na2 += ca[2] * ca[2];
        na3 += ca[3] * ca[3];
        na4 += ca[4] * ca[4];
        na5 += ca[5] * ca[5];
        na6 += ca[6] * ca[6];
        na7 += ca[7] * ca[7];

        nb0 += cb[0] * cb[0];
        nb1 += cb[1] * cb[1];
        nb2 += cb[2] * cb[2];
        nb3 += cb[3] * cb[3];
        nb4 += cb[4] * cb[4];
        nb5 += cb[5] * cb[5];
        nb6 += cb[6] * cb[6];
        nb7 += cb[7] * cb[7];
    }

    let mut dot = (dot0 + dot1) + (dot2 + dot3) + (dot4 + dot5) + (dot6 + dot7);
    let mut norm_a_sq = (na0 + na1) + (na2 + na3) + (na4 + na5) + (na6 + na7);
    let mut norm_b_sq = (nb0 + nb1) + (nb2 + nb3) + (nb4 + nb5) + (nb6 + nb7);

    for (x, y) in remainder_a.iter().zip(remainder_b.iter()) {
        dot += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    let denom = norm_a_sq.sqrt() * norm_b_sq.sqrt();
    if denom == 0.0 {
        return 0.0;
    }

    (dot / denom).clamp(-1.0, 1.0)
}

/// Compute cosine distance between two vectors: `1.0 - cosine_similarity(a, b)`.
///
/// Guaranteed to be in `[0.0, 2.0]`.
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    (1.0 - cosine_similarity(a, b)).max(0.0)
}

/// Compute the squared Euclidean distance between two vectors: $\sum (a_i - b_i)^2$.
#[inline]
pub fn euclidean_distance_squared(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    if len != b.len() || len == 0 {
        return 0.0;
    }

    let mut sum0 = 0.0f32;
    let mut sum1 = 0.0f32;
    let mut sum2 = 0.0f32;
    let mut sum3 = 0.0f32;
    let mut sum4 = 0.0f32;
    let mut sum5 = 0.0f32;
    let mut sum6 = 0.0f32;
    let mut sum7 = 0.0f32;

    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let remainder_a = chunks_a.remainder();
    let remainder_b = chunks_b.remainder();

    for (ca, cb) in chunks_a.zip(chunks_b) {
        let d0 = ca[0] - cb[0];
        let d1 = ca[1] - cb[1];
        let d2 = ca[2] - cb[2];
        let d3 = ca[3] - cb[3];
        let d4 = ca[4] - cb[4];
        let d5 = ca[5] - cb[5];
        let d6 = ca[6] - cb[6];
        let d7 = ca[7] - cb[7];

        sum0 += d0 * d0;
        sum1 += d1 * d1;
        sum2 += d2 * d2;
        sum3 += d3 * d3;
        sum4 += d4 * d4;
        sum5 += d5 * d5;
        sum6 += d6 * d6;
        sum7 += d7 * d7;
    }

    let mut total = (sum0 + sum1) + (sum2 + sum3) + (sum4 + sum5) + (sum6 + sum7);

    for (x, y) in remainder_a.iter().zip(remainder_b.iter()) {
        let d = x - y;
        total += d * d;
    }

    total
}

/// Compute the Euclidean ($L_2$) distance between two vectors: $\sqrt{\sum (a_i - b_i)^2}$.
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    euclidean_distance_squared(a, b).sqrt()
}

/// In-place $L_2$ normalize a vector: $v \leftarrow v / ||v||_2$.
///
/// If norm is zero, vector is left unchanged.
pub fn l2_normalize(v: &mut [f32]) {
    let norm = l2_norm(v);
    if norm > 0.0 {
        let inv = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv;
        }
    }
}

/// Return a normalized copy of a vector.
pub fn l2_normalized(v: &[f32]) -> Vec<f32> {
    let mut out = v.to_vec();
    l2_normalize(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product_basic() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let b = [2.0, 1.0, 0.0, -1.0, 2.0, 3.0, 1.0, 0.0, 2.0];
        let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let actual = dot_product(&a, &b);
        assert!((actual - expected).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_properties() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [2.0, 4.0, 6.0, 8.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-5);

        let orthogonal_a = [1.0, 0.0];
        let orthogonal_b = [0.0, 1.0];
        assert!(cosine_similarity(&orthogonal_a, &orthogonal_b).abs() < 1e-5);

        let opposite_a = [1.0, 2.0];
        let opposite_b = [-1.0, -2.0];
        assert!((cosine_similarity(&opposite_a, &opposite_b) + 1.0).abs() < 1e-5);

        assert_eq!(cosine_similarity(&[], &[]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 6.0, 3.0];
        // dx=3, dy=4, dz=0 => dist = 5.0
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-5);
        assert!((euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn test_l2_normalization() {
        let mut v = [3.0, 4.0];
        l2_normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-5);
        assert!((v[1] - 0.8).abs() < 1e-5);
        assert!((l2_norm(&v) - 1.0).abs() < 1e-5);
    }
}
