# RFC 0001: Harness Memory Product Boundary

## Status

Proposed

## Context

Harness Memory is the product layer that turns repository and delivery activity into durable, searchable project memory for AI agents and maintainers. Engram already provides the storage, search, graph, and MCP primitives. This RFC defines what Harness Memory is allowed to capture and how it relates to the surrounding developer workflow.

This is a boundary document only. It is intended to guide ENGRA-22, ENGRA-23, ENGRA-24, and ENGRA-25, but it does not implement ingestion, automation, schemas, MCP tools, Huly sync, GitHub sync, or CI jobs.

## Decision

Harness Memory stores curated project memory derived from engineering workflow events. Its scope is to preserve decisions, context, outcomes, and traceable references that help future agents and maintainers understand why work happened and what changed.

Harness Memory is not a general telemetry lake, secret store, log archive, personal activity tracker, or replacement for source systems such as GitHub, Huly, CI, repository documentation, or AGENTS.md.

## Product Boundary

Harness Memory should capture:

- durable engineering context that remains useful after the current session ends;
- references to source-of-truth systems rather than full copies when the source system remains available;
- normalized summaries of work, decisions, constraints, and verification results;
- relationships between work items, commits, pull requests, documents, CI runs, and agent sessions;
- enough metadata to evaluate provenance, recency, trust, and scope.

Harness Memory should not capture:

- every command, token, prompt, completion, log line, or transient tool output;
- raw artifacts when a summary plus source reference is sufficient;
- private environment values, credentials, access tokens, cookies, SSH keys, API keys, or signing material;
- unrelated personal data or personal communications that are not required to understand repository work;
- raw logs or stack traces before sensitive values have been filtered;
- source code snapshots by default when Git already provides durable version history.

## Event Types

The initial product boundary recognizes these event families. Later implementation work may add exact schemas, retention policies, and MCP/API surfaces.

### Work Item Events

Track issue-level planning and execution state.

Examples:

- Huly issue created, triaged, scoped, started, blocked, unblocked, completed, or reopened;
- backlog item linked to a GitHub issue, pull request, branch, commit, RFC, ADR, or CI run;
- issue scope changed in a way that affects future implementation.

Recommended metadata:

- source system and source identifier;
- title, status, priority, assignee or agent identifier when relevant;
- repository, branch, milestone, labels, and parent/child links;
- timestamps for created, updated, started, completed, and observed-at;
- concise summary of scope, acceptance criteria, blockers, and outcome.

### Agent Session Events

Track durable session outcomes, not raw transcripts.

Examples:

- agent accepted a task;
- agent discovered a constraint from AGENTS.md, repository docs, code, or issue context;
- agent made a product or technical decision;
- agent completed work with verification evidence;
- agent handed off a blocker or follow-up.

Recommended metadata:

- agent identifier, session identifier, workspace, repository, and branch;
- task or issue identifiers;
- files or documents referenced;
- decisions made, assumptions, constraints, and unresolved questions;
- commands run and summarized results when relevant;
- verification status and evidence links.

### Documentation Events

Track durable documentation intent and relationship to implementation.

Examples:

- AGENTS.md instructions changed;
- RFC, ADR, architecture, operations, schema, or user guide changed;
- documentation establishes a boundary, policy, invariant, or migration path.

Recommended metadata:

- document path, section, commit or pull request reference;
- type of document: AGENTS, RFC, ADR, guide, reference, invariant, operation, schema;
- summary of the policy or decision;
- affected product area, feature, or workflow;
- supersedes, depends-on, or related-document links.

### GitHub Events

Track source-control and review milestones.

Examples:

- pull request opened, reviewed, approved, merged, closed, or reverted;
- commit associated with an issue or agent session;
- release tag created;
- review comment creates a required follow-up.

Recommended metadata:

- repository, owner, branch, commit SHA, pull request number, issue number, and actor;
- event action and event time;
- title and concise summary;
- changed path list or high-level area when available;
- review state, merge state, and source URL;
- linked Huly issue, RFC, ADR, CI run, or agent session.

