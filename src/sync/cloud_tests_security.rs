use super::*;
use crate::sync::key_config::KEY_BYTES_LEN;

#[tokio::test]
async fn wrong_key_fails_non_destructively_without_key_bytes() {
    // Given: a stored encrypted object and existing local data that must survive audit failure.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(21), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let source = temp_dir.path().join("source.db");
    let destination = temp_dir.path().join("destination.db");
    tokio::fs::write(&source, b"sensitive database bytes")
        .await
        .expect("test source is written");
    writer.upload(&source).await.expect("upload succeeds");
    tokio::fs::write(&destination, b"existing local bytes")
        .await
        .expect("existing destination is written");
    let before = store.snapshot().expect("fixture object is stored");

    // When: a client with the wrong key ID tries to download.
    let wrong_reader =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(22), store.clone());
    let wrong_error = wrong_reader
        .download(&destination)
        .await
        .expect_err("wrong key id must fail before write");

    // Then: the audit is non-destructive and does not expose payload/key bytes.
    assert_eq!(
        tokio::fs::read(&destination)
            .await
            .expect("existing destination reads"),
        b"existing local bytes"
    );
    let after = store.snapshot().expect("fixture object remains stored");
    assert_eq!(after.body, before.body);
    assert_eq!(after.metadata, before.metadata);
    let wrong_message = wrong_error.to_string();
    assert!(wrong_message.contains("refusing encrypted download"));
    assert!(!wrong_message.contains("sensitive database bytes"));
    assert!(!wrong_message.contains(&hex::encode([21u8; KEY_BYTES_LEN])));
    assert!(!wrong_message.contains(&hex::encode([22u8; KEY_BYTES_LEN])));
}

