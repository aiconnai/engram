# RFC 0006: gbrain ideas prioritization

**Status:** Accepted
**Date:** 2026-06-07
**Decision owner:** Engram maintainers
**Scope:** Roadmap prioritization only; no implementation changes.

## Context

Engram is preparing for a public-facing phase. A set of nine ideas from the
gbrain review were evaluated against the current repository state. This RFC
records the priority order under the chosen lens of technical integrity and debt
reduction.

The lens intentionally prioritizes fragile contracts, security enforcement gaps,
and measured or measurable bottlenecks over onboarding and product-adoption
work. Public-readiness work remains important, but it is not the ordering lens
for this RFC.

## Findings that changed the original proposal

The original proposal assumed several gaps that are no longer true, or are
smaller than stated.

| # | Idea | Original premise | Current state |
|---:|---|---|---|
| 3 | Registry/dispatch parity | Add a hard test | Already covered by `test_dispatch_registry_parity()` in `src/mcp/tools/mod.rs` |
| 8 | Retrieval quality in CI | Add deterministic retrieval-quality checks | Benchmark metrics already exist in `benches/search.rs`; CI floors are the missing step |
| 2 | Auth scopes | Expose scopes as enforced auth | Scope-grant infrastructure exists in `src/storage/scope_grants.rs`; central transport/dispatch enforcement is the gap |

Other confirmed signals:

| # | Idea | Current signal |
|---:|---|---|
| 5 | WAL/concurrency | `Storage` owns an `Arc<Mutex<Connection>>`; `StoragePool` exists but must be validated before implementation work |
| 1 | CLI setup/connect/doctor | Not present in the current CLI surface |
| 4 | Normalized tool errors | Tool handlers still commonly encode errors inside successful JSON values |

## Decision

Prioritize the nine ideas in this order.

### Tier 1: contract and bottleneck debt

1. **Normalize MCP/tool errors.**

   Replace success-shaped `{"error": ...}` tool responses with an explicit
   handler result contract, such as `HandlerResult = Result<Value, ToolError>`.
   This is the highest-priority contract debt because every handler can drift
   independently today. The pre-public window is the right time to define the
   clean wire contract before external consumers depend on inconsistent error
   shapes.

2. **Enforce workspace/scoped auth centrally.**

   `ScopeGrant`, hierarchical permissions, and `check_scope_access()` already
   exist. The remaining gap is central enforcement: bearer identity must map to
   agent/workspace/scope context, and dispatch must reject unauthorized tool
   calls before handlers rely on implicit trust. This should build on the
   normalized error contract from priority 1.

3. **Measure and then decide read concurrency.**

   The current single mutex around SQLite connection access plausibly wastes WAL
   read concurrency. However, this is still a performance optimization, not a
   contract fix. The first required step is a contention benchmark that measures
   concurrent read latency while unrelated writes occur. If the benchmark does
   not prove material gain, stop before implementing a read pool.

### Tier 2: promote existing quality work

4. **Promote retrieval quality from benchmark signal to release signal.**

   `benches/search.rs` already measures `precision@10`, `MRR`, and `nDCG@10`
   with deterministic queries. The missing work is a normal `cargo test` release
   signal with explicit floors. This should be cheap, but the floors must be
   chosen in the implementation RFC or issue, not here.

### Tier 3: CI and developer-experience improvements

5. **Add fail-closed diff-based verification.**

   This is useful process hardening, but it is below the contract and measured
   bottleneck work under this RFC's lens.

6. **Parallelize independent checks with failure-first logs.**

   This improves feedback speed and readability. It should not displace error,
   auth, or measured concurrency work.

### Tier 4: adoption work deferred by this lens

7. **Add `engram-cli setup/connect/doctor`.**

   This is likely important for public adoption, but it is onboarding work, not
   technical-integrity debt. Defer it to an adoption/readiness RFC or roadmap
   pass.

8. **Write the team-brain guide.**

   This is product education work. Defer it under the chosen technical-debt
   lens.

### Already complete

9. **Registry/dispatch parity hard test.**

   Treat this as closed for the purpose of this prioritization. Future work may
   still simplify the registry/dispatch model by introducing a single
   `ToolSpec { definition, handler }` source of truth, but that is a separate
   design choice, not the original missing-test task.

## Compatibility stance

Engram is still in a pre-public transition window. For contract fixes such as
normalized tool errors and auth enforcement, implementation RFCs may choose a
clean breaking contract if there are no active external consumers to preserve.

If an implementation RFC identifies existing consumers, it must use the normal
compatibility discipline: additive transition, feature flag or compatibility
layer for one cycle, and explicit migration guidance.

## Required implementation RFCs

This RFC decides priority and order only. It does not authorize direct
implementation of the Tier 1 items without their own focused implementation
plans.

The expected follow-up RFCs are:

| Priority | Follow-up |
|---:|---|
| 1 | Tool error contract RFC |
| 2 | Workspace-scoped auth enforcement RFC |
| 3 | SQLite read-concurrency benchmark and decision RFC |

## What we will not copy

Do not copy a large release ceremony from gbrain. Engram should keep release
process lightweight until measured process failures justify more machinery.

Do not treat directory scoping as a security boundary. Security must be enforced
through identity, workspace, scope, and tool authorization.

Do not adopt long-lived remote bearer tokens as the final auth model without a
scope/workspace mapping and revocation story.

Do not add a large command/test taxonomy before the current harness and CI gates
show a concrete gap.

## Out of scope

This RFC does not modify code.

This RFC does not implement normalized errors, auth enforcement, read pooling,
retrieval-quality floors, CLI setup, or documentation guides.

This RFC does not choose concrete benchmark floors for retrieval quality or
read-concurrency contention. Those numbers belong in the follow-up
implementation RFCs.

## Acceptance criteria for this RFC

The RFC is accepted when:

| Criterion | Status |
|---|---|
| `docs/rfcs/0006-gbrain-ideas-prioritization.md` exists | Required |
| No code files are changed by this RFC | Required |
| Priority order is recorded with explicit deferrals | Required |
| Registry/dispatch parity is marked already complete | Required |
| Read-concurrency work is gated by benchmark evidence | Required |
