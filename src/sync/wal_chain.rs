//! SQLite WAL checksum-chain validation (C2).
//!
//! **Integrity, not authenticity.** The SQLite WAL checksum chain and the
//! SHA-256 of a [`WalDeltaPack`] payload detect accidental corruption,
//! truncation, reordering and splicing of frames. Neither authenticates the
//! emitter: anyone holding the frames can recompute both. Authenticity needs
//! a signature or MAC, which this module does not provide.
//!
//! SQLite semantics (wal.c):
//! - checksum word order is selected by the magic's low bit: `0x377f0682`
//!   means little-endian words, `0x377f0683` big-endian words;
//! - the header checksum covers header bytes `0..24` seeded with `(0, 0)`;
//! - each frame checksum covers the first 8 frame-header bytes plus the page
//!   data, seeded with the previous frame's checksum (the header checksum for
//!   frame 1);
//! - stored checksum fields are always big-endian.
//!
//! This crate's parser decodes stored checksum fields as little-endian when
//! the magic is `WAL_MAGIC_LE` (`is_little_endian_checksum`), so values are
//! normalised with [`stored_checksum`] before comparison.

use serde::{Deserialize, Serialize};

use super::wal_replay_guard::validate_page_size;
use super::wal_replication::{
    compute_wal_checksum, WalDeltaPack, WalFrame, WalReplicationError, WAL_HEADER_SIZE,
    WAL_MAGIC_LE,
};

/// Chain context a [`WalDeltaPack`] must carry for its frames to be verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalChainContext {
    /// WAL magic of the source file (selects checksum word order).
    pub magic: u32,
    /// Running checksum preceding `start_frame`, as stored in the WAL
    /// (the header checksum when `start_frame == 1`).
    pub seed: (u32, u32),
}

/// True when the checksum words are little-endian for this WAL magic.
pub fn checksum_words_le(magic: u32) -> bool {
    magic & 1 == 0
}

/// Convert a checksum decoded by this crate's parser back to its stored value.
pub fn stored_checksum(value: u32, decoded_le: bool) -> u32 {
    if decoded_le {
        value.swap_bytes()
    } else {
        value
    }
}

/// Whether this crate's parser decoded checksum fields as little-endian.
pub(crate) fn decoded_le(magic: u32) -> bool {
    magic == WAL_MAGIC_LE
}

/// SQLite checksum of one frame (8 header bytes + page data) from `seed`.
pub fn wal_frame_checksum(
    page_number: u32,
    db_size_pages: u32,
    data: &[u8],
    words_le: bool,
    seed: (u32, u32),
) -> (u32, u32) {
    let mut head = [0u8; 8];
    head[0..4].copy_from_slice(&page_number.to_be_bytes());
    head[4..8].copy_from_slice(&db_size_pages.to_be_bytes());
    let after_head = compute_wal_checksum(&head, words_le, seed);
    compute_wal_checksum(data, words_le, after_head)
}

/// Verify a raw 32-byte WAL header checksum; returns the stored checksum,
/// which seeds frame 1.
pub fn verify_header_bytes(bytes: &[u8]) -> Result<(u32, u32), WalReplicationError> {
    if bytes.len() < WAL_HEADER_SIZE {
        return Err(WalReplicationError::InvalidHeader(format!(
            "header needs {} bytes, got {}",
            WAL_HEADER_SIZE,
            bytes.len()
        )));
    }
    let word =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let magic = word(0);
    let stored = (word(24), word(28));
    let computed = compute_wal_checksum(&bytes[0..24], checksum_words_le(magic), (0, 0));
    if computed != stored {
        return Err(WalReplicationError::InvalidHeader(format!(
            "header checksum mismatch: stored {:08x}{:08x}, computed {:08x}{:08x}",
            stored.0, stored.1, computed.0, computed.1
        )));
    }
    Ok(stored)
}

/// Stored checksum of `frame` if it matches the chain from `seed`.
pub(crate) fn chain_step(frame: &WalFrame, magic: u32, seed: (u32, u32)) -> Option<(u32, u32)> {
    let computed = wal_frame_checksum(
        frame.page_number,
        frame.db_size_pages,
        &frame.data,
        checksum_words_le(magic),
        seed,
    );
    let le = decoded_le(magic);
    let stored = (
        stored_checksum(frame.checksum1, le),
        stored_checksum(frame.checksum2, le),
    );
    (computed == stored).then_some(stored)
}

/// Validate frame ordering for a single WAL generation: strictly increasing
/// `frame_index` and identical salts.
pub fn validate_frame_order(frames: &[WalFrame]) -> Result<(), WalReplicationError> {
    for pair in frames.windows(2) {
        let (prev, next) = (&pair[0], &pair[1]);
        if next.frame_index <= prev.frame_index {
            return Err(invalid(
                next,
                format!(
                    "frame_index {} does not follow {}",
                    next.frame_index, prev.frame_index
                ),
            ));
        }
        if (next.salt1, next.salt2) != (prev.salt1, prev.salt2) {
            return Err(invalid(next, "salts differ within one replay".to_string()));
        }
    }
    Ok(())
}

