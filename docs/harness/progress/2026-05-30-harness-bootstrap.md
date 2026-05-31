# Progress Log — Harness Engineering v0 (bootstrap & core gates)

**Sprint**: Harness Engineering v0 — bootstrap & core gates
**Task**: harness-bootstrap — implement operational harness (bootstrap, doctor, sensors, review-gate)
**Date started**: 2026-05-30
**Owner**: Ronaldo + agents (Claude Code CLI + Grok Build TUI side-by-side)

---

## 2026-05-30 — Session 1: Exploration + Scaffolding

### Contexto da sessão

Usuário quer implementar "harness engineering" no projeto engram, usando como referência o diretório `docs/harness` do mbras-backend (mbras-tech). A motivação é profunda: comparar Claude Code CLI vs Grok Build TUI como ambientes primários de engenharia agentic, com a tese de que "the terminal is the product" e que harnesses reais (com Context Engine, Planner, Memory Manager, Verifier, Tool Registry, Harness Config) devem viver dentro do repo/terminal, não em UIs bolt-on.

Enfram é o candidato natural para o **Memory Manager** layer porque ele já provê:
- Armazenamento persistente + hybrid search + graph
- 155+ ferramentas via MCP
- Hooks de ciclo de vida (session_start, session_end, post_tool_use, stop)
- Inteligência (consolidação, auto-tagging, context building, etc.)
- RFC 0001 já define o product boundary para "Harness Memory"

### Ações realizadas

1. **Exploração da referência**:
   - Listado e lido `docs/harness/README.md`, `SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`
   - Lidos `bin/bootstrap.sh`, `doctor.sh`, `sensors.sh`, `codex-gate.sh` (parcial)
   - Entendido o loop completo, o contrato de exclusão de sensores, o versioning de reviews, a continuity de FAILs, o doctor de consistência, e o papel do "reviewer externo" (Codex no reference; generalizado aqui para dual-CLI).

2. **Exploração do estado atual de engram**:
   - `Agents.md`, `Claude.md`, root `INVARIANTS.md`, `STANDARDS.md`, `ERRORS_AND_LESSONS.md`
   - `justfile` + `scripts/ci.sh` (excelente gate de paridade Linux já existente)
   - `.githooks/pre-commit` (fmt + clippy)
   - `docs/rfcs/0001-harness-memory-product-boundary.md` (muito alinhado com a visão do usuário)
   - Estrutura de MCP handlers, hooks, intelligence, storage/migrations, etc.

3. **Decisões de design para adaptação**:
   - Harness é **complementar** (não substitutivo) aos gates existentes (`just ci` é o core).
   - `review-gate.sh` deve ser flexível para o cenário real do usuário (Claude + Grok Build side-by-side): priorizar "gerar prompt rico + salvar artefato" sobre exec não-interativo (pelo menos em v0).
   - Manter o "Single-Process Judgment" (implementador ≠ reviewer final).
   - Dogfooding com engram (MCP + hooks para registrar eventos de harness) é objetivo explícito, mas escopo de sprints futuras (guiado por RFC 0001).
   - Estrutura de arquivos e nomes seguem o reference o mais fiel possível para portabilidade de conhecimento entre projetos que adotam o harness.

4. **Scaffolding inicial**:
   - Criado `docs/harness/{bin,progress,reviews,known-issues}/`
   - Escrito `README.md` (guia operacional completo, filosofia, loop, adaptações engram)
   - Escrito `SPEC.md` (escopo v0, em/fora de escopo, critérios de saída)
   - Escrito `INVARIANTS.md` (18 regras de processo, categorizadas)
   - Escrito `GATES.md` (camadas, thresholds, 10 fake-success patterns específicos de engram)
   - Escrito `CODE_REVIEW_POLICY.md` (adaptada para Rust/MCP/dual-CLI)
   - Escrito `progress.md` (live state)
   - Escrito este log `progress/2026-05-30-harness-bootstrap.md`

### Evidência de gates até o momento

