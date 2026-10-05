### O1 — Observabilidade sem conteúdo proprietário [P2; operação; depende C1/C6/H2]

**Files:** `docs/OPERATIONS.md`, normalização de erros, tracing/metrics já existentes, stats/risk-register do harness; novos testes de redaction no módulo dono do log.

- [ ] Definir operação/correlação/resultado/duração/retry; teste com credencial sentinela, texto privado, query, filesystem path e erro de provider. Logs não podem conter esses payloads por default; CLI stdout/protocolo continuam corretos.
- [ ] Instrumentar poucos caminhos críticos: permission_denied, SQLITE_BUSY, provider timeout, recovery, gate mismatch. Agregar contagens sem ID/tenant/query como labels de alta cardinalidade.
- [ ] Reconciliar #178/#179 com tracing/counters/rate limiter existentes antes de nova instrumentação. Cobrir parse-error fora do handler, classificação MCP-error vs sucesso HTTP, correlação/method, SSE ativo e distribuição de latência conforme contrato aprovado; items excluídos são não-goals explícitos, não green implícito. 429/Retry-After têm regressão; nenhum dashboard de produção presumido.
- [ ] Exercício local dispara falha e demonstra alerta/owner/runbook/rollback, sem enviar mensagens externas. Separar disponibilidade local de SLO real de deployment.

**Aceite:** diagnósticos úteis sem vazamento; dashboards não chamam falta de amostra de zero erro. **Rollback:** desligar export extra sem perder mensagens de erro estruturadas.
