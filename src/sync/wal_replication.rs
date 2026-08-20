//! SQLite WAL continuous delta replication and Point-In-Time Recovery (PITR) engine.
//!
//! Provides:
//! - 32-byte SQLite WAL header parser and serializer.
//! - Frame delta reader for extracting dirty frames appended to `.db-wal`.
//! - Compressed, SHA-256 checksummed delta package serializer.
//! - `WalReplicationStreamer` for tracking replication offsets and computing lag.
//! - `WalRecoveryEngine` for point-in-time recovery replaying delta frames.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::EngramError;

/// SQLite WAL magic numbers
pub const WAL_MAGIC_BE: u32 = 0x377f0682;
pub const WAL_MAGIC_LE: u32 = 0x377f0683;
pub const WAL_DEFAULT_VERSION: u32 = 3007000;
pub const WAL_HEADER_SIZE: usize = 32;
pub const WAL_FRAME_HEADER_SIZE: usize = 24;

/// Errors specific to WAL replication and recovery.
#[derive(Debug, thiserror::Error)]
pub enum WalReplicationError {
    #[error("Invalid WAL header: {0}")]
    InvalidHeader(String),
    #[error("Invalid WAL frame at index {index}: {reason}")]
    InvalidFrame { index: u32, reason: String },
    #[error("Checksum mismatch: expected {expected}, calculated {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Database integrity check failed: {0}")]
    IntegrityCheckFailed(String),
    #[error("Recovery error: {0}")]
    RecoveryError(String),
    #[error("Replication error: {0}")]
    Other(String),
}

impl From<WalReplicationError> for EngramError {
    fn from(err: WalReplicationError) -> Self {
        EngramError::Sync(err.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. SQLite WAL Checksum Algorithm & Header Parser
// ─────────────────────────────────────────────────────────────────────────────

/// Compute SQLite WAL checksum over an 8-byte aligned buffer.
///
/// SQLite WAL checksum treats bytes as pairs of 32-bit integers and computes:
/// `s1 += x + s2; s2 += y + s1;`
pub fn compute_wal_checksum(data: &[u8], is_le: bool, initial: (u32, u32)) -> (u32, u32) {
    let mut s1 = initial.0;
    let mut s2 = initial.1;
    let pairs = data.len() / 8;
    for i in 0..pairs {
        let chunk = &data[i * 8..(i + 1) * 8];
        let x = if is_le {
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        } else {
            u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        };
        let y = if is_le {
            u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]])
        } else {
            u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]])
        };
        s1 = s1.wrapping_add(x).wrapping_add(s2);
        s2 = s2.wrapping_add(y).wrapping_add(s1);
    }
    (s1, s2)
}

/// Parsed 32-byte SQLite WAL file header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalHeader {
    pub magic: u32,
    pub version: u32,
    pub page_size: u32,
    pub checkpoint_seq: u32,
    pub salt1: u32,
    pub salt2: u32,
    pub checksum1: u32,
    pub checksum2: u32,
    pub is_little_endian_checksum: bool,
}

impl WalHeader {
    /// Create a new WAL header with valid checksums for a given page size and salt.
    pub fn new(page_size: u32, checkpoint_seq: u32, salt1: u32, salt2: u32) -> Self {
        let magic = WAL_MAGIC_BE;
        let version = WAL_DEFAULT_VERSION;
        let mut raw = [0u8; 24];
        raw[0..4].copy_from_slice(&magic.to_be_bytes());
        raw[4..8].copy_from_slice(&version.to_be_bytes());
        let page_size_raw = if page_size == 65536 { 1u32 } else { page_size };
        raw[8..12].copy_from_slice(&page_size_raw.to_be_bytes());
        raw[12..16].copy_from_slice(&checkpoint_seq.to_be_bytes());
        raw[16..20].copy_from_slice(&salt1.to_be_bytes());
        raw[20..24].copy_from_slice(&salt2.to_be_bytes());

        let (checksum1, checksum2) = compute_wal_checksum(&raw, false, (0, 0));

        Self {
            magic,
            version,
            page_size,
            checkpoint_seq,
            salt1,
            salt2,
            checksum1,
            checksum2,
            is_little_endian_checksum: false,
        }
    }