### CI and Verification Events

Track verification outcomes relevant to future work.

Examples:

- required CI workflow succeeded, failed, or was skipped;
- lightweight documentation sanity check completed;
- release, benchmark, type-check, lint, or test result associated with a pull request;
- failure pattern summarized for later diagnosis.

Recommended metadata:

- CI provider, workflow, job, run identifier, and source URL;
- repository, branch, commit SHA, pull request number, and triggering actor;
- status, conclusion, duration, and observed-at;
- command or job name, not raw secret-bearing logs;
- concise failure summary after redaction;
- artifacts retained by reference only.

## Relationship to Source Systems

### AGENTS.md

AGENTS.md remains the source of truth for agent-facing repository instructions. Harness Memory may store summaries, changes, and references to AGENTS.md, but it must not override or silently reinterpret current instructions.

When AGENTS.md changes, Harness Memory should treat the change as a documentation event with provenance. Future agents should still read AGENTS.md directly before acting.

### Repository Documentation

Repository docs remain the canonical source for architecture, schemas, operational procedures, and usage instructions. Harness Memory may index decisions and relationships between docs and work items, especially when a document explains why a product boundary exists.

Harness Memory should prefer document path, section, commit, and short summary over copying full documents.

### GitHub

GitHub remains the source of truth for commits, branches, pull requests, reviews, issues, releases, and code history. Harness Memory should store durable summaries and links that connect GitHub activity to work items, agent sessions, documentation, and CI results.

Harness Memory should not duplicate repository contents or treat stored summaries as authoritative source code state.

### CI

CI remains the source of truth for workflow configuration, job logs, artifacts, and status checks. Harness Memory should store normalized verification outcomes and redacted failure summaries that help future agents understand what passed, failed, or was not run.

Harness Memory should not store raw CI logs by default.

## Default Exclusions

Harness Memory must not store these by default:

- secrets, credentials, private keys, API tokens, OAuth tokens, cookies, session identifiers, signing certificates, or password material;
- private environment values, including full `.env` contents or command output that prints environment variables;
- raw logs, raw stack traces, debug dumps, crash reports, or CI artifacts before filtering for sensitive data;
- unrelated personal data, private messages, calendar data, emails, or local machine details unrelated to repository work;
- proprietary third-party content copied from systems where a reference is sufficient;
- full source files, generated MCP files, build outputs, dependency caches, or binary artifacts unless a later RFC explicitly defines a safe, limited use case.

Implementations should default to reference, summarize, and redact. Capturing raw content should require an explicit allowlist, a documented reason, and a retention policy.

## Metadata Principles

Every stored Harness Memory event should include:

- source system and source identifier;
- repository or workspace scope;
- event type and event time;
- observed-at time;
- actor or agent identifier when relevant;
- provenance URL or file reference when available;
- concise summary;
- sensitivity classification;
- links to related work items, commits, pull requests, docs, CI runs, or agent sessions.

Metadata should be structured enough to support search, graph relationships, retention, and future conflict detection.

## Non-Goals

This RFC does not define:

- database schema changes;
- MCP tool definitions;
- Huly automation;
- GitHub webhook processing;
- CI workflow implementation;
- embedding or retrieval behavior;
- retention enforcement;
- UI behavior.

Those belong to follow-up implementation issues after the product boundary is accepted.

## Foundation for Follow-Up Work

ENGRA-22 through ENGRA-25 should use this RFC as the constraint set for implementation planning. In particular:

- ingestion should map source events into the event families above;
- storage should preserve provenance and sensitivity metadata;
- automation should avoid default raw-content capture;
- docs and MCP surfaces should describe Harness Memory as curated project memory, not source-system replacement.

## Open Questions

- Which sensitivity labels should be standardized first?
- Which event families require retention controls in the first implementation pass?
- Which source references should be required versus optional for local-only workflows?