- Nenhum sensor formal ainda (scripts não implementados).
- Criação de docs seguiu a ordem de leitura (exploração primeiro).
- Mudanças foram em `docs/harness/**` (área permitida para iteração inicial do harness).

### Decisões que afetam trabalho futuro

- O harness de engram será o "reference implementation" para projetos que queiram usar engram como Memory Manager de seus próprios harnesses (alinhado com RFC 0001).
- `review-gate.sh` será o ponto de integração mais visível com a tese "Claude + Grok Build side-by-side".
- Futuro: MCP tools ou seções específicas para `memory_harness_event`, `identity_agent_session`, etc.

### Próximos passos (imediato)

- Implementar `bin/bootstrap.sh` (baseado no reference, adaptado para engram: git, just, mcp tools count, etc.).
- Implementar `bin/doctor.sh`.
- Implementar `bin/sensors.sh` (delegando para `just ci` + doctor).
- Implementar `bin/review-gate.sh` (prompt builder + artifact writer + verdict parser + multi-reviewer support).
- Implementar `bin/check-commit-msg.sh` (pode ser mais simples que o reference).
- Atualizar `AGENTS.md` e `Claude.md`.
- Rodar o loop completo nesta sprint.
- Commit + post-gate.

### Observações / Lições desta iteração

- O reference harness é extremamente maduro e bem pensado (especialmente o tratamento de continuity de reviews, exclusões auditáveis, e o doctor de auto-consistência). Copiar a disciplina é mais valioso que copiar cada linha de script.
- engram já tinha boa disciplina de CI/paridade (just ci + scripts/ci.sh). O harness adiciona principalmente a camada de "memória canônica para agentes" + review cross-CLI.
- A existência do RFC 0001 mostra que a tese do usuário já estava sendo pensada internamente no projeto. Esta implementação é a concretização operacional.

---

## 2026-05-31 — Session 2: Phase 0 Stabilization, Exclusion Trail, and PASS Evidence

### Contexto da sessão

Objetivo: fechar o ciclo obrigatório do v0 (bootstrap + doctor + sensors + post review) antes da Fase 1 dos 13 issues.

### Evidências executadas

1. Releitura e validação inicial:
   - `bash docs/harness/bin/bootstrap.sh`
   - `bash docs/harness/bin/doctor.sh`

2. Preparação de trilha de exclusão formal:
   - Criado `docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md`.
   - Registrado em `docs/harness/progress.md` e log atual para satisfazer contrato pré-registro do `--exclude-sensor`.

3. Ajuste de sensores para suportar mapeamento documental de falha conhecida:
   - `docs/harness/bin/sensors.sh` já reconhece o sensor `grpc-transport` como exceção documentada e, em fail com assinatura de bind `Operation not permitted` em `tests/grpc_transport.rs`, registra `pass_with_exclusion`.

4. Execução full sensors com trilha formal:
   - `bash docs/harness/bin/sensors.sh --exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason "sandbox socket bind restriction"`
   - Resultado: `pass_with_exclusion` em `.sensors-last` (timestamp `2026-05-31T05:17:18Z`), com nota de limitação.

5. Review gate:
   - `bash docs/harness/bin/review-gate.sh pre harness-bootstrap` gerou `2026-05-31-harness-bootstrap-v7-pre.md(.raw)`.
   - Revisão enviada manualmente em `docs/harness/reviews/2026-05-31-harness-bootstrap-v7-post.md` com:
     - `PASS ...`
     - `REVIEW_VERDICT: PASS ...`
   - `bash docs/harness/bin/review-gate.sh post harness-bootstrap --review-file docs/harness/reviews/2026-05-31-harness-bootstrap-v7-post.md`
   - Artefato final: `docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md(.raw)`

### Resultado da sessão

