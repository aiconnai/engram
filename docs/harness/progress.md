# Engram — Harness Progress (Live State)

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Harness Engineering v0 — bootstrap & core gates` |
| Active task | `harness-bootstrap — implement operational harness (bootstrap, doctor, sensors, review-gate)` |
| Active plan | `docs/harness/progress/2026-05-30-harness-bootstrap.md` |
| Last review | `2026-05-31 — pass: docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md` |
| Last sensors | `2026-05-31T15:52:49Z — status=pass` |
| Last commit | `f2b1799` |

> Sumário curto do trabalho ativo. Logs detalhados em `progress/`.

## Sprint ativa

- **Harness Engineering v0 — bootstrap & core gates**
- **Log**: [`progress/2026-05-30-harness-bootstrap.md`](./progress/2026-05-30-harness-bootstrap.md)
- **Status**: active — scaffolding + implementation of the terminal-native agent harness discipline, adapted from the mbras-backend reference model, for the engram Rust + MCP + multi-SDK codebase.

## Contexto e Motivação

O usuário está usando Claude Code CLI e Grok Build TUI side-by-side para comparar workflows agentic em terminal. A visão é que "the terminal is the product" e que harnesses reais (Context Engine, Planner, Memory Manager, Verifier, Tool Registry, Harness Config) devem viver onde o trabalho de engenharia de mais alto sinal já acontece: o repositório + CLI.

Engram é posicionado de forma única porque ele *é* o Memory Manager para agentes. O harness de desenvolvimento do próprio engram pode dogfood o produto (futuro).

RFC 0001 (`docs/rfcs/0001-harness-memory-product-boundary.md`) já define o product boundary para Harness Memory.

Esta sprint implementa a **camada operacional** (o "harness engineering" process) que permite que agentes (e humanos) trabalhem de forma resumível, auditável e confiável.

## Trabalho em andamento (v0)

- [x] Estrutura de diretórios `docs/harness/{bin,progress,reviews,known-issues}`
- [x] `README.md` — guia operacional completo adaptado para engram/Rust/MCP/dual-CLI
- [x] `SPEC.md` — escopo da sprint v0
- [x] `INVARIANTS.md` — 18 regras de processo invioláveis (session, commits, review, harness self-consistency, Rust/engram specifics)
- [x] `GATES.md` — 3 camadas, thresholds, fake-success patterns específicos de engram (embedding features, MCP, schema version, hooks, etc.)
- [x] `CODE_REVIEW_POLICY.md` — política injetada no reviewer externo, com adaptações para dual-CLI e domínios de engram
- [x] `progress/2026-05-30-harness-bootstrap.md` — log detalhado (este arquivo)
- [x] `bin/bootstrap.sh` — script de orientação (read-only, rápido, determinístico)
- [x] `bin/doctor.sh` — consistência do harness
- [x] `bin/sensors.sh` — wrapper sobre `just ci` + doctor + engram-specific
- [x] `bin/review-gate.sh` — generalizado para claude/grok/etc, com prompt engineering, continuity, versioning, timeout
- [x] `bin/check-commit-msg.sh` — validador de commits
- [x] `docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md` — limitação formal para sensor `grpc-transport`
- [x] Atualização de `AGENTS.md` + `Claude.md` para exigir bootstrap
- [x] Execução do loop completo nesta sprint + evidência de PASS
- [ ] Integração leve com pre-commit / justfile (sem ruptura)

## Trilha de exclusão ativa

- `docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md` foi registrado para o sensor `grpc-transport`.
  - Exigir `sensors.sh --exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason \"sandbox socket bind restriction\"`.
  - `pass_with_exclusion` é aceitável apenas com trilha completa e sem fechar produção (limpeza no ambiente sem exclusão necessária).

## Últimas decisões registradas

- Harness é **complementar** aos gates existentes (`just ci`, pre-commit, GitHub Actions). Não substitui — adiciona memória persistida, review cross-CLI, e disciplina de processo.
- Review-gate será flexível para o cenário atual (prompt files + paste no outro CLI) porque Grok Build TUI e Claude Code CLI estão sendo usados side-by-side.
- Dogfooding com o próprio engram (via MCP + hooks) é objetivo explícito de longo prazo, guiado por RFC 0001, mas fora do escopo de v0 bootstrap.
- Invariants do harness são separados dos data invariants (`INVARIANTS.md` na raiz) para manter clareza.

