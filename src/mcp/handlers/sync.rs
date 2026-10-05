//! Cloud sync, advanced sync, and multi-agent sharing tool handlers.

use serde_json::{json, Value};

use super::HandlerContext;

pub fn sync_status(ctx: &HandlerContext, _params: Value) -> Value {
    #[cfg(feature = "cloud")]
    {
        use crate::sync::get_sync_status;
        ctx.storage
            .with_connection(|conn| {
                let status = get_sync_status(conn)?;
                Ok(json!(status))
            })
            .unwrap_or_else(|e| json!({"error": e.to_string()}))
    }
    #[cfg(not(feature = "cloud"))]
    {
        let _ = ctx;
        json!({"error": "Cloud sync requires the 'cloud' feature to be enabled"})
    }
}

pub fn sync_version(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::get_sync_version;

    ctx.storage
        .with_connection(|conn| {
            let version = get_sync_version(conn)?;
            Ok(json!(version))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn sync_delta(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::get_sync_delta;

    let since_version = match params.get("since_version").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return json!({"error": "since_version is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
            let delta = get_sync_delta(conn, since_version)?;
            Ok(json!(delta))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn sync_state(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::{get_agent_sync_state, update_agent_sync_state};

    let agent_id = match params.get("agent_id").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "agent_id is required"}),
    };

    // If update_version is provided, update the state first
    if let Some(version) = params.get("update_version").and_then(|v| v.as_i64()) {
        if let Err(e) = ctx
            .storage
            .with_connection(|conn| update_agent_sync_state(conn, agent_id, version))
        {
            return json!({"error": e.to_string()});
        }
    }

    ctx.storage
        .with_connection(|conn| {
            let state = get_agent_sync_state(conn, agent_id)?;
            Ok(json!(state))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn sync_cleanup(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::cleanup_sync_data;

    let older_than_days = params
        .get("older_than_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(30);

    ctx.storage
        .with_connection(|conn| {
            let deleted = cleanup_sync_data(conn, older_than_days)?;
            Ok(json!({"deleted": deleted}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_share(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::share_memory;

    let memory_id = match params.get("memory_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "memory_id is required"}),
    };

    let from_agent = match params.get("from_agent").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "from_agent is required"}),
    };

    let to_agent = match params.get("to_agent").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "to_agent is required"}),
    };

    let message = params.get("message").and_then(|v| v.as_str());

    ctx.storage
        .with_connection(|conn| {
            let share_id = share_memory(conn, memory_id, from_agent, to_agent, message)?;
            Ok(json!({"share_id": share_id}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_shared_poll(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::poll_shared_memories;

    let agent_id = match params.get("agent_id").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "agent_id is required"}),
    };

    let include_acknowledged = params
        .get("include_acknowledged")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    ctx.storage
        .with_connection(|conn| {
            let shares = poll_shared_memories(conn, agent_id, include_acknowledged)?;
            Ok(json!({"shares": shares}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_share_ack(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::acknowledge_share;

    let share_id = match params.get("share_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "share_id is required"}),
    };

    let agent_id = match params.get("agent_id").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "agent_id is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
            acknowledge_share(conn, share_id, agent_id)?;
            Ok(json!({"acknowledged": true}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_events_poll(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::poll_events;
    use chrono::DateTime;

    let since_id = params.get("since_id").and_then(|v| v.as_i64());
    let since_time = params
        .get("since_time")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let agent_id = params.get("agent_id").and_then(|v| v.as_str());
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    ctx.storage
        .with_connection(|conn| {
            let events = poll_events(conn, since_id, since_time, agent_id, limit)?;
            Ok(json!({"events": events}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_events_clear(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::clear_events;
    use chrono::DateTime;

    let before_id = params.get("before_id").and_then(|v| v.as_i64());
    let before_time = params
        .get("before_time")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let keep_recent = params
        .get("keep_recent")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    ctx.storage
        .with_connection(|conn| {
            let deleted = clear_events(conn, before_id, before_time, keep_recent)?;
            Ok(json!({"deleted": deleted}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// The streamer opens `{db_path}-wal`. Reading the active WAL is lock-safe
/// (SQLite holds no POSIX locks on it), but a crafted `db_path` could make
/// that name alias the active database or `-shm`, whose descriptor close
/// would drop SQLite's locks (G1).
fn refuse_lock_bearing_wal_source(ctx: &HandlerContext, db_path: &str) -> Result<(), Value> {
    let wal_path = format!("{db_path}-wal");
    ctx.storage
        .refuse_lock_bearing_sqlite_file(&wal_path)
        .map_err(|e| json!({"error": e.to_string()}))
}

pub fn replication_status(ctx: &HandlerContext, params: Value) -> Value {
    use crate::sync::WalReplicationStreamer;

    let db_path = params
        .get("db_path")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| ctx.storage.db_path());

    if db_path == ":memory:" {
        return json!({
            "status": "in_memory",
            "message": "In-memory database does not produce disk WAL files",
            "lag": null
        });
    }

    if let Err(e) = refuse_lock_bearing_wal_source(ctx, db_path) {
        return e;
    }

    let streamer = WalReplicationStreamer::new(db_path);
    match streamer.status() {
        Ok(status) => json!(status),
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn replication_sync_now(ctx: &HandlerContext, params: Value) -> Value {
    use crate::sync::WalReplicationStreamer;

    let db_path = params
        .get("db_path")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| ctx.storage.db_path());

    if db_path == ":memory:" {
        return json!({"error": "In-memory database does not support disk WAL replication"});
    }

    let compress = params
        .get("compress")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let identifier = params.get("identifier").and_then(|v| v.as_str());

    if let Err(e) = refuse_lock_bearing_wal_source(ctx, db_path) {
        return e;
    }

    let mut streamer = WalReplicationStreamer::new(db_path).with_compression(compress);
    if let Some(id) = identifier {
        streamer = streamer.with_identifier(id);
    }

    match streamer.flush_delta() {
        Ok(Some(pack)) => json!({
            "synced": true,
            "pack_id": pack.pack_id,
            "start_frame": pack.start_frame,
            "end_frame": pack.end_frame,
            "frame_count": pack.frame_count,
            "page_size": pack.page_size,
            "checkpoint_seq": pack.checkpoint_seq,
            "compressed": pack.compressed,
            "payload_bytes": pack.payload.len(),
            "checksum_sha256": pack.checksum_sha256,
        }),
        Ok(None) => json!({
            "synced": true,
            "message": "No new WAL frames to replicate (already up to date)",
            "frame_count": 0
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn replication_recover(ctx: &HandlerContext, params: Value) -> Value {
    replication_recover_classified(ctx, params).0
}

/// `replication_recover` plus how the call ended, for observability. Input
/// validation failures are `Rejected` (the recovery engine never ran); only an
/// error from the engine itself is `Failed`.
pub fn replication_recover_classified(
    ctx: &HandlerContext,
    params: Value,
) -> (Value, crate::observability::RecoveryOutcome) {
    use crate::observability::RecoveryOutcome as Outcome;
    use crate::sync::{RecoveryOptions, WalRecoveryEngine};
    use chrono::DateTime;
    use std::path::{Path, PathBuf};

    let rejected = |value: Value| (value, Outcome::Rejected);
    fn engine_result<E: std::fmt::Display>(
        result: std::result::Result<crate::sync::RecoveryReport, E>,
    ) -> (Value, Outcome) {
        match result {
            Ok(report) => (json!(report), Outcome::Succeeded),
            Err(e) => (json!({"error": e.to_string()}), Outcome::Failed),
        }
    }

    let target_db_path = match params.get("target_db_path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return rejected(json!({"error": "target_db_path is required"})),
    };

    let default_source = ctx.storage.db_path();
    let source_db_path = params
        .get("source_db_path")
        .and_then(|v| v.as_str())
        .unwrap_or(default_source);

    if source_db_path == ":memory:" {
        return rejected(json!({"error": "Cannot recover from in-memory database"}));
    }

    let default_wal = format!("{}-wal", source_db_path);
    let source_wal_path = params
        .get("source_wal_path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default_wal));

    let target_frame = params
        .get("target_frame")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let target_time = params
        .get("target_time")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let commit_boundary_only = params
        .get("commit_boundary_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let verify_integrity = params
        .get("verify_integrity")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let options = RecoveryOptions {
        target_frame,
        target_time,
        commit_boundary_only,
        verify_integrity,
    };

    // G1: never open/close the server's own database files with a raw File.
    let target_path = Path::new(target_db_path);
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(target_path) {
        return rejected(json!({"error": format!("Invalid target_db_path: {}", e)}));
    }
    if ctx.storage.is_active_sqlite_artifact(source_db_path) {
        let explicit_wal = params.get("source_wal_path").is_some();
        if target_frame.is_some() || target_time.is_some() || explicit_wal {
            return rejected(
                json!({"error": "point-in-time recovery from the active database to an \
                earlier frame or time is not supported while it is open; recover from a \
                closed copy (source_db_path + source_wal_path), or omit target_frame, \
                target_time and source_wal_path to recover its latest committed state"}),
            );
        }
        return engine_result(WalRecoveryEngine::recover_active_database(
            &ctx.storage,
            target_path,
            &options,
        ));
    }
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(source_db_path) {
        return rejected(json!({"error": format!("Invalid source_db_path: {}", e)}));
    }
    if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(&source_wal_path) {
        return rejected(json!({"error": format!("Invalid source_wal_path: {}", e)}));
    }

    engine_result(WalRecoveryEngine::point_in_time_recovery(
        Path::new(source_db_path),
        &source_wal_path,
        Path::new(target_db_path),
        &options,
    ))
}
