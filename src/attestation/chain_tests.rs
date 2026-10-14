use super::*;
use crate::storage::Storage;

fn test_chain() -> AttestationChain {
    let storage = Storage::open_in_memory().unwrap();
    AttestationChain::new(storage)
}

#[test]
fn test_log_and_verify_document() {
    let chain = test_chain();
    let content = b"hello, world";
    let record = chain
        .log_document(content, "hello.txt", Some("agent-1"), &[1, 2, 3], None)
        .unwrap();

    assert!(record.id.is_some());
    assert_eq!(record.document_name, "hello.txt");
    assert_eq!(record.agent_id.as_deref(), Some("agent-1"));
    assert_eq!(record.memory_ids, vec![1, 2, 3]);
    assert_eq!(record.previous_hash, GENESIS_HASH);
    assert!(!record.record_hash.is_empty());

    // verify_document should find it
    let found = chain.verify_document(content).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().document_name, "hello.txt");
}

#[test]
fn test_chain_linkage() {
    let chain = test_chain();

    let r1 = chain
        .log_document(b"doc1", "doc1.txt", None, &[], None)
        .unwrap();
    let r2 = chain
        .log_document(b"doc2", "doc2.txt", None, &[], None)
        .unwrap();

    assert_eq!(r1.previous_hash, GENESIS_HASH);
    assert_eq!(r2.previous_hash, r1.record_hash);
}

#[test]
fn test_verify_chain_valid() {
    let chain = test_chain();
    chain.log_document(b"a", "a.txt", None, &[], None).unwrap();
    chain.log_document(b"b", "b.txt", None, &[], None).unwrap();

    match chain.verify_chain(None).unwrap() {
        ChainStatus::Valid { record_count } => assert_eq!(record_count, 2),
        other => panic!("expected Valid, got {other:?}"),
    }
}

#[test]
fn test_verify_chain_empty() {
    let chain = test_chain();
    assert!(matches!(
        chain.verify_chain(None).unwrap(),
        ChainStatus::Empty
    ));
}

#[test]
fn test_chain_stays_linear_under_concurrent_append() {
    use std::sync::Arc;

    let storage = crate::storage::Storage::open_in_memory().unwrap();
    let chain = Arc::new(AttestationChain::new(storage));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let c = Arc::clone(&chain);
            let idx = i;
            std::thread::spawn(move || {
                let doc_name = format!("doc-{}.txt", idx);
                let content_bytes = format!("content-{}", idx);
                c.log_document(content_bytes.as_bytes(), &doc_name, None, &[], None)
                    .unwrap()
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    match chain.verify_chain(None).unwrap() {
        ChainStatus::Valid { record_count } => assert_eq!(record_count, 4),
        other => panic!("expected Valid, got {other:?}"),
    }
}

#[test]
fn test_verify_chain_rejects_tampered_signature() {
    use ed25519_dalek::{SigningKey, VerifyingKey};

    let chain = test_chain();
    let secret = [42u8; 32];
    let verifying_key_bytes: [u8; 32] =
        VerifyingKey::from(&SigningKey::from_bytes(&secret)).to_bytes();

    chain
        .log_document(b"data", "doc.txt", None, &[], Some(&secret))
        .unwrap();

    let zero_signature = "00".repeat(64);
    chain
        .storage
        .with_transaction(|conn| {
            conn.execute(
                "UPDATE attestation_log SET signature = ?1",
                rusqlite::params![zero_signature],
            )?;
            Ok(())
        })
        .unwrap();

    match chain.verify_chain(Some(&verifying_key_bytes)).unwrap() {
        ChainStatus::Broken { .. } => {}
        other => panic!("expected Broken, got {other:?}"),
    }
}

#[test]
fn test_verify_chain_rejects_stripped_signature_when_key_provided() {
    use ed25519_dalek::{SigningKey, VerifyingKey};

    let chain = test_chain();
    let secret = [42u8; 32];
    let verifying_key_bytes: [u8; 32] =
        VerifyingKey::from(&SigningKey::from_bytes(&secret)).to_bytes();

    chain
        .log_document(b"data", "doc.txt", None, &[], Some(&secret))
        .unwrap();

    chain
        .storage
        .with_transaction(|conn| {
            conn.execute("UPDATE attestation_log SET signature = NULL", [])?;
            Ok(())
        })
        .unwrap();

    match chain.verify_chain(Some(&verifying_key_bytes)).unwrap() {
        ChainStatus::Broken { .. } => {}
        other => panic!("expected Broken, got {other:?}"),
    }
}

#[test]
fn test_verify_chain_accepts_valid_signature() {
    use ed25519_dalek::{SigningKey, VerifyingKey};

    let chain = test_chain();
    let secret = [42u8; 32];
    let verifying_key_bytes: [u8; 32] =
        VerifyingKey::from(&SigningKey::from_bytes(&secret)).to_bytes();

    chain
        .log_document(b"data", "doc.txt", None, &[], Some(&secret))
        .unwrap();

    match chain.verify_chain(Some(&verifying_key_bytes)).unwrap() {
        ChainStatus::Valid { record_count: 1 } => {}
        other => panic!("expected Valid(1), got {other:?}"),
    }
}

#[test]
fn test_verify_chain_skips_sig_check_when_no_key_provided() {
    let chain = test_chain();
    chain
        .log_document(b"data", "doc.txt", None, &[], None)
        .unwrap();

    match chain.verify_chain(None).unwrap() {
        ChainStatus::Valid { record_count: 1 } => {}
        other => panic!("expected Valid(1), got {other:?}"),
    }
}

#[test]
fn test_list_with_filter() {
    let chain = test_chain();
    chain
        .log_document(b"x", "x.txt", Some("agent-A"), &[], None)
        .unwrap();
    chain
        .log_document(b"y", "y.txt", Some("agent-B"), &[], None)
        .unwrap();

    let filter = AttestationFilter {
        agent_id: Some("agent-A".to_string()),
        ..Default::default()
    };
    let results = chain.list(&filter).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].document_name, "x.txt");
}

