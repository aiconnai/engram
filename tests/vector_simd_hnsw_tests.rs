//! Integration & Verification Tests for Vector SIMD Math and HNSW Index (RFC 0005)

use engram::search::hnsw::{HnswConfig, HnswIndex};
use engram::search::vector::{
    cosine_distance, cosine_similarity, dot_product, euclidean_distance,
    euclidean_distance_squared, l2_norm, l2_normalize, l2_normalized, VectorMetric,
};

// ---------------------------------------------------------------------------
// Reference implementations for exact correctness testing
// ---------------------------------------------------------------------------

fn ref_dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn ref_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = ref_dot_product(a, b);
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

fn generate_pseudo_vector(dim: usize, seed: u32) -> Vec<f32> {
    let mut vec = Vec::with_capacity(dim);
    let mut state = seed as u64;
    for _ in 0..dim {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = (((state >> 16) & 0xFFFF) as f32 / 65535.0) * 2.0 - 1.0;
        vec.push(val);
    }
    vec
}

// ---------------------------------------------------------------------------
// Vector Kernel Tests
// ---------------------------------------------------------------------------

#[test]
fn test_simd_dot_product_matches_reference_across_dimensions() {
    for dim in [1, 3, 7, 8, 9, 15, 16, 31, 32, 64, 128, 384, 768, 1536] {
        let v1 = generate_pseudo_vector(dim, 100);
        let v2 = generate_pseudo_vector(dim, 200);

        let expected = ref_dot_product(&v1, &v2);
        let actual = dot_product(&v1, &v2);

        let diff = (actual - expected).abs();
        let max_mag = expected.abs().max(1.0);
        assert!(
            diff / max_mag < 1e-4,
            "dot_product mismatch at dim {dim}: expected {expected}, got {actual}"
        );
    }
}

#[test]
fn test_simd_cosine_similarity_matches_reference_across_dimensions() {
    for dim in [1, 7, 8, 16, 64, 384, 768, 1536] {
        let v1 = generate_pseudo_vector(dim, 42 + dim as u32);
        let v2 = generate_pseudo_vector(dim, 99 + dim as u32);

        let expected = ref_cosine_similarity(&v1, &v2);
        let actual = cosine_similarity(&v1, &v2);

        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-4,
            "cosine_similarity mismatch at dim {dim}: expected {expected}, got {actual}"
        );
    }
}

#[test]
fn test_cosine_similarity_boundary_cases() {
    // Identical
    let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-5);
    assert!(cosine_distance(&v, &v) < 1e-5);

    // Opposite
    let neg_v: Vec<f32> = v.iter().map(|x| -x).collect();
    assert!((cosine_similarity(&v, &neg_v) + 1.0).abs() < 1e-5);
    assert!((cosine_distance(&v, &neg_v) - 2.0).abs() < 1e-5);

    // Zero vector
    let zero = vec![0.0; 5];
    assert_eq!(cosine_similarity(&zero, &v), 0.0);
    assert_eq!(cosine_similarity(&v, &zero), 0.0);

    // Empty and length mismatch
    assert_eq!(cosine_similarity(&[], &[]), 0.0);
    assert_eq!(cosine_similarity(&[1.0, 2.0], &[1.0]), 0.0);
}

#[test]
fn test_euclidean_distance_kernels() {
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![4.0, 6.0, 3.0, 4.0]; // diffs: 3, 4, 0, 0 => sum of squares = 25

    assert!((euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-5);
    assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-5);

    // Length mismatch must return INFINITY, not 0.0
    assert!(euclidean_distance_squared(&a, &[1.0, 2.0]).is_infinite());
    assert!(euclidean_distance(&a, &[1.0, 2.0]).is_infinite());
    assert_eq!(euclidean_distance_squared(&[], &[]), 0.0);
}

#[test]
fn test_l2_normalization_and_norm() {
    let mut v = vec![0.0, 3.0, 4.0, 0.0];
    assert!((l2_norm(&v) - 5.0).abs() < 1e-5);

    l2_normalize(&mut v);
    assert!((l2_norm(&v) - 1.0).abs() < 1e-5);
    assert!((v[1] - 0.6).abs() < 1e-5);
    assert!((v[2] - 0.8).abs() < 1e-5);

    let norm_copy = l2_normalized(&[10.0, 0.0]);
    assert_eq!(norm_copy, vec![1.0, 0.0]);

    // Subnormal or zero norm vector should not panic or produce NaN
    let mut zero_vec = vec![0.0, 0.0];
    l2_normalize(&mut zero_vec);
    assert_eq!(zero_vec, vec![0.0, 0.0]);
}