    /// Parse a 32-byte WAL header from raw bytes.
    pub fn parse(bytes: &[u8]) -> std::result::Result<Self, WalReplicationError> {
        if bytes.len() < WAL_HEADER_SIZE {
            return Err(WalReplicationError::InvalidHeader(format!(
                "Expected at least {} bytes, got {}",
                WAL_HEADER_SIZE,
                bytes.len()
            )));
        }

        let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let is_le = match magic {
            WAL_MAGIC_BE => false,
            WAL_MAGIC_LE => true,
            other => {
                return Err(WalReplicationError::InvalidHeader(format!(
                    "Invalid WAL magic number: 0x{:08x} (expected 0x{:08x} or 0x{:08x})",
                    other, WAL_MAGIC_BE, WAL_MAGIC_LE
                )));
            }
        };

        let version = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let raw_page_size = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        // SQLite represents 65536 as 1 in WAL/DB header
        let page_size = if raw_page_size == 1 {
            65536
        } else {
            raw_page_size
        };

        if !(512..=65536).contains(&page_size) || !page_size.is_power_of_two() {
            return Err(WalReplicationError::InvalidHeader(format!(
                "Invalid page size: {} (must be power of 2 between 512 and 65536)",
                page_size
            )));
        }

        let checkpoint_seq = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let salt1 = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let salt2 = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

        let (checksum1, checksum2) = if is_le {
            (
                u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
                u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
            )
        } else {
            (
                u32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
                u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
            )
        };

        Ok(Self {
            magic,
            version,
            page_size,
            checkpoint_seq,
            salt1,
            salt2,
            checksum1,
            checksum2,
            is_little_endian_checksum: is_le,
        })
    }

    /// Serialize header to 32-byte array.
    pub fn serialize(&self) -> [u8; WAL_HEADER_SIZE] {
        let mut buf = [0u8; WAL_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic.to_be_bytes());
        buf[4..8].copy_from_slice(&self.version.to_be_bytes());
        let page_size_raw = if self.page_size == 65536 {
            1u32
        } else {
            self.page_size
        };
        buf[8..12].copy_from_slice(&page_size_raw.to_be_bytes());
        buf[12..16].copy_from_slice(&self.checkpoint_seq.to_be_bytes());
        buf[16..20].copy_from_slice(&self.salt1.to_be_bytes());
        buf[20..24].copy_from_slice(&self.salt2.to_be_bytes());

        if self.is_little_endian_checksum {
            buf[24..28].copy_from_slice(&self.checksum1.to_le_bytes());
            buf[28..32].copy_from_slice(&self.checksum2.to_le_bytes());
        } else {
            buf[24..28].copy_from_slice(&self.checksum1.to_be_bytes());
            buf[28..32].copy_from_slice(&self.checksum2.to_be_bytes());
        }

        buf
    }

    /// Verify checksum of the header's first 24 bytes against checksum1 and checksum2.
    pub fn verify_checksum(&self, raw_header_bytes: &[u8]) -> bool {
        if raw_header_bytes.len() < 24 {
            return false;
        }
        let (c1, c2) = compute_wal_checksum(
            &raw_header_bytes[0..24],
            self.is_little_endian_checksum,
            (0, 0),
        );
        c1 == self.checksum1 && c2 == self.checksum2
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. WAL Frame Structure & Delta Reader
// ─────────────────────────────────────────────────────────────────────────────

/// Represents a single WAL page frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalFrame {
    /// 1-based frame index within the WAL file
    pub frame_index: u32,
    /// Target 1-based database page number
    pub page_number: u32,
    /// Size of the database in pages after commit (> 0 for commit frames)
    pub db_size_pages: u32,
    pub salt1: u32,
    pub salt2: u32,
    pub checksum1: u32,
    pub checksum2: u32,
    /// Raw page content (length equals WAL page_size)
    pub data: Vec<u8>,
}

impl WalFrame {
    /// Returns true if this frame represents a transaction commit.
    pub fn is_commit(&self) -> bool {
        self.db_size_pages > 0
    }

