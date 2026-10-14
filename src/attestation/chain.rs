//! Attestation chain — blockchain-style chained records
//!
//! Manages an append-only chain of attestation records. Each record includes a
//! `previous_hash` pointing to the preceding record, forming a tamper-evident
//! chain similar to a blockchain.

use chrono::Utc;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::{EngramError, Result};
use crate::storage::Storage;

use super::types::{AttestationFilter, AttestationRecord, ChainStatus};

/// Genesis sentinel used as `previous_hash` for the first record.
///
/// This is a well-known constant, not a random or signed commitment. It means
/// the chain has no cryptographic anchor to a trusted root: an adversary with
/// direct DB access can insert a plausible first record with
/// `previous_hash = "genesis"` and a valid `record_hash`, indistinguishable
/// from a legitimate genesis record.
///
/// For stronger guarantees, replace this with a per-chain random nonce
/// generated at chain creation time and stored out-of-band (e.g. in a
/// separate `attestation_config` table).
const GENESIS_HASH: &str = "genesis";

/// Sentinel for hash scheme version 1 (legacy pipe-delimited format).
pub const LEGACY_HASH_VERSION: i32 = 1;

/// Current hash scheme version for newly created attestation records (v2 length-prefixed canonical format).
pub const CURRENT_HASH_VERSION: i32 = 2;

/// Maximum allowed document size (100 MB). Documents larger than this would
/// block the thread synchronously during SHA-256 hashing.
const MAX_DOCUMENT_BYTES: usize = 100 * 1024 * 1024;

/// Maximum number of memory IDs per attestation record. Larger slices cause
/// unbounded JSON serialization.
const MAX_MEMORY_IDS: usize = 10_000;

/// Manages the append-only attestation chain
pub struct AttestationChain {
    storage: Storage,
}

