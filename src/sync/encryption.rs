use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use super::key_config::{CloudKeyProvider, ConfiguredCloudKeyProvider};
use crate::error::{EngramError, Result};

const ENCRYPTED_PAYLOAD_MAGIC: &[u8; 4] = b"EGK1";
const ENCRYPTION_NONCE_LEN: usize = 12;

pub(super) fn encrypt_data_with_provider(
    provider: &ConfiguredCloudKeyProvider,
    data: &[u8],
) -> Result<Vec<u8>> {
    let key = provider.active_key();
    let cipher = key
        .with_bytes(|bytes| Aes256Gcm::new_from_slice(bytes))
        .map_err(|_| EngramError::Encryption("configured key is invalid".to_string()))?;

    let mut nonce_bytes = [0u8; ENCRYPTION_NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data).map_err(|_| {
        EngramError::Encryption(format!(
            "encryption failed with configured key id {}",
            key.id().as_str()
        ))
    })?;

    let key_id = key.id().as_str().as_bytes();
    let key_id_len = u8::try_from(key_id.len())
        .map_err(|_| EngramError::Encryption("configured key id is too long".to_string()))?;
    let mut result = Vec::with_capacity(
        ENCRYPTED_PAYLOAD_MAGIC.len() + 1 + key_id.len() + nonce_bytes.len() + ciphertext.len(),
    );
    result.extend_from_slice(ENCRYPTED_PAYLOAD_MAGIC);
    result.push(key_id_len);
    result.extend_from_slice(key_id);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub(super) fn decrypt_data_with_provider(
    provider: &ConfiguredCloudKeyProvider,
    data: &[u8],
) -> Result<Vec<u8>> {
    if is_versioned_payload_for_any_configured_key(data) {
        decrypt_versioned_payload(provider, data)
    } else {
        decrypt_legacy_payload(provider, data)
    }
}

pub(super) fn decrypt_data_with_provider_for_key_id(
    provider: &ConfiguredCloudKeyProvider,
    data: &[u8],
    expected_key_id: &str,
) -> Result<Vec<u8>> {
    let payload_key_id = versioned_payload_key_id(data)?.ok_or_else(|| {
        EngramError::Encryption(
            "encrypted object metadata requires a versioned payload; refusing destructive re-encryption"
                .to_string(),
        )
    })?;
    if payload_key_id != expected_key_id {
        return Err(EngramError::Encryption(format!(
            "encrypted object key id {expected_key_id} does not match payload key id {payload_key_id}; refusing destructive re-encryption"
        )));
    }

    decrypt_versioned_payload(provider, data)
}

pub(super) fn is_versioned_encrypted_payload(data: &[u8]) -> bool {
    is_versioned_payload_for_any_configured_key(data)
}

fn is_versioned_payload_for_any_configured_key(data: &[u8]) -> bool {
    if !data.starts_with(ENCRYPTED_PAYLOAD_MAGIC) {
        return false;
    }
    let header_len = ENCRYPTED_PAYLOAD_MAGIC.len() + 1;
    if data.len() < header_len {
        return true;
    }

    let key_id_len = usize::from(data[ENCRYPTED_PAYLOAD_MAGIC.len()]);
    let nonce_start = header_len + key_id_len;
    let ciphertext_start = nonce_start + ENCRYPTION_NONCE_LEN;
    if data.len() < ciphertext_start {
        return false;
    }

    std::str::from_utf8(&data[header_len..nonce_start])
        .map(is_configured_key_id)
        .unwrap_or(false)
}

fn is_configured_key_id(value: &str) -> bool {
    const KEY_ID_PREFIX: &str = "sha256:";
    let Some(hex_digest) = value.strip_prefix(KEY_ID_PREFIX) else {
        return false;
    };
    hex_digest.len() == 32 && hex_digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn versioned_payload_key_id(data: &[u8]) -> Result<Option<&str>> {
    if !data.starts_with(ENCRYPTED_PAYLOAD_MAGIC) {
        return Ok(None);
    }
    let header_len = ENCRYPTED_PAYLOAD_MAGIC.len() + 1;
    if data.len() < header_len {
        return Err(EngramError::Encryption(
            "encrypted payload header is truncated".to_string(),
        ));
    }
    let key_id_len = usize::from(data[ENCRYPTED_PAYLOAD_MAGIC.len()]);
    let nonce_start = header_len + key_id_len;
    let ciphertext_start = nonce_start + ENCRYPTION_NONCE_LEN;
    if data.len() < ciphertext_start {
        return Err(EngramError::Encryption(
            "encrypted payload metadata is truncated".to_string(),
        ));
    }
    let key_id = std::str::from_utf8(&data[header_len..nonce_start]).map_err(|_| {
        EngramError::Encryption("encrypted payload key id is malformed".to_string())
    })?;
    if !is_configured_key_id(key_id) {
        return Err(EngramError::Encryption(
            "encrypted payload key id is malformed".to_string(),
        ));
    }
    Ok(Some(key_id))
}

fn decrypt_versioned_payload(
    provider: &ConfiguredCloudKeyProvider,
    data: &[u8],
) -> Result<Vec<u8>> {
    let header_len = ENCRYPTED_PAYLOAD_MAGIC.len() + 1;
    if data.len() < header_len {
        return Err(EngramError::Encryption(
            "encrypted payload header is truncated".to_string(),
        ));
    }

    let key_id_len = usize::from(data[ENCRYPTED_PAYLOAD_MAGIC.len()]);
    let nonce_start = header_len + key_id_len;
    let ciphertext_start = nonce_start + ENCRYPTION_NONCE_LEN;
    if data.len() < ciphertext_start {
        return Err(EngramError::Encryption(
            "encrypted payload metadata is truncated".to_string(),
        ));
    }

    let payload_key_id = std::str::from_utf8(&data[header_len..nonce_start]).map_err(|_| {
        EngramError::Encryption("encrypted payload key id is malformed".to_string())
    })?;
    if !is_configured_key_id(payload_key_id) {
        return Err(EngramError::Encryption(
            "encrypted payload key id is malformed".to_string(),
        ));
    }
    let Some(key) = provider.key_for_id(payload_key_id) else {
        let configured_key_id = provider.active_key().id().as_str();
        return Err(EngramError::Encryption(format!(
            "encrypted payload key id {payload_key_id} does not match configured key id {configured_key_id}; refusing destructive re-encryption"
        )));
    };

    decrypt_aes_gcm(
        key,
        &data[nonce_start..ciphertext_start],
        &data[ciphertext_start..],
    )
}

fn decrypt_legacy_payload(provider: &ConfiguredCloudKeyProvider, data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < ENCRYPTION_NONCE_LEN {
        return Err(EngramError::Encryption(
            "encrypted payload is too short".to_string(),
        ));
    }

    let active_result = decrypt_aes_gcm(
        provider.active_key(),
        &data[..ENCRYPTION_NONCE_LEN],
        &data[ENCRYPTION_NONCE_LEN..],
    );
    match (active_result, provider.previous_key()) {
        (Ok(plaintext), _) => Ok(plaintext),
        (Err(_), Some(previous_key)) => decrypt_aes_gcm(
            previous_key,
            &data[..ENCRYPTION_NONCE_LEN],
            &data[ENCRYPTION_NONCE_LEN..],
        ),
        (Err(error), None) => Err(error),
    }
}

fn decrypt_aes_gcm(
    key: &super::key_config::CloudEncryptionKey,
    nonce_bytes: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let cipher = key
        .with_bytes(|bytes| Aes256Gcm::new_from_slice(bytes))
        .map_err(|_| EngramError::Encryption("configured key is invalid".to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext).map_err(|_| {
        EngramError::Encryption(format!(
            "decryption failed for configured key id {}; key may differ or payload is corrupted",
            key.id().as_str()
        ))
    })
}

#[cfg(test)]
#[path = "encryption_tests.rs"]
mod tests;
