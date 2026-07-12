use super::*;

#[tokio::test]
async fn rotated_client_reads_previous_key_and_writes_active_key_metadata() {
    // Given: an object encrypted before rotation and a new provider with old+new keys.
    let store = InMemoryCloudStore::default();
    let old_writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(7), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let old_source = temp_dir.path().join("old.db");
    let restored = temp_dir.path().join("restored.db");
    let new_source = temp_dir.path().join("new.db");
    tokio::fs::write(&old_source, b"old database bytes")
        .await
        .expect("old source is written");
    old_writer
        .upload(&old_source)
        .await
        .expect("old upload succeeds");
    let old_key_id = old_writer
        .encryption_key_id()
        .expect("old key id exists")
        .to_string();

    // When: the rotated client downloads the old object and uploads a replacement.
    let rotated =
        CloudStorage::test_fixture("bucket", "path.db", rotating_provider(8, 7), store.clone());
    rotated
        .download(&restored)
        .await
        .expect("rotated download succeeds");
    tokio::fs::write(&new_source, b"new database bytes")
        .await
        .expect("new source is written");
    rotated
        .upload(&new_source)
        .await
        .expect("rotated upload succeeds");
    let new_restored = temp_dir.path().join("new-restored.db");
    rotated
        .download(&new_restored)
        .await
        .expect("rotated client reads active-key upload");

    // Then: old data was readable, and future writes use the new active key identity.
    assert_eq!(
        tokio::fs::read(&restored)
            .await
            .expect("restored file reads"),
        b"old database bytes"
    );
    let stored = store.snapshot().expect("rotated object is stored");
    assert_eq!(
        stored
            .metadata
            .get(ENCRYPTION_KEY_ID_METADATA)
            .map(String::as_str),
        rotated.encryption_key_id()
    );
    assert_ne!(
        stored.metadata.get(ENCRYPTION_KEY_ID_METADATA),
        Some(&old_key_id)
    );
    assert_eq!(
        tokio::fs::read(&new_restored)
            .await
            .expect("new restored file reads"),
        b"new database bytes"
    );
    assert!(!stored
        .body
        .windows(b"new database bytes".len())
        .any(|window| window == b"new database bytes"));
}