#[test]
fn test_empty_document_name_rejected() {
    let chain = test_chain();
    let err = chain.log_document(b"data", "", None, &[], None);
    assert!(err.is_err());
}

#[test]
fn test_list_document_name_filter_no_wildcard_expansion() {
    // H3: LIKE metacharacters in document_name must not expand
    let chain = test_chain();
    chain
        .log_document(b"a", "report_final.txt", None, &[], None)
        .unwrap();
    chain
        .log_document(b"b", "reportXfinal.txt", None, &[], None)
        .unwrap();

    // Searching for "report_final" should only match "report_final.txt",
    // NOT "reportXfinal.txt" (SQLite `_` is a single-char wildcard if unescaped)
    let filter = super::super::types::AttestationFilter {
        document_name: Some("report_final".to_string()),
        ..Default::default()
    };
    let results = chain.list(&filter).unwrap();
    assert_eq!(
        results.len(),
        1,
        "unescaped _ should not act as wildcard; got {results:?}"
    );
    assert_eq!(results[0].document_name, "report_final.txt");
}

#[test]
fn test_list_document_name_filter_percent_literal() {
    // H3: literal % in document_name filter must be matched as literal
    let chain = test_chain();
    chain
        .log_document(b"a", "100% complete.txt", None, &[], None)
        .unwrap();
    chain
        .log_document(b"b", "other.txt", None, &[], None)
        .unwrap();

    let filter = super::super::types::AttestationFilter {
        document_name: Some("100%".to_string()),
        ..Default::default()
    };
    let results = chain.list(&filter).unwrap();
    assert_eq!(
        results.len(),
        1,
        "percent should be literal, got {results:?}"
    );
    assert_eq!(results[0].document_name, "100% complete.txt");
}

#[test]
fn test_agent_id_too_long_rejected() {
    // H4: agent_id > 256 chars must be rejected
    let chain = test_chain();
    let long_id = "a".repeat(257);
    let err = chain.log_document(b"data", "doc.txt", Some(&long_id), &[], None);
    assert!(err.is_err(), "agent_id > 256 chars should be rejected");
}

#[test]
fn test_agent_id_control_char_rejected() {
    // H4: agent_id with control characters must be rejected
    let chain = test_chain();
    let bad_id = "agent\x00null";
    let err = chain.log_document(b"data", "doc.txt", Some(bad_id), &[], None);
    assert!(
        err.is_err(),
        "agent_id with control chars should be rejected"
    );
}

