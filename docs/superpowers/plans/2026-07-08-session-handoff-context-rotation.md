# Session Handoff Context Rotation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the approved manual-first Session Handoff workflow: one MCP action and one CLI command produce the same copy-ready continuation packet for starting a fresh AI session.

**Architecture:** Add a shared library builder in `src/intelligence/session_handoff.rs` that owns request normalization, session fallback, section assembly, checkpoint persistence, warnings, and Markdown rendering. Migrate existing MCP surfaces (`session_land`, `harness_handoff`) and the new CLI wrapper (`engram session handoff`) onto that builder so there is one handoff product contract.

**Tech Stack:** Rust, SQLite via `Storage`/rusqlite, existing Engram `intelligence`, `context`, `mcp`, and `bin/cli` modules, serde/serde_json, clap, existing MCP registry generator.

## Global Constraints

- Use `rtk` for every shell command in this repository.
- Do not introduce a new storage schema for the MVP.
- `session_id` is optional; when omitted, resolve the most recent session in the workspace or produce a warning-marked workspace-level packet.
- `persist` defaults to `true`; `--no-persist` disables checkpoint creation.
- MCP and CLI must share the same internal builder.
- `session_land` and `harness_handoff` must not maintain separate rendering or checkpoint persistence logic.
- Do not include raw transcripts, raw command output, raw artifact content, environment dumps, tokens, cookies, or private keys by default.
- The MVP does not promise comprehensive active secret scrubbing beyond existing redaction/private-content mechanisms.
- Preserve backward compatibility where practical: existing `session_land` and `harness_handoff` callers should continue to get their familiar top-level fields while also receiving `copy_block` and `warnings`.
- If MCP tool schemas or descriptions change, update `src/mcp/tools/registry.rs`, regenerate `docs/MCP_TOOLS.md`, and run `./scripts/generate-mcp-reference.sh --check`.
- Do not touch unrelated refactors or harness sprint metadata unless a gate explicitly requires it.

---

## File Structure

Create:

- `src/intelligence/session_handoff.rs` — shared builder, request/packet structs, session fallback, safe previews, section assembly, Markdown rendering, optional checkpoint persistence, unit tests.
- `src/bin/cli/session.rs` — CLI wrapper for `engram session handoff` using the shared builder.

Modify:

- `src/intelligence/mod.rs` — expose the new module and public builder types/functions.
- `src/mcp/handlers/handoff.rs` — make `session_land` a thin adapter over the shared builder.
- `src/mcp/handlers/harness.rs` — make `handle_harness_handoff` a thin adapter over the shared builder while preserving stricter validation.
- `src/bin/cli/args.rs` — add `session handoff` clap subcommand and arguments.
- `src/bin/cli/main.rs` — route the new CLI command to `session::handle`.
- `src/mcp/tools/registry.rs` — make `session_land.session_id` optional, document fallback, and add any response/schema wording required by registry conventions.
- `src/mcp/prompts.rs` — update any prompt text that tells agents `session_land` requires `session_id`.
- `tests/mcp_protocol_tests.rs` — add protocol-level tests for `session_land` fallback and copy block.
- `docs/MCP_TOOLS.md` — regenerate only if `registry.rs` changes.

Do not create a new MCP tool unless implementation discovers a hard registry constraint. If a friendlier alias is later desired, add it in a separate follow-up.

---

### Task 1: Add the Shared Session Handoff Builder Skeleton

**Files:**
- Create: `src/intelligence/session_handoff.rs`
- Modify: `src/intelligence/mod.rs`

**Interfaces:**
- Consumes: `engram::Storage`, `crate::intelligence::session_indexing::{list_sessions, Session}`, `crate::error::Result`.
- Produces:
  - `SessionHandoffRequest`
  - `SessionHandoffPacket`
  - `HandoffItem`
  - `build_session_handoff(storage: &Storage, request: SessionHandoffRequest) -> Result<SessionHandoffPacket>`

- [ ] **Step 1: Write failing unit tests for omitted-session fallback**

