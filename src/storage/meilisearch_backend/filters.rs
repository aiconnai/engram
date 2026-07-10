use crate::error::EngramError;
use crate::types::{ListOptions, MemoryScope, SearchOptions, SortField, SortOrder};

pub(super) fn escape_filter_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn build_tags_filter(tags: &[String]) -> Option<String> {
    if tags.is_empty() {
        return None;
    }
    let clauses: Vec<String> = tags
        .iter()
        .map(|tag| format!("tags = \"{}\"", escape_filter_value(tag)))
        .collect();
    Some(clauses.join(" AND "))
}

pub(super) fn build_workspace_filter(workspaces: &[String]) -> Option<String> {
    if workspaces.is_empty() {
        return None;
    }
    if workspaces.len() == 1 {
        return Some(format!(
            "workspace = \"{}\"",
            escape_filter_value(&workspaces[0])
        ));
    }
    let values: Vec<String> = workspaces
        .iter()
        .map(|w| format!("\"{}\"", escape_filter_value(w)))
        .collect();
    Some(format!("workspace IN [{}]", values.join(", ")))
}

pub(super) fn build_scope_filter(scope: &MemoryScope) -> Vec<String> {
    let mut parts = Vec::new();
    parts.push(format!("scope = \"{}\"", scope.scope_type()));
    match scope.scope_id() {
        Some(id) => parts.push(format!("scope_id = \"{}\"", escape_filter_value(id))),
        None => parts.push("scope_id IS NULL".to_string()),
    }
    parts
}

pub(super) fn build_filter_from_search_options(
    options: &SearchOptions,
) -> Result<Option<String>, EngramError> {
    if options.filter.is_some() {
        return Err(EngramError::InvalidInput(
            "Advanced filter expressions are not supported by the Meilisearch backend.".to_string(),
        ));
    }

    let mut clauses = Vec::new();

    if let Some(scope) = &options.scope {
        clauses.extend(build_scope_filter(scope));
    }

    if let Some(memory_type) = &options.memory_type {
        clauses.push(format!(
            "memory_type = \"{}\"",
            escape_filter_value(memory_type.as_str())
        ));
    } else if !options.include_transcripts {
        clauses.push("memory_type != \"transcript_chunk\"".to_string());
    }

    if let Some(tier) = &options.tier {
        clauses.push(format!("tier = \"{}\"", escape_filter_value(tier.as_str())));
    }

    if !options.include_archived {
        clauses.push("lifecycle_state != \"archived\"".to_string());
    }

    if let Some(tags) = &options.tags {
        if let Some(tag_clause) = build_tags_filter(tags) {
            clauses.push(tag_clause);
        }
    }

    let workspaces = if let Some(workspace) = &options.workspace {
        vec![workspace.clone()]
    } else {
        options.workspaces.clone().unwrap_or_default()
    };
    if let Some(workspace_clause) = build_workspace_filter(&workspaces) {
        clauses.push(workspace_clause);
    }

    Ok(if clauses.is_empty() {
        None
    } else {
        Some(clauses.join(" AND "))
    })
}

pub(super) fn build_filter_from_list_options(
    options: &ListOptions,
) -> Result<Option<String>, EngramError> {
    if options.filter.is_some() || options.metadata_filter.is_some() {
        return Err(EngramError::InvalidInput(
            "Metadata/advanced filters are not supported by the Meilisearch backend.".to_string(),
        ));
    }

    let mut clauses = Vec::new();

    if let Some(scope) = &options.scope {
        clauses.extend(build_scope_filter(scope));
    }

    if let Some(memory_type) = &options.memory_type {
        clauses.push(format!(
            "memory_type = \"{}\"",
            escape_filter_value(memory_type.as_str())
        ));
    }

    if let Some(tier) = &options.tier {
        clauses.push(format!("tier = \"{}\"", escape_filter_value(tier.as_str())));
    }

    if !options.include_archived {
        clauses.push("lifecycle_state != \"archived\"".to_string());
    }

    if let Some(tags) = &options.tags {
        if let Some(tag_clause) = build_tags_filter(tags) {
            clauses.push(tag_clause);
        }
    }

    let workspaces = if let Some(workspace) = &options.workspace {
        vec![workspace.clone()]
    } else {
        options.workspaces.clone().unwrap_or_default()
    };
    if let Some(workspace_clause) = build_workspace_filter(&workspaces) {
        clauses.push(workspace_clause);
    }

    Ok(if clauses.is_empty() {
        None
    } else {
        Some(clauses.join(" AND "))
    })
}

pub(super) fn sort_to_meili(sort_by: SortField, sort_order: SortOrder) -> String {
    let field = match sort_by {
        SortField::CreatedAt => "created_at",
        SortField::UpdatedAt => "updated_at",
        SortField::LastAccessedAt => "last_accessed_at",
        SortField::Importance => "importance",
        SortField::AccessCount => "access_count",
    };
    let order = match sort_order {
        SortOrder::Asc => "asc",
        SortOrder::Desc => "desc",
    };
    format!("{}:{}", field, order)
}
