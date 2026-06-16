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

Engram é posicionado de forma única porque ele *é* o Memory Manager para agentes e para times que acumulam contexto proprietário mais rápido do que conseguem organizá-lo manualmente. O harness de desenvolvimento do próprio engram pode dogfood o produto (futuro).

RFC 0001 (`docs/rfcs/0001-harness-memory-product-boundary.md`) já define o product boundary para Harness Memory.

Esta sprint implementa a **camada operacional** (o "harness engineering" process) que permite que agentes (e humanos) trabalhem de forma resumível, auditável e confiável sobre a mesma memória canônica.

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
- [x] Integração leve com pre-commit / justfile (sem ruptura)

## Trilha de exclusão ativa

- `docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md` foi registrado para o sensor `grpc-transport`.
  - Exigir `sensors.sh --exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason \"sandbox socket bind restriction\"`.
  - `pass_with_exclusion` é aceitável apenas com trilha completa e sem fechar produção (limpeza no ambiente sem exclusão necessária).

## Últimas decisões registradas

- Harness é **complementar** aos gates existentes (`just ci`, pre-commit, GitHub Actions). Não substitui — adiciona memória persistida, review cross-CLI, e disciplina de processo.
- Review-gate será flexível para o cenário atual (prompt files + paste no outro CLI) porque Grok Build TUI e Claude Code CLI estão sendo usados side-by-side.
- Dogfooding com o próprio engram (via MCP + hooks) é objetivo explícito de longo prazo, guiado por RFC 0001, mas fora do escopo de v0 bootstrap.
- Invariants do harness são separados dos data invariants (`INVARIANTS.md` na raiz) para manter clareza.
- 2026-06-08: `ENGRA-103` aberto no Huly para `memory_digest`; RFC 0008 define a ferramenta como digest read-only, determinístico, sem schema novo e com provenance explícita.

## Crate maintenance — 2026-06-12

- Corrigida a revisão dos crates:
  - `engram-wasm` entrou no workspace raiz e no gate local/GitHub CI.
  - `engram-core` passou a excluir artefatos internos do pacote publicado.
  - Advisories corrigíveis foram atualizados no `Cargo.lock` (`rustls-webpki`
    0.103.13, `tar` 0.4.46, `time` 0.3.47, `rand` 0.8.6,
    `aws-lc-rs` 1.17.0 / `aws-lc-sys` 0.41.0).
  - Dependências opcionais afetadas atualizadas: `libsql` 0.9.30,
    `notify` 8.2.0, `tokenizers` 0.23.1.
  - `cargo audit` e `cargo deny check` passam; ignores restantes são
    transitivos upstream-blocked (`rustls-webpki` via AWS/libsql,
    `quinn-proto`, e unmaintained transitivos sem safe upgrade).
- Verificações:
  - `cargo check --all-targets` — PASS.
  - `cargo check --all-targets --no-default-features --features turso,local-embeddings` — PASS.
  - `cargo audit` — PASS.
  - `cargo deny check` — PASS.
  - `bash scripts/ci.sh` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Version control gate — 2026-06-12

- Corrigido `docs/harness/bin/vc-gate.sh` para não usar pipeline
  `git log | grep -q` sob `set -o pipefail`; quando o `grep -q` encontrava
  uma correspondência cedo, `git log` podia receber SIGPIPE e fazer o gate
  reportar falso negativo.
- `latest_git_mentions_issue` e `jj_current_mentions_issue` agora fazem match
  sobre strings capturadas, preservando o contrato de fechar issue apenas com
  evidência recente no Git ou descrição atual do jj.
- Evidência local:
  - `bash -n docs/harness/bin/vc-gate.sh` — PASS.
  - `bash docs/harness/bin/vc-gate.sh done ci` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Code quality maintenance — 2026-06-16

