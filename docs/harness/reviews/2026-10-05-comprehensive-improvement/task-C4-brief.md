### C4 — Abertura SQLite descriptor-bound: investigação dedicada [P1; segurança/storage; independente após C2]

**Files:** ler progress sobre reversões, `src/storage/connection.rs`, `src/storage/lock.rs`; NEW ADR dedicado em `docs/decisions/` e sandbox spike descartável separado do core.

- [ ] Reproduzir threat model e compatibilidade com locking/pool/WAL; documentar por que pathname aliases e hardlinks rejeitados não resolvem o contrato. Não reutilizar antigo PASS como closure.
- [ ] Comparar shim nativo vs VFS auditado com matriz Linux/macOS/BSD, ABI, múltiplas conexões/processos, checkpoint e symlink/race. Timebox cinco dias; sem viable boundary, registrar risco residual e owner, não implementar workaround vulnerável.
- [ ] Submeter ADR/resultado e testes de viabilidade; implementação futura exige review especialista e plano de dados/portabilidade próprio.

**Aceite desta tarefa:** decisão de viabilidade verificável ou defer explícito; não exige declarar atomic opening resolvido. **Rollback:** descartar spike sem tocar bancos reais.
