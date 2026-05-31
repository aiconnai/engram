# RFC 0004 — Markdown and Obsidian Portability Contract

**Issue:** #32  
**Date:** 2026-05-31  
**Status:** Accepted  
**Decision:** Export-first, review-mode import; Obsidian is a human-editable mirror, not the source of truth.

---

## Problem

Engram stores memories in SQLite. Users who work in Obsidian or read Markdown want to view, annotate, and occasionally import edits back. The contract must:

1. Make exported files useful standalone (no Engram running required to read them).
2. Prevent silent overwrites of canonical memory on re-import.
3. Document exactly what is lossy so users are not surprised.

---

## Canonical Frontmatter Fields

Every exported Markdown file begins with a YAML frontmatter block. All fields are required unless marked optional.

```yaml
---
engram_id: 42                          # Memory.id — integer, primary key
engram_workspace: default              # Memory.workspace
engram_scope: user                     # Memory.scope (user|session|agent|global)
engram_type: note                      # Memory.memory_type
engram_created_at: 2026-05-31T02:07:00Z  # Memory.created_at (RFC 3339 / UTC)
engram_updated_at: 2026-05-31T04:00:00Z  # Memory.updated_at (RFC 3339 / UTC)
engram_content_hash: sha256:abc123...  # Memory.content_hash — detect drift on re-import
engram_source_session: sess_abc        # Memory.metadata["source_session"] if present (optional)
engram_importance: 0.8                 # Memory.importance (0.0–1.0)
engram_tags:                           # Memory.tags
  - rust
  - architecture
engram_tier: permanent                 # Memory.tier (permanent|daily)
engram_version: 3                      # Memory.version — last known canonical version
---
```

**Namespace prefix `engram_`** is mandatory on all fields to avoid clashing with Obsidian's own frontmatter keys (`tags`, `aliases`, `cssclasses`, etc.).

### Field mapping

| Frontmatter key | Memory field | Notes |
|---|---|---|
| `engram_id` | `id` | Integer. Never re-assigned on re-import. |
| `engram_workspace` | `workspace` | Normalized: lowercase `[a-z0-9_-]`, max 64 chars. |
| `engram_scope` | `scope` | Enum: `user`, `session`, `agent`, `global`. |
| `engram_type` | `memory_type` | Enum values exported as lowercase strings. |
| `engram_created_at` | `created_at` | RFC 3339, always UTC (`Z` suffix). |
| `engram_updated_at` | `updated_at` | RFC 3339, always UTC. |
| `engram_content_hash` | `content_hash` | `sha256:<hex>` prefix. Used for drift detection. |
| `engram_source_session` | `metadata["source_session"]` | Optional. Omitted if absent. |
| `engram_importance` | `importance` | Float 0.0–1.0. |
| `engram_tags` | `tags` | YAML sequence. Empty list is `[]`. |
| `engram_tier` | `tier` | `permanent` or `daily`. |
| `engram_version` | `version` | Integer. Bump on every canonical update. |

---

## Export

### Grouping modes

Export is triggered via the CLI or MCP tool and supports these groupings:

| Mode | Flag | Output structure |
|---|---|---|
| Day | `--group day` | One directory per `YYYY-MM-DD`; files named `<id>-<slug>.md`. |
| Project / workspace | `--group workspace` | One directory per workspace. |
| Entity | `--group entity` | One directory per unique entity extracted from tags/metadata. |
| Type | `--group type` | One directory per `memory_type` (note, decision, issue, …). |
| Flat | `--group flat` (default) | All files in one directory. |

### File naming

```
<engram_id>-<slug>.md
```

`slug` = first 40 chars of content, lowercased, non-alphanumeric replaced with `-`, trailing `-` stripped.

Example: `42-authentication-is-required-for-every.md`

### Content body

The Markdown body is `Memory.content` verbatim. No reformatting is applied. Code blocks, lists, and headings in the original content are preserved.

### What is exported losslessly

