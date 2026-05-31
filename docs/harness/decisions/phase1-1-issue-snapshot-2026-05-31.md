# Fase 1.1 — Snapshot inicial de Issues Abertas

**Data:** 2026-05-31  
**Arquivo:** [docs/harness/plans/2026-05-31-code-all-issues-plan.md](../plans/2026-05-31-code-all-issues-plan.md)  
**Escopo:** 13 issues abertas no repositório `aiconnai/engram` com validação inicial de evidência local.

## Resultado

- Existem 13 issues abertas e a maior parte já tem base parcial no main, especialmente #27, #21 e #36/#34/#35 foundations.
- As atividades de implementação do produto harness (issues #34–#37 e #30/#25/#28/#29/#31/#32/#26) ainda não foram concluídas nesta fase de v0.
- O trabalho de engenharia de 13 issues só pode avançar após:
  1. encerramento explícito do V0 com `REVIEW_VERDICT: PASS` + trilha de exclusão formal (quando aplicável);
  2. consolidação das evidências no final da Fase 1;
  3. sequência de decisões/rfcs obrigatórias.

## Estado inicial por issue (alto nível)

| Issue | Classificação | Estado base (main) | Posição da Fase 1 |
|---|---|---|---|
| #37 | P1/harness | Nenhuma implementação de evidência consolidada | Auditar detalhadamente; deve ser construído junto a #34/#36 |
| #36 | P1/harness | Fallback parcial `session_land` em MCP | Requer expansão estruturada e validação anti claims |
| #35 | P1/context | Não implementado como `harness_status` | Depende de decisões de schema de status (P0/P1 decisions) |
| #34 | P1/context | Não implementado como `harness_record` | Bloco fundacional do pacote harness |
| #32 | P2/docs | Não implementado | Iniciar após decisões de design #28/#29 |
| #31 | P2/context | RFC + decisão aceitas; benchmark de ratio/recall fixo adicionado em `benches/token_reduction.rs` | Requer follow-up de recall técnico amplo antes de expansão |
| #30 | P1/context | Não unificado | Depende de decisão de compressão/token model |
| #29 | P2/search | Não implementado (RFC pendente) | Requer decisão arquitetural antes de código |
| #28 | P0/docs | Sem decisão final vinculante | Deve ser decidido antes de novas mudanças de superfície |
| #27 | P0 | **Parcialmente entregue** (`docs/MCP_TOOLS.md` gerado por script + check no CI) | Confirmar no começo da 1.3 e encadear com issue doc |
| #26 | P0/storage/search | Sem contrato final | Dependente de decisão técnica (#29) |
| #25 | P0/storage/workers | Não implementado completo | Depende da arquitetura de queue/status unificada |
| #21 | P0/maintenance | **Parcialmente entregue** (`maintenance_status` + testes em CLI) | Auditar superfície README/CLI/help vs AC |

## Saída esperada desta Fase 1

- Concluir 1.2 (fonte única do plano) e 1.3 (auditoria completa com matriz de evidências).
- Registrar decisões e evidências em `docs/harness/decisions/` e `docs/harness/plans/open-issues-audit-2026-05-31.md`.
