use super::*;

/// Update a memory
pub fn update_memory(conn: &Connection, id: i64, input: &UpdateMemoryInput) -> Result<Memory> {
    // Get current memory for versioning
    let current = get_memory_internal(conn, id, false)?;
    let now = Utc::now().to_rfc3339();

    // Build update query dynamically
    let mut updates = vec!["updated_at = ?".to_string()];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];

    if let Some(ref content) = input.content {
        updates.push("content = ?".to_string());
        values.push(Box::new(content.clone()));
        // Recalculate dedup hash when content changes
        // (dedup uses normalized hash so case/whitespace-only edits don't create duplicates)
        let new_hash = compute_dedup_hash(content);
        updates.push("content_hash = ?".to_string());
        values.push(Box::new(new_hash));
    }

    if let Some(ref memory_type) = input.memory_type {
        updates.push("memory_type = ?".to_string());
        values.push(Box::new(memory_type.as_str().to_string()));
    }

    if let Some(importance) = input.importance {
        updates.push("importance = ?".to_string());
        values.push(Box::new(importance));
    }

    if let Some(ref metadata) = input.metadata {
        let metadata_json = serde_json::to_string(metadata)?;
        updates.push("metadata = ?".to_string());
        values.push(Box::new(metadata_json));
    }

    if let Some(ref scope) = input.scope {
        updates.push("scope_type = ?".to_string());
        values.push(Box::new(scope.scope_type().to_string()));
        updates.push("scope_id = ?".to_string());
        values.push(Box::new(scope.scope_id().map(|s| s.to_string())));
    }

    // Update event_time if provided (Some(None) clears)
    if let Some(event_time) = &input.event_time {
        updates.push("event_time = ?".to_string());
        let value = event_time.as_ref().map(|dt| dt.to_rfc3339());
        values.push(Box::new(value));
    }

    // Update trigger_pattern if provided (Some(None) clears)
    if let Some(trigger_pattern) = &input.trigger_pattern {
        updates.push("trigger_pattern = ?".to_string());
        values.push(Box::new(trigger_pattern.clone()));
    }

    // Update media_url if provided (Some(None) clears, Some(Some(url)) sets)
    if let Some(media_url) = &input.media_url {
        updates.push("media_url = ?".to_string());
        values.push(Box::new(media_url.clone()));
    }

    // Handle TTL update with tier invariant enforcement
    // Normalize: ttl_seconds <= 0 means "no expiration" (consistent with create_memory)
    // Invariants:
    //   - Permanent tier: expires_at MUST be NULL
    //   - Daily tier: expires_at MUST be set
    if let Some(ttl) = input.ttl_seconds {
        if ttl <= 0 {
            // Request to remove expiration
            // Only allowed for Permanent tier; for Daily tier, this is an error
            if current.tier == MemoryTier::Daily {
                return Err(crate::error::EngramError::InvalidInput(
                    "Cannot remove expiration from a Daily tier memory. Use promote_to_permanent first.".to_string()
                ));
            }
            updates.push("expires_at = NULL".to_string());
        } else {
            // Request to set expiration
            // Only allowed for Daily tier; for Permanent tier, this is an error
            if current.tier == MemoryTier::Permanent {
                return Err(crate::error::EngramError::InvalidInput(
                    "Cannot set expiration on a Permanent tier memory. Permanent memories cannot expire.".to_string()
                ));
            }
            let expires_at = (Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339();
            updates.push("expires_at = ?".to_string());
            values.push(Box::new(expires_at));
        }
    }

    // Increment version
    updates.push("version = version + 1".to_string());

    // Execute update
    let sql = format!("UPDATE memories SET {} WHERE id = ?", updates.join(", "));
    values.push(Box::new(id));

    let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    conn.execute(&sql, params.as_slice())?;

    // Update tags if provided
    if let Some(ref tags) = input.tags {
        conn.execute("DELETE FROM memory_tags WHERE memory_id = ?", params![id])?;
        for tag in tags {
            ensure_tag(conn, tag)?;
            conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag_id)
                 SELECT ?, id FROM tags WHERE name = ?",
                params![id, tag],
            )?;
        }
    }

    // Create new version
    let new_content = input.content.as_ref().unwrap_or(&current.content);
    let new_tags = input.tags.as_ref().unwrap_or(&current.tags);
    let new_metadata = input.metadata.as_ref().unwrap_or(&current.metadata);
    let tags_json = serde_json::to_string(new_tags)?;
    let metadata_json = serde_json::to_string(new_metadata)?;

    conn.execute(
        "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
         VALUES (?, (SELECT version FROM memories WHERE id = ?), ?, ?, ?, ?)",
        params![id, id, new_content, tags_json, metadata_json, now],
    )?;

    // Re-queue for embedding if content changed
    if input.content.is_some() {
        conn.execute(
            "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at)
             VALUES (?, 'pending', ?)",
            params![id, now],
        )?;
        conn.execute(
            "UPDATE memories SET has_embedding = 0 WHERE id = ?",
            params![id],
        )?;
    }

    // Build list of changed fields for event data
    let mut changed_fields = Vec::new();
    if input.content.is_some() {
        changed_fields.push("content");
    }
    if input.tags.is_some() {
        changed_fields.push("tags");
    }
    if input.metadata.is_some() {
        changed_fields.push("metadata");
    }
    if input.importance.is_some() {
        changed_fields.push("importance");
    }
    if input.ttl_seconds.is_some() {
        changed_fields.push("ttl");
    }

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": changed_fields,
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    get_memory_internal(conn, id, false)
}
