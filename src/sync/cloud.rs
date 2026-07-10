//! Cloud storage backends (S3, R2, GCS, Azure)

use std::path::Path;

use aws_config::BehaviorVersion;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;

use super::encryption::{decrypt_data_with_provider, encrypt_data_with_provider};
use super::key_config::{CloudKeyProvider, ConfiguredCloudKeyProvider};
use crate::error::{EngramError, Result};

/// Cloud storage abstraction
pub struct CloudStorage {
    client: S3Client,
    bucket: String,
    key: String,
    encrypt: bool,
    key_provider: Option<ConfiguredCloudKeyProvider>,
}

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
            client,
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

    /// Upload local file to cloud
    pub async fn upload(&self, local_path: &Path) -> Result<u64> {
        let data = tokio::fs::read(local_path).await?;
        let size = data.len() as u64;

        let body = if self.encrypt {
            let encrypted = self.encrypt_data(&data)?;
            ByteStream::from(encrypted)
        } else {
            ByteStream::from(data)
        };

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&self.key)
            .body(body)
            .send()
            .await
            .map_err(|e| EngramError::CloudStorage(e.to_string()))?;

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
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&self.key)
            .send()
            .await
            .map_err(|e| EngramError::CloudStorage(e.to_string()))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| EngramError::CloudStorage(e.to_string()))?
            .into_bytes();

        let decrypted = if self.encrypt {
            self.decrypt_data(&data)?
        } else {
            data.to_vec()
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
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&self.key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let service_error = e.into_service_error();
                if service_error.is_not_found() {
                    Ok(false)
                } else {
                    Err(EngramError::CloudStorage(service_error.to_string()))
                }
            }
        }
    }

    /// Get remote file metadata
    pub async fn metadata(&self) -> Result<CloudMetadata> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&self.key)
            .send()
            .await
            .map_err(|e| EngramError::CloudStorage(e.to_string()))?;

        Ok(CloudMetadata {
            size: response.content_length().unwrap_or(0) as u64,
            last_modified: response.last_modified().map(|dt| dt.to_string()),
            etag: response.e_tag().map(String::from),
        })
    }

    /// Delete remote file
    pub async fn delete(&self) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&self.key)
            .send()
            .await
            .map_err(|e| EngramError::CloudStorage(e.to_string()))?;

        Ok(())
    }

    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let provider = self.configured_key_provider()?;
        encrypt_data_with_provider(provider, data)
    }

    fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let provider = self.configured_key_provider()?;
        decrypt_data_with_provider(provider, data)
    }

    fn configured_key_provider(&self) -> Result<&ConfiguredCloudKeyProvider> {
        self.key_provider.as_ref().ok_or_else(|| {
            EngramError::Encryption(
                "cloud encryption requires a configured durable key provider".to_string(),
            )
        })
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
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let passphrase = "hunter2";
        let salt = b"abcdefghijklmnop";
        let key1 = derive_key_from_passphrase(passphrase, salt).unwrap();
        let key2 = derive_key_from_passphrase(passphrase, salt).unwrap();
        assert_eq!(key1, key2, "same passphrase+salt must yield same key");
    }

    #[test]
    fn test_derive_key_different_salt() {
        let passphrase = "hunter2";
        let salt1 = b"abcdefghijklmnop";
        let salt2 = b"pqrstuvwxyz12345";
        let key1 = derive_key_from_passphrase(passphrase, salt1).unwrap();
        let key2 = derive_key_from_passphrase(passphrase, salt2).unwrap();
        assert_ne!(key1, key2, "different salts must yield different keys");
    }

    #[test]
    fn test_derive_key_length() {
        let key = derive_key_from_passphrase("secret", b"saltysalt12345678").unwrap();
        assert_eq!(key.len(), 32, "key must be 32 bytes");
    }

    #[test]
    fn missing_configured_key_fails_before_encrypted_write() {
        // Given: an encrypted storage instance without a provider.
        let sdk_config = aws_config::SdkConfig::builder()
            .behavior_version(BehaviorVersion::latest())
            .build();
        let storage = CloudStorage {
            client: S3Client::new(&sdk_config),
            bucket: "bucket".to_string(),
            key: "path.db".to_string(),
            encrypt: true,
            key_provider: None,
        };

        // When: encryption is attempted before an upload body is built.
        let error = storage
            .encrypt_data(b"db bytes")
            .expect_err("missing provider must fail");

        // Then: the failure is explicit and contains no payload bytes.
        let message = error.to_string();
        assert!(message.contains("configured durable key provider"));
        assert!(!message.contains("db bytes"));
    }
}
