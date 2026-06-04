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
    pub tier: String,
    pub created_at: String,
    pub updated_at: String,
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
pub fn export_memories(conn: &Connection) -> Result<ExportData> {
    let memories = list_memories(
        conn,
        &ListOptions {
            limit: Some(100000),
            ..Default::default()
        },
    )?;

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

        let input = CreateMemoryInput {
            content: mem.content.clone(),
            memory_type,
            tags: mem.tags.clone(),
            metadata: mem.metadata.clone(),
            importance: Some(mem.importance),
            scope: MemoryScope::Global,
            workspace: Some(mem.workspace.clone()),
            tier,
            defer_embedding: false,
            ttl_seconds: None,
            dedup_mode: if skip_duplicates {
                DedupMode::Skip
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
            Err(EngramError::Duplicate { .. }) => skipped += 1,
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