#[test]
fn test_vector_metric_scoring() {
    let a = vec![1.0, 0.0];
    let b = vec![1.0, 0.0];
    let c = vec![0.0, 1.0];

    assert_eq!(VectorMetric::Cosine.distance(&a, &b), 0.0);
    assert!((VectorMetric::Cosine.similarity(&a, &b) - 1.0).abs() < 1e-5);

    assert!((VectorMetric::Cosine.distance(&a, &c) - 1.0).abs() < 1e-5);
    assert!(VectorMetric::Cosine.similarity(&a, &c).abs() < 1e-5);

    assert_eq!(VectorMetric::DotProduct.distance(&a, &b), -1.0);
    assert_eq!(VectorMetric::DotProduct.similarity(&a, &b), 1.0);

    assert_eq!(VectorMetric::Euclidean.distance(&a, &b), 0.0);
    assert_eq!(VectorMetric::Euclidean.similarity(&a, &b), 1.0);
}

// ---------------------------------------------------------------------------
// HNSW Index Tests
// ---------------------------------------------------------------------------

#[test]
fn test_hnsw_insertion_and_search_basic() {
    let config = HnswConfig::new(4, VectorMetric::Cosine);
    let mut index = HnswIndex::new(config);

    assert!(index.is_empty());
    assert_eq!(index.len(), 0);

    // Mismatched dimension vector must be rejected cleanly
    index.insert(99, &[1.0, 0.0]);
    assert_eq!(index.len(), 0);

    index.insert(1, &[1.0, 0.0, 0.0, 0.0]);
    index.insert(2, &[0.0, 1.0, 0.0, 0.0]);
    index.insert(3, &[0.0, 0.0, 1.0, 0.0]);
    index.insert(4, &[0.0, 0.0, 0.0, 1.0]);

    assert_eq!(index.len(), 4);
    assert!(index.contains(&1));
    assert!(index.contains(&2));
    assert!(!index.contains(&999));

    // Search with wrong dimension must return empty
    assert!(index.search(&[1.0, 0.0], 2, None).is_empty());

    let query = [0.9, 0.1, 0.0, 0.0];
    let results = index.search(&query, 2, None);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, 1);
    assert!(results[0].similarity > results[1].similarity);
}

#[test]
fn test_hnsw_update_existing_id() {
    let config = HnswConfig::new(3, VectorMetric::Cosine);
    let mut index = HnswIndex::new(config);

    index.insert(10, &[1.0, 0.0, 0.0]);
    index.insert(20, &[0.0, 1.0, 0.0]);

    // Query close to 10
    let res = index.search(&[0.9, 0.0, 0.0], 1, None);
    assert_eq!(res[0].id, 10);

    // Update ID 10 to point in opposite direction
    index.insert(10, &[0.0, 0.0, 1.0]);

    let res2 = index.search(&[0.0, 0.0, 0.9], 1, None);
    assert_eq!(res2[0].id, 10);
}

#[test]
fn test_hnsw_high_dimensional_recall_accuracy() {
    let dim = 128;
    let n_items = 200;
    let config = HnswConfig::new(dim, VectorMetric::Cosine).with_ef(64, 48);

    let mut index = HnswIndex::new(config);
    let mut vectors = Vec::with_capacity(n_items);

    for id in 0..n_items {
        let vec = l2_normalized(&generate_pseudo_vector(dim, id as u32 + 1000));
        index.insert(id as i64, &vec);
        vectors.push((id as i64, vec));
    }

    assert_eq!(index.len(), n_items);

    // Run 20 queries and measure recall@5 against brute force
    let k = 5;
    let mut total_hits = 0;

    for q_idx in 0..20 {
        let query = l2_normalized(&generate_pseudo_vector(dim, q_idx as u32 + 9999));

        // Ground truth brute-force nearest neighbors
        let mut ground_truth: Vec<(i64, f32)> = vectors
            .iter()
            .map(|(id, vec)| (*id, cosine_similarity(&query, vec)))
            .collect();
        ground_truth.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_k_ground_truth: Vec<i64> = ground_truth.iter().take(k).map(|(id, _)| *id).collect();

        // HNSW search
        let hnsw_results = index.search(&query, k, Some(48));
        let hnsw_ids: Vec<i64> = hnsw_results.iter().map(|r| r.id).collect();

        for id in &hnsw_ids {
            if top_k_ground_truth.contains(id) {
                total_hits += 1;
            }
        }
    }

    let recall = (total_hits as f32) / (20.0 * k as f32);
    assert!(
        recall >= 0.90,
        "HNSW recall@5 ({recall:.3}) fell below target 0.90"
    );
}

#[test]
fn test_hnsw_euclidean_metric() {
    let config = HnswConfig::new(3, VectorMetric::Euclidean).with_ef(32, 32);
    let mut index = HnswIndex::new(config);

    index.insert(1, &[0.0, 0.0, 0.0]);
    index.insert(2, &[1.0, 1.0, 1.0]);
    index.insert(3, &[10.0, 10.0, 10.0]);

    let res = index.search(&[0.1, 0.1, 0.1], 2, None);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].id, 1);
    assert_eq!(res[1].id, 2);
}