Add this test module to the bottom of the new `src/intelligence/session_handoff.rs` file before implementation is complete:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::session_indexing::{index_conversation, ChunkingConfig, Message};
    use crate::Storage;
    use chrono::{Duration, Utc};

    fn message_at(minutes_ago: i64, content: &str) -> Message {
        Message {
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Utc::now() - Duration::minutes(minutes_ago),
            id: None,
        }
    }

    fn seed_session(storage: &Storage, session_id: &str, workspace: &str, minutes_ago: i64) {
        let messages = vec![message_at(minutes_ago, &format!("seed {session_id}"))];
        storage
            .with_connection(|conn| {
                index_conversation(
                    conn,
                    session_id,
                    &messages,
                    &ChunkingConfig::default(),
                    Some(workspace),
                    Some(session_id),
                    Some("test-agent"),
                )?;
                Ok::<_, crate::error::EngramError>(())
            })
            .expect("seed session");
    }

    #[test]
    fn omitted_session_id_uses_latest_session_in_workspace() {
        let storage = Storage::open_in_memory().expect("in-memory storage");
        seed_session(&storage, "older-session", "handoff-test", 20);
        seed_session(&storage, "newer-session", "handoff-test", 1);

        let packet = build_session_handoff(
            &storage,
            SessionHandoffRequest {
                workspace: Some("handoff-test".to_string()),
                persist: false,
                ..SessionHandoffRequest::default()
            },
        )
        .expect("handoff packet");

        assert_eq!(packet.session_id.as_deref(), Some("newer-session"));
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("session_id omitted")),
            "fallback warning missing: {:?}",
            packet.warnings
        );
        assert!(packet.copy_block.contains("# Continue this work in a new AI session"));
    }

    #[test]
    fn omitted_session_id_without_sessions_returns_workspace_packet_with_warning() {
        let storage = Storage::open_in_memory().expect("in-memory storage");

        let packet = build_session_handoff(
            &storage,
            SessionHandoffRequest {
                workspace: Some("empty-workspace".to_string()),
                persist: false,
                ..SessionHandoffRequest::default()
            },
        )
        .expect("workspace handoff packet");

        assert_eq!(packet.session_id, None);
        assert_eq!(packet.workspace, "empty-workspace");
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("No concrete session resolved")),
            "workspace warning missing: {:?}",
            packet.warnings
        );
        assert!(packet.copy_block.contains("## Source references"));
    }
}
```

- [ ] **Step 2: Run the tests and confirm they fail before implementation**

Run:

```bash
rtk cargo test session_handoff --lib
```

Expected: FAIL because `src/intelligence/session_handoff.rs`, `SessionHandoffRequest`, and `build_session_handoff` do not exist or are incomplete.

- [ ] **Step 3: Implement the minimal builder skeleton**

Create `src/intelligence/session_handoff.rs` with this starting implementation:

```rust
//! Shared session handoff builder for MCP and CLI surfaces.
//!
//! Produces a derived continuation packet for starting a fresh AI session.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::intelligence::session_indexing::list_sessions;
use crate::Storage;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionHandoffRequest {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub current_goal: Option<String>,
    #[serde(default)]
    pub next_session_hints: Vec<String>,
    #[serde(default)]
    pub files_touched: Vec<String>,
    #[serde(default)]
    pub decisions_made: Vec<String>,
    #[serde(default)]
    pub tests_run: Vec<String>,
    #[serde(default)]
    pub tests_not_run: Vec<String>,
    #[serde(default)]
    pub known_risks: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub verification_evidence: Option<String>,
    #[serde(default)]
    pub issue_numbers: Vec<i64>,
    #[serde(default)]
    pub plan_doc_paths: Vec<String>,
    #[serde(default = "default_persist")]
    pub persist: bool,
    #[serde(default = "default_true")]
    pub include_operational_context: bool,
    #[serde(default = "default_true")]
    pub include_digest: bool,
}

fn default_persist() -> bool {
    true
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct HandoffItem {
    pub title: String,
    pub detail: Option<String>,
    pub source_memory_id: Option<i64>,
    pub source_context_event_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionHandoffPacket {
    pub session_id: Option<String>,
    pub workspace: String,
    pub created_at: String,
    pub summary: String,
    pub current_goal: Option<String>,
    pub open_items: Vec<HandoffItem>,
    pub decisions: Vec<HandoffItem>,
    pub verification: Vec<HandoffItem>,
    pub risks: Vec<HandoffItem>,
    pub blockers: Vec<HandoffItem>,
    pub files_touched: Vec<String>,
    pub tests_run: Vec<String>,
    pub tests_not_run: Vec<String>,
    pub next_steps: Vec<String>,
    pub source_memory_ids: Vec<i64>,
    pub source_context_event_ids: Vec<i64>,
    pub warnings: Vec<String>,
    pub checkpoint_id: Option<i64>,
    pub copy_block: String,
}

pub fn build_session_handoff(
    storage: &Storage,
    request: SessionHandoffRequest,
) -> Result<SessionHandoffPacket> {
    let workspace = request
        .workspace
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();

    let mut warnings = Vec::new();
    let session_id = resolve_session_id(storage, request.session_id.as_deref(), &workspace, &mut warnings)?;
    let summary = request
        .summary
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| match &session_id {
            Some(id) => format!("Session {id} handoff"),
            None => format!("Workspace {workspace} handoff"),
        });

    let mut packet = SessionHandoffPacket {
        session_id,
        workspace,
        created_at: chrono::Utc::now().to_rfc3339(),
        summary,
        current_goal: request.current_goal.clone(),
        open_items: Vec::new(),
        decisions: request
            .decisions_made
            .iter()
            .map(|decision| HandoffItem {
                title: strip_private_content(decision),
                detail: None,
                source_memory_id: None,
                source_context_event_id: None,
            })
            .collect(),
        verification: verification_items(&request),
        risks: request
            .known_risks
            .iter()
            .map(|risk| HandoffItem {
                title: strip_private_content(risk),
                detail: None,
                source_memory_id: None,
                source_context_event_id: None,
            })
            .collect(),
        blockers: request
            .blockers
            .iter()
            .map(|blocker| HandoffItem {
                title: strip_private_content(blocker),
                detail: None,
                source_memory_id: None,
                source_context_event_id: None,
            })
            .collect(),
        files_touched: request.files_touched.clone(),
        tests_run: request.tests_run.clone(),
        tests_not_run: request.tests_not_run.clone(),
        next_steps: merged_next_steps(&request),
        source_memory_ids: Vec::new(),
        source_context_event_ids: Vec::new(),
        warnings,
        checkpoint_id: None,
        copy_block: String::new(),
    };

    if packet.current_goal.as_deref().unwrap_or("").trim().is_empty() {
        packet.warnings.push("No current_goal was provided or inferred.".to_string());
    }
    if packet.next_steps.is_empty() {
        packet.warnings.push("No next steps were provided or inferred.".to_string());
    }
    if packet.verification.is_empty() {
        packet
            .warnings
            .push("No verification evidence provided. Do not claim this work is complete.".to_string());
    }

    packet.copy_block = render_copy_block(&packet);
    Ok(packet)
}

fn resolve_session_id(
    storage: &Storage,
    requested: Option<&str>,
    workspace: &str,
    warnings: &mut Vec<String>,
) -> Result<Option<String>> {
    if let Some(session_id) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(Some(session_id.to_string()));
    }

    let sessions = storage.with_connection(|conn| list_sessions(conn, Some(workspace), 1))?;
    if let Some(session) = sessions.into_iter().next() {
        warnings.push(format!(
            "session_id omitted; using most recent session '{}' in workspace '{}'.",
            session.session_id, workspace
        ));
        Ok(Some(session.session_id))
    } else {
        warnings.push(format!(
            "No concrete session resolved for workspace '{}'; generated a workspace-level handoff.",
            workspace
        ));
        Ok(None)
    }
}

