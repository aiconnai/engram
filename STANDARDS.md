# STANDARDS.md — Governance & Quality Standards

Lean governance for this repository. Every rule is objectively verifiable.

**Referenced by:** [CLAUDE.md](CLAUDE.md)

**Assumed knowledge:** Language-standard conventions (PEP 8, ESLint defaults, etc.) are assumed.
This file only documents project-specific rules and deviations.

---

## 1. Core Principles

> **Iron Law:** Every rule must protect more than it costs. Remove rules that create drag without value.

- **Evidence over opinion** — decisions backed by data or tested behavior
- **Parse at the boundary** — validate external input where it enters the system
- **Errors carry context** — never swallow exceptions; log or propagate with details
- **Idempotency where it matters** — re-running should be safe or explicitly documented as unsafe
- **Document decisions that affect future work** — not all decisions, just consequential ones
- **Least powerful tool** — use the simplest approach that solves the problem
- **Verify before claiming done** — evidence before completion claims, always

---

## 2. Project Tiers

| Tier | Blast Radius | Required Gates | Max Iterations |
|------|-------------|----------------|----------------|
| **T1** | Multi-phase, writes to external systems | README, CLAUDE.md, PLAN.md, dry-run, preflight, manifest | 15 |
| **T2** | Single-phase, external reads/writes | README, CLAUDE.md, dry-run, preflight | 10 |
| **T3** | Local only — reads data, generates reports | README | 7 |
| **T4** | Reference material, static resources | Optional | 5 |

---

## 3. Required Gates by Tier

### T1/T2 — External Systems

- **Dry-run default:** no writes without `--live` flag
- **Preflight validation:** inputs exist and are well-formed before execution
- **Manifest output:** JSON manifest in `output/` after every run
- **Rollback info:** manifest contains enough data to undo manually
- **Idempotency:** README documents whether re-running is safe

### T3 — Local Processing

- Validate input files before processing
- Clear error messages on failure
- Non-zero exit code on error

### T4 — Documentation

- No execution-level gates required

---

## 4. Naming Conventions

### Python
- Files: `snake_case.py`
- Verb-first for scripts: `publish_`, `validate_`

### JavaScript/TypeScript
- Files: `camelCase.ts` or `kebab-case.ts` (be consistent)
- Components: `PascalCase.tsx`

### Rust
- Files: `snake_case.rs`

### Config & Output
- Config: YAML or TOML
- Data/output: JSON
- Secrets: `.env` (never committed)

---

## 5. Code Quality

<HARD-GATE>No hardcoded secrets or credentials in code — ever.</HARD-GATE>

| Rule | Severity |
|------|----------|
| No hardcoded secrets or credentials | CRITICAL |
| Error handling: never silently swallow exceptions | CRITICAL |
| No silent failures — if something goes wrong, it must be visible | HIGH |
| No commented-out code in commits | HIGH |
| No `TODO` without a linked issue or explanation | MEDIUM |
| Dependencies: pin versions in lock files | MEDIUM |

---

## 6. Git Conventions

- Branch naming: `feature/`, `fix/`, `chore/` prefixes
- Commit messages: imperative mood, max 72 chars first line
- One logical change per commit
- Never commit `.env`, credentials, or large binaries

---

## 7. Plan Format Standard

When writing implementation plans:

- Break work into bite-sized tasks (2-5 minutes each)
- Each task specifies: exact file paths, expected changes, verification command
- Tasks are written for someone with zero context about the codebase
- Order: setup → implement → test → verify → document
- Include expected output for verification commands

---

## 8. Documentation Relevance Rule

Document only what helps someone proceed safely with the next task.

- If a decision constrains future work → document it
- If a workaround exists for a known issue → document it in ERRORS_AND_LESSONS.md
- If documentation would be stale within a sprint → skip it
- Pressure-test documentation: if an agent rationalizes around a rule, add an explicit counter

---

## 9. Exception Rule

<HARD-GATE>Undocumented exceptions are treated as bugs.</HARD-GATE>

Any rule in this file can be overridden if:

1. The exception is documented in the PR or commit message
2. The reason explains why the rule does not apply
3. The override is scoped — it does not disable the rule globally