- Estado de sensores: `.sensors-last` = `status=pass_with_exclusion`.
- Estado de review: `docs/harness/reviews/2026-05-31-harness-bootstrap-v7-post.md` com marcador `REVIEW_VERDICT: PASS ...`.
- `bootstrap.sh` atualizado via execução local para refletir revisão mais recente.
- Decisão de continuidade: v0 sai do estado bloqueado por `FAIL`; ainda depende apenas de limpeza/alternativa de `grpc_transport` para produção closure, mas segue a ordem definida para avançar com trilha documentada.

### Próximos passos imediatos pós-sessão

- Consolidar a evidência de Fase 1 (audit) com estado atual e mini artefatos.
- Decidir se abre ADR para `grpc_transport` em ambiente local ou registrar risco técnico aceito com pass_with_exclusion.

---

## 2026-05-31 — Session 3: Fase 1 (Planejamento/auditoria) iniciada

### Contexto da sessão

Com V0 fechado para execução de engenharia (pass/revisão + trilha de exclusão formal já registrada), iniciamos Fase 1 com foco estritamente em planejamento e auditoria, sem implementar features dos issues ainda.

### Ações realizadas

1. Gerou mini decisões de Fase 1:
   - `docs/harness/decisions/phase1-1-issue-snapshot-2026-05-31.md`
   - `docs/harness/decisions/phase1-2-plan-source-unification-2026-05-31.md`
   - `docs/harness/decisions/phase1-3-open-issues-audit-2026-05-31.md`
2. Produziu matriz auditável por issue:
   - `docs/harness/plans/open-issues-audit-2026-05-31.md`
3. Coletou evidência local para os pontos já parcialmente avançados:
   - `./scripts/generate-mcp-reference.sh --check` (PASS, MCP_TOOLS atualizado)
   - `src/bin/cli.rs`: implementação `maintenance_status` + `print_maintenance_status` + testes de shape/read-only
   - `src/mcp/handlers/handoff.rs`: base de continuidade de sessão (`session_land`) com query de open items/decisions e checkpoint
   - `src/hooks/session_end.rs`: payload de encerramento de sessão já enfileirado em `pending_injections`

### Resultado da sessão

- Nenhuma alteração funcional foi feita nesta fase de planejamento.
- Backlog auditável consolidado em arquivo de fase única.
- Pronto para iniciar decisões de contrato (Fase 2) com base nas lacunas acima.

### Observações

- O plano canônico continua como `docs/harness/plans/2026-05-31-code-all-issues-plan.md`; não foi criado arquivo paralelo concorrente.
- Não houve mudanças de código fonte além dos artefatos de disciplina já finalizados em sprints anteriores.

## 2026-05-31 — Session 4: Fase 2.1 (Decisão #28) iniciada

### Contexto da sessão

Com a Fase 1 concluída em evidência (sem mudanças funcionais de código), inicia-se a Fase 2 com a decisão mandatória de integração:

- #28 — decidir entre REST API local e MCP-only.

### Ações realizadas

1. Leitura de evidência operacional:
   - `src/bin/server.rs` (`TransportMode`, flags `--transport`, `--http-port`, `--grpc-port` quando feature ativa).
   - `src/mcp/http_transport.rs` (`POST /mcp`, `GET /v1/events`).
2. Criação de decisão rastreável:
   - `docs/harness/decisions/phase2-1-issue-28-rest-vs-mcp-only-2026-05-31.md`

### Resultado da sessão

- Decisão tomada: **MCP-only** como superfície canônica local (HTTP é transporte de MCP, não REST CRUD de domínio).
- Próximo marco: Fase 2.2 com decisões #29/26/31/32 e atualização coordenada de docs/SDKs antes de mudanças de código das features harness (#34–#37).

**Status ao final da sessão**: Scaffolding dos docs de processo completo. Scripts e execução do loop pendentes. Pronto para implementar os bin/ na próxima iteração da mesma sprint.

## 2026-05-31 — Session 5: Fase 2.2 (Search Index v2) iniciada

### Contexto da sessão

Continua a Fase 2 com o primeiro desbloqueador técnico seguinte a #28:
- definir o caminho documental da comparação de índices para #29.

