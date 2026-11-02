# Markdown portability core — implementation plan

Owner approved Q1–Q3 and the consolidated design on 2026-11-02. Base:
60ed4eec9390c9ef526411c9b64ea5f69bfbbcce. Local branch/worktree:
refactor/markdown-portability-core / .worktrees/markdown-portability-core.

## Scope

Deepen export and import together under src/portability/markdown/, eliminating
HandlerContext and crate::mcp imports from that module. Keep public signatures,
reexports, JSON output, defaults, environment modes, stored-workspace authorization,
transaction ordering, confined filesystem behavior and uncertain commit outcomes.
Extract only shared authorization primitives into auth; existing MCP paths delegate.
No projection, storage schema, embeddings, HNSW, remote operation or new dependency.

## Steps and acceptance

1. Baseline existing Markdown tests with CI_REQUIRED_FEATURES and shared Cargo target.
2. Move implementation and pure tests; use typed internal options/results/errors.
   Preserve detailed MCP reports and public Rust compatibility conversions.
3. Extract shared mode/row-access policy to auth and retain MCP reexports/delegation.
4. Test domain behavior through the typed seam with SQLite and tempfile; retain
   adapter contract, safety and direct-handler authorization regression suites.
5. Run focused tests, fmt, clippy, generated MCP reference and full sensors. Record
   results and deviations in the task log. No cargo clean.
6. Freeze a local commit for independent review. Formal post-gate remains pending
   without a genuine external review and human receipt; do not fabricate either.

## Rollback

Revert the isolated refactor commit; no data migration or deployment is needed.
