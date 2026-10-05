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
    ///
    /// Known G1 hazard (INVARIANTS #27): the file is read through a descriptor
    /// that is closed afterwards. Uploading a database that this process has
    /// open through rusqlite drops SQLite's POSIX locks on it (and the raw
    /// bytes may miss WAL content). `SyncWorker` does this but is not started
    /// by any binary today; checkpoint/snapshot to a separate file first.
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
    ///
    /// The object is validated first (key identity, format) and then written
    /// to a sibling temp file, fsynced and renamed over `local_path`, so an
    /// interrupted or failed download leaves the previous file intact (no
    /// partial or truncated database) and no temp file behind. Rename swaps
    /// the directory entry: never point it at a database that is open (in this
    /// or another process); see the G1 note on [`Self::upload`].
    pub async fn download(&self, local_path: &Path) -> Result<u64> {
        refuse_hot_sqlite_target(local_path)?;
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

        replace_file_atomically(local_path, &decrypted).await?;

        tracing::info!(
            "Downloaded {} bytes from s3://{}/{}",
            size,
            self.bucket,
            self.key
        );
        Ok(size)
    }

    /// [`Self::download`] for callers that hold the open [`crate::storage::Storage`]:
    /// additionally refuses a target that is the live database or one of its
    /// side files (G1: replacing or closing those drops SQLite's locks).
    pub async fn download_checked(
        &self,
        local_path: &Path,
        storage: &crate::storage::Storage,
    ) -> Result<u64> {
        storage.refuse_active_sqlite_artifact(local_path)?;
        self.download(local_path).await
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

/// Refuse a download target that looks like an open database: a non-empty
/// `-wal`, or a `-shm` / `-journal` file, next to it means another connection
/// (or an unclean shutdown) still owns state that a replacement would corrupt.
fn refuse_hot_sqlite_target(target: &Path) -> Result<()> {
    let sidecar = |suffix: &str| {
        let mut name = target.as_os_str().to_os_string();
        name.push(suffix);
        std::path::PathBuf::from(name)
    };
    let hot_wal = std::fs::metadata(sidecar("-wal")).is_ok_and(|m| m.len() > 0);
    let hot_other = ["-shm", "-journal"]
        .iter()
        .any(|suffix| sidecar(suffix).exists());
    if hot_wal || hot_other {
        return Err(EngramError::Sync(format!(
            "refusing to replace '{}': it has a hot -wal/-shm/-journal side file \
             (database open or not cleanly closed)",
            target.display()
        )));
    }
    Ok(())
}

/// Make the rename durable. Best effort: the file is already replaced, so a
/// failure only weakens crash durability and is logged, not returned.
async fn sync_parent_dir(target: &Path) {
    #[cfg(unix)]
    {
        let parent = match target.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        let result = async {
            let dir = tokio::fs::File::open(parent).await?;
            dir.sync_all().await
        }
        .await;
        if let Err(error) = result {
            tracing::warn!(
                path = %crate::observability::redact::path_label(parent),
                error_class = %crate::observability::redact::io_class(&error),
                "fsync of download directory failed"
            );
        }
    }
    #[cfg(not(unix))]
    let _ = target;
}

/// Write `bytes` to a sibling temp file, fsync it, and rename it over `target`.
/// The temp file is removed on every failure path. An existing target keeps its
/// permissions; a new one is owner-only.
async fn replace_file_atomically(target: &Path, bytes: &[u8]) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| EngramError::Config("download target has no file name".to_string()))?;
    let temp = target.with_file_name(format!(
        ".{file_name}.download-{}.tmp",
        uuid::Uuid::new_v4().simple()
    ));

    let outcome: Result<()> = async {
        let mut options = tokio::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options.open(&temp).await?;
        file.write_all(bytes).await?;
        file.sync_all().await?;
        drop(file);
        if let Ok(existing) = tokio::fs::metadata(target).await {
            if existing.is_file() {
                tokio::fs::set_permissions(&temp, existing.permissions()).await?;
            }
        }
        tokio::fs::rename(&temp, target).await?;
        sync_parent_dir(target).await;
        Ok(())
    }
    .await;

    if outcome.is_err() {
        // Best-effort teardown: the business error is already in `outcome`.
        let _ = tokio::fs::remove_file(&temp).await;
    }
    outcome
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
