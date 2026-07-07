use engram::embedding::{
    run_embedding_queue_hygiene, EmbeddingQueueHygieneConfig, EmbeddingQueueHygieneReport,
    DEFAULT_COMPLETE_RETENTION,
};
use engram::error::Result;
use engram::storage::Storage;

pub(super) fn handle(
    storage: &Storage,
    requeue_failed: bool,
    apply: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    if apply && dry_run {
        eprintln!("--apply and --dry-run are mutually exclusive");
        std::process::exit(1);
    }
    if dry_run {
        eprintln!("WARNING: --dry-run requested explicitly; mutation will be skipped.");
    }
    let r = run_embedding_queue_maintenance(storage, requeue_failed, apply)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r)?);
    } else {
        print_maintenance_queue_hygiene(&r, apply);
    }
    Ok(())
}

pub(super) fn run_embedding_queue_maintenance(
    storage: &Storage,
    requeue_failed: bool,
    apply: bool,
) -> Result<EmbeddingQueueHygieneReport> {
    let config = EmbeddingQueueHygieneConfig {
        complete_retention: DEFAULT_COMPLETE_RETENTION,
        ..Default::default()
    };
    storage.with_connection(|conn| {
        run_embedding_queue_hygiene(conn, &config, requeue_failed, apply, true)
    })
}

fn print_maintenance_queue_hygiene(r: &EmbeddingQueueHygieneReport, apply: bool) {
    println!(
        "Embedding queue hygiene ({})",
        if apply { "APPLIED" } else { "dry-run" }
    );
    println!("  stale rows requeued:   {}", r.requeued_stale);
    println!("  stale rows failed:     {}", r.failed_exhausted);
    println!("  failed rows requeued:  {}", r.requeued_failed);
    println!("  complete rows pruned:  {}", r.pruned_complete);
    if !apply {
        println!("  (dry-run; re-run with --apply to execute)");
    }
}