impl AttestationChain {
    /// Create a new chain backed by the given storage
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Log a document ingestion, creating a chained attestation record.
    ///
    /// Steps:
    /// 1. Compute SHA-256 hash of `content`
    /// 2. Get the last record's `record_hash` (or `"genesis"` if first)
    /// 3. Build the record with `previous_hash` = last record's hash
    /// 4. Compute `record_hash` = SHA-256 of the canonical representation
    /// 5. Optionally sign with Ed25519
    /// 6. Insert into `attestation_log` and return the created record
    pub fn log_document(
        &self,
        content: &[u8],
        document_name: &str,
        agent_id: Option<&str>,
        memory_ids: &[i64],
        sign_key: Option<&[u8; 32]>,
    ) -> Result<AttestationRecord> {
        if document_name.trim().is_empty() {
            return Err(EngramError::InvalidInput(
                "document_name must not be empty".to_string(),
            ));
        }
        if document_name.len() > 1_000 {
            return Err(EngramError::InvalidInput(
                "document_name too long (max 1000 characters)".to_string(),
            ));
        }
        if document_name.contains('\0') {
            return Err(EngramError::InvalidInput(
                "document_name must not contain null bytes".to_string(),
            ));
        }
        if document_name.contains('|') {
            return Err(EngramError::InvalidInput(
                "document_name must not contain pipe '|' characters".to_string(),
            ));
        }

        if let Some(id) = agent_id {
            if id.len() > 256 {
                return Err(EngramError::InvalidInput(
                    "agent_id must not exceed 256 characters".to_string(),
                ));
            }
            if id.chars().any(|c| c.is_control()) {
                return Err(EngramError::InvalidInput(
                    "agent_id must not contain control characters".to_string(),
                ));
            }
            if id.contains('|') {
                return Err(EngramError::InvalidInput(
                    "agent_id must not contain pipe '|' characters".to_string(),
                ));
            }
        }

        // M1: reject documents that would block the thread during hashing
        if content.len() > MAX_DOCUMENT_BYTES {
            return Err(EngramError::InvalidInput(format!(
                "document too large: {} bytes (max {})",
                content.len(),
                MAX_DOCUMENT_BYTES
            )));
        }

        // M2: reject oversized or negative memory_ids
        if memory_ids.len() > MAX_MEMORY_IDS {
            return Err(EngramError::InvalidInput(format!(
                "too many memory_ids: {} (max {})",
                memory_ids.len(),
                MAX_MEMORY_IDS
            )));
        }
        if memory_ids.iter().any(|&id| id < 0) {
            return Err(EngramError::InvalidInput(
                "memory_ids must be non-negative".to_string(),
            ));
        }

        let document_hash = hash_bytes(content);
        let document_size = content.len();
        let ingested_at = Utc::now();
        let memory_ids_vec: Vec<i64> = memory_ids.to_vec();
        let agent_id_owned = agent_id.map(str::to_string);
        let sign_key_owned = sign_key.copied();

        self.storage.with_transaction(|conn| {
            let previous_hash: String = {
                let mut stmt = conn
                    .prepare("SELECT record_hash FROM attestation_log ORDER BY id DESC LIMIT 1")?;
                let mut rows = stmt.query([])?;
                match rows.next()? {
                    Some(row) => row.get(0)?,
                    None => GENESIS_HASH.to_string(),
                }
            };

            let mut record = AttestationRecord {
                id: None,
                document_hash,
                document_name: document_name.to_string(),
                document_size,
                ingested_at,
                agent_id: agent_id_owned,
                memory_ids: memory_ids_vec,
                previous_hash,
                record_hash: String::new(),
                signature: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: None,
                hash_version: CURRENT_HASH_VERSION,
            };

            record.record_hash = Self::compute_record_hash_v2(&record);

            // M3 (key zeroization): `SigningKey` is `ZeroizeOnDrop` in ed25519-dalek 2.x,
            // so the copy created from `key_bytes` is zeroed when it drops here.
            // The caller's source buffer (the `sign_key: Option<&[u8; 32]>` argument)
            // is NOT zeroed automatically. Callers that own the key material should
            // wrap it in `zeroize::Zeroizing<[u8; 32]>` to ensure the source is also
            // cleared after use.
            if let Some(key_bytes) = sign_key_owned.as_ref() {
                record.signature = Some(sign_record_hash(&record.record_hash, key_bytes)?);
            }

            let memory_ids_json =
                serde_json::to_string(&record.memory_ids).map_err(EngramError::Serialization)?;
            let metadata_json =
                serde_json::to_string(&record.metadata).map_err(EngramError::Serialization)?;

            // M4: guard against unbounded metadata at the insert boundary.
            // This is the single point all paths must pass through, so future
            // callers that set metadata on the record before insertion are
            // automatically covered.
            const MAX_METADATA_BYTES: usize = 65_536;
            if metadata_json.len() > MAX_METADATA_BYTES {
                return Err(EngramError::InvalidInput(format!(
                    "metadata too large: {} bytes (max {})",
                    metadata_json.len(),
                    MAX_METADATA_BYTES
                )));
            }

            conn.execute(
                "INSERT INTO attestation_log
                    (document_hash, document_name, document_size, ingested_at,
                     agent_id, memory_ids, previous_hash, record_hash, signature, metadata, hash_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    record.document_hash,
                    record.document_name,
                    record.document_size as i64,
                    record.ingested_at.to_rfc3339(),
                    record.agent_id,
                    memory_ids_json,
                    record.previous_hash,
                    record.record_hash,
                    record.signature,
                    metadata_json,
                    record.hash_version,
                ],
            )?;

            let id = conn.last_insert_rowid();
            let created_at_str: String = conn.query_row(
                "SELECT created_at FROM attestation_log WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok();

            Ok(AttestationRecord {
                id: Some(id),
                created_at,
                ..record
            })
        })
    }

    /// Check whether a document (by content) has already been attested.
    ///
    /// Returns the first matching record, or `None`.
    pub fn verify_document(&self, content: &[u8]) -> Result<Option<AttestationRecord>> {
        let hash = hash_bytes(content);
        self.storage.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, document_hash, document_name, document_size, ingested_at,
                        agent_id, memory_ids, previous_hash, record_hash, signature,
                        metadata, created_at, hash_version
                 FROM attestation_log
                 WHERE document_hash = ?1
                 ORDER BY id ASC
                 LIMIT 1",
            )?;
            let mut rows = stmt.query(rusqlite::params![hash])?;
            match rows.next()? {
                Some(row) => Ok(Some(row_to_record(row)?)),
                None => Ok(None),
            }
        })
    }

    /// Verify the integrity of the entire chain.
    ///
    /// Walks all records in insertion order and checks:
    /// - `previous_hash` of each record matches the `record_hash` of the preceding one
    /// - Each `record_hash` is correctly computed from the record's fields (supporting both v1 and v2 schemes)
    /// - If `verifying_key` is `Some`, every record must carry a valid Ed25519
    ///   signature for its `record_hash`
    pub fn verify_chain(&self, verifying_key: Option<&[u8; 32]>) -> Result<ChainStatus> {
        // M5: Process rows one at a time (streaming) to avoid loading the entire
        // attestation_log table into memory. Only `expected_previous` and a
        // counter are kept in memory between rows.
        self.storage.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, document_hash, document_name, document_size, ingested_at,
                        agent_id, memory_ids, previous_hash, record_hash, signature,
                        metadata, created_at, hash_version
                 FROM attestation_log
                 ORDER BY id ASC",
            )?;

            let mut expected_previous = GENESIS_HASH.to_string();
            let mut record_count: usize = 0;
            let mut rows = stmt.query([])?;

            while let Some(row) = rows.next()? {
                let record = row_to_record(row).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )),
                    )
                })?;
                record_count += 1;

                // 1. Check linkage (constant-time to prevent timing oracle)
                let linkage_ok: bool = record
                    .previous_hash
                    .as_bytes()
                    .ct_eq(expected_previous.as_bytes())
                    .into();
                if !linkage_ok {
                    return Ok(ChainStatus::Broken {
                        at_record_id: record.id.unwrap_or(-1),
                        expected_hash: expected_previous,
                        actual_hash: record.previous_hash.clone(),
                    });
                }

                // 2. Recompute record_hash and compare (constant-time).
                // Supports v1 legacy delimiter format and v2 canonical format.
                let recomputed = match record.hash_version {
                    1 => {
                        let v1_hash = Self::compute_record_hash_v1(&record);
                        if bool::from(v1_hash.as_bytes().ct_eq(record.record_hash.as_bytes())) {
                            v1_hash
                        } else {
                            let v1_meta = Self::compute_record_hash_v1_with_meta(&record);
                            if bool::from(v1_meta.as_bytes().ct_eq(record.record_hash.as_bytes())) {
                                v1_meta
                            } else {
                                v1_hash
                            }
                        }
                    }
                    _ => Self::compute_record_hash_v2(&record),
                };
                let hash_ok: bool = recomputed
                    .as_bytes()
                    .ct_eq(record.record_hash.as_bytes())
                    .into();
                if !hash_ok {
                    return Ok(ChainStatus::Broken {
                        at_record_id: record.id.unwrap_or(-1),
                        expected_hash: recomputed,
                        actual_hash: record.record_hash.clone(),
                    });
                }

                // 3. Verify Ed25519 signature when the caller provides a key.
                if let Some(vk_bytes) = verifying_key {
                    let Some(sig_hex) = record.signature.as_deref() else {
                        return Ok(ChainStatus::Broken {
                            at_record_id: record.id.unwrap_or(-1),
                            expected_hash: format!(
                                "Ed25519 signature for record_hash {}",
                                record.record_hash
                            ),
                            actual_hash: "missing signature".to_string(),
                        });
                    };

                    let valid = verify_signature(&record.record_hash, sig_hex, vk_bytes)?;
                    if !valid {
                        return Ok(ChainStatus::Broken {
                            at_record_id: record.id.unwrap_or(-1),
                            expected_hash: format!(
                                "valid Ed25519 signature for record_hash {}",
                                record.record_hash
                            ),
                            actual_hash: "invalid signature".to_string(),
                        });
                    }
                }

                expected_previous = record.record_hash;
            }

            if record_count == 0 {
                Ok(ChainStatus::Empty)
            } else {
                Ok(ChainStatus::Valid { record_count })
            }
        })
    }

    /// List attestation records with optional filters
    pub fn list(&self, filter: &AttestationFilter) -> Result<Vec<AttestationRecord>> {
        let limit = filter.limit.unwrap_or(100) as i64;
        let offset = filter.offset.unwrap_or(0) as i64;
        let agent_id = filter.agent_id.clone();
        let document_name = filter.document_name.clone();

        self.storage.with_connection(|conn| {
            // Build a flexible query with optional filters
            let mut conditions: Vec<String> = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(ref aid) = agent_id {
                conditions.push(format!("agent_id = ?{}", param_values.len() + 1));
                param_values.push(Box::new(aid.clone()));
            }
            if let Some(ref name) = document_name {
                conditions.push(format!(
                    "document_name LIKE ?{} ESCAPE '\\'",
                    param_values.len() + 1
                ));
                let escaped = name
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_");
                param_values.push(Box::new(format!("%{}%", escaped)));
            }

            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            let limit_idx = param_values.len() + 1;
            let offset_idx = param_values.len() + 2;
            param_values.push(Box::new(limit));
            param_values.push(Box::new(offset));

            let sql = format!(
                "SELECT id, document_hash, document_name, document_size, ingested_at,
                        agent_id, memory_ids, previous_hash, record_hash, signature,
                        metadata, created_at, hash_version
                 FROM attestation_log
                 {}
                 ORDER BY id ASC
                 LIMIT ?{} OFFSET ?{}",
                where_clause, limit_idx, offset_idx
            );

            let mut stmt = conn.prepare(&sql)?;
            let refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|b| b.as_ref()).collect();
            let rows = stmt.query_map(refs.as_slice(), |row| {
                row_to_record(row).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )),
                    )
                })
            })?;
            rows.collect::<std::result::Result<Vec<_>, _>>()
                .map_err(EngramError::Database)
        })
    }

    // ─── Private helpers ────────────────────────────────────────────────────

    /// Compute the record hash for a record according to its `hash_version`.
    pub fn compute_record_hash(record: &AttestationRecord) -> String {
        match record.hash_version {
            1 => Self::compute_record_hash_v1(record),
            _ => Self::compute_record_hash_v2(record),
        }
    }

    /// Compute v1 legacy record hash (delimiter-separated).
    ///
    /// Hash = SHA-256 of `document_hash|document_name|document_size|ingested_at|agent_id|memory_ids|previous_hash`
    pub fn compute_record_hash_v1(record: &AttestationRecord) -> String {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            record.document_hash,
            record.document_name,
            record.document_size,
            record.ingested_at.to_rfc3339(),
            record.agent_id.as_deref().unwrap_or(""),
            serde_json::to_string(&record.memory_ids).unwrap_or_default(),
            record.previous_hash,
        );
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    /// Compute intermediate v1 record hash with metadata appended.
    pub fn compute_record_hash_v1_with_meta(record: &AttestationRecord) -> String {
        let metadata_str = if record.metadata.is_null() {
            String::new()
        } else {
            serde_json::to_string(&record.metadata).unwrap_or_default()
        };
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            record.document_hash,
            record.document_name,
            record.document_size,
            record.ingested_at.to_rfc3339(),
            record.agent_id.as_deref().unwrap_or(""),
            serde_json::to_string(&record.memory_ids).unwrap_or_default(),
            record.previous_hash,
            metadata_str,
        );
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    /// Compute v2 canonical record hash (length-prefixed fields with metadata).
    pub fn compute_record_hash_v2(record: &AttestationRecord) -> String {
        let fields: Vec<String> = vec![
            record.document_hash.clone(),
            record.document_name.clone(),
            record.document_size.to_string(),
            record.ingested_at.to_rfc3339(),
            record.agent_id.as_deref().unwrap_or("").to_string(),
            serde_json::to_string(&record.memory_ids).unwrap_or_default(),
            record.previous_hash.clone(),
            serde_json::to_string(&record.metadata).unwrap_or_default(),
        ];
        let canonical = fields
            .iter()
            .map(|f| format!("{}:{}", f.len(), f))
            .collect::<Vec<_>>()
            .join("|");
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }
}