## Próximos passos imediatos

1. Fechar post review do estado atual com artefato `REVIEW_VERDICT: PASS`.
2. Concluir Fase 1: manter mini-artifacts dos blocos 1.1–1.3 e audit report em `docs/harness/plans/`.
3. Entrar em Fase 2 (decisões/P1/P0): 28, 29, 26, 31, 32.
4. Preparar Fase 4 com base no contrato de `harness_record` + `harness_status`.

## Nota da sessão — 2026-05-31

- Compressão: `ContextCompressor` agora expõe diagnósticos de orçamento/skips,
  e `memory_compress_for_context` retorna metadados de orçamento e memórias
  ignoradas.
- Compressão semântica: dedupe preserva endpoints técnicos distintos e normaliza
  pontuação final em tokens técnicos sem perder paths, IDs ou versões.
- Evidência de benchmark: `docs/harness/reviews/2026-05-31-compression-benchmark-ratio-recall.md`.
- Verificações desta sessão:
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `bash docs/harness/bin/sensors.sh` — PASS limpo, sem exclusão
    (`2026-05-31T15:52:49Z`).
  - `cargo test test_technical_tokens_normalize_trailing_punctuation` — PASS.
  - `cargo test test_deduplication_preserves_distinct_technical_endpoints` — PASS.
  - `cargo test --test v070_integration_tests test_memory_compress_for_context_reports_skip_metadata` — PASS.
  - `bash -n docs/harness/bin/review-gate.sh` — PASS.
  - `bash -n docs/harness/bin/check-commit-msg.sh` — PASS.
  - `bash docs/harness/bin/check-commit-msg.sh --message 'fix(harness): harden review gates'` — PASS.

## Review fixes — 2026-05-31

- `docs/harness/bin/review-gate.sh` agora usa `set -euo pipefail` e corrige a
  grafia de `REVIEWER_CLI=grok` no comentário operacional.
- `CLAUDE.md` agora lista caminhos explícitos para todos os docs obrigatórios
  do harness.
- `tests/v070_integration_tests.rs` cobre o contrato MCP de
  `memory_compress_for_context` para metadados de skip/orçamento.
- Review manual subsequente:
  - dedupe semântico preserva fatos técnicos em relação de superset;
  - `memory_compress_for_context` aceita `memory_ids` além de `ids`;
  - benchmark `fixed_corpus_ratio_recall` usa piso fixo explícito de 7/9.

## Phase 0 post-review — 2026-05-31

- Artefato manual criado por instrução explícita do usuário:
  `docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md`.
- `bash docs/harness/bin/review-gate.sh post harness-bootstrap --review-file docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md`
  retornou `POST-GATE PASS`.
- Observação: o artefato é `Codex/manual`, não cross-CLI independente.

## ENG-1295 / #26 — passo 3

- `maintenance-status` humano agora imprime seção `Derived indexes:` com
  `embeddings`, `memories_fts` e `crossrefs`.
- A renderização foi extraída para `write_maintenance_status<W: Write>` para
  permitir teste sem captura global de stdout.
- Novo teste:
  `maintenance_status_human_output_includes_derived_indexes`.
- Verificações:
  - `cargo fmt --all -- --check` — PASS.
  - `cargo test maintenance_status` — PASS.
  - `cargo test test_health_check_reports_` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## ENG-1296 / #32 — harden queue and operational hygiene surface

- Auditoria de superfície de fila de embeddings aplicada em health + status operacional:
  - Incluídos em `derived_indexes[embeddings].details`: `pending`, `processing`,
    `stale_processing`, `failed`, `retryable_failed`, `exhausted_failed`,
    `max_retry_count`, `oldest_pending_age` (+ alias
    `oldest_pending_age_seconds`).
  - Saída humana de `maintenance-status` agora exibe linha `queue-state` com esses contadores (com fallback estável para chave legada `oldest_pending_age_seconds`).
- Testes/regressões validados:
  - `cargo test maintenance_status_ -- --nocapture` — PASS.
  - `cargo test test_health_check_embedding_details_include_queue_state_counters -- --nocapture` — PASS.
  - `cargo test embedding_queue_health_counts_stale_and_retries -- --nocapture` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

---

**Nota**: Este arquivo é atualizado manualmente ao final de cada iteração significativa ou ao final de sessões. O log detalhado fica no arquivo apontado por `Active plan`.
