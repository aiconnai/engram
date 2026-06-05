# Engram Harness Spec

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Harness Engineering v0 — bootstrap & core gates` |
| Active task | `harness-bootstrap — implement operational harness (bootstrap, doctor, sensors, review-gate)` |
| Started | `2026-05-30` |
| Owner | `Ronaldo + agents (Claude + Grok Build side-by-side)` |
| Active spec | `docs/harness/SPEC.md` |
| Active plan | `docs/harness/progress/2026-05-30-harness-bootstrap.md` |
| Tracker | RFC 0001 + this harness adoption (ENGRA-22 area) |

## Escopo da sprint ativa

> **Este SPEC é o escopo da sprint/tarefa ativa do harness. Não é o blueprint completo do produto.**
> - Blueprint de Harness Memory (produto): [`docs/rfcs/0001-harness-memory-product-boundary.md`](../rfcs/0001-harness-memory-product-boundary.md)
> - Regras de dados (sistema): [`../INVARIANTS.md`](../../INVARIANTS.md)
> - Standards gerais: [`../../STANDARDS.md`](../../STANDARDS.md)
> - Erros & lições: [`../../ERRORS_AND_LESSONS.md`](../../ERRORS_AND_LESSONS.md)

## Sprint ativa: Harness Engineering v0 — bootstrap & core gates

- **Branch**: (branch atual do trabalho)
- **Progress log**: [`progress/2026-05-30-harness-bootstrap.md`](./progress/2026-05-30-harness-bootstrap.md)
- **Status**: active — implementação inicial do harness operacional inspirado no modelo mbras-backend, adaptado para Rust + MCP + multi-SDK + dual CLI (Claude Code + Grok Build).

### Em escopo (v0)

- Criar estrutura `docs/harness/` completa (README, SPEC, INVARIANTS, GATES, CODE_REVIEW_POLICY, progress.md, bin/ scripts, reviews/, known-issues/, progress/).
- Implementar `bootstrap.sh` — orientação rápida, read-only, <50 linhas, imprime estado + ordem de leitura.
- Implementar `doctor.sh` — validação de consistência do harness (drift SPEC/progress, executáveis, referências à policy, bootstrap size, etc.).
- Implementar `sensors.sh` — wrapper determinístico sobre `just ci` (fmt + clippy -D + testes paridade Linux + docs + MCP ref) + harness doctor + checks específicos de engram.
- Implementar `review-gate.sh` — generalizado para múltiplos reviewers (claude, grok, codex, local). Suporte a pre/post, range, continuity em FAILs, versionamento de artefatos, exclusão de paths do harness, timeout, prompt rico com fake-success patterns de Rust/MCP.
- Implementar `check-commit-msg.sh` — validador de Conventional Commits com scopes engram/harness.
- Criar `CODE_REVIEW_POLICY.md` adaptada para Rust, engram (MCP tools, hooks, embeddings, storage invariants, cross-SDK), e o cenário dual-CLI atual.
- Criar `GATES.md` com thresholds, fake-success patterns específicos (ex.: tests passando só com features locais mas falhando em CI Linux, schema version drift, MCP protocol breakage, embedding cache bounds violados, etc.).
- Criar `WHAT_WE_DONT_DO.md` como política explícita de escopo negativo para evitar expansão silenciosa de mudanças de harness.
- Criar `docs/harness/canvas/` com template de Review Canvas para mudanças complexas.
- Adicionar `baseline.sh`, `quarterly-audit.sh`, lanes opcionais em `sensors.sh` e guard de review para `docs/harness/bin/*`.
- Atualizar `AGENTS.md` e `Claude.md` para exigir `bootstrap.sh` no início de toda sessão.
- Atualizar pre-commit hook e/ou justfile para reforçar (sem quebrar fluxo atual).
- Seed de progresso para esta sprint + registro de decisões.
- Rodar o loop completo (bootstrap → pre → sensors → post) nesta própria implementação.
- Documentar como o harness se relaciona com o RFC 0001 (Harness Memory product boundary) e dogfooding futuro.

### Gates esperados para v0

- `bash docs/harness/bin/bootstrap.sh`
- `bash docs/harness/bin/doctor.sh`
- `bash docs/harness/bin/sensors.sh`
- `bash docs/harness/bin/review-gate.sh pre harness-bootstrap`
- `bash docs/harness/bin/review-gate.sh post harness-bootstrap`
- `just ci` (paridade mantida)

### Fora de escopo (v0)

- Implementação completa de ingestão automática de eventos de harness no próprio engram (deixa para ENGRA-22+ seguindo RFC 0001).
- Mudanças em storage schema, MCP tools novos, ou intelligence/consolidation.
- Suporte nativo non-interactive exec para todos os CLIs (foco em prompt files + paste workflow para o cenário atual Claude + Grok Build side-by-side).
- Substituição total de `.githooks` ou CI GitHub workflows (o harness complementa, não substitui).
- Reabrir ou alterar RFC 0001 neste escopo.
- Mudanças em `INVARIANTS.md` (raiz) ou `STANDARDS.md` — apenas docs de processo do harness.

## Próximas iterações (v1+)

- Dogfooding: usar engram MCP + hooks para registrar sessões de harness, reviews, gate results como memórias com provenance forte.
- `memory_harness_*` tools ou seção dedicada.
- Agentes especializados em harness (planner, verifier, context-engine) como MCP tools ou personas em `docs/harness/agents/`.
- Integração mais profunda com Grok Build TUI (se expuser APIs/exec modes).
- Suporte a Linear/GitHub sync de tasks no harness (se aplicável ao fluxo de engram).

## Emenda 2026-06-05 — cross-harness improvements

O plano `docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md` adiciona melhorias inspiradas no harness mbras sem importar comportamento de domínio externo:

- Escopo negativo explícito em `WHAT_WE_DONT_DO.md`.
- Review Canvas para evidência em mudanças complexas.
- Guard para mudanças em `docs/harness/bin/*`.
- Baseline snapshot barato.
- Lanes opcionais em `sensors.sh` que não substituem o gate completo.
- Auditoria periódica evidence-only.

## Critérios de Saída da Sprint v0

- Estrutura completa + scripts executáveis + doctor.sh verde.
- Esta própria task passou pelo loop completo (pre + sensors all-green + post PASS).
- AGENTS.md e Claude.md atualizados e bootstrap rodado com sucesso.
- README do harness explica o posicionamento de engram como Memory Manager ideal para harnesses de outros projetos.
- Nenhum drift entre SPEC.md e progress.md; doctor.sh passa limpo.

---

**Nota**: Este SPEC é mutável durante a sprint. Atualizações de escopo vão para o log de progresso e (quando relevante) para um novo version do SPEC com nota de data. Invariants do harness não mudam sem ADR + gates anteriores.