- `id`, `workspace`, `scope`, `memory_type`, `created_at`, `updated_at`, `content_hash`, `importance`, `tags`, `tier`, `version`, `content`
- `source_session` when present in metadata

### What is lossy (cannot round-trip)

| Field | Why lossy |
|---|---|
| `embedding` | Binary vector; not human-readable. Recomputed on import. |
| `access_count` | Usage metric; not meaningful outside Engram. Dropped on export. |
| `last_accessed_at` | Same — usage metric. Dropped. |
| `expires_at` | Time-bound semantics do not transfer to static files. Dropped. |
| `procedure_success_count`, `procedure_failure_count` | Procedural runtime state. Dropped. |
| `summary_of_id` | Foreign key; meaningless without the referenced memory. Dropped. |
| `lifecycle_state` | Internal state machine. Reset to `active` on import. |
| Arbitrary `metadata` keys | Only `source_session` is preserved; other keys are dropped. |
| `owner_id` | Multi-user field; not portable across Engram instances. Dropped. |
| `visibility` | Instance-specific access policy. Reset to default on import. |

---

## Import (review mode)

**Default: review mode.** Import never silently overwrites canonical memory.

### Review mode behavior

1. Parse frontmatter from each `.md` file.
2. Look up `engram_id` in the local database.
3. Compute `sha256` of the current file body.
4. Compare against `engram_content_hash` in frontmatter.

| Condition | Action |
|---|---|
| `engram_id` not found in DB | Stage as **new memory** for user confirmation. |
| `content_hash` matches DB | No-op — file is in sync. |
| `content_hash` differs, `engram_version` matches DB version | Stage as **pending update** for user review. |
| `engram_version` < DB version | **Conflict**: DB has been updated since export. Require explicit `--force-version` flag to overwrite. |
| `engram_version` > DB version | **Conflict**: file version is ahead of DB (unusual). Require explicit `--force-version`. |

### Staged updates

Staged updates are shown as a diff and require explicit `--confirm` to apply. The import tool prints:

```
[STAGED] memory #42 — content changed (hash drift)
  DB version: 3, file version: 3
  Run with --confirm to apply, or --skip to discard.
```

### What import restores

- `content` (from file body, after `--confirm`)
- `tags`, `importance`, `memory_type`, `scope`, `workspace` (from frontmatter)
- `updated_at` is set to the import timestamp (not the frontmatter value)
- `content_hash` is recomputed from the new content
- `version` is incremented by 1

### What import never restores

- Lossy fields listed in the export section above
- `engram_id` is never re-assigned — the existing row is updated in place

---

## Obsidian integration notes

Obsidian may add its own frontmatter keys (`aliases`, `cssclasses`, `position`). The import parser **ignores all keys without the `engram_` prefix**, so Obsidian annotations are safe and never cause parse errors.

Users may freely add Obsidian-native links (`[[Memory title]]`), callouts, and tags inside the body — these are preserved verbatim on export (since export outputs `content` verbatim) but are treated as plain text by Engram.

**Obsidian is a mirror, not the source of truth.** The canonical record lives in Engram's SQLite database. Obsidian edits take effect only after an explicit import with `--confirm`.

---

## Decision

| Option | Description | Decision |
|---|---|---|
| A | Obsidian as primary store, Engram syncs from it | Rejected — bidirectional sync introduces conflict surface without benefit |
| B | Engram as primary, Markdown export is read-only | Rejected — prevents useful annotations flowing back |
| **C** | Engram as primary, import in review mode (chosen) | **Accepted** — canonical record protected; human edits can flow back deliberately |

Option C with review-mode import is the correct boundary. It treats Obsidian as a human-readable projection of the memory graph, not as a peer store.

---

## Non-goals

- Real-time sync with Obsidian's sync plugin
- Git-based conflict resolution
- Exporting graph edges or knowledge graph structure (separate concern)
- Multi-vault support (single vault path only in v1)