#[tokio::test]
async fn wrong_key_emits_redacted_audit_signal() {
    // Given: encrypted cloud data and a captured structured tracing sink.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(51), store.clone());
    let wrong_reader =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(52), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let source = temp_dir.path().join("source.db");
    let destination = temp_dir.path().join("destination.db");
    tokio::fs::write(&source, b"audit-sensitive bytes")
        .await
        .expect("source is written");
    writer.upload(&source).await.expect("upload succeeds");
    let captured = CapturedLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(captured.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    // When: the wrong-key client rejects the encrypted object.
    wrong_reader
        .download(&destination)
        .await
        .expect_err("wrong key must fail");

    // Then: a stable warning is emitted without plaintext or raw key material.
    let logs = captured.snapshot();
    assert!(logs.contains("Rejected encrypted cloud object"));
    assert!(logs.contains("WARN"));
    assert!(!logs.contains("audit-sensitive bytes"));
    assert!(!logs.contains(&hex::encode([51u8; KEY_BYTES_LEN])));
    assert!(!logs.contains(&hex::encode([52u8; KEY_BYTES_LEN])));
}

#[tokio::test]
async fn wrong_key_cannot_overwrite_existing_ciphertext() {
    // Given: remote ciphertext written by a key unavailable to the replacement client.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(21), store.clone());
    let wrong_writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(22), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let original = temp_dir.path().join("original.db");
    let replacement = temp_dir.path().join("replacement.db");
    tokio::fs::write(&original, b"original encrypted bytes")
        .await
        .expect("original source is written");
    tokio::fs::write(&replacement, b"replacement bytes")
        .await
        .expect("replacement source is written");
    writer
        .upload(&original)
        .await
        .expect("initial upload succeeds");
    let before = store.snapshot().expect("fixture object is stored");

    // When: the wrong-key client attempts an upload to the same object.
    let error = wrong_writer
        .upload(&replacement)
        .await
        .expect_err("wrong key must not overwrite ciphertext");

    // Then: the remote body and metadata are unchanged and the refusal is observable.
    let after = store.snapshot().expect("fixture object remains stored");
    assert_eq!(after.body, before.body);
    assert_eq!(after.metadata, before.metadata);
    assert!(error.to_string().contains("refusing encrypted download"));
    assert!(!error.to_string().contains("replacement bytes"));
}

#[tokio::test]
async fn plaintext_client_cannot_downgrade_existing_ciphertext() {
    // Given: an encrypted remote object and a plaintext-mode replacement client.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(31), store.clone());
    let plaintext_writer = CloudStorage::test_fixture_plaintext("bucket", "path.db", store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let original = temp_dir.path().join("original.db");
    let replacement = temp_dir.path().join("replacement.db");
    tokio::fs::write(&original, b"encrypted original")
        .await
        .expect("original source is written");
    tokio::fs::write(&replacement, b"plaintext replacement")
        .await
        .expect("replacement source is written");
    writer
        .upload(&original)
        .await
        .expect("initial upload succeeds");
    let before = store.snapshot().expect("fixture object is stored");

    // When: plaintext mode attempts to replace the encrypted object.
    let error = plaintext_writer
        .upload(&replacement)
        .await
        .expect_err("plaintext overwrite must fail");

    // Then: ciphertext remains unchanged and no plaintext is persisted remotely.
    let after = store.snapshot().expect("fixture object remains stored");
    assert_eq!(after.body, before.body);
    assert_eq!(after.metadata, before.metadata);
    assert!(error.to_string().contains("refusing plaintext overwrite"));
    assert!(!after
        .body
        .windows(b"plaintext replacement".len())
        .any(|window| window == b"plaintext replacement"));
}

#[tokio::test]
async fn plaintext_client_cannot_downgrade_legacy_ciphertext() {
    // Given: metadata-less legacy ciphertext and a plaintext-mode client.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(32), store.clone());
    let plaintext_writer = CloudStorage::test_fixture_plaintext("bucket", "path.db", store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let original = temp_dir.path().join("original.db");
    let replacement = temp_dir.path().join("replacement.db");
    tokio::fs::write(&original, b"legacy encrypted original")
        .await
        .expect("original source is written");
    tokio::fs::write(&replacement, SQLITE_HEADER)
        .await
        .expect("plaintext replacement is written");
    writer
        .upload(&original)
        .await
        .expect("initial upload succeeds");
    let mut legacy = store.snapshot().expect("fixture object is stored");
    let key_id_len = usize::from(legacy.body[4]);
    legacy.body = legacy.body[(5 + key_id_len)..].to_vec();
    legacy.size = legacy.body.len() as u64;
    legacy.metadata.clear();
    let legacy_etag = legacy.etag.clone().expect("fixture etag exists");
    store
        .put(legacy, UploadCondition::Matches(legacy_etag))
        .expect("legacy fixture replaces versioned object");
    let before = store.snapshot().expect("legacy object is stored");

    // When: plaintext mode attempts to overwrite the unidentified legacy object.
    let error = plaintext_writer
        .upload(&replacement)
        .await
        .expect_err("legacy ciphertext downgrade must fail");

    // Then: legacy ciphertext remains byte-for-byte unchanged.
    let after = store.snapshot().expect("legacy object remains stored");
    assert_eq!(after.body, before.body);
    assert_eq!(after.metadata, before.metadata);
    assert_eq!(after.etag, before.etag);
    assert!(error.to_string().contains("refusing plaintext overwrite"));
}

#[tokio::test]
async fn stale_upload_condition_cannot_replace_concurrently_changed_object() {
    // Given: a validated object version that is replaced before the conditional PUT.
    let store = InMemoryCloudStore::default();
    let storage =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(33), store.clone());
    let initial = CloudObject {
        body: b"initial".to_vec(),
        metadata: HashMap::new(),
        size: 7,
        last_modified: None,
        etag: None,
    };
    store
        .put(initial, UploadCondition::DoesNotExist)
        .expect("initial fixture write succeeds");
    let stale_etag = store
        .snapshot()
        .and_then(|object| object.etag)
        .expect("initial etag exists");
    let concurrent = CloudObject {
        body: b"concurrent".to_vec(),
        metadata: HashMap::new(),
        size: 10,
        last_modified: None,
        etag: None,
    };
    store
        .put(concurrent, UploadCondition::Matches(stale_etag.clone()))
        .expect("concurrent fixture write succeeds");

    // When: a PUT uses the stale version identifier.
    let error = storage
        .backend
        .put_object(
            "bucket",
            "path.db",
            b"stale replacement".to_vec(),
            HashMap::new(),
            UploadCondition::Matches(stale_etag),
        )
        .await
        .expect_err("stale conditional upload must fail");

    // Then: the concurrent object remains authoritative.
    assert!(error.to_string().contains("stale object state"));
    assert_eq!(
        store.snapshot().expect("concurrent object remains").body,
        b"concurrent"
    );
}
