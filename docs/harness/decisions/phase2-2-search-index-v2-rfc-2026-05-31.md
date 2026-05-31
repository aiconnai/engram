# Fase 2.2 — Decisão #29: Search Index v2 RFC

**Data:** 2026-05-31
**Issue:** #29 (`Create Search Index v2 RFC`)
**Status da decisão:** Formalizada (RFC emitido e aceita).
RFC principal: `docs/rfcs/0003-search-index-v2.md` (Aceita).

## Evidência no main

- Arquitetura atual de search é SQLite-first com módulos em `src/search`:
  - BM25 + fuzzy + hybrid + ranking/rerank;
  - storage default em `src/storage/sqlite_backend.rs`.
- Existe backend meilisearch habilitado por feature (`src/storage/meilisearch_backend.rs`) e indexador opcional (`src/storage/meilisearch_indexer.rs`), tratado em `src/storage/mod.rs` como extensão planejada/alternativa.
- Não há rastreio de implementação local de `tantivy`, ANN/HNSW, `manticore`, nem camada migratória formal FTS5→v2.
- O contrato de saúde de índices derivados já existe em SQLite (`sqlite_fts_health`, `sqlite_graph_health`, `sqlite_embedding_health`) em `src/storage/sqlite_backend.rs`.
- Roadmap (`docs/ROADMAP.md`) já registra fase de Meilisearch e evolução de infraestrutura de busca, mas sem RFC de escolha de caminho “v2”.

## Decisão operacional

1. **Emissão de #29 foi concluída com RFC formal** em `docs/rfcs` com:
   - comparação explícita dos caminhos (FTS5 atual, Meilisearch, Tantivy/ANN, Fallback),
   - critérios de degradação e migração (foco em não regressão e local-first),
   - guardrails pós-incident (`schema/rebuild`, orphan handling, drift detectável),
   - plano de rollback e limpeza de dados.
2. **Resultado operacional formal:**
   - manter SQLite+FTS5 como base funcional principal de busca;
   - manter Meilisearch como opção opcional de backend (`feature-gated`);
   - não adotar ainda Tantivy/Manticore/ANN-HNSW como padrão sem RFC/benchmark dedicado;
   - manter #26 (contrato de `derived_index` health) e guardrails de rebuild/health/drift/disposability como pré-requisito para qualquer backend externo novo.

## Ação derivada

- Abrir/prolongar RFC `0002` (ou equivalente) em `docs/rfcs` com a matriz operacional completa para cumprir #29.
- Somente após esse RFC, seguir para decisões #26 e #25 com base contratual compartilhada.
