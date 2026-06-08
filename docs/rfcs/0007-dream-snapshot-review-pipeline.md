# RFC 0007: Dream Snapshot Review Pipeline

## Status

Proposed

## Context

Engram already has the primitives needed for a memory synthesis layer:

- canonical memories in SQLite, FTS, vector indexes, and graph edges;
- `dream-phase` scheduling and advisory locking;
- `memory_policy` scoring for salience, retention, retrieval priority, and
  conflict risk;
- `enrichment_events` for append-only audit of automated enrichment;
- Operational Context events, summaries, bundles, and retained artifact access;
- temporal graph and contradiction detection;
- harness records for project decisions, verification, handoffs, and risks.

The missing product layer is not more raw storage. It is a reviewable synthesis
workflow that turns existing memory and operational evidence into candidate
updates while keeping canonical truth explicit and inspectable.

Claude Dreams is useful as a job-lifecycle reference: asynchronous jobs, input
store separate from output store, progress inspection, cancellation, archival,
and partial output preserved on failure. OpenAI's memory dreaming work is useful
as a quality target: carry forward context, follow durable preferences and
constraints, and stay current as time passes.

## Decision

Engram will add a Dream Snapshot Review Pipeline.

Dream jobs read canonical memories, memory policy records, temporal signals,
context events, context summaries, artifacts, enrichment events, and harness
records. They emit review candidates into dedicated dream tables. Candidates are
derived proposals, not accepted memories.

No dream job may silently create, update, delete, expire, promote, demote, or
supersede canonical memory rows. Applying a candidate requires explicit review
and confirmation through a mutating MCP tool.

## Product Boundary

Dream snapshots should help agents answer:

- what durable project context should carry forward;
- which memories are stale, contradicted, duplicated, or low-value;
- which preferences and constraints are stable enough to surface early;
- which synthesized summaries are useful but still need provenance;
- why a candidate exists and what evidence supports it.

Dream snapshots should not:

- replace source systems such as Git, Huly, CI, repository docs, AGENTS.md, or
  harness docs;
- store raw logs, raw transcripts, terminal dumps, or secrets by default;
- treat model output or heuristic output as canonical truth;
- auto-accept high-confidence candidates in v1;
- hide rejected or edited candidate decisions;
- mutate source memories without explicit confirmation.

## Core Types

### DreamJob

A `DreamJob` is one synthesis run over a scoped set of inputs.

Required fields:

- `id`: stable job identifier;
- `workspace`: workspace being processed;
- `status`: `pending`, `running`, `completed`, `failed`, `canceled`, or
  `archived`;
- `instructions`: optional high-level synthesis policy;
- `model_profile`: deterministic local profile by default;
- `input_summary`: counts and selectors used for the job;
- `output_summary`: candidate counts and aggregate metrics;
- `error`: structured failure details when applicable;
- `created_at`, `started_at`, `finished_at`, and `archived_at`.

Job statuses are monotonic except that terminal jobs may be archived.
Cancellation is idempotent. Failed jobs retain any candidates that were already
emitted so users can inspect partial work.

### DreamCandidate

A `DreamCandidate` is a proposed memory action.

Required fields:

- `id`: stable candidate identifier;
- `job_id`: parent dream job;
- `workspace`: candidate workspace;
- `kind`: `summary`, `preference`, `constraint`, `project_state`,
  `stale_fact`, `contradiction`, `merge`, `promotion`, `decay`, or
  `temporal_update`;
- `proposed_action`: `create`, `update`, `merge`, `supersede`, `expire`,
  `promote`, `demote`, or `ignore`;
- `review_state`: `pending`, `accepted`, `edited`, `rejected`, `applied`, or
  `archived`;
- `confidence`: bounded `0.0..=1.0` score;
- `freshness_state`: `current`, `stale`, `future_due`, `expired`,
  `conflicted`, or `unknown`;
- `content_preview`: short inspectable text;
- `proposed_content`: optional full proposed content;
- `reason_codes`: stable machine-readable reason codes;
- `policy_explanation`: memory-policy explanation when source memories exist;
- `metadata`: sensitivity, retention, reducer, and scope metadata;
- timestamps for creation, review, and application.

