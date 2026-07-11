use super::*;
use crate::sync::key_config::KEY_BYTES_LEN;

#[tokio::test]
async fn missing_key_provider_fails_non_destructively_without_key_bytes() {
    // Given: an encrypted object and a reconstructed client with no configured key.
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
    let before = store.snapshot().expect("fixture object is stored");

    // When: download is attempted without the key provider.
    let missing_reader =
        CloudStorage::test_fixture_without_provider("bucket", "path.db", store.clone());
    let error = missing_reader
        .download(&destination)
        .await
        .expect_err("missing key provider must fail before write");

    // Then: neither local nor remote data changes and no key bytes are exposed.
    assert!(!destination.exists());
    let after = store.snapshot().expect("fixture object remains stored");
    assert_eq!(after.body, before.body);
    assert_eq!(after.metadata, before.metadata);
    let message = error.to_string();
    assert!(message.contains("configured durable key provider"));
    assert!(!message.contains("sensitive database bytes"));
    assert!(!format!("{error:?}").contains(&hex::encode([21u8; KEY_BYTES_LEN])));
}

#[tokio::test]
async fn missing_key_identity_metadata_fails_before_local_write() {
    // Given: encrypted bytes whose non-secret key identity metadata is absent.
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
    store.remove_metadata(ENCRYPTION_KEY_ID_METADATA);
    let before = store.snapshot().expect("fixture object is stored");

    // When: metadata is missing even though the body is still encrypted.
    let missing_error = writer
        .download(&destination)
        .await
        .expect_err("missing key metadata must fail before write");

    // Then: missing identity is rejected without overwriting or leaking secrets.
    assert!(!destination.exists());
    let after = store.snapshot().expect("fixture object remains stored");
    assert_eq!(after.body, before.body);
    assert_eq!(after.metadata, before.metadata);
    let missing_message = missing_error.to_string();
    assert!(missing_message.contains("missing encryption key id metadata"));
    assert!(!missing_message.contains("sensitive database bytes"));
    assert!(!format!("{missing_error:?}").contains(&hex::encode([21u8; KEY_BYTES_LEN])));
}

#[tokio::test]
async fn pre_metadata_versioned_object_remains_readable_after_restart() {
    // Given: a versioned encrypted object created before S3 metadata persistence shipped.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(41), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let source = temp_dir.path().join("source.db");
    let destination = temp_dir.path().join("destination.db");
    tokio::fs::write(&source, b"pre-metadata encrypted bytes")
        .await
        .expect("source is written");
    writer.upload(&source).await.expect("upload succeeds");
    store.clear_metadata();
    drop(writer);

    // When: a reconstructed client downloads using the embedded key identity.
    let reader =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(41), store.clone());
    reader
        .download(&destination)
        .await
        .expect("pre-metadata object decrypts");

    // Then: legacy compatibility returns the exact original plaintext without rewriting remote data.
    assert_eq!(
        tokio::fs::read(&destination)
            .await
            .expect("destination reads"),
        b"pre-metadata encrypted bytes"
    );
    assert!(store
        .snapshot()
        .expect("fixture object remains stored")
        .metadata
        .is_empty());
}

#[tokio::test]
async fn legacy_nonce_ciphertext_object_remains_readable_after_restart() {
    // Given: metadata-less legacy nonce+ciphertext produced with the durable key.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(61), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let source = temp_dir.path().join("source.db");
    let destination = temp_dir.path().join("destination.db");
    tokio::fs::write(&source, b"legacy cloud bytes")
        .await
        .expect("source is written");
    writer.upload(&source).await.expect("upload succeeds");
    let mut legacy = store.snapshot().expect("fixture object is stored");
    let key_id_len = usize::from(legacy.body[4]);
    legacy.body = legacy.body[(5 + key_id_len)..].to_vec();
    legacy.size = legacy.body.len() as u64;
    legacy.metadata.clear();
    let legacy_etag = legacy.etag.clone().expect("fixture etag exists");
    store
        .put(legacy, UploadCondition::Matches(legacy_etag))
        .expect("legacy fixture replaces versioned object");
    drop(writer);

    // When: a reconstructed client downloads the legacy object.
    let reader =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(61), store.clone());
    reader
        .download(&destination)
        .await
        .expect("legacy object decrypts");

    // Then: the exact plaintext is restored without rewriting remote ciphertext.
    assert_eq!(
        tokio::fs::read(&destination)
            .await
            .expect("destination reads"),
        b"legacy cloud bytes"
    );
    assert!(store
        .snapshot()
        .expect("legacy object remains stored")
        .metadata
        .is_empty());
}