- Aplicada fatia de alta confiança do relatório de qualidade:
  - Python SDK: `close()` agora é idempotente, zera `_client` após fechar e
    bloqueia `_mcp_call` em cliente fechado; `list()`/`search()` usam
    `filter_` no Python e enviam `filter` no payload MCP.
  - TypeScript SDK: request IDs JSON-RPC agora incrementam por cliente; opções
    públicas ganharam parity para `filter`, `mediaUrl`, workspaces e filtros
    de escopo suportados pela superfície MCP; testes foram reescritos para
    validar métodos públicos em vez de membros privados.
  - Cargo: removidos `anyhow`, `deadpool-sqlite`, `jsonrpc-core`,
    `levenshtein`, dev-deps não usadas, `wasm-bindgen-test` do crate WASM e o
    bin dummy `engram-core`/`src/main.rs`.
- `prost` foi mantido e documentado em `package.metadata.cargo-machete`
  porque é requerido por código gerado do recurso `grpc`.
- Fora de escopo deliberado: deleção dos módulos Rust de confiança média e
  deduplicações maiores; eles exigem revisão de compatibilidade pública.
- Review Canvas:
  `docs/harness/canvas/2026-06-16-code-quality-maintenance.md`.
- Post-review prompt gerado em
  `docs/harness/reviews/2026-06-16-code-quality-maintenance-v2-post.md.raw`;
  sem verdict independente ainda porque o fluxo exige outro CLI/reviewer.