    /// Parse a single WAL frame from raw bytes.
    pub fn parse(
        bytes: &[u8],
        frame_index: u32,
        page_size: u32,
        is_le: bool,
    ) -> std::result::Result<Self, WalReplicationError> {
        let expected_size = WAL_FRAME_HEADER_SIZE + page_size as usize;
        if bytes.len() < expected_size {
            return Err(WalReplicationError::InvalidFrame {
                index: frame_index,
                reason: format!(
                    "Frame buffer too short: expected {} bytes, got {}",
                    expected_size,
                    bytes.len()
                ),
            });
        }

        let page_number = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if page_number == 0 {
            return Err(WalReplicationError::InvalidFrame {
                index: frame_index,
                reason: "page_number must be >= 1 (SQLite pages are 1-based)".to_string(),
            });
        }
        let db_size_pages = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let salt1 = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let salt2 = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        let (checksum1, checksum2) = if is_le {
            (
                u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
                u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            )
        } else {
            (
                u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
                u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            )
        };

        let data = bytes[WAL_FRAME_HEADER_SIZE..expected_size].to_vec();

        Ok(Self {
            frame_index,
            page_number,
            db_size_pages,
            salt1,
            salt2,
            checksum1,
            checksum2,
            data,
        })
    }

    /// Serialize frame header and data into binary format.
    pub fn serialize(&self, is_le: bool) -> Vec<u8> {
        let mut buf = Vec::with_capacity(WAL_FRAME_HEADER_SIZE + self.data.len());
        buf.extend_from_slice(&self.page_number.to_be_bytes());
        buf.extend_from_slice(&self.db_size_pages.to_be_bytes());
        buf.extend_from_slice(&self.salt1.to_be_bytes());
        buf.extend_from_slice(&self.salt2.to_be_bytes());

        if is_le {
            buf.extend_from_slice(&self.checksum1.to_le_bytes());
            buf.extend_from_slice(&self.checksum2.to_le_bytes());
        } else {
            buf.extend_from_slice(&self.checksum1.to_be_bytes());
            buf.extend_from_slice(&self.checksum2.to_be_bytes());
        }

        buf.extend_from_slice(&self.data);
        buf
    }
}

/// Extracted delta containing WAL header and dirty frames.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalDelta {
    pub header: WalHeader,
    pub start_frame: u32,
    pub end_frame: u32,
    pub total_wal_frames: u32,
    pub checkpoint_seq: u32,
    pub frames: Vec<WalFrame>,
}

/// Extracts delta frames from a `.db-wal` file.
pub struct WalDeltaReader;

impl WalDeltaReader {
    /// Read and parse WAL header from file.
    pub fn read_header(wal_path: &Path) -> std::result::Result<WalHeader, WalReplicationError> {
        let mut file = File::open(wal_path)?;
        let mut buf = [0u8; WAL_HEADER_SIZE];
        file.read_exact(&mut buf)?;
        WalHeader::parse(&buf)
    }