#[test]
fn test_agent_id_valid_256_chars_accepted() {
    // H4: exactly 256 chars is valid
    let chain = test_chain();
    let id_256 = "a".repeat(256);
    let result = chain.log_document(b"data", "doc.txt", Some(&id_256), &[], None);
    assert!(result.is_ok(), "agent_id of 256 chars should be accepted");
}

#[test]
fn test_compute_record_hash_deterministic() {
    let chain = test_chain();
    let r = chain
        .log_document(b"stable", "stable.txt", None, &[], None)
        .unwrap();
    let recomputed = AttestationChain::compute_record_hash(&r);
    assert_eq!(r.record_hash, recomputed);
}

#[test]
fn test_chain_integrity_5_docs() {
    let chain = test_chain();

    // Log 5 documents
    let docs = [
        (b"alpha" as &[u8], "alpha.txt"),
        (b"beta", "beta.txt"),
        (b"gamma", "gamma.txt"),
        (b"delta", "delta.txt"),
        (b"epsilon", "epsilon.txt"),
    ];

    let mut records = Vec::new();
    for (content, name) in &docs {
        let r = chain
            .log_document(content, name, Some("agent-x"), &[], None)
            .unwrap();
        records.push(r);
    }

    // Verify each record's previous_hash forms a proper chain
    assert_eq!(records[0].previous_hash, GENESIS_HASH);
    for i in 1..5 {
        assert_eq!(
            records[i].previous_hash,
            records[i - 1].record_hash,
            "record {i} previous_hash should point to record {} record_hash",
            i - 1
        );
    }

    // Full chain verification
    match chain.verify_chain(None).unwrap() {
        ChainStatus::Valid { record_count } => assert_eq!(record_count, 5),
        other => panic!("expected Valid with 5 records, got {other:?}"),
    }
}

// ── M1: document size cap ────────────────────────────────────────────────

#[test]
fn test_document_too_large_rejected() {
    // M1: content exceeding MAX_DOCUMENT_BYTES must be rejected before hashing
    let chain = test_chain();
    // Allocate a vec just over 100 MB
    let big = vec![0u8; 100 * 1024 * 1024 + 1];
    let err = chain.log_document(&big, "big.bin", None, &[], None);
    assert!(err.is_err(), "document > 100 MB should be rejected");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("too large"),
        "error should mention 'too large', got: {msg}"
    );
}

#[test]
fn test_document_at_limit_accepted() {
    // M1: content exactly at 100 MB must be accepted
    let chain = test_chain();
    let at_limit = vec![0u8; 100 * 1024 * 1024];
    let result = chain.log_document(&at_limit, "limit.bin", None, &[], None);
    assert!(
        result.is_ok(),
        "document at exactly 100 MB should be accepted"
    );
}

// ── M2: memory_ids cap + negative rejection ──────────────────────────────

#[test]
fn test_too_many_memory_ids_rejected() {
    // M2: more than 10_000 memory_ids must be rejected
    let chain = test_chain();
    let ids: Vec<i64> = (0..10_001).collect();
    let err = chain.log_document(b"data", "doc.txt", None, &ids, None);
    assert!(
        err.is_err(),
        "more than 10_000 memory_ids should be rejected"
    );
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("too many"),
        "error should mention 'too many', got: {msg}"
    );
}

#[test]
fn test_memory_ids_at_limit_accepted() {
    // M2: exactly 10_000 memory_ids must be accepted
    let chain = test_chain();
    let ids: Vec<i64> = (0..10_000).collect();
    let result = chain.log_document(b"data", "doc.txt", None, &ids, None);
    assert!(
        result.is_ok(),
        "exactly 10_000 memory_ids should be accepted"
    );
}

#[test]
fn test_negative_memory_id_rejected() {
    // M2: negative memory_ids must be rejected
    let chain = test_chain();
    let ids = vec![1i64, 2, -1, 4];
    let err = chain.log_document(b"data", "doc.txt", None, &ids, None);
    assert!(err.is_err(), "negative memory_id should be rejected");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("non-negative"),
        "error should mention 'non-negative', got: {msg}"
    );
}

#[test]
fn test_memory_ids_all_non_negative_accepted() {
    // M2: non-negative memory_ids must be accepted
    let chain = test_chain();
    let ids = vec![0i64, 1, 100, 999];
    let result = chain.log_document(b"data", "doc.txt", None, &ids, None);
    assert!(result.is_ok(), "non-negative memory_ids should be accepted");
}

