//! Point-in-time recovery (PITR) and WAL frame replay (C2).
//!
//! Every entry point validates all frames (limits, ordering, checksum chain)
//! before the target path is created, opened or modified. Replay then runs on
//! a staging copy beside the target (see `wal_recovery_staging`), which only
//! replaces the target after fsync and a passing integrity check, so late I/O
//! or integrity failures leave a pre-existing target, the base snapshot and
//! the PITR source untouched.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::wal_chain::{validate_frame_order, validate_pack_chain};
use super::wal_recovery_staging::{open_file_sink, replay_staged, StageSeed};
use super::wal_replay_guard::{preflight_replay, ReplayLimits, MIN_WAL_PAGE_SIZE};
use super::wal_replication::{WalDeltaPack, WalDeltaReader, WalFrame, WalReplicationError};

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
    ///
    /// Applies the approved default [`ReplayLimits`] (64 GiB budget).
    pub fn replay_frames_to_db(
        target_db_path: &Path,
        page_size: u32,
        frames: &[WalFrame],
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        Self::replay_frames_to_db_with_limits(
            target_db_path,
            page_size,
            frames,
            options,
            &ReplayLimits::default(),
        )
    }

    /// Same as [`Self::replay_frames_to_db`] with explicit replay limits.
    ///
    /// Every frame is validated by [`preflight_replay`] before the target or
    /// its parent directory is created, opened or modified.
    pub fn replay_frames_to_db_with_limits(
        target_db_path: &Path,
        page_size: u32,
        frames: &[WalFrame],
        options: &RecoveryOptions,
        limits: &ReplayLimits,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        let start_time = std::time::Instant::now();
        let all: Vec<&WalFrame> = frames.iter().collect();
        preflight_replay(page_size, &all, limits)?;
        validate_frame_order(frames)?;
        let candidates = select_candidate_frames(all, options);
        replay_staged(
            target_db_path,
            StageSeed::ExistingTarget,
            page_size,
            &candidates,
            options,
            start_time,
            open_file_sink,
        )
    }

    /// Recover database from a base database snapshot and a sequence of delta packages.
    ///
    /// Applies the approved default [`ReplayLimits`] (64 GiB budget).
    pub fn recover_from_delta_packs(
        base_db_path: Option<&Path>,
        target_db_path: &Path,
        packs: &[WalDeltaPack],
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        Self::recover_from_delta_packs_with_limits(
            base_db_path,
            target_db_path,
            packs,
            options,
            &ReplayLimits::default(),
        )
    }

    /// Same as [`Self::recover_from_delta_packs`] with explicit replay limits.
    ///
    /// All packs are unpacked and validated before the base is copied.
    pub fn recover_from_delta_packs_with_limits(
        base_db_path: Option<&Path>,
        target_db_path: &Path,
        packs: &[WalDeltaPack],
        options: &RecoveryOptions,
        limits: &ReplayLimits,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        let start_time = std::time::Instant::now();
        limits.validate()?;

        let mut sorted_packs: Vec<&WalDeltaPack> = packs.iter().collect();
        sorted_packs.sort_by_key(|p| (p.checkpoint_seq, p.start_frame));
        if let Some(target_time) = options.target_time {
            sorted_packs.retain(|p| p.created_at <= target_time);
        }

        let unpacked = unpack_within_budget(&sorted_packs, limits)?;
        validate_pack_chain(&sorted_packs, &unpacked)?;
        let page_size = sorted_packs.last().map(|p| p.page_size).unwrap_or(4096);
        let all_frames: Vec<WalFrame> = unpacked.into_iter().flatten().collect();

        let all: Vec<&WalFrame> = all_frames.iter().collect();
        preflight_replay(page_size, &all, limits)?;
        let candidates = select_candidate_frames(all, options);

        let seed = match base_db_path {
            Some(base) if base.exists() => StageSeed::File(base),
            _ => StageSeed::ExistingTarget,
        };
        replay_staged(
            target_db_path,
            seed,
            page_size,
            &candidates,
            options,
            start_time,
            open_file_sink,
        )
    }

    /// Recover the latest committed state of a database that this process
    /// has open through `storage`, into `target_db`.
    ///
    /// The seed is written by SQLite (`VACUUM INTO`, which includes committed
    /// WAL content), so no raw descriptor of the active files is opened and
    /// SQLite's POSIX locks stay intact (G1). No WAL frames are replayed, so
    /// earlier points in time are not available this way. The target must
    /// not be one of the active files.
    pub fn recover_active_database(
        storage: &crate::storage::Storage,
        target_db: &Path,
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        let start_time = std::time::Instant::now();
        storage
            .refuse_active_sqlite_artifact(target_db)
            .map_err(|err| WalReplicationError::RecoveryError(err.to_string()))?;
        replay_staged(
            target_db,
            StageSeed::ActiveStorage(storage),
            MIN_WAL_PAGE_SIZE, // unused: no frames are written
            &[],
            options,
            start_time,
            open_file_sink,
        )
    }

    /// Point-In-Time Recovery replaying from source database and its active WAL file.
    ///
    /// Applies the approved default [`ReplayLimits`] (64 GiB budget).
    ///
    /// `source_db` is copied through a plain file: it must not be a database
    /// that this process has open (see [`Self::recover_active_database`]).
    pub fn point_in_time_recovery(
        source_db: &Path,
        source_wal: &Path,
        target_db: &Path,
        options: &RecoveryOptions,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        Self::point_in_time_recovery_with_limits(
            source_db,
            source_wal,
            target_db,
            options,
            &ReplayLimits::default(),
        )
    }

    /// Same as [`Self::point_in_time_recovery`] with explicit replay limits.
    ///
    /// WAL frames are extracted and validated before the source is copied.
    pub fn point_in_time_recovery_with_limits(
        source_db: &Path,
        source_wal: &Path,
        target_db: &Path,
        options: &RecoveryOptions,
        limits: &ReplayLimits,
    ) -> std::result::Result<RecoveryReport, WalReplicationError> {
        let start_time = std::time::Instant::now();
        limits.validate()?;
        if !source_db.exists() {
            return Err(WalReplicationError::RecoveryError(format!(
                "Source database not found: {}",
                source_db.display()
            )));
        }

        // 1. Extract and validate WAL frames before touching the target.
        let delta = if source_wal.exists() {
            let delta = WalDeltaReader::extract_delta_frames(source_wal, 0, None)?;
            let all: Vec<&WalFrame> = delta.frames.iter().collect();
            preflight_replay(delta.header.page_size, &all, limits)?;
            Some(delta)
        } else {
            None
        };

        // 2. If WAL file does not exist, recovery is simply the base DB
        let Some(delta) = delta else {
            // Staged copy, integrity-checked when requested, then replaced.
            return replay_staged(
                target_db,
                StageSeed::File(source_db),
                MIN_WAL_PAGE_SIZE, // unused: no frames are written
                &[],
                options,
                start_time,
                open_file_sink,
            );
        };

        // 3. Replay validated frames onto a staged copy of the source
        let candidates = select_candidate_frames(delta.frames.iter().collect(), options);
        replay_staged(
            target_db,
            StageSeed::File(source_db),
            delta.header.page_size,
            &candidates,
            options,
            start_time,
            open_file_sink,
        )
    }
}

