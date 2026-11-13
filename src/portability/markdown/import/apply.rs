use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};

use super::files::MarkdownSource;
use super::status::{classify_import_status, ImportStatus};
use crate::auth::access::ensure_memory_access;
use crate::error::{EngramError, Result};
use crate::portability::markdown::frontmatter::{
    extract_body, frontmatter_tags, import_payload, parse_frontmatter,
};
use crate::portability::markdown::MarkdownContext;
use crate::storage::queries::{compute_content_hash, compute_content_hash_raw};
use crate::types::{CreateMemoryInput, UpdateMemoryInput};

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub(crate) struct ImportFailure {
    error: EngramError,
    // Storage can fail at COMMIT or in permission revalidation after COMMIT.
    // A completed callback alone cannot prove either rollback or persistence.
    attempted: Option<u64>,
}

impl From<EngramError> for ImportFailure {
    fn from(error: EngramError) -> Self {
        Self {
            error,
            attempted: None,
        }
    }
}

impl ImportFailure {
    pub(crate) fn into_parts(self) -> (EngramError, Option<u64>) {
        (self.error, self.attempted)
    }
}

/// Complete domain result; each adapter chooses its public representation.
#[derive(Debug, Serialize)]
pub(crate) struct ImportOutcome {
    pub scanned: usize,
    pub in_sync: usize,
    pub new: usize,
    pub pending: usize,
    pub conflict: usize,
    pub applied: usize,
    pub dry_run: bool,
    pub files: Vec<ImportFile>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportFile {
    pub file: String,
    pub engram_id: Option<i64>,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

struct Input {
    file: String,
    id: Option<i64>,
    version: i64,
    fm: HashMap<String, String>,
    body: String,
}

enum Mutation {
    None,
    Create(CreateMemoryInput),
    Update(i64, UpdateMemoryInput),
}

struct Plan {
    file: String,
    id: Option<i64>,
    status: ImportStatus,
    mutation: Mutation,
}

fn positive_field(fm: &HashMap<String, String>, field: &str) -> Result<Option<i64>> {
    fm.get(field)
        .map(|value| {
            value
                .parse::<i64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    EngramError::InvalidInput(format!("{field} must be a positive integer"))
                })
        })
        .transpose()
}

fn prepare(sources: Vec<MarkdownSource>) -> Result<Vec<Input>> {
    let mut ids = std::collections::BTreeSet::new();
    sources
        .into_iter()
        .map(|source| {
            let fm = parse_frontmatter(&source.raw);
            let id = positive_field(&fm, "engram_id")?;
            if id.is_some_and(|id| !ids.insert(id)) {
                return Err(EngramError::InvalidInput(
                    "Duplicate engram_id in Markdown input".into(),
                ));
            }
            Ok(Input {
                file: source
                    .relative
                    .to_str()
                    .ok_or_else(|| {
                        EngramError::InvalidInput("Markdown input names must be UTF-8".into())
                    })?
                    .to_string(),
                id,
                version: positive_field(&fm, "engram_version")?.unwrap_or(1),
                body: extract_body(&source.raw).trim().to_string(),
                fm,
            })
        })
        .collect()
}

fn creation(
    ctx: &MarkdownContext<'_>,
    input: &Input,
    workspace: Option<&str>,
) -> Result<CreateMemoryInput> {
    let mut obj = import_payload(&input.fm, &input.body, true);
    if let Some(workspace) = workspace {
        obj.insert("workspace".into(), json!(workspace));
    }
    obj.insert("tags".into(), json!(frontmatter_tags(&input.fm)));
    let mut parsed: CreateMemoryInput = serde_json::from_value(Value::Object(obj))
        .map_err(|_| EngramError::InvalidInput("Invalid Markdown creation fields".into()))?;
    let workspace =
        crate::types::normalize_workspace(parsed.workspace.as_deref().unwrap_or("default"))
            .map_err(|error| EngramError::InvalidInput(error.to_string()))?;
    if ctx
        .principal
        .as_ref()
        .is_some_and(|principal| !principal.allows_workspace(Some(&workspace)))
    {
        return Err(EngramError::Unauthorized(
            "Markdown target workspace is not allowed".into(),
        ));
    }
    parsed.workspace = Some(workspace);
    Ok(parsed)
}

fn update(input: &Input) -> Result<UpdateMemoryInput> {
    let mut obj = import_payload(&input.fm, &input.body, false);
    if input.fm.contains_key("engram_tags_list") {
        obj.insert("tags".into(), json!(frontmatter_tags(&input.fm)));
    }
    serde_json::from_value(Value::Object(obj))
        .map_err(|_| EngramError::InvalidInput("Invalid Markdown update fields".into()))
}

