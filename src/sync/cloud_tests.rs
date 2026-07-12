use super::*;
use rand::Rng;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

struct CapturedLogWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedLogWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("test log lock is not poisoned")
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for CapturedLogs {
    type Writer = CapturedLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        CapturedLogWriter(self.0.clone())
    }
}

impl CapturedLogs {
    fn snapshot(&self) -> String {
        String::from_utf8(
            self.0
                .lock()
                .expect("test log lock is not poisoned")
                .clone(),
        )
        .expect("captured logs are utf-8")
    }
}

fn random_salt() -> [u8; 16] {
    rand::rngs::OsRng.gen()
}

#[test]
fn test_derive_key_deterministic() {
    let passphrase = "hunter2";
    let salt = random_salt();
    let key1 = derive_key_from_passphrase(passphrase, &salt).unwrap();
    let key2 = derive_key_from_passphrase(passphrase, &salt).unwrap();
    assert_eq!(key1, key2, "same passphrase+salt must yield same key");
}

#[test]
fn test_derive_key_different_salt() {
    let passphrase = "hunter2";
    let salt1 = random_salt();
    let mut salt2 = random_salt();
    while salt2 == salt1 {
        salt2 = random_salt();
    }
    let key1 = derive_key_from_passphrase(passphrase, &salt1).unwrap();
    let key2 = derive_key_from_passphrase(passphrase, &salt2).unwrap();
    assert_ne!(key1, key2, "different salts must yield different keys");
}

#[test]
fn test_derive_key_length() {
    let salt = random_salt();
    let key = derive_key_from_passphrase("secret", &salt).unwrap();
    assert_eq!(key.len(), 32, "key must be 32 bytes");
}

fn provider_from_byte(byte: u8) -> ConfiguredCloudKeyProvider {
    ConfiguredCloudKeyProvider::from_material(
        super::super::key_config::EncodedKeyMaterial::Inline(zeroize::Zeroizing::new(format!(
            "hex:{}",
            hex::encode([byte; super::super::key_config::KEY_BYTES_LEN])
        ))),
        None,
    )
    .expect("test key provider loads")
}

fn rotating_provider(active: u8, previous: u8) -> ConfiguredCloudKeyProvider {
    ConfiguredCloudKeyProvider::from_material_with_previous(
        super::super::key_config::EncodedKeyMaterial::Inline(zeroize::Zeroizing::new(format!(
            "hex:{}",
            hex::encode([active; super::super::key_config::KEY_BYTES_LEN])
        ))),
        super::super::key_config::EncodedKeyMaterial::Inline(zeroize::Zeroizing::new(format!(
            "hex:{}",
            hex::encode([previous; super::super::key_config::KEY_BYTES_LEN])
        ))),
    )
    .expect("rotating test key provider loads")
}

#[test]
fn missing_configured_key_fails_before_encrypted_write() {
    // Given: an encrypted storage instance without a provider.
    let storage = CloudStorage::test_fixture_without_provider(
        "bucket",
        "path.db",
        InMemoryCloudStore::default(),
    );

    // When: encryption is attempted before an upload body is built.
    let error = storage
        .encrypt_data(b"db bytes")
        .expect_err("missing provider must fail");

    // Then: the failure is explicit and contains no payload bytes.
    let message = error.to_string();
    assert!(message.contains("configured durable key provider"));
    assert!(!message.contains("db bytes"));
}

#[tokio::test]
async fn encrypted_upload_download_reconstructed_client_persists_key_identity() {
    // Given: a durable key and an in-memory cloud fixture, not live S3.
    let store = InMemoryCloudStore::default();
    let writer =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(42), store.clone());
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let source = temp_dir.path().join("source.db");
    let restored = temp_dir.path().join("restored.db");
    let plaintext = b"restart roundtrip database bytes";
    tokio::fs::write(&source, plaintext)
        .await
        .expect("test source is written");

    // When: one client uploads and a reconstructed client downloads.
    let uploaded = writer.upload(&source).await.expect("upload succeeds");
    let reader =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(42), store.clone());
    let expected_key_id = writer
        .encryption_key_id()
        .expect("writer key id exists")
        .to_string();
    drop(writer);
    let downloaded = reader.download(&restored).await.expect("download succeeds");

    // Then: payload metadata carries only non-secret key identity/version and no plaintext.
    let stored = store.snapshot().expect("fixture object is stored");
    assert_eq!(uploaded, plaintext.len() as u64);
    assert_eq!(downloaded, plaintext.len() as u64);
    assert_eq!(
        tokio::fs::read(&restored)
            .await
            .expect("restored file reads"),
        plaintext
    );
    assert_ne!(stored.body, plaintext);
    assert!(!stored
        .body
        .windows(plaintext.len())
        .any(|window| window == plaintext));
    assert_eq!(
        stored
            .metadata
            .get(ENCRYPTION_KEY_ID_METADATA)
            .map(String::as_str),
        Some(expected_key_id.as_str())
    );
    assert_eq!(
        stored
            .metadata
            .get(ENCRYPTION_FORMAT_VERSION_METADATA)
            .map(String::as_str),
        Some("1")
    );
    assert!(!format!("{stored:?}").contains(&hex::encode(
        [42u8; super::super::key_config::KEY_BYTES_LEN]
    )));
}

#[path = "cloud_tests_compat.rs"]
mod compatibility;
#[path = "cloud_tests_rotation.rs"]
mod rotation;
#[path = "cloud_tests_security.rs"]
mod security;
