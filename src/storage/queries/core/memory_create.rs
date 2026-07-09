use super::*;

/// Create a new memory with deduplication support
pub fn create_memory(conn: &Connection, input: &CreateMemoryInput) -> Result<Memory> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let metadata_json = serde_json::to_string(&input.metadata)?;
    let importance = input.importance.unwrap_or(0.5);

    // Compute dedup hash for duplicate detection (normalized: lowercased, whitespace-collapsed)
    // This is also the stored `content_hash`, so import/export must use the same function.
    let content_hash = compute_content_hash(&input.content);

    // Normalize workspace early for dedup checking
    let workspace = match &input.workspace {
        Some(ws) => crate::types::normalize_workspace(ws)
            .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?,
        None => "default".to_string(),
    };

    // Check for duplicates based on dedup_mode (scoped to same scope AND workspace)
    if input.dedup_mode != DedupMode::Allow {
        if let Some(existing) =
            find_by_content_hash(conn, &content_hash, &input.scope, Some(&workspace))?
        {
            match input.dedup_mode {
                DedupMode::Reject => {
                    return Err(EngramError::Duplicate {
                        existing_id: existing.id,
                        message: format!(
                            "Duplicate memory detected (id={}). Content hash: {}",
                            existing.id, content_hash
                        ),
                    });
                }
                DedupMode::Skip => {
                    // Return existing memory without modification
                    return Ok(existing);
                }
                DedupMode::Merge => {
                    // Merge: update existing memory with new tags and metadata
                    let mut merged_tags = existing.tags.clone();
                    for tag in &input.tags {
                        if !merged_tags.contains(tag) {
                            merged_tags.push(tag.clone());
                        }
                    }

                    let mut merged_metadata = existing.metadata.clone();
                    for (key, value) in &input.metadata {
                        merged_metadata.insert(key.clone(), value.clone());
                    }

                    let update_input = UpdateMemoryInput {
                        content: None, // Keep existing content
                        memory_type: None,
                        tags: Some(merged_tags),
                        metadata: Some(merged_metadata),
                        importance: input.importance, // Use new importance if provided
                        scope: None,
                        ttl_seconds: input.ttl_seconds, // Apply new TTL if provided
                        event_time: None,
                        trigger_pattern: None,
                        media_url: input.media_url.clone().map(Some),
                    };

                    return update_memory(conn, existing.id, &update_input);
                }
                DedupMode::Allow => unreachable!(),
            }
        }
    }

    // Extract scope type and id for database storage
    let scope_type = input.scope.scope_type();
    let scope_id = input.scope.scope_id().map(|s| s.to_string());

    // workspace was already normalized above for dedup checking

    // Determine tier and enforce tier invariants
    let tier = input.tier;

    // Calculate expires_at based on tier and ttl_seconds
    // Tier invariants:
    //   - Permanent: expires_at MUST be NULL (cannot expire)
    //   - Daily: expires_at MUST be set (default: created_at + 24h)
    let expires_at = match tier {
        MemoryTier::Permanent => {
            // Permanent memories cannot have an expiration
            if input.ttl_seconds.is_some() && input.ttl_seconds != Some(0) {
                return Err(EngramError::InvalidInput(
                    "Permanent tier memories cannot have a TTL. Use Daily tier for expiring memories.".to_string()
                ));
            }
            None
        }
        MemoryTier::Daily => {
            // Daily memories must have an expiration (default: 24 hours)
            let ttl = input.ttl_seconds.filter(|&t| t > 0).unwrap_or(86400); // 24h default
            Some((now + chrono::Duration::seconds(ttl)).to_rfc3339())
        }
    };

    let event_time = input.event_time.map(|dt| dt.to_rfc3339());

    conn.execute(
        "INSERT INTO memories (content, memory_type, importance, metadata, created_at, updated_at, valid_from, scope_type, scope_id, workspace, tier, expires_at, content_hash, event_time, event_duration_seconds, trigger_pattern, summary_of_id, media_url)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            input.content,
            input.memory_type.as_str(),
            importance,
            metadata_json,
            now_str,
            now_str,
            now_str,
            scope_type,
            scope_id,
            workspace,
            tier.as_str(),
            expires_at,
            content_hash,
            event_time,
            input.event_duration_seconds,
            input.trigger_pattern,
            input.summary_of_id,
            input.media_url,
        ],
    )?;

    let id = conn.last_insert_rowid();

    // Insert tags
    for tag in &input.tags {
        ensure_tag(conn, tag)?;
        conn.execute(
            "INSERT OR IGNORE INTO memory_tags (memory_id, tag_id)
             SELECT ?, id FROM tags WHERE name = ?",
            params![id, tag],
        )?;
    }

    // Queue for embedding if not deferred
    if !input.defer_embedding {
        conn.execute(
            "INSERT INTO embedding_queue (memory_id, status, queued_at)
             VALUES (?, 'pending', ?)",
            params![id, now_str],
        )?;
    }

    // Create initial version
    let tags_json = serde_json::to_string(&input.tags)?;
    conn.execute(
        "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
         VALUES (?, 1, ?, ?, ?, ?)",
        params![id, input.content, tags_json, metadata_json, now_str],
    )?;

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Created,
        Some(id),
        None,
        serde_json::json!({
            "workspace": input.workspace.as_deref().unwrap_or("default"),
            "memory_type": input.memory_type.as_str(),
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    let memory = get_memory_internal(conn, id, false)?;
    if let Err(e) = initialize_memory_policy(conn, &memory) {
        tracing::warn!(
            target = "engram::storage::memory_policy",
            memory_id = memory.id,
            error = %e,
            "failed to initialize memory policy metadata; continuing"
        );
    }

    Ok(memory)
}

fn initialize_memory_policy(conn: &Connection, memory: &Memory) -> Result<()> {
    let features = extract_features(PolicyFeatureInput {
        memory,
        existing_policy: None,
        event: None,
        hybrid_search_score: None,
        session_relevance: None,
    });
    let score = score_policy(&features);
    let policy = upsert_policy_record(
        conn,
        PolicyRecordInput {
            memory_id: memory.id,
            salience_score: score.salience_score,
            retention_score: score.retention_score,
            retrieval_priority: score.retrieval_priority,
            policy_version: score.policy_version,
            policy_reason: score.policy_reason,
        },
    )?;
    emit_policy_event(conn, "create_memory", &policy, false);
    Ok(())
}
