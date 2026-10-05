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

/// Task C3: explicit Unicode and adversarial inputs (no panic, sizes reported on the
/// original bytes, lossless decompression of plain non-abbreviable text is the identity).
#[test]
fn test_aaak_unicode_and_adversarial_inputs_do_not_panic() {
    let cases = [
        "",
        " ",
        "ß ẞ İ ı ς Σ ﬃ \u{212a}",
        "e\u{301}\u{308}\u{327} combining marks",
        "👨\u{200d}👩\u{200d}👧\u{200d}👦 family and 🏳\u{fe0f}\u{200d}🌈 flag",
        "\u{202e}bidi override\u{202c} \u{2066}isolate\u{2069}",
        "\u{0}\u{7}\u{1b}\u{7f}\u{85}\u{200b}\u{feff}",
        "The database İİİ configuration ẞ implementation requires authentication",
        &"é".repeat(10_001),
    ];
    for text in cases {
        for mode in [AaakMode::Lossless, AaakMode::UltraDense] {
            let res = AaakCompressor::compress(text, mode);
            assert_eq!(res.original_bytes, text.len(), "input {text:?}");
            let _ = AaakCompressor::decompress(&res.compressed);
        }
    }

    // Text with nothing to abbreviate survives a lossless round trip verbatim.
    for text in ["ß ẞ İ", "👨\u{200d}👩\u{200d}👧", "e\u{301}\u{308}"] {
        let res = AaakCompressor::compress(text, AaakMode::Lossless);
        assert_eq!(AaakCompressor::decompress(&res.compressed), text);
    }
}
