use super::*;

#[test]
fn test_text_similarity() {
    let a = "The quick brown fox jumps over the lazy dog";
    let b = "The quick brown fox jumps over the lazy cat";
    let c = "Something completely different";

    let sim_ab = calculate_text_similarity(a, b);
    let sim_ac = calculate_text_similarity(a, c);

    assert!(sim_ab > 0.8, "Similar texts should have high similarity");
    assert!(sim_ac < 0.3, "Different texts should have low similarity");
}

#[test]
fn test_cosine_similarity() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let c = vec![0.0, 1.0, 0.0];

    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    assert!(cosine_similarity(&a, &c).abs() < 0.001);
}

#[test]
fn test_conflict_type_parsing() {
    assert_eq!(
        "contradiction".parse::<ConflictType>().unwrap(),
        ConflictType::Contradiction
    );
    assert_eq!(
        "duplicate".parse::<ConflictType>().unwrap(),
        ConflictType::Duplicate
    );
}
