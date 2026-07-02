# RFC 0011: Model Routing Contract

Status: proposed

Date: 2026-06-09

Related issue: ENGRA-109

## Summary

Engram already has multiple model/provider touchpoints, mostly around
embeddings, multimodal processing, token budgeting, and optional LLM council
workflows. This RFC defines a minimal routing contract so future code can choose
providers explicitly, report degraded/offline behavior, and avoid silent
fallbacks.

This RFC does not add runtime provider dependencies.

## Current Touchpoints

### Embeddings

Primary code:

- `src/embedding/mod.rs`
- `src/embedding/provider.rs`
- `src/embedding/tfidf.rs`
- `src/embedding/openai.rs`
- `src/embedding/ollama.rs`
- `src/embedding/cohere.rs`
- `src/embedding/voyage.rs`
- `src/embedding/onnx.rs`
- `src/embedding/clip.rs`
- `src/embedding/queue.rs`
- `src/mcp/handlers/retrieval.rs`

Current behavior:

- `TfIdfEmbedder` is the local default and does not require network access.
- Feature-gated providers include OpenAI, Ollama, Cohere, Voyage, ONNX/local,
  and multimodal CLIP-style embeddings.
- `memory_embedding_providers` reports the active embedder model.
- `memory_embedding_migrate` can re-embed memories and record a target model.

### Retrieval and Reranking

Primary code:

- `src/search/hybrid.rs`
- `src/search/rerank.rs`
- `src/search/neural_rerank.rs`
- `src/search/semantic_cache.rs`
- `src/mcp/handlers/search.rs`

Current behavior:

- Hybrid search combines lexical and vector signals.
- Neural reranking is feature-gated.
- Some search paths are rule-based and should remain no-LLM.

### Token Budgeting and Context

Primary code:

- `src/intelligence/token_counter.rs`
- `src/intelligence/context_builder.rs`
- `src/mcp/handlers/summarize.rs`

Current behavior:

- `context_budget_check` accepts a `model` and optional encoding override.
- Token counting is deterministic and should not require provider routing.

### LLM Council

Primary code:

- `src/mcp/handlers/council.rs`
- `src/mcp/tools/registry.rs`
- `tests/mcp_protocol_tests.rs`

Current behavior:

- `memory_council` is feature-gated behind `http-client`.
- The handler calls a remote llm-council backend and returns stage outputs plus
  `final_model`.
- This is an orchestration surface, not core memory retrieval.

### Multimodal Providers

Primary code:

- `src/multimodal/audio.rs`
- `src/multimodal/vision.rs`
- `src/mcp/handlers/multimodal.rs`

Current behavior:

- Vision and audio providers are configured from environment variables.
- Some multimodal search paths fall back from CLIP-style embeddings to image
  description and text search.
- Fallback must be explicit in future route reporting.

### Deterministic Evaluations

Primary code:

- `src/dream/eval.rs`
- `docs/DREAM_SNAPSHOT_EVALS.md`

Current behavior:

- Dream snapshot evals are deterministic and should not require paid model
  access.
- These paths should be marked as `rule_based` or `deterministic_local`, not
  routed to an LLM provider.

## Route Contract

A future route descriptor should contain:

```rust
pub struct ModelRoute {
    pub purpose: ModelPurpose,
    pub provider_id: String,
    pub model_id: String,
    pub capability: ModelCapability,
    pub cost_class: CostClass,
    pub latency_class: LatencyClass,
    pub offline_policy: OfflinePolicy,
    pub fallback_policy: FallbackPolicy,
    pub feature_flag: Option<String>,
    pub requires_secret: bool,
}
```

Conceptual enums:

- `ModelPurpose`
  - `embedding_text`
  - `embedding_image`
  - `rerank`
  - `vision_describe_image`
  - `audio_transcribe`
  - `llm_council`
  - `token_count`
  - `deterministic_eval`
- `ModelCapability`
  - `local_rule_based`
  - `local_model`
  - `remote_embedding`
  - `remote_llm`
  - `remote_vision`
  - `remote_audio`
- `CostClass`
  - `free_local`
  - `metered_remote`
  - `unknown`
- `LatencyClass`
  - `inline`
  - `background`
  - `batch`
- `OfflinePolicy`
  - `works_offline`
  - `requires_local_model`
  - `requires_network`
  - `disabled_without_feature`
- `FallbackPolicy`
  - `none`
  - `explicit_local_fallback`
  - `explicit_remote_fallback`
  - `degrade_without_substitution`

The route descriptor is metadata first. It should not instantiate providers by
itself.

## Resolver Contract

A future resolver should:

- accept a `ModelPurpose` and optional caller preference;
- return a selected route or structured degraded result;
- never make network calls during route selection;
- never silently substitute a weaker provider;
- surface missing feature flags, missing credentials, and missing local model
  files as explicit states;
- include enough metadata for logs, MCP responses, and tests.

Suggested result shape:

```json
{
  "purpose": "embedding_text",
  "status": "ok",
  "provider_id": "tfidf",
  "model_id": "tfidf-128",
  "offline_policy": "works_offline",
  "fallback_used": false,
  "warnings": []
}
```

Degraded result:

```json
{
  "purpose": "embedding_text",
  "status": "unavailable",
  "provider_id": "openai",
  "model_id": "text-embedding-3-small",
  "reason": "missing_secret",
  "required_secret": "OPENAI_API_KEY",
  "fallback_available": "tfidf"
}
```

## Failure and Degraded Vocabulary

- `ok`: selected route is available.
- `missing_secret`: route needs a credential that is not configured.
- `feature_disabled`: route requires a disabled Cargo feature.
- `model_missing`: local model path is missing or invalid.
- `provider_unavailable`: provider exists but cannot be reached at execution
  time.
- `fallback_used`: explicit fallback was used and should be visible to callers.
- `unavailable`: no acceptable route exists.

## Configuration Direction

Initial routing configuration should come from existing code and environment
variables. A future config file may be added only after the route descriptor is
stable.

Possible future file:

```toml
[routes.embedding_text]
provider = "tfidf"
model = "tfidf-128"
fallback = "none"

[routes.embedding_text.remote]
provider = "openai"
model = "text-embedding-3-small"
fallback = "tfidf"
```

Do not add this file in the first implementation.

## Paths That Remain Rule-Based

These should not route to an LLM provider by default:

- token counting and budget checks;
- deterministic dream snapshot evals;
- MCP contract validation;
- harness doctor/sensors/review gates;
- BM25/fuzzy search;
- policy-rerank explanation when no model reranker is configured.

## Initial Implementation Plan

1. Add metadata-only route types in a small module.
2. Add tests for default routes:
   - text embeddings -> local TF-IDF route by default;
   - token count -> rule-based route;
   - dream eval -> deterministic local route;
   - remote provider route reports missing secret without network calls.
3. Add a read-only MCP or CLI status surface only after the metadata module is
   stable.
4. Do not change provider instantiation or search behavior in the first PR.

## Non-Goals

- No provider SDK changes.
- No paid-provider tests by default.
- No runtime failover implementation in this RFC.
- No new remote dependency.
- No automatic provider selection based on live latency or price.

## Open Questions

- Should route metadata live under `src/embedding/`, `src/intelligence/`, or a
  new `src/model_routing/` module?
- Should route status be exposed over MCP, CLI, or both?
- Should embedding vector dimensions be part of every embedding route descriptor?
- How should multimodal fallback from image embedding to image description be
  represented: one route with fallback, or two explicit routes?