Candidate content must be non-empty when the proposed action would create or
update canonical memory content. Candidates whose action targets existing state
must include target identifiers in `metadata.target_memory_ids` or the relevant
target-specific field described below.

### DreamCandidateSource

A `DreamCandidateSource` links a candidate to evidence.

Examples:

- source memory id;
- context event id;
- context artifact id;
- context summary id;
- enrichment event id;
- harness record id;
- document path and section;
- commit, issue, pull request, or CI run reference.

Every candidate must have at least one source unless it is explicitly marked as
low-trust scratch output with short retention. Low-trust scratch candidates are
not applied by default.

## Candidate Lifecycle

Default lifecycle:

1. `dream_create` creates a `pending` job.
2. The job runs and emits `pending` candidates.
3. Users or agents inspect candidates through read-only MCP tools.
4. `dream_candidate_review` marks a candidate accepted, edited, rejected, or
   archived.
5. `dream_candidate_apply` applies an accepted or edited candidate only when
   called with `confirm=true`.
6. Application emits audit rows and records the accepted canonical memory ids or
   lifecycle updates.

Rejected candidates remain useful feedback. Later jobs may use prior rejected
candidate metadata to avoid repeating the same proposal.

## Candidate Application Contract

`dream_candidate_apply` is deliberately narrow in v1. It only applies
`accepted` or `edited` candidates, requires `confirm=true`, and is idempotent:
if a candidate is already `applied`, the tool returns the recorded application
result instead of mutating canonical state again. `rejected`, `pending`, and
`archived` candidates are never applied.

Per-action semantics:

| Proposed action | Required target fields | Effect on apply |
|---|---|---|
| `create` | `proposed_content` | Creates a new canonical memory from the reviewed content and records the new memory id. |
| `update` | exactly one `metadata.target_memory_ids[]`, `proposed_content` | Creates a new canonical memory version or replacement row using existing memory update semantics; original memory id is recorded as superseded/updated provenance. |
| `merge` | two or more `metadata.target_memory_ids[]`, `proposed_content` | Creates one merged canonical memory and records all merged source memory ids; source rows are not deleted. |
| `supersede` | one `metadata.target_memory_ids[]`, optional `metadata.superseded_by_memory_id` or `proposed_content` | Marks the target as superseded when a canonical successor exists, or creates the successor from `proposed_content` first. |
| `expire` | one or more `metadata.target_memory_ids[]`, `metadata.expiration_reason` | Applies the existing lifecycle/policy mechanism for expiry or decay state without deleting source rows. |
| `promote` | one or more `metadata.target_memory_ids[]`, optional policy metadata | Raises retention/retrieval policy through existing `memory_policy` semantics; content is unchanged. |
| `demote` | one or more `metadata.target_memory_ids[]`, optional policy metadata | Lowers retention/retrieval policy through existing `memory_policy` semantics; content is unchanged. |
| `ignore` | none | No canonical mutation; records an applied no-op only for audit when the reviewer intentionally closes the candidate. |

`update`, `merge`, and `supersede` never rewrite history in place unless an
existing storage API already models that operation as versioned and auditable.
When the existing API cannot express the action safely, apply must fail with a
structured unsupported-action error and leave the candidate review state
unchanged. All successful applications set `review_state='applied'`,
`applied_at`, and an application result in candidate metadata or an audit row.

## Data Model

The implementation should add dedicated tables:

```sql
dream_jobs (
    id TEXT PRIMARY KEY,
    workspace TEXT NOT NULL,
    status TEXT NOT NULL,
    instructions TEXT,
    model_profile TEXT NOT NULL DEFAULT 'deterministic-local-v1',
    input_summary_json TEXT NOT NULL DEFAULT '{}',
    output_summary_json TEXT NOT NULL DEFAULT '{}',
    error_json TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    finished_at TEXT,
    archived_at TEXT
);

dream_candidates (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES dream_jobs(id) ON DELETE CASCADE,
    workspace TEXT NOT NULL,
    kind TEXT NOT NULL,
    proposed_action TEXT NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'pending',
    confidence REAL NOT NULL,
    freshness_state TEXT NOT NULL DEFAULT 'unknown',
    content_preview TEXT NOT NULL,
    proposed_content TEXT,
    reason_codes TEXT NOT NULL DEFAULT '[]',
    policy_explanation_json TEXT NOT NULL DEFAULT '{}',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    reviewed_at TEXT,
    applied_at TEXT
);

dream_candidate_sources (
    candidate_id TEXT NOT NULL REFERENCES dream_candidates(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    source_ref TEXT,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (candidate_id, source_type, source_id)
);
```