- Verificações:
  - `npm test` em `sdks/typescript` — PASS.
  - `npm run type-check` em `sdks/typescript` — PASS.
  - `uv run --with pytest-asyncio pytest` em `sdks/python` — PASS, 162 tests.
  - `cargo check --all-targets` — PASS.
  - `cargo check -p engram-core --features grpc --all-targets` — PASS.
  - `cargo check -p engram-wasm --all-targets` — PASS.
  - `cargo check -p engram-wasm --target wasm32-unknown-unknown` — PASS.
  - `cargo machete` — PASS.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo clippy --all-targets --all-features -- -D warnings` — PASS.
  - `git diff --check` — PASS.
  - `make ci` — PASS.
  - `bash docs/harness/bin/sensors.sh` — PASS (`make ci + doctor`).

## Próximos passos imediatos

1. Fechar post review do estado atual com artefato `REVIEW_VERDICT: PASS`.
2. Concluir Fase 1: manter mini-artifacts dos blocos 1.1–1.3 e audit report em `docs/harness/plans/`.
3. Entrar em Fase 2 (decisões/P1/P0): 28, 29, 26, 31, 32.
4. Preparar Fase 4 com base no contrato de `harness_record` + `harness_status`.

## AgentShield loop MVL — 2026-06-16

- Added the minimum viable loop components for a bounded AgentShield security
  scan:
  - Automation: `.github/workflows/agentshield-loop.yml` runs weekly and by
    manual dispatch.
  - Skill: `skills/agentshield-scan/SKILL.md`.
  - State: `docs/loops/agentshield-scan/STATE.md`.
  - Gate: `scripts/run-agentshield-loop.sh`, also exposed as `make
    loop-security` and `just loop-security`.
- Scope is static triage only: no automatic remediation, no production
  credentials, no auto-commit, and `LOOP_MAX_ITERATIONS` is capped at 5.
- The loop is optional and not part of required PR branch protection.

## Security reference harness adaptation — 2026-06-04

- Adicionada adaptação local do
  `anthropics/defending-code-reference-harness` em
  `docs/harness/security/anthropic-reference-harness.md`.
- Adicionados tuning files para scan/triage em `.claude/scan-extras.txt` e
  `.claude/fp-rules.txt`.
- Contrato adotado: static/read-only first; pipeline autônoma bloqueada por
  default até existir ADR, sandbox forte e target contract Rust.
- README e GATES do harness agora referenciam o fluxo de segurança.

## ENGRA-103 / RFC 0008 — `memory_digest` planning — 2026-06-08

- Criado contrato proposto para `memory_digest` em
  `docs/rfcs/0008-memory-digest.md`.
- Criado plano de implementação em
  `docs/harness/plans/2026-06-08-memory-digest-implementation-plan.md`.
- Criado Review Canvas em
  `docs/harness/canvas/2026-06-08-memory-digest.md`.
- Decisão de escopo:
  - v1 é ferramenta MCP read-only;
  - sem schema migration;
  - sem LLM obrigatório;
  - sem raw artifact retrieval;
  - sem mutação de memórias canônicas;
  - implementação deve orquestrar `memory_smart_retrieve`,
    `memory_build_context`, graph/crossrefs e `context_build_bundle`.
- Huly: `ENGRA-103` (`MCP memory_digest actionable retrieval digest`) criado
  com prioridade High.

## ENGRA-111 — deterministic MCP mock parity harness — 2026-06-09

- Added fixture-driven parity coverage for three public MCP shapes:
  `memory_create`/`memory_search`, `context_record`/`context_search`, and the
  unknown-tool error envelope.
- The fixture stores stable normalized expectations only; generated memory IDs,
  timestamps, scores, and UUID-like values stay out of the contract.
- SDK expansion note lives with the fixture so Python and TypeScript parity can
  reuse the same scenario names and expected normalized output later.
- Review Canvas: `docs/harness/canvas/2026-06-09-engra-111-mock-parity.md`.

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

## ENG-1296 / #25 — Add embedding queue hygiene policy

- Formalização da política de operação da fila: budget de retry e retenção para `complete` (via constants config in `src/embedding/queue.rs`).
- `maintenance queue-hygiene` adicionado ao CLI com fluxo explícito de reparo:
  - `--dry-run` (padrão) para não mutar
  - `--apply` para executar mudanças
  - `--requeue-failed` para incluir requeue de `failed` com orçamento restante
- Expansão de saúde: `zero_retry_failed`, idades `oldest_processing_age` /
  `oldest_failed_age`, e buckets de `retry_count`.
- Ajuste adicional de observabilidade: buckets `retry_count_0/1/2/3+` passaram a
  usar semântica fixa (`3+` para `>=3`) independente do `max_retries` configurado,
  e passaram a aparecer também na saída humana de `maintenance-status`.
- Estado de saúde e saída humana de `maintenance-status` continuam read-only.
- Testes adicionados/estendidos em Rust para cobertura de:
  - `health` (contadores e idades)
  - path de hygiene em modo dry-run vs apply
  - output de status e comportamento não mutável por padrão.
- Verificações na iteração:
  - `cargo fmt --all -- --check` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `cargo test maintenance_status_ -- --nocapture` — PASS.
  - `cargo test test_health_check_embedding_details_include_queue_state_counters -- --nocapture` — PASS.
  - `cargo test test_embedding_queue_health_counts_stale_and_retries -- --nocapture` — PASS.
  - `cargo test test_embedding_queue_hygiene_dry_run_does_not_mutate_and_apply_can_repair -- --nocapture` — PASS.
  - `cargo test maintenance_queue_hygiene_dry_run_does_not_mutate_and_apply_updates -- --nocapture` — PASS.
- Decisão de escopo:
  - `CHEATSHEET_CUTOVER.md` permanece fora do commit; é checklist operacional
    de deploy/cutover e não faz parte da política de higiene da fila.

## ENG-1296 / #26 — Define derived index health contract for external backends

- Padronizado `DerivedIndexHealth` para backends externos:
  - adicionado construtor `DerivedIndexHealth::external(...)` em `backend.rs`;
  - `meilisearch` e `turso` passam a preencher `derived_indexes` com entrada `kind=external`
    (com contadores base e status explícito), em vez de retornar vetor vazio.
- Contrato em `docs/SCHEMA.md` ajustado para exigir representação consistente de
  derived indexes também em backends sem analítica interna por índice.
- Validação:
  - `cargo test test_turso_health_check --test turso_backend_tests --features turso -- --nocapture` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `cargo check --tests --features meilisearch` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## ENG-1296 / #27 — Generate MCP tools reference from code (partial)

- Documentação passou a usar a referência gerada como fonte principal:
  - `README.md` (`Available MCP Tools`) agora aponta para `docs/MCP_TOOLS.md` e para o gerador.
  - `docs/AI_GUIDE.md` remove contagens manuais e passa a referenciar `docs/MCP_TOOLS.md` como origem.
- Validação:
  - `./scripts/generate-mcp-reference.sh --check` — PASS.
- O fechamento de #27 ainda depende de eliminar/normalizar quaisquer contagens ou listagens manuais remanescentes fora desse escopo.

## Attestation CRITICAL security fixes — 2026-06-01

- `src/attestation/chain.rs` agora lê o tip da cadeia e insere o novo registro
  dentro do mesmo `with_transaction`, removendo a janela TOCTOU entre SELECT e
  INSERT.
- `verify_chain` passou a aceitar `Option<&[u8; 32]>` para verificar assinaturas
  Ed25519; `None` preserva o comportamento legado, e `Some(key)` exige assinatura
  válida em todos os registros verificados.
- Testes adicionados cobrem append concorrente, assinatura válida, assinatura
  adulterada e assinatura removida.
- Verificações:
  - `cargo test --features agent-portability test_verify_chain` — PASS.
  - `cargo test --features agent-portability test_chain_stays_linear` — PASS.
  - `cargo test --features agent-portability attestation` — PASS.
  - `cargo test --features agent-portability scenario_5_chain_verify_valid` — PASS.
  - `cargo test` — PASS.
  - `cargo fmt --all -- --check` — BLOCKED por formatting drift existente
    fora do diff (`compression_semantic.rs`, `token_counter.rs`, handlers MCP).
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `cargo clippy --all-targets --all-features -- -D warnings` — BLOCKED por
    warnings existentes fora do diff (`token_counter.rs`, `harness.rs`,
    `markdown_export.rs`).

## Council workflow skill / `memory_council` — 2026-06-02

- Fluxo reutilizavel de consensus/council adicionado:
  - ferramenta MCP `memory_council`;
  - handler `src/mcp/handlers/council.rs`;
  - wrappers Python/TypeScript `CouncilSkill`;
  - skill instalavel `skills/engram-council/SKILL.md`;

- Integração de fluxo leve com gates locais concluída:
  - `.githooks/pre-commit` agora usa `just pre-commit` quando disponível e cai
    para os comandos diretos quando `just` não estiver instalado.
  - `justfile` ganhou a receita `pre-commit` como fonte única para o hook.
  - `tests/mcp_protocol_tests.rs` recebeu cobertura de round-trip do tool
    `memory_council` via `tools/call`.
  - documentacao em README, AI guide, guia de uso em repos, SDK READMEs,
    changelog e referencia MCP gerada.
- Ajustes de revisao aplicados:
  - `skills/engram-council/SKILL.md` reestruturada como playbook operacional
    para agentes, com regras de uso, checklist, template de prompt, argumentos
    MCP, interpretacao de resultado e handling de falhas;
  - truncamento de erro no handler agora e seguro para UTF-8;
  - textos publicos distinguem `engram-council` (skill instalavel) de
    `llm-council` (backend de orquestracao);
  - README TypeScript nao depende mais de trailing whitespace para quebra de
    linha.
- Verificacoes:
  - `bash docs/harness/bin/bootstrap.sh` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo test council -- --nocapture` — PASS.
  - `git diff --check` — PASS.
