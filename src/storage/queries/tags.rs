use super::*;
use std::collections::HashMap;

/// Tag with usage count
#[derive(Debug, Clone, serde::Serialize)]
pub struct TagInfo {
    pub name: String,
    pub count: i64,
    pub last_used: Option<DateTime<Utc>>,
}

/// Get all tags with their usage counts
pub fn list_tags(conn: &Connection) -> Result<Vec<TagInfo>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT t.name, COUNT(mt.memory_id) as count,
               MAX(m.updated_at) as last_used
        FROM tags t
        LEFT JOIN memory_tags mt ON t.id = mt.tag_id
        LEFT JOIN memories m ON mt.memory_id = m.id AND m.valid_to IS NULL
        GROUP BY t.id, t.name
        ORDER BY count DESC, t.name ASC
        "#,
    )?;

    let tags: Vec<TagInfo> = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let last_used: Option<String> = row.get(2)?;

            Ok(TagInfo {
                name,
                count,
                last_used: last_used.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tags)
}

/// Tag hierarchy node
#[derive(Debug, Clone, serde::Serialize)]
pub struct TagHierarchyNode {
    pub name: String,
    pub full_path: String,
    pub count: i64,
    pub children: Vec<TagHierarchyNode>,
}

/// Build tag hierarchy from slash-separated tags (e.g., "project/engram/core")
pub fn get_tag_hierarchy(conn: &Connection) -> Result<Vec<TagHierarchyNode>> {
    let tags = list_tags(conn)?;

    // Build hierarchy from slash-separated paths
    let mut root_nodes: HashMap<String, TagHierarchyNode> = HashMap::new();

    for tag in tags {
        let parts: Vec<&str> = tag.name.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        let root_name = parts[0].to_string();
        if !root_nodes.contains_key(&root_name) {
            root_nodes.insert(
                root_name.clone(),
                TagHierarchyNode {
                    name: root_name.clone(),
                    full_path: root_name.clone(),
                    count: 0,
                    children: Vec::new(),
                },
            );
        }

        // Add count to appropriate level
        if parts.len() == 1 {
            if let Some(node) = root_nodes.get_mut(&root_name) {
                node.count += tag.count;
            }
        } else {
            // For nested tags, we'd need recursive building
            // For now, just add to root count
            if let Some(node) = root_nodes.get_mut(&root_name) {
                node.count += tag.count;
            }
        }
    }

    Ok(root_nodes.into_values().collect())
}

/// Tag validation result
#[derive(Debug, Clone, serde::Serialize)]
pub struct TagValidationResult {
    pub valid: bool,
    pub orphaned_tags: Vec<String>,
    pub empty_tags: Vec<String>,
    pub duplicate_assignments: Vec<(i64, String)>,
    pub total_tags: i64,
    pub total_assignments: i64,
}

/// Validate tag consistency
pub fn validate_tags(conn: &Connection) -> Result<TagValidationResult> {
    // Find orphaned tags (tags with no memories)
    let orphaned: Vec<String> = conn
        .prepare(
            "SELECT t.name FROM tags t
             LEFT JOIN memory_tags mt ON t.id = mt.tag_id
             WHERE mt.tag_id IS NULL",
        )?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Find empty tag names
    let empty: Vec<String> = conn
        .prepare("SELECT name FROM tags WHERE name = '' OR name IS NULL")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Count totals
    let total_tags: i64 = conn.query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;
    let total_assignments: i64 =
        conn.query_row("SELECT COUNT(*) FROM memory_tags", [], |row| row.get(0))?;

    Ok(TagValidationResult {
        valid: orphaned.is_empty() && empty.is_empty(),
        orphaned_tags: orphaned,
        empty_tags: empty,
        duplicate_assignments: vec![], // Would need more complex query
        total_tags,
        total_assignments,
    })
}
