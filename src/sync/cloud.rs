//! Cloud storage backends (S3, R2, GCS, Azure)

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use std::collections::HashMap;
use std::path::Path;

use super::encryption::{
    decrypt_data_with_provider, decrypt_data_with_provider_for_key_id, encrypt_data_with_provider,
    is_versioned_encrypted_payload,
};
use super::key_config::{CloudKeyProvider, ConfiguredCloudKeyProvider};
use crate::error::{EngramError, Result};

pub(super) const ENCRYPTION_KEY_ID_METADATA: &str = "engram-key-id";
pub(super) const ENCRYPTION_FORMAT_VERSION_METADATA: &str = "engram-key-format-version";
const ENCRYPTION_ALGORITHM_METADATA: &str = "engram-key-algorithm";
const OBJECT_FORMAT_METADATA: &str = "engram-object-format";
const PLAINTEXT_OBJECT_FORMAT: &str = "sqlite-plaintext-v1";
const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";

/// Cloud storage abstraction
pub struct CloudStorage {
    backend: CloudBackend,
    bucket: String,
    key: String,
    encrypt: bool,
    key_provider: Option<ConfiguredCloudKeyProvider>,
}

#[path = "cloud_backend.rs"]
mod backend;
#[path = "cloud_encryption_policy.rs"]
mod encryption_policy;
#[cfg(test)]
use backend::InMemoryCloudStore;
use backend::{CloudBackend, CloudObject, UploadCondition};
impl CloudStorage {
    /// Create from S3-compatible URI (s3://bucket/path/to/file.db)
    pub async fn from_uri(uri: &str, encrypt: bool) -> Result<Self> {
        let uri = uri
            .strip_prefix("s3://")
            .ok_or_else(|| EngramError::Config("URI must start with s3://".to_string()))?;

        let parts: Vec<&str> = uri.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(EngramError::Config(
                "URI must be s3://bucket/path".to_string(),
            ));
        }

        let bucket = parts[0].to_string();
        let key = parts[1].to_string();

        // Load AWS config from environment
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let client = S3Client::new(&config);

        let key_provider = if encrypt {
            Some(ConfiguredCloudKeyProvider::from_env()?)
        } else {
            None
        };

        Ok(Self {
            backend: CloudBackend::S3(client),
            bucket,
            key,
            encrypt,
            key_provider,
        })
    }

    pub fn encryption_key_id(&self) -> Option<&str> {
        self.key_provider
            .as_ref()
            .map(|provider| provider.active_key().id().as_str())
    }

    pub fn encryption_rotation_metadata(
        &self,
    ) -> Option<&super::key_config::CloudKeyRotationMetadata> {
        self.key_provider
            .as_ref()
            .map(CloudKeyProvider::rotation_metadata)
    }

    /// Upload a local file with conditional replacement.
    ///
    /// Existing objects are read and validated before replacement, so callers
    /// need `HeadObject`, `GetObject`, and conditional `PutObject` permission.
    pub async fn upload(&self, local_path: &Path) -> Result<u64> {
        let condition = self.ensure_remote_object_is_replaceable().await?;
        let data = tokio::fs::read(local_path).await?;
        let size = data.len() as u64;

        let (body, metadata) = if self.encrypt {
            (
                self.encrypt_data(&data)?,
                self.encryption_object_metadata()?,
            )
        } else {
            (
                data,
                HashMap::from([(
                    OBJECT_FORMAT_METADATA.to_string(),
                    PLAINTEXT_OBJECT_FORMAT.to_string(),
                )]),
            )
        };

        self.backend
            .put_object(&self.bucket, &self.key, body, metadata, condition)
            .await?;

        tracing::info!(
            "Uploaded {} bytes to s3://{}/{}",
            size,
            self.bucket,
            self.key
        );
        Ok(size)
    }

    /// Download from cloud to local file
    pub async fn download(&self, local_path: &Path) -> Result<u64> {
        let object = self.backend.get_object(&self.bucket, &self.key).await?;

        let decrypted = if self.encrypt {
            self.decrypt_encrypted_object(&object)?
        } else if Self::has_encryption_identity(&object) {
            return self.reject_encryption_audit(
                "encrypted cloud object requires an encryption key; refusing ciphertext download in plaintext mode",
            );
        } else if !Self::is_known_plaintext_object(&object) {
            return self.reject_encryption_audit(
                "remote cloud object format is unidentified; refusing ciphertext fallback in plaintext mode",
            );
        } else {
            object.body
        };

        let size = decrypted.len() as u64;

        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(local_path, &decrypted).await?;

        tracing::info!(
            "Downloaded {} bytes from s3://{}/{}",
            size,
            self.bucket,
            self.key
        );
        Ok(size)
    }

    /// Check if remote file exists
    pub async fn exists(&self) -> Result<bool> {
        self.backend.object_exists(&self.bucket, &self.key).await
    }

    /// Get remote file metadata
    pub async fn metadata(&self) -> Result<CloudMetadata> {
        let object = self.backend.head_object(&self.bucket, &self.key).await?;

        Ok(CloudMetadata {
            size: object.size,
            last_modified: object.last_modified,
            etag: object.etag,
        })
    }

    /// Delete remote file
    pub async fn delete(&self) -> Result<()> {
        self.backend.delete_object(&self.bucket, &self.key).await
    }

    #[cfg(test)]
    fn test_fixture(
        bucket: &str,
        key: &str,
        key_provider: ConfiguredCloudKeyProvider,
        store: InMemoryCloudStore,
    ) -> Self {
        Self {
            backend: CloudBackend::Fixture(store),
            bucket: bucket.to_string(),
            key: key.to_string(),
            encrypt: true,
            key_provider: Some(key_provider),
        }
    }

    #[cfg(test)]
    fn test_fixture_without_provider(bucket: &str, key: &str, store: InMemoryCloudStore) -> Self {
        Self {
            backend: CloudBackend::Fixture(store),
            bucket: bucket.to_string(),
            key: key.to_string(),
            encrypt: true,
            key_provider: None,
        }
    }

    #[cfg(test)]
    fn test_fixture_plaintext(bucket: &str, key: &str, store: InMemoryCloudStore) -> Self {
        Self {
            backend: CloudBackend::Fixture(store),
            bucket: bucket.to_string(),
            key: key.to_string(),
            encrypt: false,
            key_provider: None,
        }
    }
}

/// Cloud file metadata
#[derive(Debug, Clone)]
pub struct CloudMetadata {
    pub size: u64,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
}

/// Derive encryption key from passphrase using Argon2id.
///
/// `salt` must be at least 8 bytes (16 bytes recommended). The returned key
/// is always 32 bytes.
#[allow(dead_code)]
pub fn derive_key_from_passphrase(passphrase: &str, salt: &[u8]) -> Result<Vec<u8>> {
    use argon2::{Algorithm, Argon2, Params, Version};
    let params = Params::new(65536, 3, 1, Some(32))
        .map_err(|e| EngramError::Sync(format!("argon2 params error: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = vec![0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| EngramError::Sync(format!("key derivation failed: {e}")))?;
    Ok(key)
}

#[cfg(test)]
#[path = "cloud_tests.rs"]
mod tests;