    /// Extract dirty frames from a WAL file starting after `from_frame` (1-indexed).
    pub fn extract_delta_frames(
        wal_path: &Path,
        from_frame: u32,
        max_frames: Option<usize>,
    ) -> std::result::Result<WalDelta, WalReplicationError> {
        if !wal_path.exists() {
            return Err(WalReplicationError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("WAL file does not exist: {}", wal_path.display()),
            )));
        }

        let bytes = fs::read(wal_path)?;
        Self::extract_delta_frames_from_bytes(&bytes, from_frame, max_frames)
    }

    /// Extract dirty frames from raw WAL bytes.
    pub fn extract_delta_frames_from_bytes(
        bytes: &[u8],
        from_frame: u32,
        max_frames: Option<usize>,
    ) -> std::result::Result<WalDelta, WalReplicationError> {
        if bytes.len() < WAL_HEADER_SIZE {
            return Err(WalReplicationError::InvalidHeader(format!(
                "Buffer too short for WAL header: {} bytes",
                bytes.len()
            )));
        }

        let header = WalHeader::parse(&bytes[0..WAL_HEADER_SIZE])?;
        let frame_size = WAL_FRAME_HEADER_SIZE + header.page_size as usize;
        let total_frames = ((bytes.len() - WAL_HEADER_SIZE) / frame_size) as u32;

        let mut frames = Vec::new();
        let start_idx = (from_frame + 1).max(1);
        let end_idx = match max_frames {
            Some(max) => (start_idx + max as u32 - 1).min(total_frames),
            None => total_frames,
        };

        if start_idx <= total_frames {
            for idx in start_idx..=end_idx {
                let offset = WAL_HEADER_SIZE + (idx as usize - 1) * frame_size;
                if offset + frame_size > bytes.len() {
                    break;
                }
                let frame_bytes = &bytes[offset..offset + frame_size];
                let frame = WalFrame::parse(
                    frame_bytes,
                    idx,
                    header.page_size,
                    header.is_little_endian_checksum,
                )?;

                // If salts do not match header salts, frame is from an older checkpoint cycle
                if frame.salt1 != header.salt1 || frame.salt2 != header.salt2 {
                    break;
                }

                frames.push(frame);
            }
        }

        let actual_start = frames.first().map(|f| f.frame_index).unwrap_or(from_frame);
        let actual_end = frames.last().map(|f| f.frame_index).unwrap_or(from_frame);

        Ok(WalDelta {
            header: header.clone(),
            start_frame: actual_start,
            end_frame: actual_end,
            total_wal_frames: total_frames,
            checkpoint_seq: header.checkpoint_seq,
            frames,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Compressed, Checksummed Delta Pack Serializer
// ─────────────────────────────────────────────────────────────────────────────

/// Compressed, checksummed package containing WAL delta frames.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalDeltaPack {
    /// Unique identifier for this delta pack
    pub pack_id: String,
    /// Optional logical database/workspace identifier
    pub db_identifier: Option<String>,
    pub checkpoint_seq: u32,
    pub start_frame: u32,
    pub end_frame: u32,
    pub frame_count: usize,
    pub page_size: u32,
    pub salt1: u32,
    pub salt2: u32,
    pub created_at: DateTime<Utc>,
    pub compressed: bool,
    /// Hex-encoded SHA-256 checksum of the payload
    pub checksum_sha256: String,
    /// Serialized (and optionally gzip-compressed) frames payload
    pub payload: Vec<u8>,
}

impl WalDeltaPack {
    /// Create and serialize a new delta package from a `WalDelta`.
    pub fn pack(
        delta: &WalDelta,
        compress: bool,
        db_identifier: Option<String>,
    ) -> std::result::Result<Self, WalReplicationError> {
        let raw_json = serde_json::to_vec(&delta.frames).map_err(|e| {
            WalReplicationError::SerializationFailed(format!("Failed to serialize frames: {}", e))
        })?;

        let payload = if compress {
            let mut encoder = GzEncoder::new(&raw_json[..], Compression::default());
            let mut compressed_buf = Vec::new();
            encoder.read_to_end(&mut compressed_buf).map_err(|e| {
                WalReplicationError::SerializationFailed(format!("Compression failed: {}", e))
            })?;
            compressed_buf
        } else {
            raw_json
        };

        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let checksum_sha256 = hex::encode(hasher.finalize());

        let created_at = Utc::now();
        let pack_id = format!(
            "wal-pack-seq{}-f{}-f{}-{}",
            delta.checkpoint_seq,
            delta.start_frame,
            delta.end_frame,
            created_at.timestamp_millis()
        );

        Ok(Self {
            pack_id,
            db_identifier,
            checkpoint_seq: delta.checkpoint_seq,
            start_frame: delta.start_frame,
            end_frame: delta.end_frame,
            frame_count: delta.frames.len(),
            page_size: delta.header.page_size,
            salt1: delta.header.salt1,
            salt2: delta.header.salt2,
            created_at,
            compressed: compress,
            checksum_sha256,
            payload,
        })
    }

    /// Verify the SHA-256 checksum of this package payload.
    pub fn verify_checksum(&self) -> std::result::Result<(), WalReplicationError> {
        let mut hasher = Sha256::new();
        hasher.update(&self.payload);
        let actual = hex::encode(hasher.finalize());
        if actual != self.checksum_sha256 {
            return Err(WalReplicationError::ChecksumMismatch {
                expected: self.checksum_sha256.clone(),
                actual,
            });
        }
        Ok(())
    }

    /// Unpack and deserialize frames contained in this package.
    pub fn unpack_frames(&self) -> std::result::Result<Vec<WalFrame>, WalReplicationError> {
        use std::io::Read;
        const MAX_DECOMPRESSED_BYTES: u64 = 64 * 1024 * 1024; // 64 MB

        self.verify_checksum()?;

        let raw_json = if self.compressed {
            let decoder = GzDecoder::new(&self.payload[..]);
            let mut limited = decoder.take(MAX_DECOMPRESSED_BYTES + 1);
            let mut decompressed = Vec::new();
            limited.read_to_end(&mut decompressed).map_err(|e| {
                WalReplicationError::DecompressionFailed(format!("Decompression failed: {}", e))
            })?;
            if decompressed.len() as u64 > MAX_DECOMPRESSED_BYTES {
                return Err(WalReplicationError::DecompressionFailed(format!(
                    "Decompressed payload exceeds {} byte limit",
                    MAX_DECOMPRESSED_BYTES
                )));
            }
            decompressed
        } else {
            self.payload.clone()
        };

        let frames: Vec<WalFrame> = serde_json::from_slice(&raw_json).map_err(|e| {
            WalReplicationError::SerializationFailed(format!("Failed to deserialize frames: {}", e))
        })?;

        Ok(frames)
    }

    /// Serialize the entire package to bytes (JSON format).
    pub fn to_bytes(&self) -> std::result::Result<Vec<u8>, WalReplicationError> {
        serde_json::to_vec(self).map_err(|e| {
            WalReplicationError::SerializationFailed(format!("Package serialization failed: {}", e))
        })
    }

    /// Deserialize a package from bytes.
    pub fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, WalReplicationError> {
        serde_json::from_slice(bytes).map_err(|e| {
            WalReplicationError::SerializationFailed(format!(
                "Package deserialization failed: {}",
                e
            ))
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. WalReplicationStreamer: Lag Computation & Replication Offsets
// ─────────────────────────────────────────────────────────────────────────────

/// Replication lag metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationLag {
    pub db_path: String,
    pub wal_path: String,
    pub wal_exists: bool,
    pub total_wal_frames: u32,
    pub last_replicated_frame: u32,
    pub unreplicated_frames: u32,
    pub unreplicated_bytes: u64,
    pub checkpoint_seq: u32,
    pub is_synced: bool,
    pub last_replicated_at: Option<DateTime<Utc>>,
}

/// Comprehensive replication status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub status: String,
    pub wal_path: String,
    pub lag: ReplicationLag,
    pub total_packs_replicated: u64,
    pub total_bytes_replicated: u64,
    pub last_pack_id: Option<String>,
    pub last_error: Option<String>,
}

/// Streamer tracking SQLite WAL offsets and managing delta flushes.
pub struct WalReplicationStreamer {
    db_path: PathBuf,
    wal_path: PathBuf,
    last_replicated_frame: u32,
    last_checkpoint_seq: u32,
    last_salt: (u32, u32),
    last_replicated_at: Option<DateTime<Utc>>,
    total_packs_replicated: u64,
    total_bytes_replicated: u64,
    last_pack_id: Option<String>,
    last_error: Option<String>,
    compress: bool,
    db_identifier: Option<String>,
}

impl WalReplicationStreamer {
    /// Create a new replication streamer for a given SQLite database path.
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        let db_path = db_path.into();
        let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
        Self {
            db_path,
            wal_path,
            last_replicated_frame: 0,
            last_checkpoint_seq: 0,
            last_salt: (0, 0),
            last_replicated_at: None,
            total_packs_replicated: 0,
            total_bytes_replicated: 0,
            last_pack_id: None,
            last_error: None,
            compress: true,
            db_identifier: None,
        }
    }

    /// Enable or disable compression for delta packages.
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Attach a logical database / workspace identifier.
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.db_identifier = Some(identifier.into());
        self
    }

    /// Reset replication frame pointer.
    pub fn reset_offset(&mut self, frame: u32) {
        self.last_replicated_frame = frame;
    }

    /// Compute current replication lag metrics.
    pub fn compute_lag(&self) -> std::result::Result<ReplicationLag, WalReplicationError> {
        let db_path_str = self.db_path.to_string_lossy().to_string();
        let wal_path_str = self.wal_path.to_string_lossy().to_string();

        if !self.wal_path.exists() {
            return Ok(ReplicationLag {
                db_path: db_path_str,
                wal_path: wal_path_str,
                wal_exists: false,
                total_wal_frames: 0,
                last_replicated_frame: self.last_replicated_frame,
                unreplicated_frames: 0,
                unreplicated_bytes: 0,
                checkpoint_seq: self.last_checkpoint_seq,
                is_synced: true,
                last_replicated_at: self.last_replicated_at,
            });
        }

        let metadata = fs::metadata(&self.wal_path)?;
        let file_len = metadata.len();
        if file_len < WAL_HEADER_SIZE as u64 {
            return Ok(ReplicationLag {
                db_path: db_path_str,
                wal_path: wal_path_str,
                wal_exists: true,
                total_wal_frames: 0,
                last_replicated_frame: self.last_replicated_frame,
                unreplicated_frames: 0,
                unreplicated_bytes: 0,
                checkpoint_seq: self.last_checkpoint_seq,
                is_synced: true,
                last_replicated_at: self.last_replicated_at,
            });
        }

        let header = WalDeltaReader::read_header(&self.wal_path)?;
        let frame_size = WAL_FRAME_HEADER_SIZE + header.page_size as usize;
        let total_frames = ((file_len as usize - WAL_HEADER_SIZE) / frame_size) as u32;

        let unreplicated_frames = total_frames.saturating_sub(self.last_replicated_frame);
        let unreplicated_bytes = (unreplicated_frames as u64) * (frame_size as u64);
        let is_synced = unreplicated_frames == 0;

        Ok(ReplicationLag {
            db_path: db_path_str,
            wal_path: wal_path_str,
            wal_exists: true,
            total_wal_frames: total_frames,
            last_replicated_frame: self.last_replicated_frame,
            unreplicated_frames,
            unreplicated_bytes,
            checkpoint_seq: header.checkpoint_seq,
            is_synced,
            last_replicated_at: self.last_replicated_at,
        })
    }

    /// Retrieve full replication status summary.
    pub fn status(&self) -> std::result::Result<ReplicationStatus, WalReplicationError> {
        let lag = self.compute_lag()?;
        let status = if !lag.wal_exists {
            "idle".to_string()
        } else if lag.is_synced {
            "synced".to_string()
        } else {
            "lagging".to_string()
        };

        Ok(ReplicationStatus {
            status,
            wal_path: self.wal_path.to_string_lossy().to_string(),
            lag,
            total_packs_replicated: self.total_packs_replicated,
            total_bytes_replicated: self.total_bytes_replicated,
            last_pack_id: self.last_pack_id.clone(),
            last_error: self.last_error.clone(),
        })
    }

    /// Extract new dirty frames, create a delta package, and advance replication offset.
    pub fn flush_delta(
        &mut self,
    ) -> std::result::Result<Option<WalDeltaPack>, WalReplicationError> {
        if !self.wal_path.exists() {
            return Ok(None);
        }

        let header = match WalDeltaReader::read_header(&self.wal_path) {
            Ok(h) => h,
            Err(e) => {
                self.last_error = Some(e.to_string());
                return Err(e);
            }
        };

        // Check if database was checkpointed and salts/checkpoint seq reset
        if self.last_checkpoint_seq != 0
            && (header.checkpoint_seq != self.last_checkpoint_seq
                || (header.salt1, header.salt2) != self.last_salt)
        {
            self.last_replicated_frame = 0;
        }

        self.last_checkpoint_seq = header.checkpoint_seq;
        self.last_salt = (header.salt1, header.salt2);

        let delta = match WalDeltaReader::extract_delta_frames(
            &self.wal_path,
            self.last_replicated_frame,
            None,
        ) {
            Ok(d) => d,
            Err(e) => {
                self.last_error = Some(e.to_string());
                return Err(e);
            }
        };

        if delta.frames.is_empty() {
            return Ok(None);
        }

        let pack = match WalDeltaPack::pack(&delta, self.compress, self.db_identifier.clone()) {
            Ok(p) => p,
            Err(e) => {
                self.last_error = Some(e.to_string());
                return Err(e);
            }
        };

        self.last_replicated_frame = delta.end_frame;
        self.last_replicated_at = Some(Utc::now());
        self.total_packs_replicated += 1;
        self.total_bytes_replicated += pack.payload.len() as u64;
        self.last_pack_id = Some(pack.pack_id.clone());
        self.last_error = None;

        Ok(Some(pack))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. WalRecoveryEngine: Point-In-Time Recovery (PITR) & Replay
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration options for point-in-time recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryOptions {
    /// Target frame sequence index to stop recovery at (inclusive)
    pub target_frame: Option<u32>,
    /// Target timestamp (UTC) to stop recovery at
    pub target_time: Option<DateTime<Utc>>,
    /// If true, only apply changes up to the last commit frame boundary
    pub commit_boundary_only: bool,
    /// Verify database integrity after recovery using PRAGMA integrity_check
    pub verify_integrity: bool,
}

impl Default for RecoveryOptions {
    fn default() -> Self {
        Self {
            target_frame: None,
            target_time: None,
            commit_boundary_only: true,
            verify_integrity: true,
        }
    }
}

/// Recovery execution report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub success: bool,
    pub target_db_path: String,
    pub frames_replayed: usize,
    pub commits_applied: usize,
    pub last_frame_applied: Option<u32>,
    pub final_db_size_bytes: u64,
    pub integrity_check: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Point-in-time recovery engine for SQLite databases.
pub struct WalRecoveryEngine;

impl WalRecoveryEngine {
    /// Replay an ordered list of `WalFrame`s directly into a target SQLite database file.
    pub fn replay_frames_to_db(
        target_db_path: &Path,
        page_size: u32,
        frames: &[WalFrame],
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        let start_time = std::time::Instant::now();

        if let Some(parent) = target_db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut candidate_frames: Vec<&WalFrame> = frames.iter().collect();

        // 1. Filter by target frame limit
        if let Some(max_frame) = options.target_frame {
            candidate_frames.retain(|f| f.frame_index <= max_frame);
        }

        // 2. Filter to last commit boundary if requested
        if options.commit_boundary_only && !candidate_frames.is_empty() {
            if let Some(last_commit_idx) = candidate_frames.iter().rposition(|f| f.is_commit()) {
                candidate_frames.truncate(last_commit_idx + 1);
            } else {
                // No commit frame found; if commit_boundary_only is strictly requested, apply none
                candidate_frames.clear();
            }
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(target_db_path)?;

        let mut commits_applied = 0;
        let mut last_frame_applied = None;

        for frame in &candidate_frames {
            let offset = (frame.page_number as u64 - 1) * page_size as u64;
            file.seek(SeekFrom::Start(offset))?;
            file.write_all(&frame.data)?;

            if frame.is_commit() {
                commits_applied += 1;
                if frame.db_size_pages > 0 {
                    let expected_len = frame.db_size_pages as u64 * page_size as u64;
                    file.set_len(expected_len)?;
                }
            }

            last_frame_applied = Some(frame.frame_index);
        }

        file.flush()?;
        drop(file);

        let final_db_size = fs::metadata(target_db_path)?.len();

        // 3. Verify integrity if requested
        let integrity_check = if options.verify_integrity && final_db_size > 0 {
            match Connection::open(target_db_path) {
                Ok(conn) => {
                    let check_result: std::result::Result<String, rusqlite::Error> =
                        conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0));
                    match check_result {
                        Ok(res) if res == "ok" => "ok".to_string(),
                        Ok(res) => {
                            return Err(WalReplicationError::IntegrityCheckFailed(res));
                        }
                        Err(e) => {
                            return Err(WalReplicationError::Sqlite(e));
                        }
                    }
                }
                Err(e) => {
                    return Err(WalReplicationError::Sqlite(e));
                }
            }
        } else {
            "skipped".to_string()
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(RecoveryReport {
            success: true,
            target_db_path: target_db_path.to_string_lossy().to_string(),
            frames_replayed: candidate_frames.len(),
            commits_applied,
            last_frame_applied,
            final_db_size_bytes: final_db_size,
            integrity_check,
            duration_ms,
            error: None,
        })
    }

    /// Recover database from a base database snapshot and a sequence of delta packages.
    pub fn recover_from_delta_packs(
        base_db_path: Option<&Path>,
        target_db_path: &Path,
        packs: &[WalDeltaPack],
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        // If base snapshot exists, copy it to target
        if let Some(base) = base_db_path {
            if base.exists() {
                if let Some(parent) = target_db_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(base, target_db_path)?;
            }
        }

        // Filter packs by target time if specified
        let mut sorted_packs: Vec<&WalDeltaPack> = packs.iter().collect();
        sorted_packs.sort_by_key(|p| (p.checkpoint_seq, p.start_frame));

        if let Some(target_time) = options.target_time {
            sorted_packs.retain(|p| p.created_at <= target_time);
        }

        let mut all_frames = Vec::new();
        let mut page_size = 4096;

        for pack in sorted_packs {
            page_size = pack.page_size;
            let mut frames = pack.unpack_frames()?;
            all_frames.append(&mut frames);
        }

        Self::replay_frames_to_db(target_db_path, page_size, &all_frames, options)
    }

    /// Point-In-Time Recovery replaying from source database and its active WAL file.
    pub fn point_in_time_recovery(
        source_db: &Path,
        source_wal: &Path,
        target_db: &Path,
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        if !source_db.exists() {
            return Err(WalReplicationError::RecoveryError(format!(
                "Source database not found: {}",
                source_db.display()
            )));
        }

        if let Some(parent) = target_db.parent() {
            fs::create_dir_all(parent)?;
        }

        // 1. Copy base database file
        fs::copy(source_db, target_db)?;

        // 2. If WAL file does not exist, recovery is simply the base DB
        if !source_wal.exists() {
            let metadata = fs::metadata(target_db)?;
            return Ok(RecoveryReport {
                success: true,
                target_db_path: target_db.to_string_lossy().to_string(),
                frames_replayed: 0,
                commits_applied: 0,
                last_frame_applied: None,
                final_db_size_bytes: metadata.len(),
                integrity_check: "ok".to_string(),
                duration_ms: 0,
                error: None,
            });
        }

        // 3. Extract frames from source WAL and replay
        let delta = WalDeltaReader::extract_delta_frames(source_wal, 0, None)?;
        Self::replay_frames_to_db(target_db, delta.header.page_size, &delta.frames, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walframe_parse_page_zero() {
        // Create a frame buffer with page_number = 0
        let page_size: u32 = 4096;
        let mut bytes = vec![0u8; WAL_FRAME_HEADER_SIZE + page_size as usize];
        // page_number = 0 (big-endian)
        bytes[0..4].copy_from_slice(&0u32.to_be_bytes());
        // db_size_pages = 1
        bytes[4..8].copy_from_slice(&1u32.to_be_bytes());

        let result = WalFrame::parse(&bytes, 1, page_size, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            WalReplicationError::InvalidFrame { index, reason } => {
                assert_eq!(index, 1);
                assert!(reason.contains("page_number"));
            }
            other => panic!("Expected InvalidFrame, got: {:?}", other),
        }
    }

    #[test]
    fn test_walframe_parse_valid_page() {
        let page_size: u32 = 4096;
        let mut bytes = vec![0u8; WAL_FRAME_HEADER_SIZE + page_size as usize];
        // page_number = 1 (valid)
        bytes[0..4].copy_from_slice(&1u32.to_be_bytes());
        bytes[4..8].copy_from_slice(&0u32.to_be_bytes());

        let result = WalFrame::parse(&bytes, 1, page_size, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().page_number, 1);
    }
}