fn verification_items(request: &SessionHandoffRequest) -> Vec<HandoffItem> {
    let mut items: Vec<HandoffItem> = request
        .tests_run
        .iter()
        .map(|test| HandoffItem {
            title: strip_private_content(test),
            detail: Some("test_run".to_string()),
            source_memory_id: None,
            source_context_event_id: None,
        })
        .collect();
    if let Some(evidence) = request
        .verification_evidence
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        items.push(HandoffItem {
            title: strip_private_content(evidence),
            detail: Some("verification_evidence".to_string()),
            source_memory_id: None,
            source_context_event_id: None,
        });
    }
    items
}

fn merged_next_steps(request: &SessionHandoffRequest) -> Vec<String> {
    let mut next_steps = request.next_steps.clone();
    for hint in &request.next_session_hints {
        if !hint.trim().is_empty() && !next_steps.iter().any(|step| step == hint) {
            next_steps.push(hint.clone());
        }
    }
    next_steps
}

fn strip_private_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(start) = remaining.find("<private>") {
        result.push_str(&remaining[..start]);
        if let Some(end_offset) = remaining[start..].find("</private>") {
            remaining = &remaining[start + end_offset + "</private>".len()..];
        } else {
            return result;
        }
    }
    result.push_str(remaining);
    result
}

fn render_copy_block(packet: &SessionHandoffPacket) -> String {
    let mut output = String::new();
    output.push_str("# Continue this work in a new AI session\n\n");
    output.push_str("## Essential context\n");
    output.push_str(&format!("Workspace: {}\n", packet.workspace));
    if let Some(session_id) = &packet.session_id {
        output.push_str(&format!("Session: {}\n", session_id));
    }
    output.push_str(&format!("Summary: {}\n\n", packet.summary));

    output.push_str("## Current goal\n");
    output.push_str(packet.current_goal.as_deref().unwrap_or("No current goal captured."));
    output.push_str("\n\n");

    output.push_str("## Decisions\n");
    push_items(&mut output, &packet.decisions);
    output.push_str("\n## Verification\n");
    push_items(&mut output, &packet.verification);
    if !packet.tests_not_run.is_empty() {
        output.push_str("\nTests not run:\n");
        for test in &packet.tests_not_run {
            output.push_str(&format!("- {}\n", strip_private_content(test)));
        }
    }

    output.push_str("\n## Risks and blockers\n");
    push_items(&mut output, &packet.risks);
    push_items(&mut output, &packet.blockers);

    output.push_str("\n## Next steps\n");
    if packet.next_steps.is_empty() {
        output.push_str("- No next steps captured.\n");
    } else {
        for (index, step) in packet.next_steps.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", index + 1, strip_private_content(step)));
        }
    }

    output.push_str("\n## Source references\n");
    output.push_str(&format!("- Memory IDs: {:?}\n", packet.source_memory_ids));
    output.push_str(&format!("- Context event IDs: {:?}\n", packet.source_context_event_ids));
    output.push_str(&format!("- Files: {:?}\n", packet.files_touched));

    if !packet.warnings.is_empty() {
        output.push_str("\n## Warnings\n");
        for warning in &packet.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}