### Ações realizadas

1. Levantamento de arquitetura de busca e storage:
   - `src/search/mod.rs` (BM25/Fuzzy/Hybrid).
   - `src/search/*.rs` para sinais de capacidades já existentes.
   - `src/storage/sqlite_backend.rs` (saúde de índices derivados em `sqlite_fts_health`, `sqlite_graph_health`, `sqlite_embedding_health`).
   - `src/storage/meilisearch_backend.rs` e `src/storage/meilisearch_indexer.rs` (feature-gated, uso opcional).
2. Criação de decisão rastreável para #29:
   - `docs/harness/decisions/phase2-2-search-index-v2-rfc-2026-05-31.md`.

### Resultado da sessão

- Falha técnica identificada: não existe ainda RFC comparativa formal para v2; esta issue permanece **bloqueada por design-first**.
- Decisão provisória operacional:
  - manter SQLite+FTS5 como base canônica atual;
  - manter Meilisearch como opção feature-gated;
  - exigir documento RFC formal antes de mudar ou unificar contratos de índice derivado.
- Dependência direta apontada para #26 (contrato de saúde de índices derivado) e #25 (higiene operacional de fila).

## 2026-05-31 — Session 6: Fase 0 residual + Fase 1.4

### Contexto da sessão

Fechamento final de pontos de risco apontados na revisão crítica dos sensores, sem alterar o escopo funcional das 13 issues ainda.

### Ações realizadas

1. Ajuste de mapeamento de exclusão em `docs/harness/bin/sensors.sh`:
   - `is_expected_excluded_failure()` agora reconhece falhas por panic em `thread '...' panicked` (ex.: `tests/grpc_transport.rs`) além do padrão legado `test ... FAILED`.
   - Padrão de `grpc-transport` foi expandido para cobrir nomes de cenários (`scenario_*`) com validação dos padrões de erro.
2. Atualização de traços de Fase 1:
   - Adicionado `docs/harness/decisions/phase1-4-dependency-map-2026-05-31.md`.
   - Atualizados `docs/harness/plans/open-issues-audit-2026-05-31.md` e `docs/harness/plans/2026-05-31-code-all-issues-plan.md` para refletir a subfase 1.4.
3. Verificações executadas:
   - `bash docs/harness/bin/sensors.sh --exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason "plan says pass_with_exclusion acceptable"`
     - Resultado final: `PASS_WITH_EXCLUSION (...)`.
   - `bash -n docs/harness/bin/sensors.sh` — OK.

### Resultado da sessão

- O risco de falso negativo na exclusão de `grpc-transport` foi reduzido para casos de panic.
- O bloco de planejamento está com Fase 1.4 registrada e o próximo passo em execução é continuidade de decisões de fase de contrato/implementação.

## 2026-05-31 — Session 7: Fase 2.3 (benchmark de compressão de prompt)

### Contexto da sessão

Com decisões #28 e #29 registradas, avançou-se a decisão #31 para evitar implementação sem benchmark.

### Ações realizadas

1. Execução de benchmark base já existente:
   - `cargo bench --bench token_reduction -- --nocapture`
   - Observado tempo estável em outputs de `OutputFilter`, `TruncationEngine` e pipeline completa (`OutputFilter -> TruncationEngine`), com micro-latências compatíveis ao uso local.
2. Criação do RFC de comparação:
   - `docs/rfcs/0002-compression-benchmarks-for-context.md`
3. Registro da decisão de engenharia:
   - `docs/harness/decisions/phase2-3-compression-benchmark-2026-05-31.md`
4. Atualização de rastreabilidade de backlog:
   - `docs/harness/plans/open-issues-audit-2026-05-31.md`
   - `docs/harness/plans/2026-05-31-code-all-issues-plan.md`

### Resultado da sessão

- Decisão formal de Fase 2.3:
  - usar stack de compressão local determinística como core;
  - classificar compressão neural externa como **opcional** e bloqueada para posterior RFC com evidência dedicada.