/// Validate a sorted pack set and their unpacked frames before replay:
/// chain context present, page_size consistent, frame indices contiguous,
/// salts matching, checksum chain intact within and across packs of the same
/// WAL generation.
pub fn validate_pack_chain(
    packs: &[&WalDeltaPack],
    unpacked: &[Vec<WalFrame>],
) -> Result<(), WalReplicationError> {
    if packs.len() != unpacked.len() {
        return Err(WalReplicationError::RecoveryError(
            "pack/frame list length mismatch".to_string(),
        ));
    }
    let Some(first) = packs.first() else {
        return Ok(());
    };
    validate_page_size(first.page_size)?;

    let mut prev: Option<PackTail> = None;
    for (pack, frames) in packs.iter().zip(unpacked) {
        let chain = pack.chain.ok_or_else(|| {
            pack_error(pack, "missing checksum-chain context (pre-C2 pack format)")
        })?;
        if pack.page_size != first.page_size {
            return Err(pack_error(pack, "page_size differs from previous packs"));
        }
        check_pack_shape(pack, frames)?;
        check_pack_link(pack, &chain, prev.as_ref())?;

        let mut running = chain.seed;
        for frame in frames {
            if (frame.salt1, frame.salt2) != (pack.salt1, pack.salt2) {
                return Err(invalid(
                    frame,
                    "frame salts differ from pack salts".to_string(),
                ));
            }
            running = chain_step(frame, chain.magic, running)
                .ok_or_else(|| invalid(frame, "SQLite checksum chain mismatch".to_string()))?;
        }
        prev = Some(PackTail {
            checkpoint_seq: pack.checkpoint_seq,
            salts: (pack.salt1, pack.salt2),
            end_frame: pack.end_frame,
            magic: chain.magic,
            last_checksum: running,
        });
    }
    Ok(())
}

/// Chain state at the end of the previous pack.
struct PackTail {
    checkpoint_seq: u32,
    salts: (u32, u32),
    end_frame: u32,
    magic: u32,
    last_checksum: (u32, u32),
}

fn check_pack_shape(pack: &WalDeltaPack, frames: &[WalFrame]) -> Result<(), WalReplicationError> {
    if frames.is_empty() {
        return Err(pack_error(pack, "pack contains no frames"));
    }
    if frames.len() != pack.frame_count {
        return Err(pack_error(pack, "frame_count does not match payload"));
    }
    for (offset, frame) in frames.iter().enumerate() {
        let expected = u32::try_from(offset)
            .ok()
            .and_then(|o| pack.start_frame.checked_add(o));
        if Some(frame.frame_index) != expected {
            return Err(invalid(
                frame,
                format!("frame_index out of sequence (expected {:?})", expected),
            ));
        }
    }
    if frames.last().map(|f| f.frame_index) != Some(pack.end_frame) {
        return Err(pack_error(pack, "end_frame does not match payload"));
    }
    Ok(())
}

/// Link a pack to the previous one: same generation must be contiguous and
/// seeded by the previous tail; a new generation must restart at frame 1.
fn check_pack_link(
    pack: &WalDeltaPack,
    chain: &WalChainContext,
    prev: Option<&PackTail>,
) -> Result<(), WalReplicationError> {
    let Some(prev) = prev else {
        return Ok(());
    };
    let same_generation =
        pack.checkpoint_seq == prev.checkpoint_seq && (pack.salt1, pack.salt2) == prev.salts;
    if same_generation {
        if Some(pack.start_frame) != prev.end_frame.checked_add(1) {
            return Err(pack_error(pack, "gap or overlap with previous pack"));
        }
        if chain.magic != prev.magic || chain.seed != prev.last_checksum {
            return Err(pack_error(
                pack,
                "chain seed does not continue previous pack",
            ));
        }
        return Ok(());
    }
    if pack.checkpoint_seq <= prev.checkpoint_seq || (pack.salt1, pack.salt2) == prev.salts {
        return Err(pack_error(pack, "ambiguous WAL generation ordering"));
    }
    if pack.start_frame != 1 {
        return Err(pack_error(pack, "new WAL generation must start at frame 1"));
    }
    Ok(())
}

fn invalid(frame: &WalFrame, reason: String) -> WalReplicationError {
    WalReplicationError::InvalidFrame {
        index: frame.frame_index,
        reason,
    }
}

fn pack_error(pack: &WalDeltaPack, reason: &str) -> WalReplicationError {
    WalReplicationError::RecoveryError(format!("pack {}: {}", pack.pack_id, reason))
}
