# RFC 0005: Engram Operational Context

## Status

Proposed

## Context

Coding agents repeatedly rediscover operational context across sessions: what was
already tried, which command outputs mattered, which artifacts explain a
decision, what verification was skipped, and which previous context is now stale.
Engram already provides persistent memory, hybrid retrieval, graph relationships,
MCP tools, and provenance-oriented storage. Operational Context defines the
product contract for applying those primitives to agent engineering workflows.

Engram Operational Context is MCP-first, provenance-backed operational memory for
coding agents. It is designed to make future sessions resume work with less
rediscovery and better evidence, without turning Engram into a shell recorder,
terminal proxy, or raw log warehouse.

This RFC is a product/spec contract only. It does not implement storage schema,
MCP tools, hooks, adapters, reducers, SDK calls, retention enforcement, or UI.

## Decision

Engram will treat Operational Context as a curated layer of operational memory
derived from coding-agent activity. The canonical product shape is:

- events that describe operational facts observed during agent work;
- artifacts that preserve references, summaries, and optional raw payloads;
- summaries that compress evidence into lossy, attributed context;
- bundles that assemble scoped context for later sessions.

The MVP is MCP-first. Agents should read and write Operational Context through
MCP tools or MCP-compatible transports. HTTP may exist as an MCP transport, but
the MVP is not a REST CRUD product and not a CLI proxy.

Operational Context is also not an RTK clone. RTK can inform adapter design and
noise-reduction patterns, but Engram's product boundary is durable operational
memory with provenance, scope, retention, and retrieval semantics.

## Product Boundary

Operational Context should help agents answer:

- what happened in prior sessions;
- why a decision was made;
- which command, file, issue, PR, review, or artifact supports a claim;
- whether stored context is current enough to trust;
- which raw evidence exists, if access is explicitly allowed;
- which facts are summaries, not authoritative source state.

Operational Context should not:

- capture every command, stdout line, stderr line, prompt, completion, or token;
- proxy shell commands or rewrite user commands in the MVP;
- replace Git, CI, issue trackers, repository docs, AGENTS.md, or harness docs;
- promise first-pass command-output compression before an agent sees output;
- store raw logs or artifacts by default;
- treat derived summaries as source-of-truth state.

## Problem Solved

Operational Context solves cross-session rediscovery. It helps a later agent find
the important parts of earlier work after the original terminal output,
conversation, or local scratch state is gone.

Operational Context does not primarily solve first-pass command-output bloat.
In the MVP, a command may still produce verbose output during the session that ran
it. Engram can record a summarized, provenance-backed representation afterward,
but it is not the first-pass shell-output filter unless a later adapter explicitly
defines that flow.

## Core Types

### ContextEvent

A `ContextEvent` is a structured operational fact observed during agent work.

Required fields:

- `id`: stable event identifier;
- `kind`: event family, such as `command_observed`, `file_read`,
  `file_changed`, `decision_made`, `verification_run`, `verification_skipped`,
  `review_result`, `blocker_found`, `handoff_created`, or `context_invalidated`;
- `scope`: repository, worktree, branch, issue, task, agent session, or user
  scope;
- `occurred_at`: when the underlying activity happened;
- `observed_at`: when Engram observed or received it;
- `actor`: human, agent, hook, adapter, or system identity;
- `summary`: concise operational statement;
- `provenance`: source reference required by default;
- `sensitivity`: classification used for retention and access policy.

Recommended fields:

- `related_files`: paths referenced by the event;
- `related_artifacts`: artifact identifiers or external URLs;
- `related_work_items`: issue, PR, RFC, ADR, or ticket identifiers;
- `confidence`: confidence in the derived statement;
- `staleness_policy`: how the event should be invalidated or downgraded.

### ContextArtifact

A `ContextArtifact` is evidence associated with one or more events.

Required fields:

- `id`: stable artifact identifier;
- `artifact_type`: `command_output_summary`, `raw_command_output`,
  `diagnostic_log`, `review_artifact`, `diff_reference`, `test_report`,
  `benchmark_report`, `screenshot`, `external_url`, or another explicit type;
- `scope`: the same isolation boundary used for related events;
- `provenance`: command, file, URL, CI run, review file, or source system;
- `retention_policy`: explicit policy, especially for raw artifacts;
- `sensitivity`: classification for access control;
- `created_at`: artifact creation time.

Artifacts may contain:

- a short summary;
- a normalized digest or hash;
- a pointer to a source-of-truth system;
- a redacted excerpt;
- a raw payload only when allowed by policy.

Raw artifact retention is off by default. If raw content is stored, the artifact
must carry an explicit reason, retention window, sensitivity label, and access
policy. Raw artifacts are not automatically included in context bundles.

### ContextSummary

A `ContextSummary` is a derived, lossy representation of events and artifacts.

Required fields:

- `id`: stable summary identifier;
- `source_event_ids`: events used to derive the summary;
- `source_artifact_ids`: artifacts used to derive the summary;
- `scope`: repository, worktree, task, or session boundary;
- `summary`: concise text intended for retrieval and agent handoff;
- `derivation_method`: reducer, model, template, or human process used;
- `created_at`: summary creation time;
- `provenance`: references back to the inputs.

Summaries are not source-of-truth state. They must be labeled as derived and
lossy. When a future agent needs authority, it should follow provenance to the
source event, artifact, repository file, CI run, review, issue, or commit.

### ContextBundle

A `ContextBundle` is a scoped package of operational context assembled for an
agent session or task.

Required fields:

