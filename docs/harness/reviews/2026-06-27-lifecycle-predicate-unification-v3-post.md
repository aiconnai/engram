REVIEW_VERDICT: FAIL code/tests verify clean, but ROADMAP still has stale salience/lifecycle ownership wording

- [MED] `docs/ROADMAP.md:123` still says “Temporal decay transitions memories through lifecycle states” under “Phase 8: Salience Scoring.” This keeps the prior README/ROADMAP contradiction only partially fixed: README now points lifecycle transitions to `lifecycle_run`, but ROADMAP still implies salience/temporal decay owns lifecycle transitions.

Checked clean areas: evidence chain exists (plan/spec/chair/codex/grok/prompt), canvas has required sections, compression paths are Archived-only and idempotent in repeat-run tests, `lifecycle_run` is the only decay-derived lifecycle writer, salience/policy no longer write lifecycle, MCP docs/metadata are updated, `SCHEMA_VERSION` remains 44, and targeted/full verification passed (`cargo test --workspace --all-targets --locked`: 1227 passed; clippy/check/MCP reference passed).
