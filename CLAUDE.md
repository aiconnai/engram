---
project: engram
stack: python, node, rust
framework: axum
tier: T1
generated_by: claude-primer v1.8.1
last_updated: 2026-03-09
---

# CLAUDE.md

<!-- Target: keep this file under 300 lines. Split detail into STANDARDS.md or local CLAUDE.md files. -->

This file provides guidance to Claude Code when working in this repository.

**Quick reference:** [docs/QUICKSTART.md](docs/QUICKSTART.md)
**Standards:** [STANDARDS.md](STANDARDS.md)
**Mistakes:** [ERRORS_AND_LESSONS.md](ERRORS_AND_LESSONS.md)

---

## Routing Rules

If the task is inside a subdirectory that has its own CLAUDE.md:
1. **Read the local CLAUDE.md first** — it is the primary source for that scope.
2. Use this root file only as general context.
3. If the local file conflicts with this file, **the local file wins**.

---

## Repository Overview

Persistent memory for AI agents and teams working on proprietary context. Engram ingests interviews, meetings, docs, transcripts, and decisions, then exposes them through hybrid search, knowledge graphs, and MCP so agents can query the source of truth directly.

**Tech stack:** python, node, rust
**Frameworks:** axum
**Suggested tier:** T1 (medium confidence) — review required
**Tier rationale:** deploy platform detected; external-facing framework
**Deploy:** docker, github_actions

### Directory Structure

```
engram/
├── src/ (engram-core), crates/ (engram-types), engram-wasm/, proto/
├── sdks/ (python/, typescript/), examples/, skills/, fuzz/, benches/
├── tests/ (Rust integration), scripts/ (ci.sh, MCP reference, quality/security checkers)
├── docs/ (docs/harness/ = agent harness), governance/
├── Makefile, justfile, Cargo.toml, deny.toml, rust-toolchain.toml
├── README.md, CONTRIBUTING.md, CHANGELOG.md
```

---

## Environment

<!-- [inferred] -->
- **Python:** 3.11+ recommended
- **Node.js:** 22.12+ recommended
- **Rust:** stable toolchain

---

## Harness Discipline (Required at Session Start)

Every agent session must begin with:

```bash
bash docs/harness/bin/bootstrap.sh
```

Then read in order: `docs/harness/SPEC.md`, `docs/harness/INVARIANTS.md`,
`docs/harness/WHAT_WE_DONT_DO.md`, `docs/harness/GATES.md`,
`docs/harness/CODE_REVIEW_POLICY.md`,
`docs/harness/security/anthropic-reference-harness.md`,
`docs/harness/README.md`, `docs/harness/progress.md`, the active plan,
`AGENTS.md`, root `INVARIANTS.md`, `STANDARDS.md`, and
`ERRORS_AND_LESSONS.md`.

This operational harness (inspired by proven backend patterns) ensures resumable, auditable, high-signal work across Claude Code CLI and Claude Code Sonnet reviewer sessions while keeping the same operational truth available to the team and its agents.

## Common Commands

<!-- [inferred] -->
```bash
bash docs/harness/bin/bootstrap.sh
bash docs/harness/bin/doctor.sh
bash docs/harness/bin/sensors.sh   # full gates: same steps as scripts/ci.sh + doctor (does not call just)
```

```bash
pip install -e sdks/python        # no root requirements.txt
(cd sdks/typescript && npm install)  # no root package.json
```

```bash
cargo build
cargo test
cargo run
make ci                           # scripts/ci.sh; `just ci` is an alias only if `just` is installed
```

---

## Testing

Test directories: `tests/` (Rust), `sdks/python/tests/`, `sdks/typescript/src/` (vitest).

```bash
cargo test
(cd sdks/python && pytest tests/)
(cd sdks/typescript && npm test)
```

---

## Code Architecture

The Rust core separates canonical SQLite storage, retrieval, MCP adapters and background work.
Markdown export/import share `src/portability/markdown/`; CLI and MCP adapt that core.

