### C5 — Refatoração seletiva da persistência no handler [P2; core; depende C7/Q4]

**Files:** `src/mcp/handlers/memory_crud/create.rs`, `src/storage/queries/` e `tests/mcp_protocol_tests.rs`; inventário Q2 decide eventual split adicional, sempre em outro PR.

- [ ] Caracterizar criação com/sem embeddings, provider failure, duplicate e transaction failure. Assert resposta MCP, estado persistido e flags/index coerentes.
- [ ] Extrair escrita SQL de embedding para função storage tipada usando a conexão/transaction existente, sem fazer chamada de rede dentro dela. A criação principal já usa storage: não vender isso como correção de bypass inexistente.
- [ ] Testes before/after e benchmarks do hotspot; nenhum rename público nem split dos 98 arquivos em lote.

**Aceite:** fronteira mais clara com comportamento compatível e ausência de lock ampliado. **Rollback:** revert normal mantendo regression tests.