fn plan(
    ctx: &MarkdownContext<'_>,
    conn: &Connection,
    input: &Input,
    workspace: Option<&str>,
    force: bool,
) -> Result<Plan> {
    let state: Option<(String, i64, String)> = if let Some(id) = input.id {
        // The file's claimed workspace never authorizes an existing ID.
        // Same connection/snapshot as classification and mutation.
        ensure_memory_access(conn, ctx.principal, id)?;
        conn.query_row(
            "SELECT content_hash, version, workspace FROM memories WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
    } else {
        None
    };
    let expected_ws = match workspace {
        Some(ws) => Some(ws.to_string()),
        None => input
            .fm
            .get("engram_workspace")
            .map(|ws| {
                crate::types::normalize_workspace(ws)
                    .map_err(|error| EngramError::InvalidInput(error.to_string()))
            })
            .transpose()?,
    };
    if let (Some((_, _, ref stored_ws)), Some(ref target_ws)) = (&state, &expected_ws) {
        if stored_ws != target_ws {
            return Err(EngramError::InvalidInput(format!(
                "engram_id {} belongs to workspace '{stored_ws}', not target workspace '{target_ws}'",
                input.id.unwrap_or_default()
            )));
        }
    }
    let (hash, baseline) = match input.fm.get("engram_content_hash") {
        Some(baseline) => (compute_content_hash_raw(&input.body), baseline.clone()),
        None => (
            compute_content_hash(&input.body),
            state
                .as_ref()
                .map(|(hash, _, _)| hash.clone())
                .unwrap_or_default(),
        ),
    };
    let status = classify_import_status(
        state
            .as_ref()
            .map(|(hash, version, _)| (hash.as_str(), *version)),
        &hash,
        &baseline,
        input.version,
        force,
    );
    let mutation = match status {
        ImportStatus::New => Mutation::Create(creation(ctx, input, workspace)?),
        ImportStatus::PendingUpdate => Mutation::Update(
            input
                .id
                .ok_or_else(|| EngramError::Internal("Missing import ID".into()))?,
            update(input)?,
        ),
        _ => Mutation::None,
    };
    Ok(Plan {
        file: input.file.clone(),
        id: input.id,
        status,
        mutation,
    })
}

fn run(
    ctx: &MarkdownContext<'_>,
    conn: &Connection,
    inputs: &[Input],
    workspace: Option<&str>,
    force: bool,
    confirm: bool,
) -> Result<ImportOutcome> {
    let normalized_workspace = match workspace {
        Some(raw) => {
            let norm = crate::types::normalize_workspace(raw)
                .map_err(|error| EngramError::InvalidInput(error.to_string()))?;
            if ctx
                .principal
                .is_some_and(|principal| !principal.allows_workspace(Some(&norm)))
            {
                return Err(EngramError::Unauthorized(
                    "Markdown target workspace is not allowed".into(),
                ));
            }
            Some(norm)
        }
        None => None,
    };
    // Authorize and classify the entire batch before the first write. With
    // confirm, BEGIN IMMEDIATE excludes concurrent writers through application.
    let plans = inputs
        .iter()
        .map(|input| plan(ctx, conn, input, normalized_workspace.as_deref(), force))
        .collect::<Result<Vec<_>>>()?;
    let (mut synced, mut new, mut pending, mut conflicts, mut applied) = (0, 0, 0, 0, 0);
    let mut files = Vec::new();
    for plan in plans {
        let mut id = plan.id;
        let mut changed = false;
        if confirm {
            match plan.mutation {
                Mutation::Create(input) => {
                    id = Some(crate::storage::queries::create_memory(conn, &input)?.id);
                    changed = true;
                }
                Mutation::Update(id, input) => {
                    crate::storage::queries::update_memory(conn, id, &input)?;
                    changed = true;
                }
                Mutation::None => (),
            }
        }
        applied += usize::from(changed);
        let (status, reason) = match plan.status {
            ImportStatus::New => {
                new += 1;
                ("new", None)
            }
            ImportStatus::InSync => {
                synced += 1;
                ("in_sync", None)
            }
            ImportStatus::PendingUpdate => {
                pending += 1;
                ("pending_update", None)
            }
            ImportStatus::Conflict(reason) => {
                conflicts += 1;
                ("conflict", Some(reason))
            }
        };
        files.push(ImportFile {
            file: plan.file,
            engram_id: id,
            status,
            applied: (confirm && changed).then_some(true),
            reason,
        });
    }
    Ok(ImportOutcome {
        scanned: inputs.len(),
        in_sync: synced,
        new,
        pending,
        conflict: conflicts,
        applied,
        dry_run: !confirm,
        files,
    })
}

pub(super) fn import_batch(
    ctx: &MarkdownContext<'_>,
    sources: Vec<MarkdownSource>,
    workspace: Option<&str>,
    force: bool,
    confirm: bool,
) -> std::result::Result<ImportOutcome, ImportFailure> {
    let inputs = prepare(sources)?; // Parsing and filesystem IO happen before DB transactions.
    let mut attempted = None;
    let result = if confirm {
        ctx.storage.with_transaction(|conn| {
            let report = run(ctx, conn, &inputs, workspace, force, true)?;
            attempted = Some(report.applied as u64);
            Ok(report)
        })
    } else {
        ctx.storage.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let result = run(ctx, &tx, &inputs, workspace, force, false)?;
            tx.commit()?;
            Ok(result)
        })
    };
    result.map_err(|error| ImportFailure { error, attempted })
}