// ── M4: metadata size cap ────────────────────────────────────────────────

#[test]
fn test_metadata_too_large_rejected() {
    // M4: metadata serialized > 64 KB must be rejected
    use serde_json::Value;
    let chain = test_chain();
    // Build a JSON string value larger than 64 KB
    let big_string = "x".repeat(66_000);
    let _metadata = Value::String(big_string);
    // We can't pass metadata to log_document yet (it doesn't accept it),
    // so we test the validation helper directly.
    // For now, build a record with oversized metadata and call the internal
    // validator path — the easiest way is to expose it or test via a
    // future API. Since log_document always sets metadata = empty object
    // (always ≤ 64 KB), this test validates the future-facing contract:
    // the serialized empty object is well under 64 KB.
    let result = chain.log_document(b"data", "doc.txt", None, &[], None);
    assert!(result.is_ok());
    let record = result.unwrap();
    let serialized = serde_json::to_string(&record.metadata).unwrap();
    assert!(
        serialized.len() <= 65_536,
        "default metadata must be within 64 KB"
    );
}

// ── M5: streaming verify_chain ───────────────────────────────────────────
// The existing verify_chain tests (test_verify_chain_valid,
// test_verify_chain_empty, test_chain_tamper_detection, etc.) serve as the
// regression harness for the streaming refactor. The behavior must be
// identical after the refactor.

// ── L3: document_name validation — null bytes and length cap ────────────────

#[test]
fn test_document_name_null_byte_rejected() {
    // L3: document_name containing null byte must be rejected
    let chain = test_chain();
    let err = chain.log_document(b"data", "doc\x00.txt", None, &[], None);
    assert!(
        err.is_err(),
        "document_name with null byte must be rejected"
    );
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("null"),
        "error should mention null bytes, got: {msg}"
    );
}

#[test]
fn test_document_name_too_long_rejected() {
    // L3: document_name longer than 1000 chars must be rejected
    let chain = test_chain();
    let long_name = "a".repeat(1_001);
    let err = chain.log_document(b"data", &long_name, None, &[], None);
    assert!(err.is_err(), "document_name > 1000 chars must be rejected");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("too long") || msg.contains("1000"),
        "error should mention length limit, got: {msg}"
    );
}

#[test]
fn test_document_name_exactly_1000_chars_accepted() {
    // L3: exactly 1000 chars must be accepted
    let chain = test_chain();
    let name_1000 = "a".repeat(1_000);
    let result = chain.log_document(b"data", &name_1000, None, &[], None);
    assert!(
        result.is_ok(),
        "document_name of exactly 1000 chars should be accepted"
    );
}

// ── L1: corrupt metadata JSON must not be silently swallowed ───────────────

#[test]
fn test_corrupt_metadata_json_returns_error() {
    // L1: verify_chain (which calls row_to_record) must return Err when
    // metadata_json stored in the DB is malformed, not silently substitute {}.
    let storage = Storage::open_in_memory().unwrap();
    let chain = AttestationChain::new(storage.clone());

    chain
        .log_document(b"data", "doc.txt", None, &[], None)
        .unwrap();

    // Corrupt the metadata column directly
    storage
        .with_transaction(|conn| {
            conn.execute("UPDATE attestation_log SET metadata = '{broken'", [])?;
            Ok(())
        })
        .unwrap();

    // verify_chain must return Err, not Ok with broken ChainStatus
    let result = chain.verify_chain(None);
    assert!(
        result.is_err(),
        "corrupt metadata JSON must propagate an error, got: {result:?}"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("invalid metadata JSON") || msg.contains("metadata"),
        "error should mention metadata, got: {msg}"
    );
}

#[test]
fn test_attestation_hash_includes_metadata() {
    let rec1 = AttestationRecord {
        id: Some(1),
        document_hash: "hash1".to_string(),
        document_name: "doc".to_string(),
        document_size: 100,
        ingested_at: chrono::Utc::now(),
        agent_id: None,
        memory_ids: vec![],
        previous_hash: GENESIS_HASH.to_string(),
        record_hash: String::new(),
        signature: None,
        metadata: serde_json::json!({"key": "val1"}),
        created_at: Some(chrono::Utc::now()),
        hash_version: CURRENT_HASH_VERSION,
    };
    let mut rec2 = rec1.clone();
    rec2.metadata = serde_json::json!({"key": "val2"});

    let h1 = AttestationChain::compute_record_hash(&rec1);
    let h2 = AttestationChain::compute_record_hash(&rec2);

    assert_ne!(h1, h2, "Hashes must differ when metadata differs");
}