// ─── Standalone helpers ──────────────────────────────────────────────────────

/// Compute SHA-256 hash of raw bytes and return as `"sha256:{hex}"` string
fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Sign the `record_hash` string with an Ed25519 secret key.
/// Returns hex-encoded signature.
///
fn sign_record_hash(record_hash: &str, secret_key_bytes: &[u8; 32]) -> Result<String> {
    use ed25519_dalek::{Signature, Signer, SigningKey};

    let signing_key = SigningKey::from_bytes(secret_key_bytes);
    let signature: Signature = signing_key.sign(record_hash.as_bytes());
    Ok(hex::encode(signature.to_bytes()))
}

/// Verify an Ed25519 signature stored as a hex string.
///
/// Returns `Ok(true)` if valid, `Ok(false)` if the signature is invalid, and
/// `Err` if the key bytes or signature bytes are malformed.
fn verify_signature(
    record_hash: &str,
    signature_hex: &str,
    verifying_key_bytes: &[u8; 32],
) -> Result<bool> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let key = VerifyingKey::from_bytes(verifying_key_bytes)
        .map_err(|e| EngramError::InvalidInput(format!("invalid verifying key: {e}")))?;

    let sig_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(false),
    };

    let sig_array: [u8; 64] = match sig_bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(_) => return Ok(false),
    };

    let signature = Signature::from_bytes(&sig_array);
    Ok(key.verify(record_hash.as_bytes(), &signature).is_ok())
}

