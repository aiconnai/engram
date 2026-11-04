//! Memory lifecycle and retention policy tool handlers.

use chrono::{DateTime, Utc};
use rusqlite::params;
use serde_json::{json, Value};

use super::HandlerContext;
use crate::intelligence::{decide_lifecycle_state, LifecycleConfig};
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
use crate::storage::queries::update_memory_lifecycle_state;
use crate::types::{LifecycleState, Memory, MemoryScope, MemoryTier, MemoryType, Visibility};

fn resolve_optional_workspace(
    ctx: &HandlerContext,
    tool_name: &str,
    params: &Value,
) -> std::result::Result<Option<String>, Value> {
    let normalized = match params.get("workspace") {
        None | Some(Value::Null) => None,
        Some(Value::String(ws)) => match crate::types::normalize_workspace(ws) {
            Ok(norm) => Some(norm),
            Err(e) => return Err(json!({"error": format!("Invalid workspace: {e}")})),
        },
        Some(_) => return Err(json!({"error": "workspace must be a string"})),
    };
    if let Some(ref p) = ctx.principal {
        if normalized.is_none()
            && matches!(p, crate::auth::TransportPrincipal::AnonymousLoopback(_))
        {
            return Err(crate::mcp::permission::workspace_claim_denied(tool_name, p));
        }
        if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
            tool_name,
            p,
            normalized.as_deref(),
        ) {
            return Err(denial);
        }
    }
    Ok(normalized)
}

fn resolve_required_workspace(
    ctx: &HandlerContext,
    tool_name: &str,
    params: &Value,
) -> std::result::Result<String, Value> {
    let raw = match params.get("workspace") {
        None | Some(Value::Null) => return Err(json!({"error": "workspace is required"})),
        Some(Value::String(ws)) => ws.as_str(),
        Some(_) => return Err(json!({"error": "workspace must be a string"})),
    };
    let normalized = match crate::types::normalize_workspace(raw) {
        Ok(norm) => norm,
        Err(e) => return Err(json!({"error": format!("Invalid workspace: {e}")})),
    };
    if let Some(ref p) = ctx.principal {
        if let Some(denial) =
            crate::mcp::permission::permission_denial_for_principal(tool_name, p, Some(&normalized))
        {
            return Err(denial);
        }
    }
    Ok(normalized)
}

fn optional_bool(params: &Value, key: &str) -> Result<Option<bool>, String> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("{key} must be a boolean")),
    }
}

