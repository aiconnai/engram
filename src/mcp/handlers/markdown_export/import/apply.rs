use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};

use super::files::MarkdownSource;
use super::status::{classify_import_status, ImportStatus};
use crate::error::{EngramError, Result};
use crate::mcp::handlers::markdown_export::frontmatter::{
    extract_body, frontmatter_tags, import_payload, parse_frontmatter,
};
use crate::mcp::handlers::HandlerContext;
use crate::mcp::workspace_guard::ensure_memory_access;
use crate::storage::queries::{compute_content_hash, compute_content_hash_raw};
use crate::types::{CreateMemoryInput, UpdateMemoryInput};

#[derive(Debug)]
pub(super) struct ImportFailure {
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
    pub(super) fn into_value(self) -> Value {
        use crate::mcp::error::ToolError;
        if let Some(attempted) = self.attempted {
            let mut value = ToolError::internal(
                "Import completion is uncertain; inspect the canonical database before retrying",
            )
            .into_value();
            value["applied"] = Value::Null;
            value["attempted"] = json!(attempted);
            value["outcome"] = json!("unknown");
            value
        } else {
            let mut value = ToolError::from(self.error).into_value();
            value["applied"] = json!(0);
            value["outcome"] = json!("not_applied");
            value
        }
    }
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
    ctx: &HandlerContext,
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
    ctx: &HandlerContext,
    conn: &Connection,
    input: &Input,
    workspace: Option<&str>,
    force: bool,
) -> Result<Plan> {
    let state: Option<(String, i64)> = if let Some(id) = input.id {
        // The file's claimed workspace never authorizes an existing ID.
        // Same connection/snapshot as classification and mutation.
        ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
        conn.query_row(
            "SELECT content_hash, version FROM memories WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
    } else {
        None
    };
    let (hash, baseline) = match input.fm.get("engram_content_hash") {
        Some(baseline) => (compute_content_hash_raw(&input.body), baseline.clone()),
        None => (
            compute_content_hash(&input.body),
            state
                .as_ref()
                .map(|(hash, _)| hash.clone())
                .unwrap_or_default(),
        ),
    };
    let status = classify_import_status(
        state
            .as_ref()
            .map(|(hash, version)| (hash.as_str(), *version)),
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
    ctx: &HandlerContext,
    conn: &Connection,
    inputs: &[Input],
    workspace: Option<&str>,
    force: bool,
    confirm: bool,
) -> Result<Value> {
    // Authorize and classify the entire batch before the first write. With
    // confirm, BEGIN IMMEDIATE excludes concurrent writers through application.
    let plans = inputs
        .iter()
        .map(|input| plan(ctx, conn, input, workspace, force))
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
        let mut detail = json!({"file":plan.file,"engram_id":id,"status":status});
        if confirm && changed {
            detail["applied"] = json!(true);
        }
        if let Some(reason) = reason {
            detail["reason"] = json!(reason);
        }
        files.push(detail);
    }
    Ok(
        json!({"scanned":inputs.len(),"in_sync":synced,"new":new,"pending":pending,"pending_updates":pending,"conflict":conflicts,"conflicts":conflicts,"applied":applied,"dry_run":!confirm,"files":files}),
    )
}

pub(super) fn import_batch(
    ctx: &HandlerContext,
    sources: Vec<MarkdownSource>,
    workspace: Option<&str>,
    force: bool,
    confirm: bool,
) -> std::result::Result<Value, ImportFailure> {
    let inputs = prepare(sources)?; // Parsing and filesystem IO happen before DB transactions.
    let mut attempted = None;
    let result = if confirm {
        ctx.storage.with_transaction(|conn| {
            let report = run(ctx, conn, &inputs, workspace, force, true)?;
            attempted = Some(
                report["applied"]
                    .as_u64()
                    .ok_or_else(|| EngramError::Internal("Missing import action count".into()))?,
            );
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
