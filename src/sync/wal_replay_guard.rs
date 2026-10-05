//! Pre-replay validation for WAL frame replay (C2).
//!
//! Every frame handed to the recovery engine is validated *before* the target
//! database file (or its parent directory) is opened, created or modified.
//!
//! Two independent caps apply:
//! - a page cap (`MAX_ALLOWED_DB_PAGES`) on `page_number` and `db_size_pages`;
//! - a byte budget (`ReplayLimits::max_target_bytes`, default 64 GiB) on every
//!   computed offset/length, using checked arithmetic.
//!
//! The page cap alone is not enough: 100 million pages of 64 KiB is ~6.55 TB.

use serde::{Deserialize, Serialize};

use super::wal_replication::{WalFrame, WalReplicationError};

/// Approved ceiling (and default) for the reconstructed target size: 64 GiB.
pub const DEFAULT_MAX_REPLAY_TARGET_BYTES: u64 = 64 * 1024 * 1024 * 1024;

/// Upper bound for `page_number` and `db_size_pages` of a replayed frame.
pub const MAX_ALLOWED_DB_PAGES: u32 = 100_000_000;

/// Smallest page size SQLite supports.
pub const MIN_WAL_PAGE_SIZE: u32 = 512;

/// Largest page size SQLite supports.
pub const MAX_WAL_PAGE_SIZE: u32 = 65536;

/// Injectable limits for WAL replay.
///
/// `Default` yields the approved 64 GiB budget. A budget of zero or above the
/// approved ceiling is refused by [`ReplayLimits::validate`]; there is no
/// "unlimited" setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLimits {
    /// Maximum size, in bytes, the reconstructed target may reach through replay.
    pub max_target_bytes: u64,
}

impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            max_target_bytes: DEFAULT_MAX_REPLAY_TARGET_BYTES,
        }
    }
}

impl ReplayLimits {
    /// Build limits with an explicit byte budget (validated at replay time).
    pub fn with_max_target_bytes(max_target_bytes: u64) -> Self {
        Self { max_target_bytes }
    }

    /// Refuse a zero budget or one above the approved ceiling.
    pub fn validate(&self) -> Result<(), WalReplicationError> {
        if self.max_target_bytes == 0 {
            return Err(WalReplicationError::ReplayBudgetExceeded(
                "replay byte budget must be greater than zero".to_string(),
            ));
        }
        if self.max_target_bytes > DEFAULT_MAX_REPLAY_TARGET_BYTES {
            return Err(WalReplicationError::ReplayBudgetExceeded(format!(
                "replay byte budget {} exceeds approved ceiling {}",
                self.max_target_bytes, DEFAULT_MAX_REPLAY_TARGET_BYTES
            )));
        }
        Ok(())
    }
}

/// Summary of a successful preflight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReplayPreflight {
    /// Highest byte offset any frame write reaches (`offset + page_size`).
    pub max_write_end_bytes: u64,
    /// Largest length any commit frame truncates/extends the file to.
    pub max_commit_len_bytes: u64,
    /// Sum of all frame payload bytes.
    pub total_frame_bytes: u64,
}

/// Validate page size, frame sizes, page caps and the byte budget for every
/// frame, without touching the filesystem.
///
/// All arithmetic is checked; any overflow is reported as a budget refusal.
pub fn preflight_replay(
    page_size: u32,
    frames: &[&WalFrame],
    limits: &ReplayLimits,
) -> Result<ReplayPreflight, WalReplicationError> {
    limits.validate()?;
    validate_page_size(page_size)?;

    let budget = limits.max_target_bytes;
    let mut summary = ReplayPreflight::default();
    for frame in frames {
        check_frame_shape(frame, page_size)?;

        let write_end = page_offset(frame.page_number, page_size)
            .and_then(|offset| offset.checked_add(frame.data.len() as u64));
        let write_end = within_budget(write_end, budget, frame, "write end")?;
        summary.max_write_end_bytes = summary.max_write_end_bytes.max(write_end);

        if frame.is_commit() {
            let commit_len = (frame.db_size_pages as u64).checked_mul(page_size as u64);
            let commit_len = within_budget(commit_len, budget, frame, "commit length")?;
            summary.max_commit_len_bytes = summary.max_commit_len_bytes.max(commit_len);
        }

        let total = summary
            .total_frame_bytes
            .checked_add(frame.data.len() as u64);
        summary.total_frame_bytes = within_budget(total, budget, frame, "total frame bytes")?;
    }
    Ok(summary)
}

/// Refuse page sizes SQLite cannot produce.
pub fn validate_page_size(page_size: u32) -> Result<(), WalReplicationError> {
    if !(MIN_WAL_PAGE_SIZE..=MAX_WAL_PAGE_SIZE).contains(&page_size) || !page_size.is_power_of_two()
    {
        return Err(WalReplicationError::InvalidHeader(format!(
            "unsupported page size {} (must be a power of two in {}..={})",
            page_size, MIN_WAL_PAGE_SIZE, MAX_WAL_PAGE_SIZE
        )));
    }
    Ok(())
}

/// Byte offset of a 1-based page, or `None` for page 0 / overflow.
pub(crate) fn page_offset(page_number: u32, page_size: u32) -> Option<u64> {
    (page_number as u64)
        .checked_sub(1)?
        .checked_mul(page_size as u64)
}

fn check_frame_shape(frame: &WalFrame, page_size: u32) -> Result<(), WalReplicationError> {
    let invalid = |reason: String| WalReplicationError::InvalidFrame {
        index: frame.frame_index,
        reason,
    };
    if frame.page_number == 0 || frame.page_number > MAX_ALLOWED_DB_PAGES {
        return Err(invalid(format!(
            "page_number {} outside 1..={}",
            frame.page_number, MAX_ALLOWED_DB_PAGES
        )));
    }
    if frame.db_size_pages > MAX_ALLOWED_DB_PAGES {
        return Err(invalid(format!(
            "db_size_pages {} exceeds {}",
            frame.db_size_pages, MAX_ALLOWED_DB_PAGES
        )));
    }
    if frame.data.len() as u64 != page_size as u64 {
        return Err(invalid(format!(
            "frame payload is {} bytes, expected page_size {}",
            frame.data.len(),
            page_size
        )));
    }
    Ok(())
}

fn within_budget(
    value: Option<u64>,
    budget: u64,
    frame: &WalFrame,
    what: &str,
) -> Result<u64, WalReplicationError> {
    match value {
        Some(v) if v <= budget => Ok(v),
        Some(v) => Err(WalReplicationError::ReplayBudgetExceeded(format!(
            "frame {}: {} {} exceeds budget {}",
            frame.frame_index, what, v, budget
        ))),
        None => Err(WalReplicationError::ReplayBudgetExceeded(format!(
            "frame {}: {} overflows u64",
            frame.frame_index, what
        ))),
    }
}