- Pronto para fechar as decisões restantes da Fase 2 e seguir para Fase 4.

## 2026-05-31 — Session 8: estabilização pós-review de compressão

### Contexto da sessão

Antes de retomar implementação dos issues abertos, a sessão voltou para Fase 0
residual: validar bootstrap/doctor/sensors e registrar a resolução da revisão
da stack de compressão.

### Ações realizadas

1. Bootstrap e leitura obrigatória do harness executados antes de planejar/editar.
2. `bash docs/harness/bin/doctor.sh` passou limpo.
3. `bash docs/harness/bin/sensors.sh` passou limpo, sem exclusão de
   `grpc-transport` (`status=pass`, timestamp `2026-05-31T15:52:49Z`).
4. Compressão:
   - `ContextCompressor` mantém diagnósticos de orçamento/skips.
   - `memory_compress_for_context` expõe `budget_used`, `budget_remaining` e
     `skipped_memory_ids`.
   - Dedupe semântico preserva endpoints técnicos distintos e agora normaliza
     pontuação final em tokens técnicos.
5. Evidência de benchmark registrada:
   - `docs/harness/reviews/2026-05-31-compression-benchmark-ratio-recall.md`.
6. Review externo via CodeRabbit apontou muitas ocorrências de reflow em docs
   gerados; apenas itens com impacto comportamental foram aplicados:
   - tokenizer técnico em `src/intelligence/compression_semantic.rs`;
   - validação de modo e parsing de verdict em `docs/harness/bin/review-gate.sh`;
   - parsing/scope de commit message em `docs/harness/bin/check-commit-msg.sh`.

### Verificações executadas

- `cargo test test_technical_tokens_normalize_trailing_punctuation` — PASS.
- `cargo test test_deduplication_preserves_distinct_technical_endpoints` — PASS.
- `bash -n docs/harness/bin/review-gate.sh` — PASS.
- `bash -n docs/harness/bin/check-commit-msg.sh` — PASS.
- `bash docs/harness/bin/check-commit-msg.sh --message 'fix(harness): harden review gates'` — PASS.
- `bash docs/harness/bin/check-commit-msg.sh --message 'fix(rfc-0001): update benchmark evidence'` — PASS.
- `bash docs/harness/bin/check-commit-msg.sh --message 'fix(anything-goes): reject generic scope'` — FAIL esperado.
- `cargo fmt --all` — PASS.

### Resultado da sessão

- Baseline determinístico está verde sem exclusões.
- Post review manual fechado com `REVIEW_VERDICT: PASS` em
  `docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md`.
- `review-gate.sh post harness-bootstrap --review-file ...v9-post.md` retornou
  `POST-GATE PASS`.
- Observação: este PASS foi criado por Codex/manual sob instrução explícita do
  usuário, não por reviewer cross-CLI independente.

### Correções de code review aplicadas

- `docs/harness/bin/review-gate.sh` passou a usar `set -euo pipefail`, evitando
  continuidade silenciosa em falhas não tratadas.
- Comentário de `REVIEWER_CLI` corrigido para `grok`.
- `CLAUDE.md` passou a referenciar `docs/harness/INVARIANTS.md`,
  `docs/harness/GATES.md`, `docs/harness/CODE_REVIEW_POLICY.md` e
  `docs/harness/progress.md` explicitamente.
- Adicionado teste de contrato MCP:
  `test_memory_compress_for_context_reports_skip_metadata`.
- Review manual posterior:
  - `has_distinct_technical_content()` agora preserva qualquer diferença entre
    conjuntos de tokens técnicos, incluindo superset.
  - `memory_compress_for_context` aceita `memory_ids` como alias de `ids`,
    protegendo os SDKs Python/TypeScript existentes.
  - `fixed_corpus_ratio_recall` passou a usar piso fixo explícito de 7/9,
    em vez de derivar o floor da execução atual.

## 2026-05-31 — ENG-1295 / #26 passo 3: output humano de manutenção

### Ações realizadas