fn push_items(output: &mut String, items: &[HandoffItem]) {
    if items.is_empty() {
        output.push_str("- None captured.\n");
        return;
    }
    for item in items {
        output.push_str(&format!("- {}\n", item.title));
    }
}
```

Modify `src/intelligence/mod.rs`:

```rust
pub mod session_handoff;
```

Add near the existing `pub use session_indexing` exports:

```rust
pub use session_handoff::{
    build_session_handoff, HandoffItem, SessionHandoffPacket, SessionHandoffRequest,
};
```

- [ ] **Step 4: Run the builder tests**

Run:

```bash
rtk cargo test session_handoff --lib
```

Expected: PASS for the two fallback tests.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
rtk git add src/intelligence/session_handoff.rs src/intelligence/mod.rs && rtk git commit -m "feat(intelligence): add session handoff builder"
```

Expected: commit succeeds.

---

### Task 2: Add Explicit Field Sections, Safe Previews, and Local Memory Retrieval

**Files:**
- Modify: `src/intelligence/session_handoff.rs`

**Interfaces:**
- Consumes: Task 1 builder types.
- Produces:
  - Open todo/issue memory retrieval.
  - Recent decision memory retrieval.
  - Explicit fields overriding inferred text.
  - Private-content stripping in rendered memory previews.

- [ ] **Step 1: Add failing tests for explicit fields and private-content stripping**

Append these tests to the existing test module in `src/intelligence/session_handoff.rs`:

```rust
use crate::storage::queries::{create_memory, list_memories};
use crate::types::{CreateMemoryInput, ListOptions, MemoryTier, MemoryType};

fn seed_memory(storage: &Storage, workspace: &str, content: &str, memory_type: MemoryType) -> i64 {
    storage
        .with_transaction(|conn| {
            let memory = create_memory(
                conn,
                &CreateMemoryInput {
                    content: content.to_string(),
                    memory_type,
                    workspace: Some(workspace.to_string()),
                    tier: MemoryTier::Permanent,
                    ..Default::default()
                },
            )?;
            Ok::<_, crate::error::EngramError>(memory.id)
        })
        .expect("seed memory")
}

#[test]
fn explicit_fields_are_rendered_and_override_empty_inference() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("explicit-workspace".to_string()),
            current_goal: Some("Finish the shared handoff builder".to_string()),
            files_touched: vec!["src/intelligence/session_handoff.rs".to_string()],
            decisions_made: vec!["Use one shared builder for MCP and CLI".to_string()],
            tests_run: vec!["rtk cargo test session_handoff --lib".to_string()],
            tests_not_run: vec!["full make ci not run in focused task".to_string()],
            known_risks: vec!["MCP schema still needs migration".to_string()],
            blockers: vec!["No blockers".to_string()],
            next_steps: vec!["Wire session_land to the builder".to_string()],
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert!(packet.copy_block.contains("Finish the shared handoff builder"));
    assert!(packet.copy_block.contains("Use one shared builder for MCP and CLI"));
    assert!(packet.copy_block.contains("rtk cargo test session_handoff --lib"));
    assert!(packet.copy_block.contains("full make ci not run in focused task"));
    assert!(packet.copy_block.contains("Wire session_land to the builder"));
}

#[test]
fn rendered_content_strips_private_tags_from_memory_previews() {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    seed_memory(
        &storage,
        "safe-workspace",
        "Public decision <private>SECRET_TOKEN</private> after text",
        MemoryType::Decision,
    );

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("safe-workspace".to_string()),
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert!(packet.copy_block.contains("Public decision"));
    assert!(!packet.copy_block.contains("SECRET_TOKEN"));
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```bash
rtk cargo test session_handoff --lib
```

Expected: FAIL because the builder does not yet query memories.

- [ ] **Step 3: Add memory retrieval helpers and merge them into the packet**

Add these helper functions above `render_copy_block`. They intentionally mirror the current `session_land` SQL so the migration preserves behavior while moving assembly into the builder:

```rust
fn collect_open_items(storage: &Storage, workspace: &str) -> Result<Vec<HandoffItem>> {
    storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type \
             FROM memories \
             WHERE workspace = ?1 \
               AND memory_type IN ('todo', 'issue') \
               AND (lifecycle_state IS NULL OR lifecycle_state != 'archived') \
             ORDER BY importance DESC, created_at DESC \
             LIMIT 50",
        )?;
        let items = stmt
            .query_map(rusqlite::params![workspace], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                let memory_type: String = row.get(2)?;
                Ok(HandoffItem {
                    title: truncate_preview(&strip_private_content(&content), 200),
                    detail: Some(memory_type),
                    source_memory_id: Some(id),
                    source_context_event_id: None,
                })
            })?
            .filter_map(|row| row.ok())
            .collect();
        Ok(items)
    })
}

