# RFC 0002: Prompt Compression Benchmark and Compression Strategy

## Status

Accepted

## Context

Issue #31 requires a benchmark before adopting any neural prompt compression backend. Current context preparation in `src/intelligence` already has:

- `OutputFilter` (command-aware, rule-based line and noise filtering) (`src/intelligence/output_filter.rs`)
- `TruncationEngine` (token-budget-driven truncation, char-count heuristic) (`src/intelligence/truncation_engine.rs`)
- `SoftTrim` utilities (`src/intelligence/content_utils.rs`)
- `ContextCompressor` and `CompressionLevel` presets (`src/intelligence/context_compression.rs`)
- `SemanticCompressor` (`src/intelligence/compression_semantic.rs`)

These are all local, deterministic, and synchronous.

No neural/LLM-only compression backend is currently required by product flow, and this RFC gates that decision on concrete evidence.

## Measurement design

1. **Latency** — Criterion benches on compression stage primitives.
2. **Token reduction** — measured through:
   - deterministic benchmark assertions currently in `benches/token_reduction.rs`;
   - compression ratio returned by `semantic_compressor` metadata (`CompressedMemory::ratio`);
   - budget fit guarantees in pipeline truncation.
3. **Quality proxy** — preserve-correctness tests:
   - key terms/entities still present after compression (`compression_semantic` tests).
4. **Citation/grounding proxy** — presence of extracted `key_entities` / `key_facts`.
5. **Recall proxy** — fixed-corpus fact-preservation assertions on a deterministic dataset.

## Runbook

```bash
cargo bench --bench token_reduction -- --nocapture
```

Observed in this session:

- output-filter throughput is typically stable around 1.0–1.1 Gelem/s on cargo-like payloads (100–2000 lines),
- truncation throughput is typically stable from 2.45–2.9 Gelem/s for budgets 500–8000,
- full pipeline (filter→truncation) is typically 1.05–1.13 Gelem/s on tested cargo payloads, with run-to-run variance expected to exceed 5%.
- fixed semantic benchmark now logs avg_ratio and avg_recall from `token_reduction/semantic_compression` in `benches/token_reduction.rs` for comparison across runs.

## Proposal

### Option set evaluated

| Strategy | What it is | Latency risk | Fidelity risk | Cost model |
|---|---|---|---|---|
| A | **No compression** (baseline) | low | none | zero |
| B | **Current deterministic compression stack** (filter + soft trims + truncation + optional semantic compression) | low–medium | medium (depends on strategy) | zero |
| C | External neural summarizer (LLM/embedding pipeline) | high | medium–high (model drift, hallucination) | external API + latency + privacy risk |

### Decision

Adopt **Option B as core** and classify **Option C as optional future**.

Core implementation remains deterministic and local in v0-v1 because:

- it already provides measurable reductions with low, stable latency;
- it is deterministic/reproducible under harness gates;
- it does not add external dependencies or credential risk;
- it supports explicit fallback and budget guarantees that are test-covered.

Option C is optional in this RFC and should only be adopted if a later issue proves improved quality/recall without harming citation fidelity, and if budget/privacy constraints justify the dependency.

## Blocking completion criteria before any follow-on implementation touches

- This RFC is blocking-accepted by:
  - `docs/harness/decisions/phase2-3-compression-benchmark-2026-05-31.md`
  - issue `#31`
  - harness traceability issue `#31` follow-up comment and decision artifacts.
  - `docs/harness/reviews/2026-05-31-compression-benchmark-ratio-recall.md` (benchmark artifact)
- Completed follow-up gates:
  - fixed-corpus benchmark added in `benches/token_reduction.rs` with explicit ratio+recall output.
  - deterministic recall checks against a fixed corpus of required facts.
  - explicit failure ceiling for deduplication in semantically similar technical sentences.
  - explicit "non-local neural path" condition: Option C may only proceed via a dedicated RFC/issue with cold/warm/failure/failure-masking profiles.

## Acceptance update for issue #31

- Benchmark plan and decision outcome are now recorded in `docs/harness/decisions/phase2-3-compression-benchmark-2026-05-31.md`.
- Decision: deterministic local compression stack remains **core**; external neural compression remains **optional** pending a dedicated RFC and benchmark profile.

## Open questions

- What fixed corpus defines acceptable recall/fidelity for high-safety documentation contexts?
- Should semantic compression be default on all flows or opt-in by criticality and source confidence?
