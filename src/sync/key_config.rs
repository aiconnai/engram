//! Durable cloud encryption key configuration.
//!
//! Cloud encryption is configured explicitly through one of these sources:
//!
//! - `ENGRAM_CLOUD_ENCRYPTION_KEY`: base64 or hex encoded 32-byte key material.
//! - `ENGRAM_CLOUD_ENCRYPTION_KEY_FILE`: path to a file containing the same
//!   encoded key material.
//! - `ENGRAM_CLOUD_ENCRYPTION_PREVIOUS_KEY`: optional previous key material for
//!   a controlled read-old/write-new rotation window.
//! - `ENGRAM_CLOUD_ENCRYPTION_PREVIOUS_KEY_FILE`: file-backed form of the
//!   optional previous key.
//!
//! The key bytes are never printed by Debug or error output. The stable key ID
//! is derived from SHA-256(key bytes) and is safe to use as rotation metadata.

use std::env;
#[path = "key_material.rs"]
mod key_material;
use key_material::{derive_key_id, parse_encoded_key};

use std::fmt;
use std::path::PathBuf;

use thiserror::Error;
use zeroize::Zeroizing;

use crate::error::EngramError;

pub const KEY_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_KEY";
pub const KEY_FILE_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_KEY_FILE";
pub const PREVIOUS_KEY_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_PREVIOUS_KEY";
pub const PREVIOUS_KEY_FILE_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_PREVIOUS_KEY_FILE";
pub const PREVIOUS_KEY_ID_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_PREVIOUS_KEY_ID";
pub const KEY_BYTES_LEN: usize = 32;

/// Stable, non-secret identifier derived from configured key material.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CloudKeyId(String);

impl CloudKeyId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CloudKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CloudKeyId").field(&self.0).finish()
    }
}

pub struct CloudEncryptionKey {
    id: CloudKeyId,
    bytes: Zeroizing<[u8; KEY_BYTES_LEN]>,
}

impl CloudEncryptionKey {
    fn new(bytes: Zeroizing<[u8; KEY_BYTES_LEN]>) -> Self {
        let id = derive_key_id(&bytes);
        Self { id, bytes }
    }

    pub fn id(&self) -> &CloudKeyId {
        &self.id
    }

    pub(super) fn with_bytes<T>(&self, f: impl FnOnce(&[u8; KEY_BYTES_LEN]) -> T) -> T {
        f(&self.bytes)
    }
}

impl fmt::Debug for CloudEncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CloudEncryptionKey")
            .field("id", &self.id)
            .field("bytes", &"<redacted>")
            .finish()
    }
}

/// Deterministic metadata attached to encrypted payloads and rotation checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudKeyRotationMetadata {
    pub format_version: u8,
    pub algorithm: &'static str,
    pub active_key_id: String,
    pub previous_key_id: Option<String>,
}

/// Configured provider contract for cloud encryption keys.
pub trait CloudKeyProvider {
    fn active_key(&self) -> &CloudEncryptionKey;
    fn rotation_metadata(&self) -> &CloudKeyRotationMetadata;
}

pub struct ConfiguredCloudKeyProvider {
    active_key: CloudEncryptionKey,
    previous_key: Option<CloudEncryptionKey>,
    rotation_metadata: CloudKeyRotationMetadata,
}

impl ConfiguredCloudKeyProvider {
    pub fn from_env() -> Result<Self, KeyConfigError> {
        let inline_key = read_optional_key_env_value()?;
        let key_file_env = env::var_os(KEY_FILE_ENV);
        let key_value = match (inline_key, key_file_env) {
            (Some(_inline), Some(_)) => return Err(KeyConfigError::AmbiguousSources),
            (Some(value), None) => EncodedKeyMaterial::Inline(value),
            (None, Some(path)) => EncodedKeyMaterial::File(PathBuf::from(path)),
            (None, None) => return Err(KeyConfigError::MissingSource),
        };

        let previous_inline_key = read_optional_env_value(PREVIOUS_KEY_ENV)?;
        let previous_key_file_env = env::var_os(PREVIOUS_KEY_FILE_ENV);
        let previous_key_value = match (previous_inline_key, previous_key_file_env) {
            (Some(_inline), Some(_)) => return Err(KeyConfigError::AmbiguousPreviousSources),
            (Some(value), None) => Some(EncodedKeyMaterial::Inline(value)),
            (None, Some(path)) => Some(EncodedKeyMaterial::File(PathBuf::from(path))),
            (None, None) => None,
        };

        let previous_key_id_hint = match env::var(PREVIOUS_KEY_ID_ENV) {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            Err(_) => None,
        };

        match previous_key_value {
            Some(previous) => Self::from_material_with_previous(key_value, previous),
            None => Self::from_material(key_value, previous_key_id_hint),
        }
    }

