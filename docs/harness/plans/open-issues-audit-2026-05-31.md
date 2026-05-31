# Open Issues Audit — 2026-05-31

**Scope:** 13 issues abertas (aiconnai/engram) + estado de `main` em 2026-05-31  
**Author:** engram harness operator (Claude Code CLI + Grok Build side-by-side)  
**Parent plan:** [docs/harness/plans/2026-05-31-code-all-issues-plan.md](./2026-05-31-code-all-issues-plan.md)  
**Status:** Início da Fase 1 (somente leitura + evidência)

## Evidência executada nesta sessão para Fase 1

- `./scripts/generate-mcp-reference.sh --check` → `docs/MCP_TOOLS.md is up to date`
- `git log --oneline -n 8 -- docs/MCP_TOOLS.md` → HEAD relacionado a `#38` no contexto de reference generator
- Inspeções de código:
  - `src/bin/cli.rs` (maintenance status + testes de shape/read-only)
  - `src/mcp/handlers/handoff.rs` (session_land atual; fundamentos de continuidade de sessão)
  - `src/hooks/session_end.rs` (produção de payload em `pending_injections`)
  - `src/intelligence/{session_context,project_context,context_builder}.rs` (bases para status/context)
- `docs/harness/` scripts e CI path filters ajustados (sem tocar no código produto nesta fase).

## Matriz por issue (estado de implementação vs AC)

| # | Título | Prioridade/Área | Status atual vs AC | Evidência chave | Próximo passo |
|---|---|---|---|---|---|
| 37 | Add harness verification evidence workflow | P1 / harness | **Não implementado**: sem convenção de manifesto persistente ligada a status, sem AC de suporte negativo e sem integração com records | Nenhum artefato dedicado em `src/` ou docs encontrado | Executar após #34 e #35 |
| 36 | Add harness_handoff for next-agent continuity | P1 / harness/context | **Parcial fundacional**: `session_land` existe, com campos ricos e checkpoint; faltam campos AC e regra de “sem evidência, sem claim de conclusão” | `src/mcp/handlers/handoff.rs` | Extensão do contrato + validações + opção de persistência via harness_record |
| 35 | Add harness_status assembler | P1 / context | **Não implementado como ferramenta CLI/MCP dedicada** | não há `harness_status` em `src` | Implementar após base de decisão (#28/#29/#30) |
| 34 | Add harness_record for decisions/handoffs/failures/verification | P1 / context | **Não implementado** | ausência de tool/módulo dedicado em `src/mcp` | Construir novo caminho principal de eventos (#36 dependente) |
| 32 | Markdown/Obsidian portability design | P2 / portability | **Não implementado** | nenhum documento novo de frontmatter/round-trip | Criar decisão/ADR |
| 31 | Benchmark prompt compression | P2 / context | **Decisão concluída (Fase 2.3):** stack local determinística como core; neural/external opcional | `docs/rfcs/0002-compression-benchmarks-for-context.md`; `cargo bench --bench token_reduction -- --nocapture`; testes de `compression_semantic` | Consolidar corpus de recall/fidelidade e decidir follow-up de implantação |
| 30 | Unify token budget + chunking | P1 / context | **Não implementado** | sem contador/token-aware no ponto de compaction | Depende de contratos de token model / compression |
| 29 | Search Index v2 RFC | P2 / search | **Não implementado** | sem RFC/ADR novo em `docs/` | Elaborar RFC com comparação operacional |
| 28 | Decide REST API vs MCP-only | P0 / docs | **Não implementado (status proposto pendente)** | sem decisão técnica final vinculante | Decisão vinculante antes de novos contratos de integração |
| 27 | Generate MCP tools reference from code | P0 / tooling | **Parcialmente avançado**: generator e check no CI parecem operacionais; requer validação contínua e remoção de contagens manuais | `./scripts/generate-mcp-reference.sh --check` passou; commit `abeda17` | Auditar drift residual e fechar linguagem de evidência |
| 26 | Define derived index health contract | P0 / storage/search | **Não implementado** | ausência de contrato de bytes esperados/lógicos e parâmetros ANN por doc formal | Criar RFC/ADR e testes |
| 25 | Add embedding queue hygiene policy | P0 / storage/workers | **Não implementado completo** | ausência de política de retry/pending/failed com retenção por doc formal + testes | Implementar em conjunto com #26/#30 |
| 21 | Add CLI maintenance status | P0 / maintenance | **Parcialmente implementado**: `maintenance_status` + `print_maintenance_status` + testes de shape/read-only em `src/bin/cli.rs` | `maintenance_status_matches_storage_health_shape`, `maintenance_status_is_read_only_for_storage_tables` | Validar AC completa de saída humana e JSON contra prompt de issue |

## Dependências de implementação (resumo)

- Decisões de produto (#28, #29, #31, #32, #26) devem preceder implementações de larga escala em #34–#37/#30/#25.
- #27 e #21 entram como pré-requisitos de higiene/robustez:
  - garantir que o MCP reference continue canônico no pipeline;
  - garantir que manutenção tenha contrato e cobertura estável antes de novas mudanças.
- Ordem executável formalizada em:
  - [phase1-4 dependency map](../decisions/phase1-4-dependency-map-2026-05-31.md)

## Conclusão de Fase 1 inicial

- O backlog ainda é em grande parte vivo, com 2 issues com fundação parcial (#27 e #21), 3 issues com fundação parcial de continuidade (#34–#36).
- Não houve alteração funcional de produto nesta Fase 1.
- Próximo bloco: fechar decisões contratuais (1.4/decisões) e preparar pacotes de implementação com eventos de `harness_record`.
