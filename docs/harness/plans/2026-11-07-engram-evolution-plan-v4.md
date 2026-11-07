# Engram evolution — revision 4, 2026-11-07

Status: updated execution proposal; individual contracts/operational decisions
below retain their own acceptance. Release closeout is the current execution task.
This revision replaces obsolete planning assumptions, not the historical records.

## Current base and completed prerequisites

Main `7ca7a9b8` contains PR264: release hardening, review-default Markdown import,
export/import filesystem and workspace protections, the shared Markdown core, and
cloud cancellation fixes. PH0/P0 implementation is integrated. Remote CI/live SDKs
and full manual Nightly41966511833 passed on that SHA. The separate SDK release
dry-run failed; [closeout](2026-11-07-release-closeout.md) handles its bounded fix.

The v3 proposal of 01/11 and RFC0004a remain untracked in the primary checkout;
their original bytes and opinions are preserved there. This self-contained revision
does not turn them into accepted policy. It uses the current repository's existing
RFCs and the [original backlog](2026-10-27-repository-execution-plan.md).

All IDs below use EV- to avoid collisions with the original plan's H1/S1/P1.

## Execution order and ownership

| ID / result | Input / owner | Verifiable completion |
|---|---|---|
| EV-B0 — current baseline | Integrator; accepted merged source and clean scoped worktree | Record refs/paths/receipts and assign exclusive ownership. Preserve earlier worktrees and old review scopes. Reserve migrations and serialize Cargo/wiring. |
| EV-C0 — shared contracts | EV-B0; integrator with independent review | Freeze scoped identity, curated-event envelope, retention/lease/ack states, quotas and compatibility. Resolve the decisions listed below before their consumers implement. |
| EV-Q0/Q1/Q2/Q4 — quality baseline and pipeline | EV-C0 for implementation; evaluation owner | Separate component retrieval from ingestion/authenticated pipeline, holdout/negatives/abstention. Preserve accepted v2 corpus/floors. Record source/toolchain/provider/corpus hashes and per-category results. No floor promotion by self-approval. |
| EV-S0/S1/S2/S3 — reversible setup | EV-C0; setup owner | Preview/static status avoids DB create/migration and continuity consumption. Apply checks expected config hashes, owns only its fragment, backs up safely and rolls back conflicts. Probe actual MCP with ephemeral DB. Support two demonstrated adapters before expanding client coverage. No real HOME/client configuration in tests. |
| EV-H1/H2/H3/H4 — recoverable continuity | EV-C0; continuity owner, integrator for schema/SDK wiring | Separate opt-in durable delivery from legacy drain. Persist checkpoint/outbox; test claim/lease/retry/ack, restart without SessionEnd, duplicate consumers and denied principals. Acknowledged delivery is not exactly-once task execution. H1 does not depend on the quality corpus merely for scheduling. |
| EV-E0/E1 — bounded curated evidence | EV-C0; evidence owner | Bound session input/chunking and spool; events contain decisions/results/references by default. Exclude raw prompts, arguments, logs, environment and secrets. Test zero limits, Unicode, saturation, retention and crash durability. Detailed capture is separately opt-in. |
| EV-P1A — projection policy | EV-C0; portability owner and decision owner | Review a versioned RFC amendment: per-memory/destination/revision opt-in denied by default, authorized content, scan-before-write, retention and human-edit handling. No managed Git projection implementation before acceptance. |
| EV-P1B/P1C — managed projection | EV-P1A accepted; current Markdown core already integrated | Stable IDs/names, manifest, directional relationships, deterministic tree, conflicts/pruning/roundtrip and no unauthorized publication. Consume `src/portability/markdown/{export,import}/`; do not rebuild the core or use removed handler query paths. Separate acceptance from ordinary export. |
| EV-Z-essential — first integrated result | Essential Q/S/H/E contracts selected and accepted | Reversible setup → curated capture → checkpoint → crash/restart → another agent → scoped retrieval. Negative auth/retention, SDKs, full gates and independent review on the exact candidate. No dependency on optional experiments just to close this result. |
| EV-E2/E3 — evidence-ranking experiment | EV-E1 + EV-Q4 | Authorization/TTL/opt-in before voting or ranking; cap/dedup parent contributions; prove no hidden-source score leak. Measure against baseline and accept or reject independently. |
| EV-Q3 — external LongMemEval adapter | Evaluation contract and explicit dataset/license/download decision | Distinguish the repository's synthetic suite from official data. Retrieval-only adapter and provenance; no implicit upstream scripts, LLM calls or downloads. |
| EV-M1/M2, EV-R1/R2 — optional model experiments | EV-Q4 and approved model acquisition; model owner | Candle vs ONNX identity/numerics/cost, or real reranker tokenizer/forward pass/limits. Record adoption or rejection. No automatic backend/default change or ANN latency claim. |
| EV-D0/D1 — honest claims and documentation | Alongside each accepted result; integrator | Name available/optional/experimental/unimplemented behavior and source evidence. Generate MCP docs when surface changes; version public contracts deliberately. |

EV-Q0 and EV-S0 may investigate in parallel during closeout using read-only fixtures
and source. Implementation consumers wait for their actual contract inputs. The
integrator serializes schema, public wiring, manifests, shared docs and heavy checks;
workers have exclusive paths. Capture adapter ownership is transferred explicitly
between setup/evidence owners. A review occupies a real slot and can unblock settled
work without a barrier across unrelated tasks.

## Decisions retained

- EV-C0 fixes concrete lease/retention/quotas and authenticated envelope behavior.
  Missing values block only affected consumers, not independent diagnosis.
- RFC0004a / managed publication approval remains a product/policy decision. Existing
  export directories are not automatically safe to publish as a wiki.
- Legacy error normalization and complete CLI diff preview retain public-contract
  decisions. Safe review default is delivered; a count summary is not the RFC's diff.
- B1 hygiene remains outside the dependency graph until a specific source/list and
  reproduction justify it. Keep the unexplained child.pid failure distinct from the
  fixes already integrated; do not claim a passing rerun diagnosed it.
- HNSW default/ANN, federation and storage changes remain separate measured decisions.
  Candle improves model inference only if measured; it is not an index acceleration.

## Acceptance and authority

Each selected delivery has its own candidate, focused regressions, full applicable
gates, independent review, receipt and operational decision. Optional experiments,
projection and external benchmarks have separate terminal accept/reject outcomes;
they do not make the first useful setup/continuity delivery wait indefinitely.
This revision installs no supervisor, edits no real client, downloads no model,
calls no paid provider and grants no deployment/publication authority.