    pub fn from_material(
        material: EncodedKeyMaterial,
        previous_key_id: Option<String>,
    ) -> Result<Self, KeyConfigError> {
        let raw = material.read_redacted()?;
        let bytes = parse_encoded_key(raw.as_str())?;
        let active_key = CloudEncryptionKey::new(bytes);
        let rotation_metadata = CloudKeyRotationMetadata {
            format_version: 1,
            algorithm: "AES-256-GCM",
            active_key_id: active_key.id().as_str().to_string(),
            previous_key_id,
        };

        Ok(Self {
            active_key,
            previous_key: None,
            rotation_metadata,
        })
    }

    pub fn from_material_with_previous(
        material: EncodedKeyMaterial,
        previous_material: EncodedKeyMaterial,
    ) -> Result<Self, KeyConfigError> {
        let raw = material.read_redacted()?;
        let bytes = parse_encoded_key(raw.as_str())?;
        let active_key = CloudEncryptionKey::new(bytes);
        let previous_raw = previous_material.read_redacted()?;
        let previous_bytes = parse_encoded_key(previous_raw.as_str())?;
        let previous_key = CloudEncryptionKey::new(previous_bytes);
        let rotation_metadata = CloudKeyRotationMetadata {
            format_version: 1,
            algorithm: "AES-256-GCM",
            active_key_id: active_key.id().as_str().to_string(),
            previous_key_id: Some(previous_key.id().as_str().to_string()),
        };

        Ok(Self {
            active_key,
            previous_key: Some(previous_key),
            rotation_metadata,
        })
    }

    pub(super) fn key_for_id(&self, key_id: &str) -> Option<&CloudEncryptionKey> {
        if self.active_key.id().as_str() == key_id {
            return Some(&self.active_key);
        }

        self.previous_key
            .as_ref()
            .filter(|key| key.id().as_str() == key_id)
    }

    pub(super) fn can_decrypt_key_id(&self, key_id: &str) -> bool {
        self.key_for_id(key_id).is_some()
    }

    pub(super) fn previous_key(&self) -> Option<&CloudEncryptionKey> {
        self.previous_key.as_ref()
    }
}

impl fmt::Debug for ConfiguredCloudKeyProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfiguredCloudKeyProvider")
            .field("active_key", &self.active_key)
            .field("rotation_metadata", &self.rotation_metadata)
            .finish()
    }
}

impl CloudKeyProvider for ConfiguredCloudKeyProvider {
    fn active_key(&self) -> &CloudEncryptionKey {
        &self.active_key
    }

    fn rotation_metadata(&self) -> &CloudKeyRotationMetadata {
        &self.rotation_metadata
    }
}

/// Encoded key input from either env or a file path.
pub enum EncodedKeyMaterial {
    Inline(Zeroizing<String>),
    File(PathBuf),
}

impl EncodedKeyMaterial {
    fn read_redacted(self) -> Result<Zeroizing<String>, KeyConfigError> {
        match self {
            EncodedKeyMaterial::Inline(value) => Ok(value),
            EncodedKeyMaterial::File(path) => std::fs::read_to_string(&path)
                .map(Zeroizing::new)
                .map_err(|source| KeyConfigError::ReadFile {
                    path_display: path.display().to_string(),
                    source,
                }),
        }
    }
}

fn read_optional_key_env_value() -> Result<Option<Zeroizing<String>>, KeyConfigError> {
    read_optional_env_value(KEY_ENV)
}

fn read_optional_env_value(name: &str) -> Result<Option<Zeroizing<String>>, KeyConfigError> {
    match env::var(name) {
        Ok(value) => Ok(Some(Zeroizing::new(value))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(KeyConfigError::MalformedKey),
    }
}

/// Redacted key configuration errors.
#[derive(Debug, Error)]
pub enum KeyConfigError {
    #[error(
        "cloud encryption requires {KEY_ENV} or {KEY_FILE_ENV}; refusing encrypted cloud writes"
    )]
    MissingSource,
    #[error(
        "cloud encryption key source is ambiguous; set only one of {KEY_ENV} or {KEY_FILE_ENV}"
    )]
    AmbiguousSources,
    #[error(
        "previous cloud encryption key source is ambiguous; set only one of {PREVIOUS_KEY_ENV} or {PREVIOUS_KEY_FILE_ENV}"
    )]
    AmbiguousPreviousSources,
    #[error("cloud encryption key file could not be read at {path_display}: {source}")]
    ReadFile {
        path_display: String,
        source: std::io::Error,
    },
    #[error(
        "cloud encryption key is malformed; expected base64 or hex encoded 32-byte key material"
    )]
    MalformedKey,
}

impl From<KeyConfigError> for EngramError {
    fn from(value: KeyConfigError) -> Self {
        EngramError::Config(value.to_string())
    }
}
