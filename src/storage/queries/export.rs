use super::*;
use std::collections::HashMap;

/// Exported memory format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportedMemory {
    pub id: i64,
    pub content: String,
    pub memory_type: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub importance: f32,
    pub workspace: String,
    #[serde(default = "default_scope_type")]
    pub scope_type: String,
    #[serde(default)]
    pub scope_id: Option<String>,
    pub tier: String,
    pub created_at: String,
    pub updated_at: String,
}

fn default_scope_type() -> String {
    "global".to_string()
}

/// Export format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: String,
    pub memory_count: usize,
    pub memories: Vec<ExportedMemory>,
}

/// Export all memories to JSON-serializable format
pub fn export_memories(conn: &Connection, workspace: Option<&str>) -> Result<ExportData> {
    let list_options = ListOptions {
        limit: Some(100_000),
        workspace: workspace
            .map(crate::types::normalize_workspace)
            .transpose()
            .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {e}")))?,
        ..Default::default()
    };

    let memories = list_memories(conn, &list_options)?;

    let exported: Vec<ExportedMemory> = memories
        .into_iter()
        .map(|m| ExportedMemory {
            id: m.id,
            content: m.content,
            memory_type: m.memory_type.as_str().to_string(),
            tags: m.tags,
            metadata: m.metadata,
            importance: m.importance,
            workspace: m.workspace,
            scope_type: m.scope.scope_type().to_string(),
            scope_id: m.scope.scope_id().map(str::to_string),
            tier: m.tier.as_str().to_string(),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(ExportData {
        version: "1.0".to_string(),
        exported_at: Utc::now().to_rfc3339(),
        memory_count: exported.len(),
        memories: exported,
    })
}

/// Import result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Import memories from exported format
pub fn import_memories(
    conn: &Connection,
    data: &ExportData,
    skip_duplicates: bool,
) -> Result<ImportResult> {
    let mut imported = 0;
    let mut skipped = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for mem in &data.memories {
        let memory_type = mem.memory_type.parse().unwrap_or(MemoryType::Note);
        let tier = mem.tier.parse().unwrap_or(MemoryTier::Permanent);
        let scope = match mem.scope_type.as_str() {
            "global" | "" => MemoryScope::Global,
            "user" => match &mem.scope_id {
                Some(scope_id) if !scope_id.is_empty() => MemoryScope::User {
                    user_id: scope_id.clone(),
                },
                _ => {
                    failed += 1;
                    errors.push(format!(
                        "Failed to import memory {}: missing scope_id for scope_type=user",
                        mem.id
                    ));
                    continue;
                }
            },
            "session" => match &mem.scope_id {
                Some(scope_id) if !scope_id.is_empty() => MemoryScope::Session {
                    session_id: scope_id.clone(),
                },
                _ => {
                    failed += 1;
                    errors.push(format!(
                        "Failed to import memory {}: missing scope_id for scope_type=session",
                        mem.id
                    ));
                    continue;
                }
            },
            "agent" => match &mem.scope_id {
                Some(scope_id) if !scope_id.is_empty() => MemoryScope::Agent {
                    agent_id: scope_id.clone(),
                },
                _ => {
                    failed += 1;
                    errors.push(format!(
                        "Failed to import memory {}: missing scope_id for scope_type=agent",
                        mem.id
                    ));
                    continue;
                }
            },
            _ => {
                failed += 1;
                errors.push(format!(
                    "Failed to import memory {}: unknown scope_type {}",
                    mem.id, mem.scope_type
                ));
                continue;
            }
        };

        let input = CreateMemoryInput {
            content: mem.content.clone(),
            memory_type,
            tags: mem.tags.clone(),
            metadata: mem.metadata.clone(),
            importance: Some(mem.importance),
            scope,
            workspace: Some(mem.workspace.clone()),
            tier,
            defer_embedding: false,
            ttl_seconds: None,
            dedup_mode: if skip_duplicates {
                DedupMode::Reject
            } else {
                DedupMode::Allow
            },
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        };

        match create_memory(conn, &input) {
            Ok(_) => imported += 1,
            Err(EngramError::Duplicate { .. }) if skip_duplicates => skipped += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Failed to import memory {}: {}", mem.id, e));
            }
        }
    }

    Ok(ImportResult {
        imported,
        skipped,
        failed,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn create_test_memory(
        conn: &Connection,
        content: &str,
        workspace: &str,
        scope: MemoryScope,
    ) -> Result<()> {
        create_memory(
            conn,
            &CreateMemoryInput {
                content: content.to_string(),
                memory_type: MemoryType::Note,
                workspace: Some(workspace.to_string()),
                scope,
                ..Default::default()
            },
        )?;
        Ok(())
    }

    fn exported_memory(
        id: i64,
        content: &str,
        scope_type: &str,
        scope_id: Option<&str>,
    ) -> ExportedMemory {
        ExportedMemory {
            id,
            content: content.to_string(),
            memory_type: "note".to_string(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            importance: 0.5,
            workspace: "default".to_string(),
            scope_type: scope_type.to_string(),
            scope_id: scope_id.map(str::to_string),
            tier: "permanent".to_string(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn export_memories_filters_by_workspace() {
        let storage = Storage::open_in_memory().unwrap();
        storage
            .with_connection(|conn| {
                create_test_memory(conn, "default memory", "default", MemoryScope::Global)?;
                create_test_memory(conn, "team memory", "team", MemoryScope::Global)?;

                let export = export_memories(conn, Some("TEAM"))?;

                assert_eq!(export.memory_count, 1);
                assert_eq!(export.memories[0].content, "team memory");
                assert_eq!(export.memories[0].workspace, "team");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn import_memories_preserves_non_global_scope() {
        let data = ExportData {
            version: "1.0".to_string(),
            exported_at: Utc::now().to_rfc3339(),
            memory_count: 1,
            memories: vec![exported_memory(
                7,
                "session scoped",
                "session",
                Some("session-1"),
            )],
        };

        let storage = Storage::open_in_memory().unwrap();
        storage
            .with_connection(|conn| {
                let result = import_memories(conn, &data, true)?;
                assert_eq!(result.imported, 1);
                assert_eq!(result.failed, 0);

                let (scope_type, scope_id): (String, Option<String>) = conn.query_row(
                    "SELECT scope_type, scope_id FROM memories WHERE content = ?1",
                    ["session scoped"],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                assert_eq!(scope_type, "session");
                assert_eq!(scope_id.as_deref(), Some("session-1"));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn import_memories_rejects_scoped_memory_without_scope_id() {
        let data = ExportData {
            version: "1.0".to_string(),
            exported_at: Utc::now().to_rfc3339(),
            memory_count: 1,
            memories: vec![exported_memory(8, "bad scope", "user", None)],
        };

        let storage = Storage::open_in_memory().unwrap();
        storage
            .with_connection(|conn| {
                let result = import_memories(conn, &data, true)?;
                assert_eq!(result.imported, 0);
                assert_eq!(result.failed, 1);
                assert!(result.errors[0].contains("missing scope_id"));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn import_memories_counts_duplicates_as_skipped() {
        let data = ExportData {
            version: "1.0".to_string(),
            exported_at: Utc::now().to_rfc3339(),
            memory_count: 1,
            memories: vec![exported_memory(9, "duplicate", "global", None)],
        };

        let storage = Storage::open_in_memory().unwrap();
        storage
            .with_connection(|conn| {
                let first = import_memories(conn, &data, true)?;
                let second = import_memories(conn, &data, true)?;

                assert_eq!(first.imported, 1);
                assert_eq!(second.imported, 0);
                assert_eq!(second.skipped, 1);
                assert_eq!(second.failed, 0);
                Ok(())
            })
            .unwrap();
    }
}