- Validacao de skill:
  - `python3 .../skill-creator/scripts/quick_validate.py skills/engram-council`
    bloqueado porque `PyYAML` nao esta instalado no Python global.
  - `rg -n '[[:blank:]]+$' skills/engram-council/SKILL.md` — PASS sem
    trailing whitespace.
  - `LC_ALL=C rg -n "[^ -~]" skills/engram-council/SKILL.md` — PASS sem
    caracteres nao ASCII.
- Limitacoes locais:
  - `pytest sdks/python/tests/test_client.py -k council` bloqueado porque o
    ambiente global nao tem `pytest-asyncio` ativo.
  - `npm run type-check` bloqueado porque `tsc` nao esta instalado no SDK
    TypeScript local.

## ENG-1241 — MCP HTTP auth contract and client docs

- Diagnostico atualizado: o HTTP transport ja aplicava Bearer auth em
  `POST /mcp` e `GET /v1/events` quando `ENGRAM_HTTP_API_KEY`/`--http-api-key`
  estava configurado; o gap real era contrato publico inconsistente.
- `src/mcp/http_transport.rs` agora constroi o router em helper testavel e
  aceita `POST /v1/mcp` como alias compativel de `POST /mcp`, com o mesmo
  contrato de auth.
- Cobertura adicionada para:
  - `POST /mcp` rejeitar request sem Bearer quando API key configurada;
  - `POST /mcp` aceitar Bearer correto;
  - `POST /v1/mcp` usar o mesmo contrato de auth.
