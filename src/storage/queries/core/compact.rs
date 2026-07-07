use super::*;

/// A compact memory representation for efficient list views.
/// Contains only essential fields and a truncated content preview.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompactMemoryRow {
    /// Memory ID
    pub id: i64,
    /// Content preview (first line or N chars)
    pub preview: String,
    /// Whether content was truncated
    pub truncated: bool,
    /// Memory type
    pub memory_type: MemoryType,
    /// Tags
    pub tags: Vec<String>,
    /// Importance score
    pub importance: f32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Workspace name
    pub workspace: String,
    /// Memory tier
    pub tier: MemoryTier,
    /// Original content length in chars
    pub content_length: usize,
    /// Number of lines in original content
    pub line_count: usize,
}

/// List memories in compact format with preview only.
///
/// This is more efficient than `list_memories` when you don't need full content,
/// such as for browsing/listing UIs.
///
/// # Arguments
/// * `conn` - Database connection
/// * `options` - List filtering/pagination options
/// * `preview_chars` - Max chars for preview (default: 100)
pub fn list_memories_compact(
    conn: &Connection,
    options: &ListOptions,
    preview_chars: Option<usize>,
) -> Result<Vec<CompactMemoryRow>> {
    use crate::intelligence::compact_preview;

    let now = Utc::now().to_rfc3339();
    let max_preview = preview_chars.unwrap_or(100);

    let mut sql = String::from(
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance,
                m.created_at, m.updated_at, m.workspace, m.tier
         FROM memories m",
    );

    let mut conditions = vec!["m.valid_to IS NULL".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    // Exclude expired memories
    conditions.push("(m.expires_at IS NULL OR m.expires_at > ?)".to_string());
    params.push(Box::new(now));

    // Tag filter (requires join)
    if let Some(ref tags) = options.tags {
        if !tags.is_empty() {
            sql.push_str(
                " JOIN memory_tags mt ON m.id = mt.memory_id
                  JOIN tags t ON mt.tag_id = t.id",
            );
            let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
            conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
            for tag in tags {
                params.push(Box::new(tag.clone()));
            }
        }
    }

    // Type filter
    if let Some(ref memory_type) = options.memory_type {
        conditions.push("m.memory_type = ?".to_string());
        params.push(Box::new(memory_type.as_str().to_string()));
    }

    // Metadata filter (JSON)
    if let Some(ref metadata_filter) = options.metadata_filter {
        for (key, value) in metadata_filter {
            metadata_value_to_param(key, value, &mut conditions, &mut params)?;
        }
    }

    // Scope filter
    if let Some(ref scope) = options.scope {
        conditions.push("m.scope_type = ?".to_string());
        params.push(Box::new(scope.scope_type().to_string()));
        if let Some(scope_id) = scope.scope_id() {
            conditions.push("m.scope_id = ?".to_string());
            params.push(Box::new(scope_id.to_string()));
        } else {
            conditions.push("m.scope_id IS NULL".to_string());
        }
    }

    // Workspace filter
    if let Some(ref workspace) = options.workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(workspace.clone()));
    }

    // Tier filter
    if let Some(ref tier) = options.tier {
        conditions.push("m.tier = ?".to_string());
        params.push(Box::new(tier.as_str().to_string()));
    }

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));

    // Sorting
    let sort_field = match options.sort_by.unwrap_or_default() {
        SortField::CreatedAt => "m.created_at",
        SortField::UpdatedAt => "m.updated_at",
        SortField::LastAccessedAt => "m.last_accessed_at",
        SortField::Importance => "m.importance",
        SortField::AccessCount => "m.access_count",
    };
    let sort_order = match options.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_field, sort_order));

    // Pagination
    let limit = options.limit.unwrap_or(100);
    let offset = options.offset.unwrap_or(0);
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<CompactMemoryRow> = stmt
        .query_map(param_refs.as_slice(), |row| {
            let id: i64 = row.get("id")?;
            let content: String = row.get("content")?;
            let memory_type_str: String = row.get("memory_type")?;
            let importance: f32 = row.get("importance")?;
            let created_at_str: String = row.get("created_at")?;
            let updated_at_str: String = row.get("updated_at")?;
            let workspace: String = row.get("workspace")?;
            let tier_str: String = row.get("tier")?;

            let memory_type = memory_type_str.parse().unwrap_or(MemoryType::Note);
            let tier = tier_str.parse().unwrap_or_default();

            // Generate compact preview
            let (preview, truncated) = compact_preview(&content, max_preview);
            let content_length = content.len();
            let line_count = content.lines().count();

            Ok(CompactMemoryRow {
                id,
                preview,
                truncated,
                memory_type,
                tags: vec![], // Will be loaded separately
                importance,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                workspace,
                tier,
                content_length,
                line_count,
            })
        })?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}