### Patterns
Keep database transactions local and bounded; perform filesystem/network preparation first.
Enforce transport/workspace policy in adapters and stored-workspace checks at the data boundary.

### Key Modules
`storage` owns canonical data; `search`/`embedding` derive retrieval state; `mcp` exposes tools.
`intelligence`, `hooks` and `sync` handle consolidation, lifecycle and optional replication.

### Data Flow
MCP/HTTP/gRPC → permission/workspace guard → domain/storage → response.
CLI invokes the same domain core. Cloud backup uses SQLite snapshots.

---

## Invariants

> **Iron Law:** Read before writing. Understand existing code before changing it.

- Validate external input at system boundaries
- Never silently swallow errors — log or propagate with context
- Prefer dry-run for operations with external side effects
- Document decisions that affect future tasks
- Read local CLAUDE.md before modifying scoped code

---

## Decision Heuristics

When in doubt, apply these in order:

1. **Reversible over perfect** — prefer actions you can undo over waiting for certainty
2. **Smallest viable change** — solve the immediate problem, nothing more
3. **Existing patterns over new abstractions** — follow what the codebase already does
4. **Explicit failure over silent success** — if unsure something worked, make it loud
5. **Data over debate** — run the test, check the log, read the error
6. **Ask over assume** — when a decision has consequences you cannot reverse, ask the user

---

## Verification Standard

> **Iron Law:** Evidence before claims, always.

- Run the actual command — don't assume success
- Fresh verification after every change — stale results are lies
- Independent verification — don't trust agent output without checking
- Verify at every layer the data passes through (defense-in-depth)

---

## Red Flags

If you catch yourself thinking any of these, **STOP and follow the process:**

- "This is just a quick fix" → Follow the full process anyway
- "I don't need to test this" → You definitely need to test this
- "It should work now" → RUN the verification
- "One more attempt should fix it" → 3+ failures = architectural problem, step back
- "Too simple to need a plan" → Simple changes break complex systems
- "I'll clean it up later" → Later never comes. Do it right now

---

## Stuck Protocol

If you have tried **3+ approaches** to the same problem without progress:

1. **Stop** — do not attempt another fix
2. **Document** the blocker: what you tried, what failed, what you suspect
3. **List** remaining untried approaches (if any)
4. **Skip** — move to the next task or ask the user for guidance

Spinning without progress is the most expensive failure mode. Detecting it early is critical.

---

## Key Decisions