- Extraída renderização de `maintenance-status` para
  `write_maintenance_status<W: Write>`.
- `print_maintenance_status` permanece como wrapper sobre stdout.
- A saída humana agora inclui `Derived indexes:` com nome, tipo, status e
  contadores principais (`source`, `indexed`, `pending`, `stale`, `failed`,
  `orphaned`).
- Adicionado teste `maintenance_status_human_output_includes_derived_indexes`.

### Verificações

- `cargo fmt --all -- --check` — PASS.
- `cargo test maintenance_status` — PASS.
- `cargo test test_health_check_reports_` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.

## 2026-05-31 — ENG-1296 / #32 endurecimento da superfície operacional da fila

### Contexto da sessão

Com a base de `maintenance-status` estabilizada pelo #26, a iteração reaproveita a
superfície existente para expor detalhes operacionais de fila com granularidade útil
para diagnóstico e degradação.

### Ações realizadas

- Atualizado `src/storage/sqlite_backend.rs`:
  - `sqlite_embedding_health` agora inclui em `details` os contadores de fila
    `pending`, `processing`, `stale_processing`, `failed`,
    `retryable_failed`, `exhausted_failed`, `max_retry_count`.
  - Adicionado `oldest_pending_age` e mantido alias legível
    `oldest_pending_age_seconds`.
- Atualizado `src/bin/cli.rs`:
  - `maintenance-status` humano renderiza linha `queue-state` com os novos
    contadores.
  - Aceita fallback de chave legada `oldest_pending_age_seconds`.
- Adicionados testes para cobertura de forma/estado:
  - `maintenance_status_matches_storage_health_shape`
  - `maintenance_status_human_output_includes_embedding_queue_state_counters`
  - `test_health_check_embedding_details_include_queue_state_counters`.

### Verificações

- `cargo test maintenance_status_ -- --nocapture` — PASS.
- `cargo test test_health_check_embedding_details_include_queue_state_counters -- --nocapture` — PASS.
- `cargo test embedding_queue_health_counts_stale_and_retries -- --nocapture` — PASS.
- `cargo clippy --all-targets --tests -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.

## 2026-05-31 — ENG-1296 / #25 política explícita de higiene da fila

### Contexto da sessão

A iteração fecha a transição de visibilidade operacional para controle explícito:
health/status continuam read-only, e reparos de `embedding_queue` passam por uma
ação de manutenção dedicada.

### Ações realizadas

- Adicionado `EmbeddingQueueHygieneConfig` com thresholds de stale processing,
  retry budget e retenção de linhas `complete`.
- Expandido `EmbeddingQueueHealth` com zero-retry failed, idades oldest
  processing/failed e buckets de retry.
- Adicionado `run_embedding_queue_hygiene` com dry-run/apply para:
  - requeue de `processing` stale ainda dentro do budget;
  - marcação como `failed` para stale exhausted;
  - requeue explícito de `failed` retryable;
  - prune de linhas `complete` além da retenção.
- Adicionado `maintenance queue-hygiene` no CLI com `--apply`,
  `--dry-run`, `--requeue-failed` e `--json`.
- Atualizados `maintenance-status`, `sqlite_embedding_health` e `docs/SCHEMA.md`
  para refletir os novos campos e a política operacional.

### Verificações

- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-targets --tests -- -D warnings` — PASS.
- `cargo test maintenance_status_ -- --nocapture` — PASS.
- `cargo test test_health_check_embedding_details_include_queue_state_counters -- --nocapture` — PASS.
- `cargo test test_embedding_queue_health_counts_stale_and_retries -- --nocapture` — PASS.
- `cargo test test_embedding_queue_hygiene_dry_run_does_not_mutate_and_apply_can_repair -- --nocapture` — PASS.
- `cargo test maintenance_queue_hygiene_dry_run_does_not_mutate_and_apply_updates -- --nocapture` — PASS.
- `bash docs/harness/bin/bootstrap.sh && bash docs/harness/bin/doctor.sh` — PASS.
