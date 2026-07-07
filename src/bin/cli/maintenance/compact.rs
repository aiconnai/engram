use engram::error::Result;
use engram::storage::queries::rebuild_derived_indexes;
use engram::storage::Storage;

pub(super) fn handle_compact(storage: &Storage, apply: bool, json: bool) -> Result<()> {
    let r = storage.compact(apply)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r)?);
        return Ok(());
    }

    println!(
        "Storage compaction ({})",
        if r.applied { "APPLIED" } else { "dry-run" }
    );
    println!("  database size:        {} bytes", r.db_size_bytes);
    println!(
        "  WAL / SHM:            {} / {} bytes",
        r.wal_bytes, r.shm_bytes
    );
    println!(
        "  reclaimable (VACUUM): {} bytes ({} free page(s))",
        r.reclaimable_bytes, r.freelist_count
    );
    println!(
        "  queue prunable:       {} complete, {} failed",
        r.queue_complete_prunable, r.queue_failed_prunable
    );
    println!("  orphan embeddings:    {}", r.orphan_embeddings);
    let free = if r.free_space_bytes < 0 {
        "unknown".to_string()
    } else {
        format!("{} bytes", r.free_space_bytes)
    };
    println!(
        "  free space:           {} (vacuum safe: {})",
        free, r.vacuum_safe
    );
    println!("  operations:");
    for op in &r.operations {
        let status = if op.applied {
            "applied".to_string()
        } else if let Some(reason) = &op.skipped_reason {
            format!("skipped ({reason})")
        } else {
            "dry-run".to_string()
        };
        println!(
            "    - {:<22} candidates={} [{}]",
            op.name, op.candidates, status
        );
    }
    if !r.applied {
        println!("  (dry-run; re-run with --apply to execute)");
    }
    Ok(())
}

pub(super) fn handle_rebuild(
    storage: &Storage,
    fts: bool,
    embeddings: bool,
    apply: bool,
    json: bool,
) -> Result<()> {
    let (do_fts, do_embeddings) = if !fts && !embeddings {
        (true, true)
    } else {
        (fts, embeddings)
    };
    let r = storage
        .with_transaction(|conn| rebuild_derived_indexes(conn, do_fts, do_embeddings, apply))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r)?);
        return Ok(());
    }

    println!(
        "Derived-index rebuild ({})",
        if r.applied { "APPLIED" } else { "dry-run" }
    );
    println!("  live memories (preserved): {}", r.memories);
    if r.fts_targeted {
        println!(
            "  FTS: indexed {} -> {}, drift {} -> {} (rebuilt: {})",
            r.fts_indexed_before,
            r.fts_indexed_after,
            r.fts_drift_before,
            r.fts_drift_after,
            r.fts_rebuilt
        );
    }
    if r.embeddings_targeted {
        println!(
            "  embeddings: {} present, {} missing, {} requeued",
            r.embeddings_present, r.embeddings_missing, r.embeddings_requeued
        );
    }
    if !r.applied {
        println!("  (dry-run; re-run with --apply to execute)");
    }
    Ok(())
}