pub fn lifecycle_status(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = match resolve_optional_workspace(ctx, "lifecycle_status", &params) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| {
            let query = if workspace.is_some() {
                "SELECT lifecycle_state, COUNT(*) as count
                 FROM memories
                 WHERE workspace = ? AND valid_to IS NULL
                 GROUP BY lifecycle_state"
            } else {
                "SELECT lifecycle_state, COUNT(*) as count
                 FROM memories
                 WHERE valid_to IS NULL
                 GROUP BY lifecycle_state"
            };

            let mut stmt = conn.prepare(query)?;
            let rows: Vec<(String, i64)> = if let Some(ref ws) = workspace {
                stmt.query_map(params![ws], |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?
                            .unwrap_or_else(|| "active".to_string()),
                        row.get::<_, i64>(1)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
            } else {
                stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?
                            .unwrap_or_else(|| "active".to_string()),
                        row.get::<_, i64>(1)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
            };

            let mut active = 0i64;
            let mut stale = 0i64;
            let mut archived = 0i64;

            for (state, count) in rows {
                match state.as_str() {
                    "active" => active = count,
                    "stale" => stale = count,
                    "archived" => archived = count,
                    _ => active += count,
                }
            }

            Ok(json!({
                "active": active,
                "stale": stale,
                "archived": archived,
                "total": active + stale + archived,
                "workspace": workspace
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

fn collect_lifecycle_transitions(
    conn: &rusqlite::Connection,
    workspace: Option<&str>,
    now: DateTime<Utc>,
    config: &LifecycleConfig,
) -> crate::error::Result<Vec<LifecycleTransition>> {
    let candidate_query = if workspace.is_some() {
        "SELECT id, content, memory_type, importance, access_count,
                created_at, updated_at, last_accessed_at, lifecycle_state,
                workspace, tier
         FROM memories
         WHERE valid_to IS NULL
           AND COALESCE(lifecycle_state, 'active') != 'archived'
           AND workspace = ?
         ORDER BY id ASC"
    } else {
        "SELECT id, content, memory_type, importance, access_count,
                created_at, updated_at, last_accessed_at, lifecycle_state,
                workspace, tier
         FROM memories
         WHERE valid_to IS NULL
           AND COALESCE(lifecycle_state, 'active') != 'archived'
         ORDER BY id ASC"
    };

    let candidates: Vec<Memory> = {
        let mut stmt = conn.prepare(candidate_query)?;
        if let Some(ws) = workspace {
            stmt.query_map(params![ws], |row| lifecycle_memory_from_row(row, now))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| lifecycle_memory_from_row(row, now))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        }
    };

    Ok(candidates
        .into_iter()
        .filter_map(|memory| {
            let next = decide_lifecycle_state(&memory, now, config);
            if next == memory.lifecycle_state {
                return None;
            }
            Some(LifecycleTransition {
                id: memory.id,
                workspace: memory.workspace,
                preview: memory.content.chars().take(50).collect(),
                next,
            })
        })
        .collect())
}

pub fn lifecycle_run(ctx: &HandlerContext, params: Value) -> Value {
    let dry_run = match optional_bool(&params, "dry_run") {
        Ok(val) => val.unwrap_or(true),
        Err(message) => return json!({"error": message}),
    };
    let workspace = match resolve_optional_workspace(ctx, "lifecycle_run", &params) {
        Ok(ws) => ws,
        Err(err) => return err,
    };
    let config = match lifecycle_config_from_params(&params) {
        Ok(config) => config,
        Err(message) => return json!({"error": message}),
    };
    let now = Utc::now();

    if dry_run {
        return ctx
            .storage
            .with_connection(|conn| {
                let transitions =
                    collect_lifecycle_transitions(conn, workspace.as_deref(), now, &config)?;
                let stale_candidates = transition_previews(&transitions, LifecycleState::Stale);
                let archive_candidates =
                    transition_previews(&transitions, LifecycleState::Archived);
                // Counts must reflect every transition, not the capped preview
                // arrays (which are limited to 10 for response size).
                Ok(json!({
                    "dry_run": true,
                    "would_mark_stale": count_transitions(&transitions, LifecycleState::Stale),
                    "would_archive": count_transitions(&transitions, LifecycleState::Archived),
                    "stale_candidates": stale_candidates,
                    "archive_candidates": archive_candidates
                }))
            })
            .unwrap_or_else(|e| json!({"error": e.to_string()}));
    }

    ctx.storage
        .with_transaction(|conn| {
            let transitions =
                collect_lifecycle_transitions(conn, workspace.as_deref(), now, &config)?;
            let mut stale_count = 0;
            let mut archive_count = 0;

            for transition in &transitions {
                update_memory_lifecycle_state(conn, transition.id, transition.next)?;
                match transition.next {
                    LifecycleState::Active => {}
                    LifecycleState::Stale => stale_count += 1,
                    LifecycleState::Archived => archive_count += 1,
                }
            }

            let operation_id = uuid::Uuid::new_v4().to_string();
            for transition in &transitions {
                let new_state = transition.next.to_string();
                emit_best_effort(
                    conn,
                    &EnrichmentEvent {
                        operation_id: &operation_id,
                        event_type: "lifecycle_transition",
                        memory_id: Some(transition.id),
                        version_id: None,
                        triggered_by: "lifecycle_run",
                        agent_id: None,
                        workspace: Some(transition.workspace.as_str()),
                        params: json!({"dry_run": false}),
                        outcome: json!({"new_state": new_state}),
                        status: "completed",
                        dry_run: false,
                    },
                );
            }

            Ok(json!({
                "dry_run": false,
                "marked_stale": stale_count,
                "archived": archive_count
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

#[derive(Clone)]
struct LifecycleTransition {
    id: i64,
    workspace: String,
    preview: String,
    next: LifecycleState,
}

/// Read an optional integer param, distinguishing "absent" (Ok(None)) from
/// "present but wrong type" (Err). A wrong-typed value must not silently fall
/// back to a default — these params drive irreversible bulk lifecycle writes.
fn optional_i64(params: &Value, key: &str) -> Result<Option<i64>, String> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| format!("{key} must be an integer")),
    }
}

/// Read an optional float param, distinguishing "absent" from "wrong type".
fn optional_f64(params: &Value, key: &str) -> Result<Option<f64>, String> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| format!("{key} must be a number")),
    }
}

/// Build a [`LifecycleConfig`] from MCP params, validating types and ranges.
///
/// `lifecycle_run` / `lifecycle_config` are external-input boundaries: a single
/// malformed call (`max_importance_mult < 1.0`, negative windows, or an inverted
/// `stale_days > archive_days`) could otherwise drive `decide_lifecycle_state`
/// to archive the entire active corpus. Reject rather than clamp so the caller
/// learns their input was wrong.
fn lifecycle_config_from_params(params: &Value) -> Result<LifecycleConfig, String> {
    let defaults = LifecycleConfig::default();

    let stale_days_base = optional_i64(params, "stale_days")?.unwrap_or(defaults.stale_days_base);
    let archive_days_base =
        optional_i64(params, "archive_days")?.unwrap_or(defaults.archive_days_base);
    let hard_idle_cap_days =
        optional_i64(params, "hard_idle_cap_days")?.unwrap_or(defaults.hard_idle_cap_days);
    let max_importance_mult = optional_f64(params, "max_importance_mult")?
        .map(|v| v as f32)
        .unwrap_or(defaults.max_importance_mult);

    if stale_days_base < 0 {
        return Err("stale_days must be >= 0".to_string());
    }
    if archive_days_base < 0 {
        return Err("archive_days must be >= 0".to_string());
    }
    if hard_idle_cap_days < 0 {
        return Err("hard_idle_cap_days must be >= 0".to_string());
    }
    if archive_days_base < stale_days_base {
        return Err("archive_days must be >= stale_days".to_string());
    }
    // A multiplier < 1.0 shrinks the decay windows below the configured base,
    // inverting "importance protects" into immediate archival; NaN (which fails
    // every comparison) is rejected here too.
    if max_importance_mult.is_nan() || max_importance_mult < 1.0 {
        return Err("max_importance_mult must be >= 1.0".to_string());
    }

    Ok(LifecycleConfig {
        stale_days_base,
        archive_days_base,
        hard_idle_cap_days,
        max_importance_mult,
    })
}

fn lifecycle_memory_from_row(
    row: &rusqlite::Row<'_>,
    now: DateTime<Utc>,
) -> rusqlite::Result<Memory> {
    let created_at = parse_rfc3339_or_now(row.get::<_, String>(5)?, now);
    let updated_at = parse_rfc3339_or_now(row.get::<_, String>(6)?, now);
    let last_accessed_at = row
        .get::<_, Option<String>>(7)?
        .map(|value| parse_rfc3339_or_now(value, now));
    let lifecycle_state = row
        .get::<_, Option<String>>(8)?
        .and_then(|state| state.parse::<LifecycleState>().ok())
        .unwrap_or(LifecycleState::Active);
    let memory_type = row
        .get::<_, String>(2)?
        .parse::<MemoryType>()
        .unwrap_or(MemoryType::Note);
    let tier = row
        .get::<_, String>(10)?
        .parse::<MemoryTier>()
        .unwrap_or(MemoryTier::Permanent);

    Ok(Memory {
        id: row.get(0)?,
        content: row.get(1)?,
        memory_type,
        tags: Vec::new(),
        metadata: std::collections::HashMap::new(),
        importance: row.get(3)?,
        access_count: row.get(4)?,
        created_at,
        updated_at,
        last_accessed_at,
        owner_id: None,
        visibility: Visibility::Private,
        scope: MemoryScope::Global,
        workspace: row
            .get::<_, Option<String>>(9)?
            .unwrap_or_else(|| "default".to_string()),
        tier,
        version: 1,
        has_embedding: false,
        expires_at: None,
        content_hash: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        procedure_success_count: 0,
        procedure_failure_count: 0,
        summary_of_id: None,
        lifecycle_state,
        stability: 1.0,
        media_url: None,
    })
}

fn parse_rfc3339_or_now(value: String, now: DateTime<Utc>) -> DateTime<Utc> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(&value) {
        return dt.with_timezone(&Utc);
    }
    for fmt in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&value, fmt) {
            return naive.and_utc();
        }
    }
    tracing::warn!(
        target = "engram::lifecycle",
        "unparseable memory timestamp in lifecycle scan; falling back to now"
    );
    now
}

/// Total number of transitions targeting `state`, uncapped. Used for the
/// dry_run counts so they match what apply will actually do; the preview arrays
/// (see [`transition_previews`]) are intentionally capped and must not be used
/// as a count source.
fn count_transitions(transitions: &[LifecycleTransition], state: LifecycleState) -> usize {
    transitions
        .iter()
        .filter(|transition| transition.next == state)
        .count()
}

fn transition_previews(transitions: &[LifecycleTransition], state: LifecycleState) -> Vec<Value> {
    transitions
        .iter()
        .filter(|transition| transition.next == state)
        .take(10)
        .map(|transition| {
            json!({
                "id": transition.id,
                "preview": transition.preview,
                "target_state": transition.next.to_string()
            })
        })
        .collect()
}

pub fn memory_set_lifecycle(ctx: &HandlerContext, params: Value) -> Value {
    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };

    let state = match params.get("state").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "state is required"}),
    };

    if !["active", "stale", "archived"].contains(&state) {
        return json!({"error": "state must be one of: active, stale, archived"});
    }
    let lifecycle_state = match state.parse::<LifecycleState>() {
        Ok(state) => state,
        Err(_) => return json!({"error": "state must be one of: active, stale, archived"}),
    };

    ctx.storage
        .with_transaction(|conn| {
            crate::mcp::workspace_guard::ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let updated = match update_memory_lifecycle_state(conn, id, lifecycle_state) {
                Ok(mem) => mem,
                Err(crate::error::EngramError::NotFound(_)) => {
                    return Ok(json!({"error": "Memory not found"}));
                }
                Err(e) => return Err(e),
            };

            let operation_id = uuid::Uuid::new_v4().to_string();
            emit_best_effort(
                conn,
                &EnrichmentEvent {
                    operation_id: &operation_id,
                    event_type: "lifecycle_transition",
                    memory_id: Some(id),
                    version_id: None,
                    triggered_by: "memory_set_lifecycle",
                    agent_id: None,
                    workspace: Some(updated.workspace.as_str()),
                    params: json!({"state": state}),
                    outcome: json!({"new_state": state}),
                    status: "completed",
                    dry_run: false,
                },
            );

            Ok(json!({"id": id, "lifecycle_state": state, "updated": true}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn lifecycle_config(_ctx: &HandlerContext, params: Value) -> Value {
    let config = match lifecycle_config_from_params(&params) {
        Ok(config) => config,
        Err(message) => return json!({"error": message}),
    };

    json!({
        "stale_days": config.stale_days_base,
        "archive_days": config.archive_days_base,
        "hard_idle_cap_days": config.hard_idle_cap_days,
        "max_importance_mult": config.max_importance_mult,
        "lifecycle_enabled": std::env::var("ENGRAM_LIFECYCLE_ENABLED")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true),
        "note": "Pass values to update configuration"
    })
}

pub fn retention_policy_set(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::set_retention_policy;

    let workspace = match resolve_required_workspace(ctx, "retention_policy_set", &params) {
        Ok(ws) => ws,
        Err(err) => return err,
    };
    let max_age_days = match optional_i64(&params, "max_age_days") {
        Ok(v) => v,
        Err(msg) => return json!({"error": msg}),
    };
    let max_memories = match optional_i64(&params, "max_memories") {
        Ok(v) => v,
        Err(msg) => return json!({"error": msg}),
    };
    let compress_after_days = match optional_i64(&params, "compress_after_days") {
        Ok(v) => v,
        Err(msg) => return json!({"error": msg}),
    };
    let compress_max_importance = match optional_f64(&params, "compress_max_importance") {
        Ok(Some(f)) if f.is_finite() => Some(f as f32),
        Ok(Some(_)) => return json!({"error": "compress_max_importance must be a finite number"}),
        Ok(None) => None,
        Err(msg) => return json!({"error": msg}),
    };
    let compress_min_access = match optional_i64(&params, "compress_min_access") {
        Ok(Some(i)) => match i32::try_from(i) {
            Ok(val) => Some(val),
            Err(_) => return json!({"error": "compress_min_access is out of range"}),
        },
        Ok(None) => None,
        Err(msg) => return json!({"error": msg}),
    };
    let auto_delete_after_days = match optional_i64(&params, "auto_delete_after_days") {
        Ok(v) => v,
        Err(msg) => return json!({"error": msg}),
    };
    let exclude_types: Option<Vec<String>> = match params.get("exclude_types") {
        None | Some(Value::Null) => None,
        Some(Value::Array(arr)) => {
            let mut types = Vec::with_capacity(arr.len());
            for item in arr {
                match item.as_str() {
                    Some(s) => types.push(s.to_string()),
                    None => {
                        return json!({"error": "exclude_types must be an array of strings"});
                    }
                }
            }
            Some(types)
        }
        Some(_) => return json!({"error": "exclude_types must be an array of strings"}),
    };

    ctx.storage
        .with_transaction(|conn| {
            let policy = set_retention_policy(
                conn,
                &workspace,
                max_age_days,
                max_memories,
                compress_after_days,
                compress_max_importance,
                compress_min_access,
                auto_delete_after_days,
                exclude_types,
            )?;
            Ok(json!(policy))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn retention_policy_get(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_retention_policy;

    let workspace = match resolve_required_workspace(ctx, "retention_policy_get", &params) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_connection(|conn| match get_retention_policy(conn, &workspace)? {
            Some(policy) => Ok(json!(policy)),
            None => Ok(json!({
                "workspace": workspace,
                "policy": null,
                "note": "No retention policy set for this workspace"
            })),
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn retention_policy_list(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::list_retention_policies;

    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "retention_policy_list",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }

    ctx.storage
        .with_connection(|conn| {
            let policies = list_retention_policies(conn)?;
            Ok(json!({"policies": policies, "count": policies.len()}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn retention_policy_delete(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::delete_retention_policy;

    let workspace = match resolve_required_workspace(ctx, "retention_policy_delete", &params) {
        Ok(ws) => ws,
        Err(err) => return err,
    };

    ctx.storage
        .with_transaction(|conn| {
            let deleted = delete_retention_policy(conn, &workspace)?;
            Ok(json!({"deleted": deleted, "workspace": workspace}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn retention_policy_apply(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::apply_retention_policies;

    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "retention_policy_apply",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }

    let dry_run = match optional_bool(&params, "dry_run") {
        Ok(val) => val.unwrap_or(false),
        Err(message) => return json!({"error": message}),
    };

    if dry_run {
        use crate::storage::queries::list_retention_policies;
        return ctx
            .storage
            .with_connection(|conn| {
                let policies = list_retention_policies(conn)?;
                Ok(json!({
                    "dry_run": true,
                    "policies_count": policies.len(),
                    "policies": policies,
                    "note": "Set dry_run=false to apply"
                }))
            })
            .unwrap_or_else(|e| json!({"error": e.to_string()}));
    }

    let tx_result = ctx.storage.with_transaction(apply_retention_policies);

    match tx_result {
        Ok(affected) => {
            // Emit SUCCESS event in a separate connection, outside the now-committed transaction.
            if affected > 0 {
                let operation_id = uuid::Uuid::new_v4().to_string();
                ctx.storage
                    .with_connection(|conn| {
                        emit_best_effort(
                            conn,
                            &EnrichmentEvent {
                                operation_id: &operation_id,
                                event_type: "lifecycle_transition",
                                memory_id: None,
                                version_id: None,
                                triggered_by: "retention_policy_apply",
                                agent_id: None,
                                workspace: None,
                                params: json!({}),
                                outcome: json!({"memories_affected": affected}),
                                status: "completed",
                                dry_run: false,
                            },
                        );
                        Ok::<_, crate::error::EngramError>(())
                    })
                    .ok();
            }
            json!({"applied": true, "memories_affected": affected})
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

/// Unified facade for memory lifecycle mutations: promotion, decay, expiration, and scoring.
pub fn memory_lifecycle_update(ctx: &HandlerContext, params: Value) -> Value {
    use rusqlite::OptionalExtension;

    let action = match params.get("action") {
        None | Some(Value::Null) => "promote",
        Some(Value::String(s)) => s.as_str(),
        Some(_) => return json!({"error": "action must be a string"}),
    };

    match action {
        "promote" => super::memory_policy::memory_promote(ctx, params),
        "promote_permanent" | "permanent" => {
            let mut p = params;
            if let Value::Object(ref mut map) = p {
                map.insert("canonical_tier".to_string(), json!(true));
            }
            super::memory_policy::memory_promote(ctx, p)
        }
        "decay" => {
            let mut p = params;
            let Some(id) = p.get("id").and_then(|v| v.as_i64()) else {
                return json!({"error": "id is required"});
            };
            let claimed_workspace = match p.get("workspace") {
                None | Some(Value::Null) => None,
                Some(Value::String(raw)) => match crate::types::normalize_workspace(raw) {
                    Ok(ws) => Some(ws),
                    Err(e) => return json!({"error": e.to_string()}),
                },
                Some(_) => return json!({"error": "workspace must be a string"}),
            };
            let lookup = ctx.storage.with_connection(|conn| {
                crate::mcp::workspace_guard::ensure_memory_access(
                    conn,
                    ctx.principal.as_ref(),
                    id,
                )?;
                let ws: Option<String> = conn
                    .query_row(
                        "SELECT workspace FROM memories WHERE id = ?1 AND valid_to IS NULL",
                        [id],
                        |row| row.get(0),
                    )
                    .optional()?;
                ws.ok_or(crate::error::EngramError::NotFound(id))
            });
            let mem_ws = match lookup {
                Ok(ws) => ws,
                Err(e) => return json!({"error": e.to_string()}),
            };
            if let Some(ref claimed) = claimed_workspace {
                if claimed != &mem_ws {
                    return json!({
                        "error": format!(
                            "workspace mismatch for memory {id}: requested '{claimed}', memory belongs to '{mem_ws}'"
                        )
                    });
                }
            }
            if let Value::Object(ref mut map) = p {
                map.insert("workspace".to_string(), json!(mem_ws));
            }
            super::memory_policy::memory_decay(ctx, p)
        }
        "expire" | "set_expiration" => super::memory_crud::set_expiration(ctx, params),
        "score" => super::memory_policy::memory_score(ctx, params),
        "explain" => super::memory_policy::memory_explain(ctx, params),
        "transition" | "set_state" | "set_lifecycle" => memory_set_lifecycle(ctx, params),
        "restore" => {
            let mut p = params;
            if let Value::Object(ref mut map) = p {
                map.insert("state".to_string(), json!("active"));
            }
            memory_set_lifecycle(ctx, p)
        }
        other => json!({
            "error": format!(
                "unsupported lifecycle action '{other}': expected 'promote', 'promote_permanent', 'decay', 'expire', 'score', 'explain', 'transition', or 'restore'"
            )
        }),
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    fn test_ctx() -> super::super::HandlerContext {
        use crate::embedding::{create_embedder, EmbeddingCache};
        use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
        use crate::storage::Storage;
        use crate::types::EmbeddingConfig;
        use parking_lot::Mutex;
        use std::sync::Arc;

        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        super::super::HandlerContext {
            storage,
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(
                    embedder.dimensions(),
                    crate::search::VectorMetric::Cosine,
                ),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
            principal: None,
        }
    }

    fn seed_lifecycle_memory(
        ctx: &super::super::HandlerContext,
        content: &str,
        importance: f32,
        access_count: i32,
        idle_days: i64,
        lifecycle_state: LifecycleState,
    ) -> i64 {
        use chrono::{Duration, Utc};

        let old_ts = (Utc::now() - Duration::days(idle_days)).to_rfc3339();
        let target_content = content;
        let target_importance = importance;
        let target_access_count = access_count;
        let target_state = lifecycle_state.to_string();
        ctx.storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (
                         content, memory_type, workspace, tier, importance,
                         access_count, lifecycle_state, created_at, updated_at,
                         last_accessed_at
                     )
                     VALUES (?1, 'note', 'default', 'permanent', ?2, ?3, ?4, ?5, ?5, ?5)",
                    rusqlite::params![
                        target_content,
                        target_importance,
                        target_access_count,
                        target_state,
                        old_ts
                    ],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .expect("seed lifecycle memory")
    }

    fn lifecycle_state(ctx: &super::super::HandlerContext, id: i64) -> String {
        ctx.storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COALESCE(lifecycle_state, 'active') FROM memories WHERE id = ?1",
                    [id],
                    |row| row.get::<_, String>(0),
                )?)
            })
            .expect("query lifecycle state")
    }

    fn candidate_ids(result: &Value, key: &str) -> Vec<i64> {
        result[key]
            .as_array()
            .expect("candidate array")
            .iter()
            .map(|candidate| {
                candidate["id"]
                    .as_i64()
                    .expect("candidate id should be an integer")
            })
            .collect()
    }

    #[test]
    fn test_lifecycle_run_archives_high_importance_high_access_candidate() {
        let ctx = test_ctx();
        let id = seed_lifecycle_memory(
            &ctx,
            "important abandoned memory",
            1.0,
            100,
            370,
            LifecycleState::Active,
        );

        let result = lifecycle_run(
            &ctx,
            json!({
                "dry_run": false,
                "workspace": "default",
                "min_importance": 0.0
            }),
        );

        assert!(result.get("error").is_none(), "unexpected error: {result}");
        assert_eq!(result["archived"].as_i64(), Some(1), "{result}");
        assert_eq!(lifecycle_state(&ctx, id), "archived");
    }

    #[test]
    fn test_lifecycle_run_dry_run_apply_parity() {
        let ctx = test_ctx();
        let stale_id =
            seed_lifecycle_memory(&ctx, "stale candidate", 0.0, 0, 35, LifecycleState::Active);
        let archive_id = seed_lifecycle_memory(
            &ctx,
            "archive candidate",
            0.0,
            0,
            95,
            LifecycleState::Active,
        );

        let dry = lifecycle_run(
            &ctx,
            json!({
                "dry_run": true,
                "workspace": "default"
            }),
        );
        assert!(
            dry.get("error").is_none(),
            "unexpected dry-run error: {dry}"
        );
        assert_eq!(candidate_ids(&dry, "stale_candidates"), vec![stale_id]);
        assert_eq!(candidate_ids(&dry, "archive_candidates"), vec![archive_id]);

        let applied = lifecycle_run(
            &ctx,
            json!({
                "dry_run": false,
                "workspace": "default"
            }),
        );
        assert!(
            applied.get("error").is_none(),
            "unexpected apply error: {applied}"
        );
        assert_eq!(applied["marked_stale"].as_i64(), Some(1), "{applied}");
        assert_eq!(applied["archived"].as_i64(), Some(1), "{applied}");
        assert_eq!(lifecycle_state(&ctx, stale_id), "stale");
        assert_eq!(lifecycle_state(&ctx, archive_id), "archived");
    }

    #[test]
    fn test_lifecycle_run_apply_is_idempotent() {
        let ctx = test_ctx();
        let id = seed_lifecycle_memory(
            &ctx,
            "idempotent archive",
            0.0,
            0,
            95,
            LifecycleState::Active,
        );

        let first = lifecycle_run(&ctx, json!({"dry_run": false, "workspace": "default"}));
        let second = lifecycle_run(&ctx, json!({"dry_run": false, "workspace": "default"}));

        assert!(
            first.get("error").is_none(),
            "unexpected first error: {first}"
        );
        assert!(
            second.get("error").is_none(),
            "unexpected second error: {second}"
        );
        assert_eq!(first["archived"].as_i64(), Some(1), "{first}");
        assert_eq!(second["marked_stale"].as_i64(), Some(0), "{second}");
        assert_eq!(second["archived"].as_i64(), Some(0), "{second}");
        assert_eq!(lifecycle_state(&ctx, id), "archived");
    }

    #[test]
    fn test_lifecycle_run_allows_direct_active_to_archived_transition() {
        let ctx = test_ctx();
        let id = seed_lifecycle_memory(&ctx, "direct archive", 0.0, 0, 90, LifecycleState::Active);

        let result = lifecycle_run(&ctx, json!({"dry_run": false, "workspace": "default"}));

        assert!(result.get("error").is_none(), "unexpected error: {result}");
        assert_eq!(result["marked_stale"].as_i64(), Some(0), "{result}");
        assert_eq!(result["archived"].as_i64(), Some(1), "{result}");
        assert_eq!(lifecycle_state(&ctx, id), "archived");
    }

    #[test]
    fn test_lifecycle_run_emits_enrichment_event() {
        use chrono::{Duration, Utc};

        let ctx = test_ctx();

        // Seed a memory old enough to trigger stale transition:
        // created_at far in the past, low importance, low access_count.
        let old_ts = (Utc::now() - Duration::days(60)).to_rfc3339();
        let memory_id = ctx
            .storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, importance,
                             access_count, lifecycle_state, created_at)
                     VALUES ('old memory content', 'note', 'default', 0.1, 0, 'active', ?1)",
                    rusqlite::params![old_ts],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .expect("seed old memory");

        // Run lifecycle_run with dry_run: false
        let result = lifecycle_run(
            &ctx,
            json!({
                "dry_run": false,
                "stale_days": 30,
                "workspace": "default"
            }),
        );
        assert!(
            result.get("error").is_none(),
            "lifecycle_run returned error: {result}"
        );
        assert!(
            result["marked_stale"].as_i64().unwrap_or(0) > 0,
            "expected at least one memory marked stale"
        );

        // Assert enrichment event was emitted
        let event_count: i64 = ctx
            .storage
            .with_connection(|conn| {
                let n: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM enrichment_events
                     WHERE event_type = 'lifecycle_transition'
                       AND triggered_by = 'lifecycle_run'",
                    [],
                    |row| row.get(0),
                )?;
                Ok(n)
            })
            .expect("query enrichment_events");

        assert!(
            event_count > 0,
            "expected enrichment_events rows for lifecycle_transition, got 0"
        );

        let update_event_count: i64 = ctx
            .storage
            .with_connection(|conn| {
                let n: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM memory_events
                     WHERE memory_id = ?1
                       AND data LIKE '%lifecycle_state%'",
                    rusqlite::params![memory_id],
                    |row| row.get(0),
                )?;
                Ok(n)
            })
            .expect("query memory_events");

        assert_eq!(
            update_event_count, 1,
            "expected one memory update event for lifecycle_state"
        );

        let version_count: i64 = ctx
            .storage
            .with_connection(|conn| {
                let n: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM memory_versions WHERE memory_id = ?1",
                    rusqlite::params![memory_id],
                    |row| row.get(0),
                )?;
                Ok(n)
            })
            .expect("query memory_versions");

        assert_eq!(
            version_count, 1,
            "expected lifecycle transition to create a memory version"
        );
    }

    #[test]
    fn test_lifecycle_run_dry_run_counts_are_uncapped() {
        // Seed more than the preview cap (10) of archive candidates so that the
        // reported count must come from the full transition set, not the capped
        // preview array.
        let ctx = test_ctx();
        let total = 12;
        for i in 0..total {
            seed_lifecycle_memory(
                &ctx,
                &format!("archive candidate {i}"),
                0.0,
                0,
                95,
                LifecycleState::Active,
            );
        }

        let dry = lifecycle_run(&ctx, json!({"dry_run": true, "workspace": "default"}));
        assert!(
            dry.get("error").is_none(),
            "unexpected dry-run error: {dry}"
        );

        // The count must reflect every memory that would transition, not the
        // preview cap of 10.
        assert_eq!(
            dry["would_archive"].as_i64(),
            Some(total),
            "would_archive must count all transitions, not the capped preview: {dry}"
        );
        // Preview array stays bounded.
        assert!(
            dry["archive_candidates"].as_array().expect("array").len() <= 10,
            "preview array should stay capped at 10: {dry}"
        );

        // dry_run count must equal what apply actually does.
        let applied = lifecycle_run(&ctx, json!({"dry_run": false, "workspace": "default"}));
        assert_eq!(
            applied["archived"].as_i64(),
            dry["would_archive"].as_i64(),
            "dry_run would_archive must match apply archived count"
        );
    }

    #[test]
    fn test_lifecycle_run_rejects_importance_mult_below_one() {
        let ctx = test_ctx();
        let id = seed_lifecycle_memory(&ctx, "fresh important", 1.0, 0, 1, LifecycleState::Active);

        let result = lifecycle_run(
            &ctx,
            json!({"dry_run": false, "workspace": "default", "max_importance_mult": 0.5}),
        );

        assert!(
            result.get("error").is_some(),
            "max_importance_mult < 1.0 must be rejected, got: {result}"
        );
        // The fresh memory must NOT have been archived.
        assert_eq!(lifecycle_state(&ctx, id), "active", "{result}");
    }

    #[test]
    fn test_lifecycle_run_rejects_negative_days() {
        let ctx = test_ctx();
        let id = seed_lifecycle_memory(&ctx, "fresh memory", 0.0, 0, 1, LifecycleState::Active);

        let result = lifecycle_run(
            &ctx,
            json!({"dry_run": false, "workspace": "default", "archive_days": -1}),
        );

        assert!(
            result.get("error").is_some(),
            "negative archive_days must be rejected, got: {result}"
        );
        assert_eq!(lifecycle_state(&ctx, id), "active", "{result}");
    }

    #[test]
    fn test_lifecycle_run_rejects_wrong_type_param() {
        let ctx = test_ctx();
        let _id = seed_lifecycle_memory(&ctx, "fresh memory", 0.0, 0, 1, LifecycleState::Active);

        // stale_days as a string is a wrong-type input: it must be rejected,
        // not silently defaulted to 30.
        let result = lifecycle_run(
            &ctx,
            json!({"dry_run": false, "workspace": "default", "stale_days": "30"}),
        );

        assert!(
            result.get("error").is_some(),
            "wrong-typed stale_days must be rejected, got: {result}"
        );
    }

    #[test]
    fn test_lifecycle_run_rejects_stale_exceeding_archive() {
        let ctx = test_ctx();
        let _id = seed_lifecycle_memory(&ctx, "fresh memory", 0.0, 0, 1, LifecycleState::Active);

        let result = lifecycle_run(
            &ctx,
            json!({
                "dry_run": false,
                "workspace": "default",
                "stale_days": 120,
                "archive_days": 90
            }),
        );

        assert!(
            result.get("error").is_some(),
            "stale_days > archive_days must be rejected, got: {result}"
        );
    }

    #[test]
    fn test_lifecycle_config_rejects_invalid_params() {
        let ctx = test_ctx();
        let result = lifecycle_config(&ctx, json!({"max_importance_mult": 0.0}));
        assert!(
            result.get("error").is_some(),
            "lifecycle_config must reject invalid params, got: {result}"
        );
    }

    #[test]
    fn test_lifecycle_run_ages_sqlite_space_separated_timestamps() {
        let ctx = test_ctx();
        let id = ctx
            .storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (
                         content, memory_type, workspace, tier, importance,
                         access_count, lifecycle_state, created_at, updated_at, last_accessed_at
                     )
                     VALUES ('sqlite format memory', 'note', 'default', 'permanent', 0.0, 0, 'active',
                             '2020-01-01 00:00:00', '2020-01-01 00:00:00', '2020-01-01 00:00:00')",
                    [],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .expect("seed sqlite timestamp memory");

        let result = lifecycle_run(&ctx, json!({"dry_run": false, "workspace": "default"}));
        assert!(result.get("error").is_none(), "unexpected error: {result}");
        assert_eq!(result["archived"].as_i64(), Some(1), "{result}");
        assert_eq!(lifecycle_state(&ctx, id), "archived");
    }

    #[test]
    fn test_memory_set_lifecycle_records_workspace_and_rejects_purged() {
        let ctx = test_ctx();
        let id = ctx
            .storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, tier, importance, access_count, lifecycle_state)
                     VALUES ('scoped lifecycle row', 'note', 'tenant-lifecycle', 'permanent', 0.5, 0, 'active')",
                    [],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .expect("seed memory");

        let purged = memory_set_lifecycle(&ctx, json!({"id": id, "state": "purged"}));
        assert!(
            purged.get("error").is_some(),
            "purged is not a valid persisted lifecycle state: {purged}"
        );
        assert_eq!(lifecycle_state(&ctx, id), "active");

        let updated = memory_set_lifecycle(&ctx, json!({"id": id, "state": "archived"}));
        assert!(
            updated.get("error").is_none(),
            "unexpected error: {updated}"
        );
        assert_eq!(lifecycle_state(&ctx, id), "archived");

        let event_ws: Option<String> = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT workspace FROM enrichment_events
                     WHERE triggered_by = 'memory_set_lifecycle' AND memory_id = ?1
                     ORDER BY id DESC LIMIT 1",
                    [id],
                    |row| row.get(0),
                )?)
            })
            .expect("query enrichment_events workspace");
        assert_eq!(event_ws.as_deref(), Some("tenant-lifecycle"));
    }

    #[test]
    fn test_retention_policy_set_rejects_wrong_typed_json_fields() {
        let ctx = test_ctx();
        for bad in [
            json!({"workspace": "default", "max_age_days": "30"}),
            json!({"workspace": "default", "max_memories": "100"}),
            json!({"workspace": "default", "compress_max_importance": "0.5"}),
            json!({"workspace": "default", "exclude_types": "note"}),
            json!({"workspace": "default", "exclude_types": [123]}),
        ] {
            let res = retention_policy_set(&ctx, bad.clone());
            assert!(
                res.get("error").is_some(),
                "expected error for wrong-typed payload {bad}, got: {res}"
            );
        }
    }

    #[test]
    fn test_memory_lifecycle_update_decay_requires_integer_id_and_matching_workspace() {
        let ctx = test_ctx();
        let (ws_a_id, ws_b_id) = ctx
            .storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, tier, importance, access_count, lifecycle_state)
                     VALUES ('decay row a', 'note', 'ws-a', 'permanent', 0.8, 0, 'active')",
                    [],
                )?;
                let a = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, tier, importance, access_count, lifecycle_state)
                     VALUES ('decay row b', 'note', 'ws-b', 'permanent', 0.8, 0, 'active')",
                    [],
                )?;
                let b = conn.last_insert_rowid();
                Ok((a, b))
            })
            .expect("seed memories");
        let _ = crate::mcp::handlers::memory_policy::memory_score(
            &ctx,
            json!({"id": ws_a_id, "persist": true}),
        );
        let _ = crate::mcp::handlers::memory_policy::memory_score(
            &ctx,
            json!({"id": ws_b_id, "persist": true}),
        );
        let score_a_before: f64 = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT retention_score FROM memory_policy WHERE memory_id = ?1",
                    [ws_a_id],
                    |row| row.get(0),
                )?)
            })
            .expect("read initial ws-a retention_score");
        let score_b_before: f64 = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT retention_score FROM memory_policy WHERE memory_id = ?1",
                    [ws_b_id],
                    |row| row.get(0),
                )?)
            })
            .expect("read initial ws-b retention_score");

        for invalid in [
            json!({"action": "decay", "dry_run": false}),
            json!({"id": "999999", "action": "decay", "workspace": "ws-b", "dry_run": false}),
            json!({"id": ws_a_id, "action": "decay", "workspace": "ws-b", "dry_run": false}),
            json!({"id": ws_a_id, "action": "decay", "workspace": 42, "dry_run": false}),
        ] {
            let res = memory_lifecycle_update(&ctx, invalid.clone());
            assert!(
                res.get("error").is_some(),
                "expected error for invalid decay payload {invalid}, got: {res}"
            );
        }

        let valid_ok = memory_lifecycle_update(
            &ctx,
            json!({"id": ws_a_id, "action": "decay", "workspace": "WS-A", "dry_run": false}),
        );
        assert!(
            valid_ok.get("error").is_none(),
            "matching normalized workspace should succeed: {valid_ok}"
        );
        let score_a_after: f64 = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT retention_score FROM memory_policy WHERE memory_id = ?1",
                    [ws_a_id],
                    |row| row.get(0),
                )?)
            })
            .expect("read ws-a retention_score");
        assert!(
            score_a_after < score_a_before,
            "ws-a policy score should decay ({score_a_before} -> {score_a_after})"
        );

        let score_b_after: f64 = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT retention_score FROM memory_policy WHERE memory_id = ?1",
                    [ws_b_id],
                    |row| row.get(0),
                )?)
            })
            .expect("read ws-b retention_score");
        assert!(
            (score_b_after - score_b_before).abs() < 1e-6,
            "ws-b policy score must remain untouched ({score_b_before}), got {score_b_after}"
        );
    }
}
