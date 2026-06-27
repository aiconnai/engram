# Reference Intake Checklist

Use this checklist when Engram harness work cites, imports lessons from, or
adapts an external harness, standard, article, repo, benchmark, or tool catalog.
It keeps external material useful without turning it into an implicit dependency,
license risk, or unreviewed process change.

This checklist is evidence for review. It does not approve the change by itself.
The local harness invariants, gates, security boundary, and negative scope still
win over any external source.

## When It Applies

Apply this checklist when a change:

- adds a new external reference to `docs/harness/**`, `AGENTS.md`, `CLAUDE.md`,
  or project governance docs;
- uses an external source to justify a harness gate, reviewer rule, skill,
  sensor, taxonomy, or exception process;
- imports ideas from an awesome list, benchmark suite, lifecycle standard,
  agent framework, MCP server, or prompt/workflow kit;
- creates a local-only reference file or ignored source artifact.

Routine links in user-facing product docs do not need this checklist unless they
change engineering process, gates, or agent instructions.

## Intake Record

Record the following in `progress.md`, the active plan, or the matching Review
Canvas:

| Field | Required evidence |
|---|---|
| Source identity | URL, owner/publisher, date read, and commit/tag/version when available |
| Source type | Primary implementation, primary article, benchmark, standard, awesome list, secondary article, or marketing page |
| License boundary | License if known; whether copying text, prompts, scripts, commands, or checklists is prohibited |
| Harness relevance | Which harness primitive it affects: context, memory/state, verifier, tool registry, runtime control, security, evals, workflow, or governance |
| Placement | Where the adapted idea belongs locally and why adjacent sections were rejected |
| Adaptation | The Engram-local rule, gate, checklist, or follow-up derived from the source |
| Exclusions | What is explicitly not being imported, vendored, executed, or treated as authoritative |
| Verification | `doctor.sh`, sensors lane, diff check, reviewer evidence, or explicit reason no executable verification applies |

## Include / Exclude Rubric

Include a source when it is:

- directly about constraining, resuming, evaluating, observing, orchestrating, or
  safely running agents;
- a primary source or inspectable implementation, or a secondary source with a
  clearly unique synthesis;
- specific enough to become an Engram-local criterion, checklist item, or
  follow-up task;
- compatible with the local security boundary and license policy.

Exclude or defer a source when it is:

- generic AI commentary, model-launch material, or agent-framework marketing
  without concrete harness design guidance;
- duplicative with an existing source and less primary or less practical;
- under a license that would make copying unsafe without explicit review;
- useful only if Engram adopted a new runtime, service, credentialed workflow,
  or autonomous execution path outside the current scope.

## Placement Guidance

Use the narrowest local home:

- **Memory / State / Resumability** — durable state, handoff records, campaign
  state, progress ledgers, replay, and continuity across sessions.
- **Context Engineering** — token budget, context shape, working memory, file
  selection, compaction, and retrieval inputs.
- **Verifier / Evals / Observability** — deterministic checks, trace grading,
  benchmarks, measurement, logs, dashboards, and review evidence.
- **Runtime / Tool Registry / MCP** — tool boundaries, tool discovery, sandboxed
  execution, dispatch, adapters, and protocol contracts.
- **Security / Safe Autonomy** — sandboxing, egress, credentials, prompt
  injection, capability risk, and autonomous-execution boundaries.
- **Workflow / Governance** — agent instructions, specs, PR/review flow,
  progress discipline, decision logs, and exception processes.

If a source fits several sections, record the rejected placements in the Review
Canvas or progress log before adding broad policy language.

## Exception Discipline

Avoid inline unexplained exceptions. If a source needs special handling, record:

- the exact URL/path or class of source;
- why the exception exists;
- owner or reviewer;
- date added and review cadence;
- removal condition.

Examples: link-check allowlists, local-only licensed references, WAF-blocked
sources, temporary benchmark exclusions, or copied-source prohibitions.

## Reviewer Rule

Reviewers should flag missing reference-intake evidence as:

- `[HIGH]` when a harness/process change relies on an external source without an
  intake record;
- `[BLOCKER]` when the change copies licensed text/prompts/scripts, weakens
  gates, imports autonomous execution, or makes an external source authoritative
  over local invariants.
