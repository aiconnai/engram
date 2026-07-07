use super::*;

/// Parse a memory from a database row
pub fn memory_from_row(row: &Row) -> rusqlite::Result<Memory> {
    let id: i64 = row.get("id")?;
    let content: String = row.get("content")?;
    let memory_type_str: String = row.get("memory_type")?;
    let importance: f32 = row.get("importance")?;
    let access_count: i32 = row.get("access_count")?;
    let created_at: String = row.get("created_at")?;
    let updated_at: String = row.get("updated_at")?;
    let last_accessed_at: Option<String> = row.get("last_accessed_at")?;
    let owner_id: Option<String> = row.get("owner_id")?;
    let visibility_str: String = row.get("visibility")?;
    let version: i32 = row.get("version")?;
    let has_embedding: i32 = row.get("has_embedding")?;
    let metadata_str: String = row.get("metadata")?;

    // Scope columns (with fallback for backward compatibility)
    let scope_type: String = row
        .get("scope_type")
        .unwrap_or_else(|_| "global".to_string());
    let scope_id: Option<String> = row.get("scope_id").unwrap_or(None);

    // TTL column (with fallback for backward compatibility)
    let expires_at: Option<String> = row.get("expires_at").unwrap_or(None);

    // Content hash column (with fallback for backward compatibility)
    let content_hash: Option<String> = row.get("content_hash").unwrap_or(None);

    let memory_type = memory_type_str.parse().unwrap_or(MemoryType::Note);
    let visibility = match visibility_str.as_str() {
        "shared" => Visibility::Shared,
        "public" => Visibility::Public,
        _ => Visibility::Private,
    };

    // Parse scope from type and id
    let scope = match (scope_type.as_str(), scope_id) {
        ("user", Some(id)) => MemoryScope::User { user_id: id },
        ("session", Some(id)) => MemoryScope::Session { session_id: id },
        ("agent", Some(id)) => MemoryScope::Agent { agent_id: id },
        _ => MemoryScope::Global,
    };

    let metadata: HashMap<String, serde_json::Value> =
        serde_json::from_str(&metadata_str).unwrap_or_default();

    // Workspace column (with fallback for backward compatibility)
    let workspace: String = row
        .get("workspace")
        .unwrap_or_else(|_| "default".to_string());

    // Tier column (with fallback for backward compatibility)
    let tier_str: String = row.get("tier").unwrap_or_else(|_| "permanent".to_string());
    let tier = tier_str.parse().unwrap_or_default();

    let event_time: Option<String> = row.get("event_time").unwrap_or(None);
    let event_duration_seconds: Option<i64> = row.get("event_duration_seconds").unwrap_or(None);
    let trigger_pattern: Option<String> = row.get("trigger_pattern").unwrap_or(None);
    let procedure_success_count: i32 = row.get("procedure_success_count").unwrap_or(0);
    let procedure_failure_count: i32 = row.get("procedure_failure_count").unwrap_or(0);
    let summary_of_id: Option<i64> = row.get("summary_of_id").unwrap_or(None);
    let lifecycle_state_str: Option<String> = row.get("lifecycle_state").unwrap_or(None);

    let lifecycle_state = lifecycle_state_str
        .and_then(|s| s.parse().ok())
        .unwrap_or(crate::types::LifecycleState::Active);

    // media_url column (additive, nullable — with fallback for older schema versions)
    let media_url: Option<String> = row.get("media_url").unwrap_or(None);

    Ok(Memory {
        id,
        content,
        memory_type,
        tags: vec![], // Loaded separately
        metadata,
        importance,
        access_count,
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        last_accessed_at: last_accessed_at.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        owner_id,
        visibility,
        scope,
        workspace,
        tier,
        version,
        has_embedding: has_embedding != 0,
        expires_at: expires_at.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        content_hash,
        event_time: event_time.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        event_duration_seconds,
        trigger_pattern,
        procedure_success_count,
        procedure_failure_count,
        summary_of_id,
        lifecycle_state,
        media_url,
    })
}
