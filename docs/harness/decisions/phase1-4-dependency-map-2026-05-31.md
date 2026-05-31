# Fase 1.4 — Ordenação de Dependências e Corte de WIP

**Data:** 2026-05-31  
**Referência:** [open issues audit](../plans/open-issues-audit-2026-05-31.md)

## Resultado

- A auditoria identificou 13 issues com mistura de fundamentos parciais e lacunas.
- O trabalho não pode prosseguir em bloco único porque há bloqueios de decisão documentáveis em #28, #29, #31, #32 e #26.
- O próximo **slice executável** é:
  1) Fechar decisões/rfc de arquitetura e contratos (Fase 2),
  2) consolidar higiene de saída/geração (#27, #21, #25),
  3) implementar suíte harness core (#34–#37),
  4) fechar contratos complementares (token/unificação #30).

## DAG recomendado (ordem de execução)

- Fase 0 (sprint v0) concluída com evidência.
- Decisões fundacionais:
  - #28 MCP-only + alinhamento HTTP/SDK
  - #29 Search Index v2 RFC
  - #26 Contrato de saúde de índices derivados (depende de #29)
  - #31 Benchmark de compressão/preprocessamento (decisão antes de implementar tokenização unificada)
  - #32 Portability design (decisão antes de execução)
- Hygiene/contrato técnico:
  - #27 gerar MCP tools reference + checks contínuos (deve permanecer canônico)
  - #21 maintenance status (contrato completo de saída + testes)
  - #25 embedding queue hygiene (após contratos de estado/índices)
- Núcleo harness:
  - #34 harness_record
  - #35 harness_status
  - #36 harness_handoff (depende #34 para persistência opcional e #35 para consistência de estado)
  - #37 verification evidence manifest (depende #34/#35)
- Complementares:
  - #30 unificação token budget/chunking (depende decisão #31)

## Critérios de aprovação deste mini-artifato

- Cada nó do DAG acima com link de evidência em `open-issues-audit.md`.
- Apenas um bloco de WIP ativo por vez no log (evitar mudanças em duas frentes de decisão técnica sem trilha de handoff entre elas).
- Nenhum issue de implementação iniciado sem:
  - decisão bloqueadora concluída,
  - plano no docs/harness (referência única),
  - trilha de progresso atualizada.

## Resultado

- **Aprovado.** Próxima ação: executar Fase 2.2 (finalizar decisões de #29 com RFC) e #31/#32, em seguida fechar blocos de implantação por dependência.