| Decision | Rationale | Status |
|----------|-----------|--------|
| Agent harness hardening v1: writer evidence bound to the commit, static/read-only default, runners only with a fake writer | No self-approval; no autonomous execution without ADR, sandbox and target contract (`docs/decisions/2026-07-21-agent-harness-hardening-v1.md`) | Accepted 2026-10-05 |
| Turso/libSQL sunset (ADR-STORAGE-20260816-1): SQLite is the only backend | Drop an unmaintained second backend (`docs/adr/2026-08-16-turso-backend-deprecation-and-sunset.md`) | Accepted; done in `08cb17a4`; post-gate pending human receipt |
| WAL replay budget 64 GiB, configurable | Bounds sparse-file and decompression abuse (root `INVARIANTS.md` #25, `ReplayLimits`) | Active (owner, 2026-10-05) |
| `defer_embedding=true` queues a background job | Code keeps the catalog promise (`CHANGELOG.md` C7; `docs/harness/progress/2026-10-05-improvement-lane-r.md`) | Active (owner, 2026-10-05) |
| `default = ["openai"]`, restricted loopback principal, char-based chunking | Lean default build; by-ID tools stay admin-only (`docs/decisions/2026-10-14-default-features-loopback-and-chunking-signoff.md`) | Accepted 2026-10-14 |

---

## Active Risks

State after PRs #266-#276 on `main` (`37ef480a`) plus the 2026-11-12 pre-`v0.24.0` remediation candidate; release status: `docs/releases/v0.24.0-readiness.md`.
Risk disposition and receipt scopes: `docs/harness/risk-register.yaml`, `docs/harness/known-issues/`.

- **Authz regression from `e04d3307`** (RISK-0003): fixed via #247, #254 and #255; stays listed until the post-gate receipt.
- **Keyless HTTP, read-only, and scope gaps** (A-1..A-7, Q-A): fixed across #268 and the 2026-11-12 pre-`v0.24.0` candidate (`Host`/`Origin` + `CorsLayer` loopback alignment, `palace_visualize` write gating/atomic writes/`write_flag_params`, `ENGRAM_MEDIA_BASE_DIR` confinement on `memory_search_by_image`, workspace-scoped `/v1/events`, context assembly/graph confinement, `memory_digest`/`harness_status`/`memory_agent_status` guards, compression `ID_SCOPED_TOOLS`, single-ID `decay`, and `read_only` hook suppression).
- **False success** (B-1/B-2): `memory_consolidate_batch` is report-only and `memory_embedding_migrate` refuses writes unless `dry_run=true` since #270; real merge and re-embedding are not implemented.
- **Unredacted logs** (RISK-0002): sync/cloud, hooks, multimodal and watcher swept; use `observability::redact` in new logs.
- **No required review on `main`** (D8, RISK-0006): `enforce_admins=false`, no PR review rule; H1 human receipts remain pending for the 2026-10-27 stack, #266-#276, and the pre-`v0.24.0` candidate.
- **C4 SQLite descriptor-bound open** (RISK-0007): ADR still *Proposed*; Invariant #27 mitigates by path/inode checks.
- **Advisories** (RISK-0008): RUSTSEC-2026-0459 fixed by #267 (rusqlite 0.41.1); secret scanning, push protection, Dependabot and private vulnerability reporting enabled 2026-11-10.
- **Unreleased Markdown export/import fixes** (RISK-0010, RISK-0011): `v0.21.2` remains the latest published release tag; fixes are prepared for `v0.24.0` while draft advisories `GHSA-8xxr-p5q8-c7cg` and `GHSA-6q7w-w273-gq6p` wait for the signed `v0.24.0` release.
- **Evidence provenance** (RISK-0009): several PASS verdicts lack a human H1 receipt.

---

## Formatting Standards

- Use consistent indentation (spaces or tabs, not mixed)
- Maximum line length: 100 characters
- Files end with a single newline
- No trailing whitespace
- Use descriptive variable and function names
- Keep functions focused — one responsibility per function
- Prefer explicit over implicit

---

## Pre-Task Protocol

### Announce at Start

Before writing any code, announce:

1. **What approach** you are using (fix, feature, refactor, etc.)
2. **Which files** you expect to modify
3. **What verification** you will run when done

### Checklist

Before starting any task:

- [ ] Read ERRORS_AND_LESSONS.md for known pitfalls
- [ ] Check if a local CLAUDE.md exists in the working directory
- [ ] Understand the existing code before making changes
- [ ] Run tests after changes to verify nothing broke
- [ ] Keep changes minimal and focused on the task
- [ ] Verify `.env` configuration is up to date

### Post-Task

Before ending a session or completing a task:

- [ ] Update ERRORS_AND_LESSONS.md if you hit a non-obvious problem
- [ ] Record any decision that constrains future work in Key Decisions
- [ ] If work is incomplete, leave a clear note about what remains
- [ ] Run final verification to confirm nothing is broken

---

## Parallel Development

Use git worktrees for parallel tasks without branch-switching conflicts:

```bash
claude --worktree feature-name    # isolated worktree + Claude session
claude -w bugfix-123 --tmux       # worktree in tmux session
git worktree list                 # see all active worktrees
```

- Each worktree gets its own branch and working directory
- Worktrees share git history — no duplicate clones
- Focus independent tasks in parallel — avoid editing same files
- Cleanup is automatic when Claude session ends without changes

---

## Provenance

Assembled from `README.md` and hand updates through 2026-11-12 (commands, Key Decisions, Active Risks).

---

## Document Information

**Last Updated:** 2026-11-12
**Generated by:** claude-primer v1.8.1
