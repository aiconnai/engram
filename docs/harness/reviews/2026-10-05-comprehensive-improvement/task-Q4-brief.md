### Q4 — Cobertura por risco + contratos MCP/SDK [P1; QA/SDK; depende C1/Q1]

**Files:** criar `docs/quality/test-coverage-map.md`; `tests/mcp_protocol_tests.rs`, `tests/canonical_journey.rs`, `scripts/test-canonical-journey.sh`, `sdks/python/tests/`, `sdks/typescript/src/index.test.ts`, scripts live e geração de referência MCP existentes.

- [ ] Mapear invariant/contrato → teste → features → lane; distinguir mocks, integração real, protocolos e coverage executada. Não chamar percentual de arquivos com `#[test]` de code coverage.
- [ ] Cobrir bearer inválido, workspace cross-tenant, unknown tool/invalid params, envelopes/erros normalizados, timeout, async lifecycle, snake_case/camelCase e paginação. Usar o mesmo banco isolado na jornada stdio+HTTP; após denial, ler e confirmar que não houve mutação.
- [ ] Rodar `rtk proxy bash scripts/test-canonical-journey.sh`, suites MCP, `rtk npm --prefix sdks/typescript test`, `rtk npm --prefix sdks/typescript run type-check`, `rtk proxy python3 -m pytest sdks/python/tests -q`; scripts live aprovados separadamente. Pacotes wheel/tarball instalados também precisam provar contrato antes de publicação futura.
- [ ] Separar aceite offline (mocks/serialização) de live SDK compatibility. Esta última exige autorização de build/install/start e execução de `scripts/test-python-sdk-live.sh` e `scripts/test-typescript-sdk-live.sh` com wheel/tarball instalados contra HTTP local isolado, sem provider/produção. Se não autorizada/executada, reportar live como pendente, nunca inferir das suites mock.

**Aceite:** regressão de contrato quebra suite real; schema/reference/SDKs alinhados. **Rollback:** mudança pública e clients revertidos coordenadamente, sem publicar.