- Docs alinhadas:
  - novo `docs/MCP_AUTH.md` para clientes externos;
  - README, AI guide, getting started e guia de uso em repos atualizados para
    flags reais (`--transport http --http-port`, `ENGRAM_HTTP_API_KEY`) e MCP
    JSON-RPC em vez de REST local antigo.
- Validacoes:
  - `cargo test http_transport --lib` — PASS.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo clippy --lib -- -D warnings` — PASS.
  - `cargo clippy --lib --tests -- -D warnings` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `git diff --check` — PASS.
  - `bash docs/harness/bin/review-gate.sh post eng-1241-mcp-http-auth-docs`
    gerou prompt de post-review em
    `docs/harness/reviews/2026-06-03-eng-1241-mcp-http-auth-docs-v2-post.md.raw`;
    sem verdict porque falta reviewer externo.
  - Escopo deliberadamente fora desta iteracao: rate-limit MCP, metricas/tracing
    especificas de transport, e execucao/verificacao de deploy Fly.io.

## ENG-1241 follow-ups — MCP HTTP follow-through

- **ENGRA-58 (implementado):** Rate limit para MCP HTTP (`/mcp` e `/v1/mcp`) via token-bucket, com chaves por IP/header, teto de buckets e stale cleanup.
- **ENGRA-59 (implementado):** observabilidade do transporte (métricas, tracing e `GET /health` com estado de proteção).
- **ENGRA-60 (implementado):** rollout/documentação de validação de deploy (Fly.io) para auth + rate limit.

## ENGRA-84 — MCP HTTP rate-limit hardening

- Auth agora é avaliado antes do rate limit no `POST /mcp` e `POST /v1/mcp`;
  requests sem Bearer válido continuam retornando `401` e não consomem bucket.
- Regressões adicionadas para interação auth/rate-limit, fallback por
  `x-real-ip`, cleanup de buckets stale e eviction sob pressão de
  `max_buckets`.

## Security fixes — 2026-06-04

- **Merkle `hash_pair` length-separation** (OBS. 9133):
  `hash_pair` alterado de `left || right` para
  `len(left) || left || len(right) || right` para eliminar colisao de
  segunda pre-imagem. Backwards compat mantida via `scheme_version: u8`
  no `MerkleProof` (v1 = concat. naive; v2 = length-sep).
  Novas provas usam v2; provas v1 pre-existentes continuam verificaveis.

- **Attestation signature verification** (OBS. 9132):
  MCP handler `attestation_chain_verify` agora aceita `verifying_key`
  opcional (hex Ed25519). Quando fornecido, valida assinatura de todo
  registro. Schema MCP atualizado em `tools.rs` para expor o parametro.

- **Dedup normalization regression** (ACHADO AUDITORIA):
  `create_memory` e `update_memory` agora usam `compute_dedup_hash`
  (normalizado) na coluna `content_hash`. Backends alternativos
  (`turso_backend`, `meilisearch_backend`) sincronizados.
  `compute_content_hash_raw(cfg(test))` existe para byte-exact checks.
  Markdown import detecta edits case-only como `PendingUpdate` (teste
  `test_import_in_sync_when_body_normalized_matches`).

  Verificacoes:
  - `cargo test --lib` — 988 PASS.
  - `cargo test --features agent-portability --lib` — 1,063 PASS.
  - `cargo clippy --all-targets --all-features` — clean.

---

**Nota**: Este arquivo e atualizado manualmente ao final de cada iteracao significativa ou ao final de sessoes. O log detalhado fica no arquivo apontado por `Active plan`.

## Harness cross-improvements — 2026-06-05

- Plano Engram-only executado a partir de `docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md`.
- Criado `docs/harness/WHAT_WE_DONT_DO.md` para escopo negativo do harness.
- Criado `docs/harness/canvas/` com README e template de Review Canvas para mudanças complexas.
- Criados `docs/harness/bin/baseline.sh` e `docs/harness/bin/quarterly-audit.sh` como evidência de drift/auditoria, sem substituir sensores ou CI.
- `sensors.sh` ganhou modos opcionais `full`, `quick`, `docs`, `mcp` e `baseline`; o modo sem argumentos permanece o gate completo canônico.
- `review-gate.sh` passou a incluir `WHAT_WE_DONT_DO.md`, Review Canvas e guard para mudanças em `docs/harness/bin/*`.
- `doctor.sh` agora valida a nova política, canvas, baseline, audit, sensor lanes e referências cruzadas.

### Verificações planejadas nesta execução

- `bash docs/harness/bin/doctor.sh`.
- `bash docs/harness/bin/sensors.sh baseline`.
- `bash -n docs/harness/bin/bootstrap.sh docs/harness/bin/doctor.sh docs/harness/bin/sensors.sh docs/harness/bin/review-gate.sh docs/harness/bin/baseline.sh docs/harness/bin/quarterly-audit.sh`.

## Huly tracking — 2026-06-05

- Issues criados no Huly para execução do plano:
  - ENGRA-78 — negative-scope policy.
  - ENGRA-79 — Review Canvas.
  - ENGRA-80 — harness script review guard.
  - ENGRA-81 — baseline snapshot.
  - ENGRA-82 — optional sensor lanes.
  - ENGRA-83 — evidence-only quarterly audit.

### Verificações executadas — 2026-06-05 harness cross-improvements

- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh baseline` — PASS; gravou `docs/harness/.baseline-last`.
- `bash -n docs/harness/bin/bootstrap.sh docs/harness/bin/doctor.sh docs/harness/bin/sensors.sh docs/harness/bin/review-gate.sh docs/harness/bin/baseline.sh docs/harness/bin/quarterly-audit.sh` — PASS.
- `bash docs/harness/bin/quarterly-audit.sh` — PASS; gravou `docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md` e `docs/harness/.quarterly-audit-last`.
- `bash docs/harness/bin/doctor.sh` final — PASS.

Limite deliberado: o full `bash docs/harness/bin/sensors.sh` nao foi executado nesta iteracao; a validacao executada foi a lane `baseline` especifica do plano de melhoria do harness.

## Version-control gate / jj adoption — 2026-06-05

- Adicionado `docs/harness/bin/vc-gate.sh` como gate opcional de fronteira de issue e gate recomendado antes de release.
- Contrato adotado:
  - `jj` pode ser usado como camada local para evoluir, splitar e descrever work-in-progress;
  - Git continua canônico para release commits, tags e `cargo publish`;
  - o gate nao cria commits, nao roda `jj new`, nao move tags e nao publica.
- Modos documentados: `status`, `start ISSUE`, `done ISSUE`, `release VERSION`.
- Criado Review Canvas em `docs/harness/canvas/2026-06-05-jj-version-control-gate.md` porque a mudanca toca scripts/policy do harness.
- Validação nao executada nesta iteração por instrução operacional atual de nao rodar verificações sem pedido explícito.

## vc-gate release guard review fix — 2026-06-06

- Post-review for `memory-policy-layer` found `vc-gate.sh release` could print `release_gate=pass` without a release version.
- Fixed `docs/harness/bin/vc-gate.sh` so release mode requires `VERSION` or `vVERSION`.
- Targeted validation:
  - `bash -n docs/harness/bin/vc-gate.sh` — PASS.
  - `bash docs/harness/bin/vc-gate.sh release --allow-dirty` — expected FAIL with `release requires VERSION or vVERSION`.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Memory policy layer Phase 1 — 2026-06-06

- Added deterministic `heuristic-v1` memory policy scoring with durable `memory_policy` records.
- Added MCP tools for score, promote, decay, explain, and conflict reconciliation.
- Integrated optional retrieval-time policy reranking through `policy_rerank` without changing default search behavior.
- Preserved SQLite/FTS/vector/graph/provenance as canonical state; no automatic synthesized memory writes were added.
- Validation passed: focused checks, `make ci`, `doctor.sh`, and post-review gate.
- Post-review artifact: `docs/harness/reviews/2026-06-06-memory-policy-layer-v2-post.md` with `REVIEW_VERDICT: PASS`.

## Root folder organization — 2026-06-06

- Moved cloud/API reference docs from the repository root into `docs/`:
  `ARCHITECTURE.md`, `OPERATIONS.md`, `QUICKSTART.md`, `REFERENCE.md`, and
  `CONTROL_PLANE_SCHEMA.sql`.
- Kept root reserved for repository identity, build/config entrypoints,
  GitHub-visible policy files, and agent context files consumed by tooling.
- Updated known links in `AGENTS.md`, `CLAUDE.md`, `docs/QUICKSTART.md`, and
  `docs/README.md`.
- No `src/`, runtime schema, MCP surface, hooks, SDK, or harness gate behavior
  changes are included in this branch.

## Security reference harness enforcement — 2026-06-06

- Added `docs/harness/security/anthropic-reference-harness.md` as the canonical
  local contract for `ENGRAM-HARNESS-SECURITY-CONTRACT-v1`.
- Added versioned tuning files `.claude/scan-extras.txt` and
  `.claude/fp-rules.txt`; they augment scan/triage behavior and do not replace
  core invariants, gates, or review policy.
- Updated `doctor.sh` to fail closed when the security note, contract anchors,
  tuning files, or required cross-references are missing.
- Updated `bootstrap.sh`, `sensors.sh`, and `review-gate.sh` to surface the
  security boundary without adding autonomous execution to the default harness
  flow.
- Updated harness docs and onboarding docs so reviewers flag autonomous Engram
  execution, sandbox drift, credential mounts, egress expansion, and C/C++/ASAN
  pipeline import unless an ADR and explicit target contract exist.
- No autonomous execution pipeline, target runner, credential mount, or `src/`
  change is included in this branch.

## CI superseded-run cancellation + ENGRA-92 — 2026-06-07

- Added GitHub Actions `concurrency` for CI runs keyed by
  workflow/event/ref, so superseded pushes on `main` cancel older in-progress
  runs instead of leaving obsolete extended jobs consuming runners.
- Preserved the required PR gate shape: `fmt`, `clippy`, `Test
  (ubuntu-latest)`, and `Documentation`.
- Confirmed `Full Feature Tests` remains schedule/manual only for new pushes.
- Fixed `ENGRA-92` by making `.claude/scan-extras.txt` and
  `.claude/fp-rules.txt` identify themselves with the literal filenames that
  `doctor.sh` validates.
- Verification:
  - `git diff --check` — PASS.
  - YAML parse of `.github/workflows/ci.yml` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Huly backlog audit + ENGRA-74 — 2026-06-07

- Used the local Huly skill and Platform API token flow to read project `ENGRA`.
- Huly returned 22 `Backlog` issues; most were stale against the current repo
  state (rate-limit/observability/deploy docs, harness cross-improvements, and
  Operational Context foundations already exist locally).
- Implemented the remaining visible code gap for **ENGRA-74**:
  `context_get_artifact` now exposes explicit retained artifact retrieval over
  MCP with `artifact_id`, `reason`, scope fields, `max_bytes`, staleness, and
  redaction checks.
- Updated `docs/MCP_TOOLS.md` through the generator.
- Focused validation:
  - `cargo test context_get_artifact --test mcp_protocol_tests -- --nocapture`
    — PASS.
  - `./scripts/generate-mcp-reference.sh --check` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `make ci` — PASS.

## ENGRA-103 `memory_digest` MCP implementation — 2026-06-08

- Added read-only MCP tool `memory_digest` as the actionable retrieval entry
  point defined by RFC 0008.
- The v1 implementation is a thin orchestrator over existing primitives:
  `memory_smart_retrieve`, `memory_build_context`, `crossrefs`, and
  `context_build_bundle`.
- Preserved the product boundary: no schema migration, no LLM call, no memory
  mutation, no Dream candidate application, and no raw artifact content return.
- Response includes extractive summary/key points, top memory previews with
  IDs, relationships, Operational Context sections, next actions, provenance,
  and warnings.
- Updated MCP registry/dispatch and regenerated `docs/MCP_TOOLS.md`
  (`Total tools: 277`).
- Added protocol coverage for tools/list read-only metadata, successful
  dispatch with source relationships, input validation, and empty-source
  warnings.
- Validation passed: focused `memory_digest` protocol tests, MCP reference
  check, fmt, clippy, doctor, and full `sensors.sh` (`make ci` + doctor).
