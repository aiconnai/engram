use clap::Subcommand;
use engram::error::Result;
use engram::storage::Storage;

mod compact;
mod queue;
mod status;

#[derive(Subcommand)]
pub(crate) enum MaintenanceAction {
    /// Show storage health and maintenance status without mutating the database
    Status {
        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Report (dry-run) or apply compaction: prune queue, checkpoint WAL, and
    /// VACUUM when there is enough free disk space
    Compact {
        /// Perform the operations (default is a read-only dry-run)
        #[arg(long)]
        apply: bool,
        /// Emit JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Report (dry-run) or apply a rebuild of derived indexes — the FTS index
    /// and embedding requeue. Canonical memories are never touched
    Rebuild {
        /// Rebuild the FTS index (if neither flag is set, both are rebuilt)
        #[arg(long)]
        fts: bool,
        /// Requeue memories missing embeddings (if neither flag is set, both)
        #[arg(long)]
        embeddings: bool,
        /// Perform the rebuild (default is a read-only dry-run)
        #[arg(long)]
        apply: bool,
        /// Emit JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Run explicit queue hygiene: stale repair, optional failed retries, and complete-row retention pruning
    QueueHygiene {
        /// Explicitly requeue failed rows with retry budget left
        #[arg(long)]
        requeue_failed: bool,
        /// Perform writes (default is a read-only dry-run)
        #[arg(long)]
        apply: bool,
        /// Keep dry-run explicit when you do not want to mutate
        #[arg(long)]
        dry_run: bool,
        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn handle(storage: &Storage, action: MaintenanceAction) -> Result<()> {
    match action {
        MaintenanceAction::Status { json } => status::handle(storage, json),
        MaintenanceAction::QueueHygiene {
            requeue_failed,
            apply,
            dry_run,
            json,
        } => queue::handle(storage, requeue_failed, apply, dry_run, json),
        MaintenanceAction::Compact { apply, json } => compact::handle_compact(storage, apply, json),
        MaintenanceAction::Rebuild {
            fts,
            embeddings,
            apply,
            json,
        } => compact::handle_rebuild(storage, fts, embeddings, apply, json),
    }
}

#[cfg(test)]
mod tests;
