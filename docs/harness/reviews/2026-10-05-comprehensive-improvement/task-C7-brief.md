### C7 — Embeddings, fila e health coerentes end-to-end [P1; storage/search; depende C2/Q4-offline]

**Files:** `src/embedding/queue/tests.rs`, `src/storage/sqlite_backend/health.rs`, `src/mcp/handlers/memory_crud/create.rs` e queries existentes; novo teste de integração somente se os módulos atuais não exercitarem a jornada completa.
**Interfaces:** manter enqueue no fluxo storage atual e embedding imediato após commit; health derivado de rows/flags/backlog existentes, não novo contador paralelo.

- [ ] Testar create deferred/immediate → worker → retry/drain/reopen com embedder determinístico. Em cada etapa conferir row de embeddings, `has_embedding`, queue, orphan count e backlog; falha após cálculo ou após enqueue não pode produzir sucesso inconsistente.
- [ ] Resolver contrato público `defer_embedding`: catálogo promete background queue, mas enqueue e embedding imediato são guardados por `!defer_embedding`. `test_deferred_create_enqueues_and_drains` exerce MCP real → job pending/health → drain → embedding/flag coerentes. Se intenção é nunca gerar embedding, mudança de contrato exige aprovação e compatibilidade, não silently redefinir defer.
- [ ] Injetar falha entre escrita de embedding, flag e completion; crash/reopen com job processing stale. Definir transação das escritas locais e retry idempotente; provider fora da transação. Recuperação automática ou manutenção explícita têm owner/runbook e health pendente visível; não chamar memória sem embedding/sem fila de saudável por ausência de backlog.
- [ ] Corrigir só discrepâncias demonstradas. Definir transição/idempotência de job por memória antes de alterar query; provider externo continua fora da transação. Missing row, row sem flag e órfãos são diagnósticos distintos.
- [ ] Rodar `rtk cargo test --lib embedding::queue::tests`, unit tests de health e protocolo create com matriz de features. Registrar casos executados e health após reopen, não só retorno do worker.

**Aceite:** caminhos de sucesso/falha convergem ou reportam pendência explícita; retry não duplica estado; API pública preservada. **Rollback:** revert de código; fila persistida exige compatibilidade com jobs já enfileirados.