/// Unpack packs in order, keeping a checked running sum of decoded payload
/// bytes. Each pack may decode at most the remaining budget (and never more
/// than the per-pack cap), so decoding stops as soon as the cumulative size
/// would exceed `limits.max_target_bytes`.
fn unpack_within_budget(
    packs: &[&WalDeltaPack],
    limits: &ReplayLimits,
) -> std::result::Result<Vec<Vec<WalFrame>>, WalReplicationError> {
    let budget = limits.max_target_bytes;
    let mut decoded_total: u64 = 0;
    let mut unpacked = Vec::with_capacity(packs.len());
    for pack in packs {
        let remaining = budget.saturating_sub(decoded_total);
        let per_pack_cap = WalDeltaPack::DEFAULT_MAX_DECOMPRESSED_BYTES;
        let cap = remaining.min(per_pack_cap);
        let Some((frames, decoded)) = pack.unpack_frames_counted(cap)? else {
            return Err(if remaining < per_pack_cap {
                WalReplicationError::ReplayBudgetExceeded(format!(
                    "pack {}: cumulative decoded payload exceeds budget {}",
                    pack.pack_id, budget
                ))
            } else {
                WalReplicationError::DecompressionFailed(format!(
                    "pack {}: decoded payload exceeds {} byte limit",
                    pack.pack_id, per_pack_cap
                ))
            });
        };
        decoded_total = decoded_total.checked_add(decoded).ok_or_else(|| {
            WalReplicationError::ReplayBudgetExceeded("decoded byte count overflow".to_string())
        })?;
        unpacked.push(frames);
    }
    Ok(unpacked)
}

/// Apply `target_frame` and commit-boundary filters to validated frames.
fn select_candidate_frames<'a>(
    mut candidates: Vec<&'a WalFrame>,
    options: &RecoveryOptions,
) -> Vec<&'a WalFrame> {
    if let Some(max_frame) = options.target_frame {
        candidates.retain(|f| f.frame_index <= max_frame);
    }
    if options.commit_boundary_only && !candidates.is_empty() {
        match candidates.iter().rposition(|f| f.is_commit()) {
            Some(last_commit_idx) => candidates.truncate(last_commit_idx + 1),
            // No commit frame: strict commit boundary applies none.
            None => candidates.clear(),
        }
    }
    candidates
}
