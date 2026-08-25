//! Integration tests for AAAK ultra-dense compression engine.

use engram::intelligence::aaak::{AaakCompressor, AaakMode};

#[test]
fn test_aaak_lossless_abbreviation_roundtrip() {
    let source =
        "The database configuration implementation requires authentication and permission checks.";
    let res = AaakCompressor::compress(source, AaakMode::Lossless);

    assert!(res.compressed.starts_with("[AAAK:v1:lossless]\n"));
    assert!(res.compressed.contains("db"));
    assert!(res.compressed.contains("cfg"));
    assert!(res.compressed.contains("impl"));
    assert!(res.compressed.contains("auth"));
    assert!(res.compressed.contains("perm"));

    let expanded = AaakCompressor::decompress(&res.compressed);
    assert!(expanded.contains("database"));
    assert!(expanded.contains("configuration"));
    assert!(expanded.contains("implementation"));
    assert!(expanded.contains("authentication"));
    assert!(expanded.contains("permission"));
}

#[test]
fn test_aaak_ultra_dense_token_savings() {
    let transcript = "Sure, I can help with that! In order to configure the database implementation, the assistant must verify the environment variables without exception. Furthermore, the repository authentication function requires asynchronous transaction processing, and the maximum connection timeout parameter should be set to five seconds. Please let me know if you need anything else!";
    let res = AaakCompressor::compress(transcript, AaakMode::UltraDense);

    assert!(res.compressed.starts_with("[AAAK:v1:dense]\n"));
    assert!(!res
        .compressed
        .to_lowercase()
        .contains("sure, i can help with that"));
    assert!(res.compressed.contains("cfg"));
    assert!(res.compressed.contains("db"));
    assert!(res.compressed.contains("impl"));
    assert!(res.compressed.contains("env"));
    assert!(res.compressed.contains("vars"));
    assert!(res.compressed.contains("w/o"));
    assert!(res.compressed.contains("repo"));
    assert!(res.compressed.contains("auth"));
    assert!(res.compressed.contains("fn"));
    assert!(res.compressed.contains("async"));
    assert!(res.compressed.contains("tx"));
    assert!(res.compression_ratio > 1.3);
    assert!(res.token_savings_pct > 5.0);
}

#[test]
fn test_aaak_jsonl_transcript_compaction() {
    let jsonl = r#"{"role":"user","content":"Please configure the authentication database."}
{"role":"assistant","content":"I have configured the authentication database with JWT tokens."}"#;

    let res = AaakCompressor::compress(jsonl, AaakMode::Transcript);

    assert!(res.compressed.starts_with("[AAAK:v1:transcript]\n"));
    assert!(res.compressed.contains("[U]"));
    assert!(res.compressed.contains("[A]"));
    assert!(res.compressed.contains("cfg"));
    assert!(res.compressed.contains("auth"));
    assert!(res.compressed.contains("db"));
    assert!(res.compressed.contains("cfg'd"));
    assert!(res.compressed.contains("w/"));
}
