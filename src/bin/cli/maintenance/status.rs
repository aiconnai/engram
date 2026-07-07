use engram::error::Result;
use engram::storage::queries::get_stats;
use engram::storage::{health_check_storage, HealthStatus, Storage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct MaintenanceStatus {
    #[serde(flatten)]
    pub(super) health: HealthStatus,
    stats: MaintenanceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaintenanceStats {
    total_memories: i64,
    total_tags: i64,
    total_crossrefs: i64,
    total_versions: i64,
    memories_with_embeddings: i64,
    memories_pending_embedding: i64,
    sync_pending: bool,
    storage_mode: String,
    schema_version: i32,
    db_size_bytes: i64,
}

pub(super) fn handle(storage: &Storage, json: bool) -> Result<()> {
    let status = maintenance_status(storage)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        print_maintenance_status(&status);
    }
    Ok(())
}

pub(super) fn maintenance_status(storage: &Storage) -> Result<MaintenanceStatus> {
    let health = storage_health(storage)?;
    let stats = storage.with_connection(get_stats)?;

    Ok(MaintenanceStatus {
        health,
        stats: MaintenanceStats {
            total_memories: stats.total_memories,
            total_tags: stats.total_tags,
            total_crossrefs: stats.total_crossrefs,
            total_versions: stats.total_versions,
            memories_with_embeddings: stats.memories_with_embeddings,
            memories_pending_embedding: stats.memories_pending_embedding,
            sync_pending: stats.sync_pending,
            storage_mode: stats.storage_mode,
            schema_version: stats.schema_version,
            db_size_bytes: stats.db_size_bytes,
        },
    })
}

fn storage_health(storage: &Storage) -> Result<HealthStatus> {
    health_check_storage(storage)
}

fn json_enum_name<T: Serialize + std::fmt::Debug>(value: T) -> String {
    serde_json::to_string(&value)
        .unwrap_or_else(|_| format!("{value:?}"))
        .trim_matches('"')
        .to_string()
}

pub(super) fn write_maintenance_status<W: std::io::Write>(
    mut writer: W,
    status: &MaintenanceStatus,
) -> Result<()> {
    writeln!(
        writer,
        "Storage: {}",
        if status.health.healthy {
            "healthy"
        } else {
            "unhealthy"
        }
    )?;
    writeln!(writer, "Latency: {:.2} ms", status.health.latency_ms)?;
    writeln!(writer, "Database: {}", status.health.details["db_path"])?;
    writeln!(writer, "Storage mode: {}", status.stats.storage_mode)?;
    writeln!(writer, "Schema version: {}", status.stats.schema_version)?;
    if let Some(quick_check) = status.health.details.get("quick_check") {
        writeln!(writer, "PRAGMA quick_check: {}", quick_check)?;
    }
    write_database_size(&mut writer, status)?;
    if let Some(warning) = status.health.details.get("warning") {
        writeln!(writer, "Warning: {}", warning)?;
    }
    writeln!(writer, "Memories: {}", status.stats.total_memories)?;
    writeln!(
        writer,
        "Embeddings: {} ready, {} pending",
        status.stats.memories_with_embeddings, status.stats.memories_pending_embedding
    )?;
    writeln!(writer, "Tags: {}", status.stats.total_tags)?;
    writeln!(writer, "Cross-refs: {}", status.stats.total_crossrefs)?;
    writeln!(writer, "Versions: {}", status.stats.total_versions)?;
    writeln!(writer, "Sync pending: {}", status.stats.sync_pending)?;
    write_derived_indexes(&mut writer, status)?;
    if let Some(error) = &status.health.error {
        writeln!(writer, "Error: {}", error)?;
    }
    Ok(())
}

fn write_database_size<W: std::io::Write>(
    writer: &mut W,
    status: &MaintenanceStatus,
) -> Result<()> {
    if let (
        Some(page_size),
        Some(page_count),
        Some(db_size_bytes),
        Some(freelist_count),
        Some(reclaimable_bytes),
    ) = (
        status.health.details.get("page_size"),
        status.health.details.get("page_count"),
        status.health.details.get("db_size_bytes"),
        status.health.details.get("freelist_count"),
        status.health.details.get("reclaimable_bytes"),
    ) {
        writeln!(
            writer,
            "Database pages: {} pages @ {} bytes ({} free, {} reclaimable bytes)",
            page_count, page_size, freelist_count, reclaimable_bytes
        )?;
        writeln!(writer, "Database size: {} bytes", db_size_bytes)?;
    } else {
        writeln!(
            writer,
            "Database size: {} bytes",
            status.stats.db_size_bytes
        )?;
    }
    Ok(())
}

fn write_derived_indexes<W: std::io::Write>(
    writer: &mut W,
    status: &MaintenanceStatus,
) -> Result<()> {
    if status.health.derived_indexes.is_empty() {
        return Ok(());
    }

    writeln!(writer, "Derived indexes:")?;
    for index in &status.health.derived_indexes {
        writeln!(
            writer,
            "  {} ({}): {} source={} indexed={} pending={} stale={} failed={} orphaned={}",
            index.name,
            json_enum_name(index.kind),
            json_enum_name(index.status),
            index.source_count,
            index.indexed_count,
            index.pending_count,
            index.stale_count,
            index.failed_count,
            index.orphaned_count
        )?;

        if index.name == "embeddings" {
            write_embedding_details(writer, index)?;
        }
        if index.name == "memories_fts" {
            let drift = index
                .details
                .get("drift_rows")
                .or_else(|| index.details.get("missing_rows"))
                .map(String::as_str)
                .unwrap_or("0");
            writeln!(writer, "    drift: {}", drift)?;
        }
    }
    Ok(())
}

fn write_embedding_details<W: std::io::Write>(
    writer: &mut W,
    index: &engram::storage::DerivedIndexHealth,
) -> Result<()> {
    let get = |key: &str| index.details.get(key).map(String::as_str).unwrap_or("0");
    let oldest_pending_age = index
        .details
        .get("oldest_pending_age")
        .or_else(|| index.details.get("oldest_pending_age_seconds"))
        .map(String::as_str)
        .unwrap_or("none");
    let oldest_processing_age = index
        .details
        .get("oldest_processing_age")
        .or_else(|| index.details.get("oldest_processing_age_seconds"))
        .map(String::as_str)
        .unwrap_or("none");
    let oldest_failed_age = index
        .details
        .get("oldest_failed_age")
        .or_else(|| index.details.get("oldest_failed_age_seconds"))
        .map(String::as_str)
        .unwrap_or("none");

    writeln!(
        writer,
        "    queue-state: pending={} processing={} stale_processing={} failed={} zero_retry_failed={} retryable_failed={} exhausted_failed={} max_retry_count={} oldest_pending_age={} oldest_processing_age={} oldest_failed_age={} retry_count_0={} retry_count_1={} retry_count_2={} retry_count_3+={}",
        get("pending"),
        get("processing"),
        get("stale_processing"),
        get("failed"),
        get("zero_retry_failed"),
        get("retryable_failed"),
        get("exhausted_failed"),
        get("max_retry_count"),
        oldest_pending_age,
        oldest_processing_age,
        oldest_failed_age,
        get("retry_count_0"),
        get("retry_count_1"),
        get("retry_count_2"),
        get("retry_count_3_plus"),
    )?;
    writeln!(
        writer,
        "    embedding profile: rows={} total_bytes={} avg_bytes={} min_bytes={} max_bytes={}",
        get("embedding_profile_rows"),
        get("embedding_profile_bytes_total"),
        get("embedding_profile_bytes_avg"),
        get("embedding_profile_bytes_min"),
        get("embedding_profile_bytes_max")
    )?;
    Ok(())
}

fn print_maintenance_status(status: &MaintenanceStatus) {
    if let Err(e) = write_maintenance_status(std::io::stdout(), status) {
        eprintln!("Failed to write maintenance status: {}", e);
    }
}