### Common Legitimate Exceptions

| Scenario | Minimum Requirement |
|----------|-------------------|
| Prototype/spike (will be discarded) | Mark branch as throwaway, no merge to main |
| Third-party/vendored code | Document source and version |
| Emergency hotfix | Post-incident review within 48 hours |
| Generated code (codegen, migrations) | Document generator and regeneration steps |
| One-time script | Comment with purpose and expiration at top of file |

---

## 10. Error Catalog

All recurring errors must be documented in [ERRORS_AND_LESSONS.md](ERRORS_AND_LESSONS.md).

---

## 11. Embedded Storage & Concurrency Discipline (SQLite / WAL)

### 11.1. Scoped Connection Lifecycle Across Async Boundaries
- **No `Connection` across `.await`:** `rusqlite::Connection` handles must never be held across Tokio await points (`!Send`/`!Sync`). Isolate database operations to synchronous closures checked out from a connection pool (`with_connection(|conn| ...)`).
- **Zero network I/O inside transactions:** Never issue HTTP calls, LLM requests, embedding API queries, or remote sync operations while holding an open transaction. Keep locks bounded to microsecond durations to prevent `SQLITE_BUSY` contention across concurrent readers and writers.

### 11.2. Upper-Bound Defense on Byte Replay (Sparse-File Attack)
- **Upper-bound page checks:** When streaming or replaying WAL frames from peers or untrusted sources, validate that `page_number` does not exceed `MAX_ALLOWED_DB_PAGES` (e.g., 100,000,000 pages). Seeking to arbitrary `u32` offsets silently allocates multi-terabyte sparse files on disk, exhausting backup systems and storage quotas.
- **Saturating decompression bomb guards:** Enforce streaming byte limits on decompressed archives using saturating readers (`read_entry_limited`) rather than checking size post-inflation.

---

## 12. Multilingual Unicode & String Safety

### 12.1. The Case-Folding Slicing Trap
- **Never slice original strings using byte offsets calculated on lowercased or normalized text (`&s[start..end]` or `s.drain(start..end)`):** Multi-byte Unicode characters alter byte lengths upon casing changes (e.g., German `ß` [2 bytes] vs. uppercase `ẞ` [3 bytes] or `SS` [2 ASCII bytes]; Greek and Turkish casing variances).
- **Safe string manipulation:** Always slice using character iterators, Unicode grapheme clusters (`unicode-segmentation`), or regex engine word boundaries (`\b`).

---

## 13. Agentic Tool Governance & Model Transparency

### 13.1. Anti-IDOR Tenant Enforcement on Integer Primary Keys
- **Autonomous agents traverse numerical IDs (`id: 123`):** Every lookup by primary key must enforce tenant and workspace verification against the authenticated principal (`principal.allows_workspace`).
- **Fail-closed permission ladders:** Enforce runtime permission modes (`read_only < scoped_write < maintenance < admin`) at the dispatcher layer so agents cannot execute mutating actions in read-only sessions.
- **Case-insensitive reserved metadata shielding:** Prevent callers from spoofing system governance keys by casing variations (reject `Workspace`, `_Scope`, `Principal` case-insensitively).

### 13.2. Zero Silent Degradation in Model Fallbacks
- **No silent downgrades:** When API keys are missing or compile-time features are disabled, never silently fall back to a lower-tier provider without returning explicit degradation metadata.
- **Deterministic offline resolution:** Model routing must resolve deterministically and offline without network probes, returning explicit status codes (`missing_secret`, `feature_disabled`, `fallback_used`).

### 13.3. Anti-Self-Approval & Negative Scope Contracts
- **AI agents must never approve their own policy or harness changes:** Enforce an explicit **Negative Scope** ([WHAT_WE_DONT_DO.md](docs/harness/WHAT_WE_DONT_DO.md)) declaring security boundaries that reject autonomous mutation without human ADR sign-off.
- **Adversarial fixture verification:** Prove task and evidence validators against adversarial fixtures (`wrong-sha`, `scope-violation`, `missing-required-fields`) that fail closed.

---

**Created:** 2026-03-09  
**Last Updated:** 2026-09-03

