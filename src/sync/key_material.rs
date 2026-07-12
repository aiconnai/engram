use base64::Engine;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::{CloudKeyId, KeyConfigError, KEY_BYTES_LEN};

pub(super) fn derive_key_id(bytes: &[u8; KEY_BYTES_LEN]) -> CloudKeyId {
    let digest = Sha256::digest(bytes);
    CloudKeyId(format!("sha256:{}", hex::encode(&digest[..16])))
}

pub(super) fn parse_encoded_key(
    raw: &str,
) -> Result<Zeroizing<[u8; KEY_BYTES_LEN]>, KeyConfigError> {
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