- `id`: stable bundle identifier;
- `scope`: requested repository, worktree, branch, task, issue, or agent session;
- `query`: why the bundle was requested;
- `included_summary_ids`: summaries selected for the bundle;
- `included_event_ids`: events selected directly;
- `included_artifact_refs`: artifact references included directly;
- `excluded_items`: omitted events or artifacts with reasons when relevant;
- `budget`: token, item, or byte budget used to assemble the bundle;
- `created_at`: bundle creation time.

Bundles should include enough provenance for the receiving agent to audit claims.
They should also expose staleness signals, such as branch mismatch, file changed
since observation, commit mismatch, expired retention, or superseded decision.

## Safety Defaults

Operational Context defaults to safe, bounded memory:

- raw retention is off by default;
- summaries are derived, lossy, and labeled as such;
- provenance is required for stored operational claims;
- context is scoped by repository, worktree, branch, task, session, and actor
  where available;
- raw artifact access requires explicit permission and policy;
- redaction happens before storage of logs, command output, stack traces, and
  external artifacts;
- context bundles should prefer summaries and references over raw payloads;
- stale context must be detectable and visible to agents.

Default exclusions:

- secrets, credentials, tokens, cookies, SSH keys, private keys, signing
  material, and environment dumps;
- raw terminal transcripts and command output without redaction;
- raw CI logs or crash dumps without filtering;
- source file snapshots when Git is sufficient;
- private communications unrelated to repository work;
- third-party proprietary material when a reference is sufficient.

## Provenance Contract

Every event, artifact, summary, and bundle must make its evidence path explicit.
At minimum, provenance should identify the source system and source reference.
Examples include:

- repository path plus commit or worktree reference;
- command string plus normalized exit status and observation time;
- CI provider, workflow, job, and run URL;
- review artifact path;
- issue, PR, RFC, or ADR identifier;
- MCP tool call identifier or agent session identifier.

If provenance is missing, the item should either be rejected, stored as
low-trust scratch context with short retention, or clearly marked as
unattributed. Unattributed context must not be promoted into durable bundles by
default.

## RTK Relationship

RTK is an optional source, adapter, and design reference. It is not a hard
dependency, not the product surface, and not the required runtime for Engram
Operational Context.

RTK references named by ENGRA-67 should be treated as examples:

- `docs/contributing/ARCHITECTURE.md`: command lifecycle, filtering, and
  fail-safe behavior can inform future adapter contracts;
- `src/core/tracking.rs`: token/context savings tracking is a useful concept,
  but Engram must report honest metrics and avoid inflated savings claims;
- `src/core/tee.rs`: raw output recovery is a useful pattern when raw access is
  explicitly allowed, but Engram keeps raw retention off by default;
- `src/hooks/rewrite_cmd.rs`: shell interception illustrates why command
  rewriting and interception are deferred out of the MVP.

An RTK adapter may later emit `ContextEvent` and `ContextArtifact` records into
Engram. That adapter must obey Engram's redaction, retention, provenance, scope,
and access-control contracts.

## MVP Scope

In scope for a future MVP implementation:

- MCP-first creation and retrieval of operational events, artifacts, summaries,
  and bundles;
- summary-first storage of command results, verification outcomes, decisions,
  blockers, and handoffs;
- provenance links to files, commands, reviews, issues, PRs, CI runs, and agent
  sessions;
- explicit sensitivity, retention, and scope metadata;
- stale-context signals in retrieval and bundle assembly;
- reducer contracts that are deterministic enough to regression test.

Out of scope for the MVP:

- shell command interception or rewrite;
- CLI proxy mode;
- mandatory RTK runtime integration;
- raw command transcript storage by default;
- first-pass command-output filtering before the agent receives output;
- replacing existing source systems or repository harness docs.

## Launch Blockers

Operational Context should not launch until these blockers are resolved:

- redaction policy and implementation for command output, logs, stack traces,
  environment-like text, and external artifacts;
- retention policy for events, summaries, bundles, and especially raw artifacts;
- scope isolation across repositories, worktrees, branches, tasks, sessions, and
  actors;
- reducer regression tests that prove summaries preserve required operational
  facts and label lossiness;
- raw artifact access controls, including explicit allowlists, audit trail, and
  default exclusion from bundles;
- stale-context detection for file changes, branch changes, commit mismatch,
  superseded decisions, expired artifacts, and source-system drift;
- honest token metrics that distinguish observed size, stored size, retrieved
  size, included bundle size, and any claimed savings.

## Honest Token Metrics

Operational Context may report token or byte savings only with clear definitions.
Metrics should distinguish:

- `observed_input_size`: raw or source-system content observed before redaction;
- `stored_artifact_size`: bytes or tokens retained after redaction and policy;
- `summary_size`: size of derived summaries;
- `retrieved_context_size`: size retrieved before bundle filtering;
- `bundle_size`: size actually handed to the agent;
- `excluded_size`: size omitted because of budget, safety, scope, or staleness;
- `estimated_savings`: clearly labeled estimate, not a guarantee.

Savings claims must not count unavailable raw content as if it had been safely
stored, must not hide safety exclusions, and must not imply first-pass shell
compression when the benefit is cross-session retrieval.

## Open Questions

- Which sensitivity labels should be shared with Harness Memory from RFC 0001?
- Which MCP tools should be introduced first, and should they reuse existing
  memory tools or define a separate operational namespace?
- What minimum provenance is acceptable for local-only command events that do
  not have URLs?
- Which reducer regression corpus should become the launch baseline?
- Should stale-context checks be computed at retrieval time, bundle assembly
  time, or both?