fn collect_recent_decisions(storage: &Storage, workspace: &str) -> Result<Vec<HandoffItem>> {
    storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, content \
             FROM memories \
             WHERE workspace = ?1 \
               AND memory_type = 'decision' \
             ORDER BY created_at DESC \
             LIMIT 20",
        )?;
        let items = stmt
            .query_map(rusqlite::params![workspace], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                Ok(HandoffItem {
                    title: truncate_preview(&strip_private_content(&content), 200),
                    detail: Some("memory_decision".to_string()),
                    source_memory_id: Some(id),
                    source_context_event_id: None,
                })
            })?
            .filter_map(|row| row.ok())
            .collect();
        Ok(items)
    })
}

fn truncate_preview(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        return content.to_string();
    }
    let mut preview: String = content.chars().take(max_chars.saturating_sub(1)).collect();
    preview.push('…');
    preview
}

fn push_source_ids(packet: &mut SessionHandoffPacket) {
    let mut ids: Vec<i64> = packet
        .open_items
        .iter()
        .chain(packet.decisions.iter())
        .chain(packet.verification.iter())
        .chain(packet.risks.iter())
        .chain(packet.blockers.iter())
        .filter_map(|item| item.source_memory_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    packet.source_memory_ids = ids;
}
```

Inside `build_session_handoff`, after creating `packet` and before warning generation, add:

```rust
match collect_open_items(storage, &packet.workspace) {
    Ok(open_items) => packet.open_items = open_items,
    Err(err) => packet
        .warnings
        .push(format!("Open item retrieval failed: {err}")),
}

match collect_recent_decisions(storage, &packet.workspace) {
    Ok(mut inferred_decisions) => {
        if packet.decisions.is_empty() {
            packet.decisions = inferred_decisions;
        } else {
            packet.decisions.append(&mut inferred_decisions);
        }
    }
    Err(err) => packet
        .warnings
        .push(format!("Decision retrieval failed: {err}")),
}

push_source_ids(&mut packet);
```

Update `render_copy_block` so open items and files are visible:

```rust
output.push_str("## What changed\n");
if packet.files_touched.is_empty() {
    output.push_str("- No touched files captured.\n\n");
} else {
    for file in &packet.files_touched {
        output.push_str(&format!("- {}\n", file));
    }
    output.push('\n');
}

output.push_str("## Open items\n");
push_items(&mut output, &packet.open_items);
output.push('\n');
```

Place that block between the `## Current goal` and `## Decisions` sections.

- [ ] **Step 4: Run focused tests**

Run:

```bash
rtk cargo test session_handoff --lib
```

Expected: PASS.

- [ ] **Step 5: Commit Task 2**

Run:

```bash
rtk git add src/intelligence/session_handoff.rs && rtk git commit -m "feat(intelligence): assemble session handoff sections"
```

Expected: commit succeeds.

---

### Task 3: Add Checkpoint Persistence and Operational Context Bundle Integration

**Files:**
- Modify: `src/intelligence/session_handoff.rs`

**Interfaces:**
- Consumes: `crate::context::{build_context_bundle, ContextBundleRequest}`, `crate::storage::queries::create_memory`, `crate::types::CreateMemoryInput`.
- Produces:
  - `checkpoint_id` when `persist=true`.
  - Persistence warnings instead of losing the packet on non-fatal persistence failure.
  - Operational Context warnings/source IDs where available.

- [ ] **Step 1: Add failing tests for persistence and no-persist**

Append tests:

```rust
#[test]
fn persist_true_creates_checkpoint_memory() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("persist-workspace".to_string()),
            current_goal: Some("Persist packet".to_string()),
            next_steps: vec!["Inspect checkpoint".to_string()],
            persist: true,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    let checkpoint_id = packet.checkpoint_id.expect("checkpoint id");
    let memories = storage
        .with_connection(|conn| {
            list_memories(
                conn,
                &ListOptions {
                    workspace: Some("persist-workspace".to_string()),
                    memory_type: Some(MemoryType::Checkpoint),
                    ..Default::default()
                },
            )
        })
        .expect("list checkpoints");

    assert!(memories.iter().any(|memory| memory.id == checkpoint_id));
    assert!(packet.copy_block.contains("Persist packet"));
}

#[test]
fn persist_false_does_not_create_checkpoint_memory() {
    let storage = Storage::open_in_memory().expect("in-memory storage");

    let packet = build_session_handoff(
        &storage,
        SessionHandoffRequest {
            workspace: Some("no-persist-workspace".to_string()),
            persist: false,
            ..SessionHandoffRequest::default()
        },
    )
    .expect("handoff packet");

    assert_eq!(packet.checkpoint_id, None);
    let memories = storage
        .with_connection(|conn| {
            list_memories(
                conn,
                &ListOptions {
                    workspace: Some("no-persist-workspace".to_string()),
                    memory_type: Some(MemoryType::Checkpoint),
                    ..Default::default()
                },
            )
        })
        .expect("list checkpoints");
    assert!(memories.is_empty());
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```bash
rtk cargo test session_handoff --lib
```

Expected: FAIL because checkpoint persistence is not implemented.

- [ ] **Step 3: Implement checkpoint persistence**

Add imports:

```rust
use crate::storage::queries::create_memory;
use crate::types::{CreateMemoryInput, MemoryTier};
```

Add helper:

```rust
fn persist_checkpoint(storage: &Storage, packet: &SessionHandoffPacket) -> Result<i64> {
    let content = serde_json::to_string_pretty(packet)
        .unwrap_or_else(|_| packet.copy_block.clone());
    let mut tags = vec!["session-handoff".to_string()];
    if let Some(session_id) = &packet.session_id {
        tags.push(format!("session:{session_id}"));
    }

    storage.with_transaction(|conn| {
        let memory = create_memory(
            conn,
            &CreateMemoryInput {
                content,
                memory_type: MemoryType::Checkpoint,
                tags,
                workspace: Some(packet.workspace.clone()),
                importance: Some(0.9),
                tier: MemoryTier::Permanent,
                ..Default::default()
            },
        )?;
        Ok(memory.id)
    })
}
```

Near the end of `build_session_handoff`, replace the final render assignment with:

```rust
packet.copy_block = render_copy_block(&packet);

if request.persist {
    match persist_checkpoint(storage, &packet) {
        Ok(id) => packet.checkpoint_id = Some(id),
        Err(err) => packet
            .warnings
            .push(format!("Checkpoint persistence failed: {err}")),
    }
    packet.copy_block = render_copy_block(&packet);
}

Ok(packet)
```

- [ ] **Step 4: Add Operational Context bundle warning integration**

Add import:

```rust
use crate::context::{build_context_bundle, ContextBundleRequest};
```

Add helper:

```rust
fn attach_operational_context(
    storage: &Storage,
    request: &SessionHandoffRequest,
    packet: &mut SessionHandoffPacket,
) {
    if !request.include_operational_context {
        return;
    }

    let query = request
        .current_goal
        .clone()
        .or_else(|| request.summary.clone())
        .or_else(|| Some("session handoff".to_string()));

    let bundle_request = ContextBundleRequest {
        query,
        workspace: Some(packet.workspace.clone()),
        session_id: packet.session_id.clone(),
        max_results: Some(80),
        section_limit: Some(12),
        include_artifact_pointers: true,
        ..Default::default()
    };

    match storage.with_connection(|conn| build_context_bundle(conn, &bundle_request)) {
        Ok(bundle) => {
            for stale in &bundle.stale_warnings {
                packet.warnings.push(format!("Stale context: {:?}", stale.warning));
            }
            for entry in bundle
                .recent_decisions
                .iter()
                .chain(bundle.unresolved_blockers.iter())
                .chain(bundle.failures.iter())
            {
                if let Ok(id) = entry.provenance.source_id.parse::<i64>() {
                    packet.source_context_event_ids.push(id);
                }
            }
            packet.source_context_event_ids.sort_unstable();
            packet.source_context_event_ids.dedup();
        }
        Err(err) => packet
            .warnings
            .push(format!("Operational Context bundle failed: {err}")),
    }
}
```

Call it before `push_source_ids(&mut packet);`:

```rust
attach_operational_context(storage, &request, &mut packet);
```

If `ContextProvenance.source_id` is not a string in the actual type, adjust only that parse line to the real field type and keep the test behavior unchanged.

- [ ] **Step 5: Run focused tests**

Run:

```bash
rtk cargo test session_handoff --lib
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

Run:

```bash
rtk git add src/intelligence/session_handoff.rs && rtk git commit -m "feat(intelligence): persist session handoff checkpoints"
```

Expected: commit succeeds.

---

### Task 4: Migrate `session_land` to the Shared Builder

**Files:**
- Modify: `src/mcp/handlers/handoff.rs`
- Modify: `src/mcp/tools/registry.rs`
- Modify: `src/mcp/prompts.rs`
- Test: existing tests in `src/mcp/handlers/handoff.rs`

**Interfaces:**
- Consumes: `build_session_handoff` and `SessionHandoffRequest`.
- Produces: `session_land` accepts omitted `session_id`, returns old `handoff` shape plus `copy_block`, `warnings`, and `checkpoint_id`.

- [ ] **Step 1: Add failing handler tests for omitted `session_id` and copy block**

In `src/mcp/handlers/handoff.rs`, update or add tests inside the existing test module:

```rust
#[test]
fn test_session_land_without_session_id_returns_workspace_packet() {
    let ctx = crate::mcp::handlers::tests::test_context();
    let result = session_land(
        &ctx,
        json!({
            "workspace": "default",
            "summary": "Manual session rotation"
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result:?}");
    assert!(result["handoff"]["copy_block"]
        .as_str()
        .expect("copy block")
        .contains("# Continue this work in a new AI session"));
    assert!(result["handoff"]["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .any(|warning| warning.as_str().unwrap_or("").contains("No concrete session resolved")));
}
```

If `crate::mcp::handlers::tests::test_context()` does not exist, keep the assertion body and create a local helper that mirrors the `TestHandler::new` setup from `tests/mcp_protocol_tests.rs`: `Storage::open_in_memory`, default embedder, `FuzzyEngine`, `EmbeddingCache`, and `SearchResultCache`.

- [ ] **Step 2: Run handler tests and confirm failure**

Run:

```bash
rtk cargo test session_land --lib
```

Expected: FAIL because `session_land` still requires `session_id` and does not expose `copy_block`.

- [ ] **Step 3: Replace `session_land` internals with builder adapter**

In `src/mcp/handlers/handoff.rs`, replace the body of `session_land` with:

```rust
pub fn session_land(ctx: &HandlerContext, params: Value) -> Value {
    let mut request: crate::intelligence::SessionHandoffRequest =
        match serde_json::from_value(params.clone()) {
            Ok(request) => request,
            Err(e) => return json!({"error": e.to_string()}),
        };

    if request.workspace.is_none() {
        request.workspace = params
            .get("workspace")
            .and_then(|value| value.as_str())
            .map(str::to_string);
    }

    match crate::intelligence::build_session_handoff(&ctx.storage, request) {
        Ok(packet) => json!({
            "handoff": packet,
            "checkpoint_id": packet.checkpoint_id,
        }),
        Err(e) => json!({"error": format!("Failed to build session handoff: {e}")}),
    }
}
```

Remove the old private `build_bootstrap_prompt` helper only after updating tests that directly call it. If tests depend on it, replace them with assertions against `packet.copy_block`.

- [ ] **Step 4: Update MCP registry and prompts**

In `src/mcp/tools/registry.rs`, update the `session_land` tool schema:

- `session_id` must no longer be listed in required inputs.
- The description should say omitted `session_id` falls back to the most recent session in the workspace.
- The `summary` and `next_session_hints` inputs remain optional.

In `src/mcp/prompts.rs`, replace text that says `session_land` requires a `session_id` with wording like:

```text
Call session_land with session_id when the host exposes it; otherwise omit it and let Engram use the most recent session in the workspace.
```

- [ ] **Step 5: Run focused tests**

Run:

```bash
rtk cargo test session_land --lib
```

Expected: PASS.

- [ ] **Step 6: Commit Task 4**

Run:

```bash
rtk git add src/mcp/handlers/handoff.rs src/mcp/tools/registry.rs src/mcp/prompts.rs && rtk git commit -m "feat(mcp): route session_land through handoff builder"
```

Expected: commit succeeds.

---

### Task 5: Migrate `harness_handoff` to the Shared Builder Without Weakening Validation

**Files:**
- Modify: `src/mcp/handlers/harness.rs`

**Interfaces:**
- Consumes: `SessionHandoffRequest` fields from Tasks 1-3.
- Produces: `handle_harness_handoff` keeps required `current_goal` and non-empty `next_steps`, preserves `completion_warning`, and returns builder `copy_block`.

- [ ] **Step 1: Update existing tests to assert copy block and shared packet fields**

In `src/mcp/handlers/harness.rs`, extend `test_harness_handoff_basic`:

```rust
assert!(result["copy_block"]
    .as_str()
    .expect("copy_block")
    .contains("# Continue this work in a new AI session"));
assert_eq!(result["current_goal"], json!("Implement feature X"));
assert_eq!(result["completion_claimed"], json!(true));
```

Extend `test_harness_handoff_no_verification_evidence`:

```rust
assert!(result["completion_warning"]
    .as_str()
    .expect("completion_warning")
    .contains("No verification evidence provided"));
assert!(result["copy_block"]
    .as_str()
    .expect("copy_block")
    .contains("Do not claim this work is complete"));
```

Keep `test_harness_handoff_missing_goal` and `test_harness_handoff_empty_next_steps` unchanged so the stricter harness validation remains enforced.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```bash
rtk cargo test harness_handoff --lib
```

Expected: FAIL because existing handler has no `copy_block`.

- [ ] **Step 3: Replace persistence/rendering section with builder call**

In `handle_harness_handoff`, keep validation for `current_goal` and `next_steps`, keep all existing parameter extraction, then replace the manual persistence and response-building block from the `// ── Persist if requested` comment through the final `response` return with:

```rust
let packet = match crate::intelligence::build_session_handoff(
    &ctx.storage,
    crate::intelligence::SessionHandoffRequest {
        workspace: Some(workspace.clone()),
        current_goal: Some(current_goal.clone()),
        files_touched: files_touched.clone(),
        decisions_made: decisions_made.clone(),
        tests_run: tests_run.clone(),
        tests_not_run: tests_not_run.clone(),
        known_risks: known_risks.clone(),
        blockers: blockers.clone(),
        next_steps: next_steps.clone(),
        verification_evidence: verification_evidence.clone(),
        issue_numbers: issue_numbers.clone(),
        plan_doc_paths: plan_doc_paths.clone(),
        persist,
        ..Default::default()
    },
) {
    Ok(packet) => packet,
    Err(e) => return json!({"error": format!("Failed to build handoff: {e}")}),
};

let has_evidence = verification_evidence
    .as_deref()
    .map(|value| !value.trim().is_empty())
    .unwrap_or(false);

let mut response = json!({
    "handoff_id": packet.checkpoint_id,
    "workspace": packet.workspace,
    "current_goal": current_goal,
    "files_touched": files_touched,
    "decisions_made": decisions_made,
    "tests_run": tests_run,
    "tests_not_run": tests_not_run,
    "known_risks": known_risks,
    "blockers": blockers,
    "next_steps": next_steps,
    "issue_numbers": issue_numbers,
    "plan_doc_paths": plan_doc_paths,
    "verification_evidence": verification_evidence,
    "completion_claimed": has_evidence,
    "persisted": persist && packet.checkpoint_id.is_some(),
    "created_at": packet.created_at,
    "warnings": packet.warnings,
    "copy_block": packet.copy_block,
});

if !has_evidence {
    response["completion_warning"] =
        json!("No verification evidence provided. Do not claim this work is complete.");
}

response
```

- [ ] **Step 4: Run focused tests**

Run:

```bash
rtk cargo test harness_handoff --lib
```

Expected: PASS.

- [ ] **Step 5: Commit Task 5**

Run:

```bash
rtk git add src/mcp/handlers/harness.rs && rtk git commit -m "feat(mcp): share harness handoff builder"
```

Expected: commit succeeds.

---

### Task 6: Add the CLI Wrapper `engram session handoff`

**Files:**
- Create: `src/bin/cli/session.rs`
- Modify: `src/bin/cli/args.rs`
- Modify: `src/bin/cli/main.rs`

**Interfaces:**
- Consumes: `engram::intelligence::{build_session_handoff, SessionHandoffRequest}`.
- Produces: `engram session handoff` with Markdown default and `--json` structured output.

- [ ] **Step 1: Add clap argument types**

Modify `src/bin/cli/args.rs` imports:

```rust
use clap::{Args, Parser, Subcommand};
```

Add after the existing optional action imports:

```rust
use crate::session::SessionAction;
```

Add a new enum variant in `Commands` before `Maintenance`:

```rust
    /// Session continuation and handoff workflows
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
```

- [ ] **Step 2: Create the CLI session module**

Create `src/bin/cli/session.rs`:

```rust
use clap::{Args, Subcommand};
use engram::error::Result;
use engram::intelligence::{build_session_handoff, SessionHandoffRequest};
use engram::storage::Storage;

#[derive(Subcommand)]
pub(crate) enum SessionAction {
    /// Generate a copy-ready packet for continuing work in a new AI session
    Handoff(SessionHandoffArgs),
}

#[derive(Args)]
pub(crate) struct SessionHandoffArgs {
    /// Session identifier. When omitted, Engram uses the latest session in the workspace.
    #[arg(long)]
    pub(crate) session: Option<String>,

    /// Workspace scope
    #[arg(long, default_value = "default")]
    pub(crate) workspace: String,

    /// Human-provided summary for the handoff packet
    #[arg(long)]
    pub(crate) summary: Option<String>,

    /// Current goal for the next AI session
    #[arg(long)]
    pub(crate) current_goal: Option<String>,

    /// Next step. Repeat the flag for multiple steps.
    #[arg(long = "next")]
    pub(crate) next_steps: Vec<String>,

    /// Do not persist a checkpoint memory
    #[arg(long)]
    pub(crate) no_persist: bool,

    /// Print structured JSON instead of copy-ready Markdown
    #[arg(long)]
    pub(crate) json: bool,
}

pub(crate) fn handle(storage: &Storage, action: SessionAction) -> Result<()> {
    match action {
        SessionAction::Handoff(args) => handoff(storage, args),
    }
}

fn handoff(storage: &Storage, args: SessionHandoffArgs) -> Result<()> {
    let packet = build_session_handoff(
        storage,
        SessionHandoffRequest {
            session_id: args.session,
            workspace: Some(args.workspace),
            summary: args.summary,
            current_goal: args.current_goal,
            next_steps: args.next_steps,
            persist: !args.no_persist,
            ..Default::default()
        },
    )?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&packet)?);
    } else {
        println!("{}", packet.copy_block);
    }
    Ok(())
}
```

- [ ] **Step 3: Route the command**

Modify `src/bin/cli/main.rs` module list:

```rust
mod session;
```

Add to the command match:

```rust
        Commands::Session { action } => session::handle(&storage, action)?,
```

Place it near `Commands::Maintenance`.

- [ ] **Step 4: Build the CLI**

Run:

```bash
rtk cargo build --bin engram-cli
```

Expected: PASS.

If the binary is named through the package default rather than `engram-cli`, run:

```bash
rtk cargo build --bins
```

Expected: PASS.

- [ ] **Step 5: Smoke test the CLI JSON path**

Run:

```bash
rtk cargo run --bin engram-cli -- --db-path /tmp/engram-handoff-smoke.db session handoff --workspace smoke --summary "Smoke handoff" --next "Continue in a new session" --no-persist --json
```

Expected: JSON output includes `