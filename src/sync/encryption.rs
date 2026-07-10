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
    let configured_key_id = provider.active_key().id().as_str();
    if payload_key_id != configured_key_id {
        return Err(EngramError::Encryption(format!(
            "encrypted payload key id {payload_key_id} does not match configured key id {configured_key_id}; refusing destructive re-encryption"
        )));
    }

    decrypt_aes_gcm(
        provider,
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

    decrypt_aes_gcm(
        provider,
        &data[..ENCRYPTION_NONCE_LEN],
        &data[ENCRYPTION_NONCE_LEN..],
    )
}

fn decrypt_aes_gcm(
    provider: &ConfiguredCloudKeyProvider,
    nonce_bytes: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let key = provider.active_key();
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
mod tests {
    use super::*;
    use crate::sync::key_config::{EncodedKeyMaterial, KEY_BYTES_LEN};
    use zeroize::Zeroizing;

    fn provider_from_byte(byte: u8) -> ConfiguredCloudKeyProvider {
        let key = [byte; KEY_BYTES_LEN];
        ConfiguredCloudKeyProvider::from_material(
            EncodedKeyMaterial::Inline(Zeroizing::new(format!("hex:{}", hex::encode(key)))),
            None,
        )
        .expect("test key provider loads")
    }

    fn encrypt_legacy_payload_with_nonce(
        provider: &ConfiguredCloudKeyProvider,
        plaintext: &[u8],
        nonce_bytes: [u8; ENCRYPTION_NONCE_LEN],
    ) -> Result<Vec<u8>> {
        let key = provider.active_key();
        let cipher = key
            .with_bytes(|bytes| Aes256Gcm::new_from_slice(bytes))
            .map_err(|_| EngramError::Encryption("configured key is invalid".to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EngramError::Encryption("test encryption failed".to_string()))?;
        let mut payload = Vec::with_capacity(ENCRYPTION_NONCE_LEN + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
        Ok(payload)
    }

    #[test]
    fn configured_key_reload_decrypts_versioned_payload() {
        // Given: two providers loaded from the same durable key material.
        let (provider_before_restart, provider_after_restart) =
            (provider_from_byte(42), provider_from_byte(42));
        let plaintext = b"database bytes";

        // When: data encrypted before restart is decrypted after restart.
        let encrypted = encrypt_data_with_provider(&provider_before_restart, plaintext)
            .expect("test payload encrypts");
        let decrypted = decrypt_data_with_provider(&provider_after_restart, &encrypted)
            .expect("test payload decrypts after reload");

        // Then: decrypt succeeds and the versioned payload carries the stable key ID.
        assert_eq!(decrypted, plaintext);
        assert!(encrypted.starts_with(ENCRYPTED_PAYLOAD_MAGIC));
        assert_eq!(
            provider_before_restart.active_key().id(),
            provider_after_restart.active_key().id()
        );
    }

    #[test]
    fn different_configured_key_returns_explicit_non_destructive_error() {
        // Given: a payload written with a different durable key.
        let (writer, reader) = (provider_from_byte(7), provider_from_byte(8));
        let encrypted =
            encrypt_data_with_provider(&writer, b"secret db").expect("test payload encrypts");

        // When: decrypt is attempted with the wrong configured key.
        let message = decrypt_data_with_provider(&reader, &encrypted)
            .expect_err("wrong configured key must fail")
            .to_string();

        // Then: the error names the key mismatch and avoids plaintext/key bytes.
        assert!(message.contains("does not match configured key id"));
        assert!(message.contains("refusing destructive re-encryption"));
        assert!(!message.contains("secret db"));
        assert!(!message.contains(&hex::encode([7u8; KEY_BYTES_LEN])));
    }

    #[test]
    fn corrupted_versioned_payload_does_not_fall_back_to_legacy() {
        // Given: a well-formed versioned payload for the configured key with corrupted ciphertext.
        let provider = provider_from_byte(9);
        let mut encrypted =
            encrypt_data_with_provider(&provider, b"secret db").expect("test payload encrypts");
        *encrypted
            .last_mut()
            .expect("encrypted payload is not empty") ^= 0x01;

        // When: decrypt is attempted.
        let message = decrypt_data_with_provider(&provider, &encrypted)
            .expect_err("corruption must fail")
            .to_string();

        // Then: the versioned envelope remains authoritative and does not retry as legacy.
        assert!(message.contains("decryption failed for configured key id"));
        assert!(!message.contains("secret db"));
    }

    #[test]
    fn configured_key_decrypts_legacy_nonce_ciphertext_payloads() {
        // Given: legacy encrypted payload bytes without versioned key-id metadata.
        let (writer, reader) = (provider_from_byte(11), provider_from_byte(11));
        let plaintext = b"legacy database bytes";
        let legacy_payload =
            encrypt_legacy_payload_with_nonce(&writer, plaintext, [3u8; ENCRYPTION_NONCE_LEN])
                .expect("legacy payload encrypts");

        // When: the configured provider decrypts the legacy nonce+ciphertext format.
        let decrypted = decrypt_data_with_provider(&reader, &legacy_payload)
            .expect("legacy payload decrypts with configured key");

        // Then: read compatibility is preserved for data encrypted with the same durable key.
        assert_eq!(decrypted, plaintext);
        assert!(!legacy_payload.starts_with(ENCRYPTED_PAYLOAD_MAGIC));
    }

    #[test]
    fn legacy_payload_with_magic_nonce_prefix_decrypts_as_legacy() {
        // Given: a legacy payload whose random nonce starts with the versioned magic bytes.
        let (writer, reader) = (provider_from_byte(12), provider_from_byte(12));
        let plaintext = b"legacy nonce collision bytes";
        let nonce = *b"EGK1legacy!!";
        let legacy_payload = encrypt_legacy_payload_with_nonce(&writer, plaintext, nonce)
            .expect("legacy payload encrypts");

        // When: decrypt sees the EGK1 prefix.
        let decrypted = decrypt_data_with_provider(&reader, &legacy_payload)
            .expect("legacy magic-prefix payload decrypts with configured key");

        // Then: the nonce prefix is treated as legacy unless a well-formed key-id envelope follows.
        assert_eq!(decrypted, plaintext);
        assert!(legacy_payload.starts_with(ENCRYPTED_PAYLOAD_MAGIC));
    }

    #[test]
    fn malformed_versioned_payload_returns_redacted_error() {
        // Given: a versioned payload with a truncated metadata section.
        let (provider, malformed) = (provider_from_byte(1), ENCRYPTED_PAYLOAD_MAGIC.to_vec());

        // When: decrypt is attempted.
        let error = decrypt_data_with_provider(&provider, &malformed)
            .expect_err("malformed payload must fail");

        // Then: the error is explicit and redacted.
        assert!(error.to_string().contains("header is truncated"));
        assert!(!format!("{error:?}").contains(&hex::encode([1u8; KEY_BYTES_LEN])));
    }
}
