### O4 — Standing checks read-only com ownership [P2; operação/CI; depende H3/H5/Q3]

**Files NEW:** `docs/harness/goals/registry.json`, `docs/harness/schemas/goal-v1.schema.json`, `docs/harness/tests/test_standing_checks.py`; workflow scheduled separado apenas após autorização de CI.
**Interfaces:** goal contém ID de check aprovado, owner, schedule enum, timeout e `on_failure=alert_only`; nenhum comando shell livre. Registry H3 fornece argv e limites máximos.

- [ ] Testar unknown check, goal com shell/comando extra, timeout acima da policy, owner ausente e concorrência do mesmo goal → recusa. Timeout/failure/missing artifact jamais vira pass; testar dispatched/manual/scheduled.
- [ ] Executar somente check aprovado em sandbox/read-only, com concurrency lock, run SHA/policy/toolchain/log e nenhuma credencial de produção. Nenhuma auto-remediação/commit/PR criada pelo scheduler.
- [ ] Rodar unittest offline com fake check. Handoff de falha para owner exige canal e autorização humana próprios; relatório local pode demonstrar alerta sem enviar mensagem real.

**Aceite:** agendamento não amplia autoridade e tem falha rastreável; scheduler não inicia writer. **Rollback:** pausar schedule e preservar receipts, sem mudar baseline do goal.
