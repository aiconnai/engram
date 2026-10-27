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
- **Node.js:** 18+ recommended
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

<!-- [placeholder] -->

### Patterns
<!-- Ex: MVC, Clean Architecture, Event-driven, Layered, etc. -->

### Key Modules
<!-- List the main modules/packages and their responsibilities -->

### Data Flow
<!-- Describe the primary data flow of the application -->

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

State on `remediation/integration`, 2026-10-26 (not yet on `main`). Detail: `docs/harness/risk-register.yaml`, `docs/harness/known-issues/`.

- **Authz regression from `e04d3307`** (RISK-0003): a restricted principal could write to a foreign workspace via `memory_create_batch` / `memory_ingest_fact_batch`. Fixed on `remediation/integration` (`c50b552e`, `f53b75a0`, gRPC e2e `83e3816c`); stays listed until a human merge puts it on `main`.
- **Unredacted logs** (RISK-0002): sync/cloud, hooks, multimodal and watcher were swept (41 sites, capture tests); the rest of the repository still needs a sweep; use `observability::redact` in new logs.
- **No required review on `main`** (D8, RISK-0006): `enforce_admins=false`, no PR review rule; human merge is process, not a technical control.
- **C4 SQLite descriptor-bound open** (RISK-0007): the ADR is still *Proposed*; Invariant #27 mitigates by path only.
- **Advisories** (RISK-0008): resolved on `remediation/integration` (`cargo audit` 0 vulnerabilities, `cargo deny` ok, no new exception; 2 governed exceptions expire 2026-12-31). Scheduled CI on `main` stays red until the merge.
- **Evidence provenance** (RISK-0009): the 2026-10-14 verdict and the sunset review are PASS without a human receipt; `.sensors-log` has no row from 2026-09-03 to 2026-10-23.

---

## Formatting Standards

<!-- [placeholder] -->
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

<!-- [placeholder] -->
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

Content in this file was assembled from:

- `README.md`
- hand corrections of 2026-10-26 (structure, commands, Key Decisions, Active Risks)

Sections containing `migrated` in a comment came from existing files — verify accuracy.
Sections containing `inferred` were detected from project structure — may need correction.
Sections containing `placeholder` need manual input.

---

## Document Information

**Last Updated:** 2026-10-26
**Generated by:** claude-primer v1.8.1
