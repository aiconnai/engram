//! Durable cloud encryption key configuration.
//!
//! Cloud encryption is configured explicitly through one of these sources:
//!
//! - `ENGRAM_CLOUD_ENCRYPTION_KEY`: base64 or hex encoded 32-byte key material.
//! - `ENGRAM_CLOUD_ENCRYPTION_KEY_FILE`: path to a file containing the same
//!   encoded key material.
//!
//! The key bytes are never printed by Debug or error output. The stable key ID
//! is derived from SHA-256(key bytes) and is safe to use as rotation metadata.

use std::env;
use std::fmt;
use std::path::PathBuf;

use base64::Engine;
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroizing;

use crate::error::EngramError;

pub const KEY_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_KEY";
pub const KEY_FILE_ENV: &str = "ENGRAM_CLOUD_ENCRYPTION_KEY_FILE";
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

        let previous_key_id = match env::var(PREVIOUS_KEY_ID_ENV) {
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

        Self::from_material(key_value, previous_key_id)
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
            rotation_metadata,
        })
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
    match env::var(KEY_ENV) {
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

fn derive_key_id(bytes: &[u8; KEY_BYTES_LEN]) -> CloudKeyId {
    let digest = Sha256::digest(bytes);
    CloudKeyId(format!("sha256:{}", hex::encode(&digest[..16])))
}

fn parse_encoded_key(raw: &str) -> Result<Zeroizing<[u8; KEY_BYTES_LEN]>, KeyConfigError> {
    let trimmed = raw.trim();
    if let Some(value) = trimmed.strip_prefix("base64:") {
        decode_exact_key(decode_base64(value)?)
    } else if let Some(value) = trimmed.strip_prefix("hex:") {
        decode_exact_key(decode_hex(value)?)
    } else {
        parse_unprefixed_key(trimmed)
    }
}

fn parse_unprefixed_key(raw: &str) -> Result<Zeroizing<[u8; KEY_BYTES_LEN]>, KeyConfigError> {
    if let Ok(decoded) = decode_base64(raw) {
        if decoded.len() == KEY_BYTES_LEN {
            return decode_exact_key(decoded);
        }
    }

    decode_exact_key(decode_hex(raw)?)
}

fn decode_exact_key(
    decoded: Zeroizing<Vec<u8>>,
) -> Result<Zeroizing<[u8; KEY_BYTES_LEN]>, KeyConfigError> {
    if decoded.len() != KEY_BYTES_LEN {
        return Err(KeyConfigError::MalformedKey);
    }

    let mut bytes = Zeroizing::new([0u8; KEY_BYTES_LEN]);
    bytes.copy_from_slice(&decoded);
    Ok(bytes)
}

fn decode_base64(raw: &str) -> Result<Zeroizing<Vec<u8>>, KeyConfigError> {
    base64::engine::general_purpose::STANDARD
        .decode(raw)
        .map(Zeroizing::new)
        .map_err(|_| KeyConfigError::MalformedKey)
}

fn decode_hex(raw: &str) -> Result<Zeroizing<Vec<u8>>, KeyConfigError> {
    hex::decode(raw)
        .map(Zeroizing::new)
        .map_err(|_| KeyConfigError::MalformedKey)
}