/// Deserialise a database row into an `AttestationRecord`
fn row_to_record(row: &rusqlite::Row<'_>) -> Result<AttestationRecord> {
    let id: i64 = row.get(0)?;
    let document_hash: String = row.get(1)?;
    let document_name: String = row.get(2)?;
    let document_size: i64 = row.get(3)?;
    let ingested_at_str: String = row.get(4)?;
    let agent_id: Option<String> = row.get(5)?;
    let memory_ids_json: String = row.get(6)?;
    let previous_hash: String = row.get(7)?;
    let record_hash: String = row.get(8)?;
    let signature: Option<String> = row.get(9)?;
    let metadata_json: String = row.get(10)?;
    let created_at_str: Option<String> = row.get(11)?;
    let hash_version: i32 = row.get(12).unwrap_or(1);

    // M6: rusqlite's `Error::display()` does not embed the DB file path in its
    // string representation, so these errors are safe to propagate as-is.
    // However, they should be mapped to generic user-facing messages at the
    // API boundary (MCP handler level) rather than surfaced directly, to avoid
    // leaking internal schema details.
    let ingested_at = chrono::DateTime::parse_from_rfc3339(&ingested_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| EngramError::Storage(format!("invalid ingested_at: {e}")))?;

    let memory_ids: Vec<i64> = serde_json::from_str(&memory_ids_json)
        .map_err(|e| EngramError::Storage(format!("invalid memory_ids JSON: {e}")))?;

    let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
        .map_err(|e| EngramError::Storage(format!("invalid metadata JSON: {e}")))?;

    let created_at = created_at_str.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(AttestationRecord {
        id: Some(id),
        document_hash,
        document_name,
        document_size: document_size as usize,
        ingested_at,
        agent_id,
        memory_ids,
        previous_hash,
        record_hash,
        signature,
        metadata,
        created_at,
        hash_version,
    })
}

#[cfg(test)]
#[path = "chain_tests.rs"]
mod tests;