Implementation issues may refine column names and checks, but the separation
between unaccepted candidates and accepted memories is required.

## MCP Surface

Initial tools:

- `dream_create`: create and optionally run a reviewable dream job;
- `dream_get`: inspect one job;
- `dream_list`: list jobs by workspace and status;
- `dream_cancel`: cancel pending or running jobs idempotently;
- `dream_archive`: archive terminal jobs;
- `dream_candidates_list`: list candidates with filters;
- `dream_candidate_get`: inspect one candidate and its sources;
- `dream_candidate_review`: accept, edit, reject, or archive a candidate;
- `dream_candidate_apply`: apply an accepted or edited candidate with
  `confirm=true`;
- `dream_eval_run`: run fixed local evaluation fixtures.

Existing `dream_run_now` remains available under the `dream-phase` feature for
compatibility. It should not be removed or silently repurposed in v1.

## Freshness Semantics

Dream candidates must be time-aware.

Examples:

- A future planned event becomes `future_due` when its date is near.
- A future planned event becomes `expired` or `stale` after its date passes
  without confirmation.
- A later confirmed event may produce a `temporal_update` candidate.
- Overlapping temporal claims may produce a `contradiction` candidate.

Freshness logic must use RFC3339 UTC internally and must not panic on malformed
metadata, invalid timestamps, empty queries, or missing source records.

## Provenance And Audit

Every candidate creation, review decision, and application must emit or preserve
auditable provenance.

Use `enrichment_events` for append-only audit rows. Candidate application must
record:

- candidate id;
- job id;
- reviewer or agent id when available;
- source ids;
- applied canonical memory ids or lifecycle state changes;
- dry-run status;
- result status and error details.

Default `memory_search`, `context_search`, and `context_build_bundle` results
must not include unaccepted dream candidates. Candidate summaries may appear
only in explicit dream tools or in future opt-in parameters that label them as
proposals, not facts. Applied candidates can surface through the canonical
memory rows they created or updated. Raw candidate source artifacts are
retrieved only through explicit artifact tools when retention and access policy
allow it.

## Safety Defaults

- Local deterministic synthesis is the default profile.
- External LLM synthesis is out of scope for v1 unless a later RFC defines
  privacy, cost, failure, and provenance rules.
- Candidate application requires `confirm=true`.
- Raw payloads are excluded by default.
- Secrets and environment dumps are never stored.
- Candidates with missing provenance are low-trust and short-retention.
- Candidate review history is retained.

## Evaluation Requirements

The evaluation suite should measure:

- carry forward useful context;
- follow durable preferences and constraints;
- stay current over time;
- preserve provenance correctness;
- reject unsafe raw-log capture;
- avoid canonical mutation before explicit apply.

The evals are local and deterministic in v1. They should not depend on a paid or
networked model provider.

## Rollout

Implement in this order:

1. Storage schema and lifecycle helpers.
2. Deterministic candidate generator.
3. MCP review tools.
4. Freshness engine.
5. Harness dogfooding and context bundle integration.
6. Eval suite and documentation.

Each phase must keep default canonical memory behavior backward-compatible.

## Non-Goals

- UI for candidate review.
- Automatic acceptance of candidates.
- Model-weight memory.
- External LLM default dependency.
- Raw transcript/log archive.
- Replacing Operational Context or Harness Memory RFCs.
- Replacing existing source systems.

## Open Questions

- Which candidate source types should be required for minimum trust?
- Should rejected candidates influence future candidate generation in v1 or v2?
- Should dream jobs be scheduled automatically, or remain explicitly triggered
  until the review workflow is proven?
