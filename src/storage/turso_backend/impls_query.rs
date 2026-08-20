//! List/count/search method bodies for `impl StorageBackend for TursoBackend`.
//!
//! Split out of `impls.rs` (ENG storage split) to keep files under the
//! repository's line-count limit. These are `pub(super)` free functions
//! called directly from the trait method stubs in `impls.rs`; behavior is
//! unchanged from the original single-file implementation.

use super::core::{TursoBackend, MEMORY_COLUMNS};
use crate::error::{EngramError, Result};
use crate::storage::backend::StorageBackend;
use crate::types::{
    ListOptions, MatchInfo, Memory, SearchOptions, SearchResult, SearchStrategy, SortField,
    SortOrder,
};

pub(super) fn list_memories(backend: &TursoBackend, options: ListOptions) -> Result<Vec<Memory>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            let mut sql = format!(
                "SELECT {} FROM memories WHERE valid_to IS NULL",
                MEMORY_COLUMNS
            );
            let mut params: Vec<libsql::Value> = Vec::new();

            if let Some(ref workspace) = options.workspace {
                sql.push_str(" AND workspace = ?");
                params.push(libsql::Value::Text(workspace.clone()));
            } else if let Some(ref workspaces) = options.workspaces {
                if !workspaces.is_empty() {
                    let placeholders = vec!["?"; workspaces.len()].join(", ");
                    sql.push_str(&format!(" AND workspace IN ({})", placeholders));
                    for workspace in workspaces {
                        params.push(libsql::Value::Text(workspace.clone()));
                    }
                }
            }

            if let Some(ref scope) = options.scope {
                sql.push_str(" AND scope_type = ?");
                params.push(libsql::Value::Text(scope.scope_type().to_string()));
                if let Some(scope_id) = scope.scope_id() {
                    sql.push_str(" AND scope_id = ?");
                    params.push(libsql::Value::Text(scope_id.to_string()));
                } else {
                    sql.push_str(" AND scope_id IS NULL");
                }
            }

            if let Some(ref memory_type) = options.memory_type {
                sql.push_str(" AND memory_type = ?");
                params.push(libsql::Value::Text(memory_type.as_str().to_string()));
            }

            if let Some(ref tier) = options.tier {
                sql.push_str(" AND tier = ?");
                params.push(libsql::Value::Text(tier.as_str().to_string()));
            }

            if let Some(ref tags) = options.tags {
                if !tags.is_empty() {
                    let placeholders = vec!["?"; tags.len()].join(", ");
                    sql.push_str(&format!(
                        " AND id IN (
                        SELECT mt.memory_id
                        FROM memory_tags mt
                        JOIN tags t ON t.id = mt.tag_id
                        WHERE t.name IN ({})
                        GROUP BY mt.memory_id
                        HAVING COUNT(DISTINCT t.name) = ?
                    )",
                        placeholders
                    ));
                    for tag in tags {
                        params.push(libsql::Value::Text(tag.clone()));
                    }
                    params.push(libsql::Value::Integer(tags.len() as i64));
                }
            }

            if !options.include_archived {
                sql.push_str(" AND (lifecycle_state IS NULL OR lifecycle_state != 'archived')");
            }

            let sort_field = options.sort_by.unwrap_or(SortField::CreatedAt);
            let sort_order = options.sort_order.unwrap_or(SortOrder::Desc);
            let sort_column = match sort_field {
                SortField::CreatedAt => "created_at",
                SortField::UpdatedAt => "updated_at",
                SortField::LastAccessedAt => "last_accessed_at",
                SortField::Importance => "importance",
                SortField::AccessCount => "access_count",
            };
            let sort_dir = match sort_order {
                SortOrder::Asc => "ASC",
                SortOrder::Desc => "DESC",
            };
            sql.push_str(&format!(" ORDER BY {} {}", sort_column, sort_dir));

            if let Some(limit_val) = options.limit {
                sql.push_str(" LIMIT ");
                sql.push_str(&limit_val.to_string());
            }

            if let Some(offset_val) = options.offset {
                sql.push_str(" OFFSET ");
                sql.push_str(&offset_val.to_string());
            }

            backend.query_memories(&sql, params).await
        })
    })
}

pub(super) fn count_memories(backend: &TursoBackend, options: ListOptions) -> Result<i64> {
    let mut options = options;
    options.limit = None;
    options.offset = None;
    let memories = backend.list_memories(options)?;
    Ok(memories.len() as i64)
}

pub(super) fn search_memories(
    backend: &TursoBackend,
    query: &str,
    options: SearchOptions,
) -> Result<Vec<SearchResult>> {
    let rt = tokio::runtime::Handle::try_current()
        .map_err(|_| EngramError::Storage("No tokio runtime available".to_string()))?;
    let search_query = query.to_string();

    tokio::task::block_in_place(|| {
        rt.block_on(async {
            // Simple LIKE-based search (full hybrid search would need vector support)
            let mut sql = format!(
                "SELECT {} FROM memories WHERE valid_to IS NULL AND content LIKE ?",
                MEMORY_COLUMNS
            );
            let mut params = vec![libsql::Value::Text(format!("%{}%", search_query))];

            if !options.include_archived {
                sql.push_str(" AND (lifecycle_state IS NULL OR lifecycle_state != 'archived')");
            }

            if let Some(ref workspace) = options.workspace {
                sql.push_str(" AND workspace = ?");
                params.push(libsql::Value::Text(workspace.clone()));
            } else if let Some(ref workspaces) = options.workspaces {
                if !workspaces.is_empty() {
                    let placeholders = vec!["?"; workspaces.len()].join(", ");
                    sql.push_str(&format!(" AND workspace IN ({})", placeholders));
                    for workspace in workspaces {
                        params.push(libsql::Value::Text(workspace.clone()));
                    }
                }
            }

            if let Some(ref scope) = options.scope {
                sql.push_str(" AND scope_type = ?");
                params.push(libsql::Value::Text(scope.scope_type().to_string()));
                if let Some(scope_id) = scope.scope_id() {
                    sql.push_str(" AND scope_id = ?");
                    params.push(libsql::Value::Text(scope_id.to_string()));
                } else {
                    sql.push_str(" AND scope_id IS NULL");
                }
            }

            if let Some(ref memory_type) = options.memory_type {
                sql.push_str(" AND memory_type = ?");
                params.push(libsql::Value::Text(memory_type.as_str().to_string()));
            } else if !options.include_transcripts {
                sql.push_str(" AND memory_type != 'transcript_chunk'");
            }

            if let Some(ref tier) = options.tier {
                sql.push_str(" AND tier = ?");
                params.push(libsql::Value::Text(tier.as_str().to_string()));
            }

            sql.push_str(" ORDER BY importance DESC");
            if let Some(limit_val) = options.limit {
                sql.push_str(" LIMIT ");
                sql.push_str(&limit_val.to_string());
            } else {
                sql.push_str(" LIMIT 20");
            }

            let memories = backend.query_memories(&sql, params).await?;

            Ok(memories
                .into_iter()
                .map(|memory| SearchResult {
                    memory,
                    score: 1.0,
                    match_info: MatchInfo {
                        strategy: SearchStrategy::KeywordOnly,
                        matched_terms: vec![search_query.clone()],
                        highlights: Vec::new(),
                        semantic_score: None,
                        keyword_score: Some(1.0),
                    },
                })
                .collect())
        })
    })
}