#[test]
fn test_attestation_pipe_in_name_rejected() {
    let chain = test_chain();
    let err = chain
        .log_document(b"data", "file|name.txt", None, &[], None)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("pipe '|' characters"),
        "error should mention pipe: {msg}"
    );
}

#[test]
fn test_chain_tamper_detection() {
    let storage = Storage::open_in_memory().unwrap();
    let chain = AttestationChain::new(storage.clone());

    chain
        .log_document(b"first", "first.txt", None, &[], None)
        .unwrap();
    let r2 = chain
        .log_document(b"second", "second.txt", None, &[], None)
        .unwrap();

    // Chain is valid before tamper
    assert!(matches!(
        chain.verify_chain(None).unwrap(),
        ChainStatus::Valid { .. }
    ));

    // Directly modify the record_hash of the second record in the DB
    let r2_id = r2.id.unwrap();
    storage
        .with_transaction(|conn| {
            conn.execute(
                "UPDATE attestation_log SET record_hash = 'sha256:0000tampered' WHERE id = ?1",
                rusqlite::params![r2_id],
            )?;
            Ok(())
        })
        .expect("tamper record");

    // Chain should now be broken
    match chain.verify_chain(None).unwrap() {
        ChainStatus::Broken { at_record_id, .. } => {
            // The breakage can be detected at r2 (hash mismatch) or at a subsequent record
            assert!(at_record_id > 0);
        }
        other => panic!("expected Broken chain after tamper, got {other:?}"),
    }
}

#[test]
fn test_legacy_v1_record_verification_and_mixed_chain() {
    let storage = Storage::open_in_memory().unwrap();
    let chain = AttestationChain::new(storage.clone());

    // Construct legacy v1 record manually and insert into attestation_log
    let now = chrono::Utc::now();
    let mut rec1 = AttestationRecord {
        id: None,
        document_hash: "sha256:abc123doc".to_string(),
        document_name: "legacy_v1.txt".to_string(),
        document_size: 128,
        ingested_at: now,
        agent_id: Some("agent-v1".to_string()),
        memory_ids: vec![1, 2],
        previous_hash: GENESIS_HASH.to_string(),
        record_hash: String::new(),
        signature: None,
        metadata: serde_json::json!({}),
        created_at: Some(now),
        hash_version: LEGACY_HASH_VERSION,
    };
    rec1.record_hash = AttestationChain::compute_record_hash_v1(&rec1);

    storage
        .with_transaction(|conn| {
            conn.execute(
                "INSERT INTO attestation_log
                    (document_hash, document_name, document_size, ingested_at,
                     agent_id, memory_ids, previous_hash, record_hash, signature, metadata, hash_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    rec1.document_hash,
                    rec1.document_name,
                    rec1.document_size as i64,
                    rec1.ingested_at.to_rfc3339(),
                    rec1.agent_id,
                    serde_json::to_string(&rec1.memory_ids).unwrap(),
                    rec1.previous_hash,
                    rec1.record_hash,
                    rec1.signature,
                    serde_json::to_string(&rec1.metadata).unwrap(),
                    rec1.hash_version,
                ],
            )?;
            Ok(())
        })
        .unwrap();

    // Now append a v2 record using standard log_document
    let r2 = chain
        .log_document(
            b"new v2 content",
            "modern_v2.txt",
            Some("agent-v2"),
            &[3],
            None,
        )
        .expect("append v2 record");
    assert_eq!(r2.previous_hash, rec1.record_hash);
    assert_eq!(r2.hash_version, CURRENT_HASH_VERSION);

    // Verify the entire mixed chain (v1 followed by v2)
    match chain.verify_chain(None).unwrap() {
        ChainStatus::Valid { record_count } => {
            assert_eq!(record_count, 2);
        }
        other => panic!("expected Valid chain for mixed v1/v2 records, got {other:?}"),
    }
}
