# 2026-10-26 — Triagem dos minors de review adiados (programa de outubro)

**Tipo**: achados de review adiados. **Não** é exclusão de sensor: nenhum sensor é excluído por este
arquivo, e ele não serve como `--known-issue` de `sensors.sh`.
**Origem**: `docs/harness/reviews/2026-10-05-comprehensive-improvement/ledger.md` (linhas citadas abaixo).
**Estado de referência**: `origin/main` em `2e84a594` + branch local `fix/resolve-all-2026-10-29` (2026-10-29).
**Registrado em**: [`progress/2026-10-26-remediation.md`](../progress/2026-10-26-remediation.md) e
`risk-register.yaml` (RISK-0003, RISK-0004, RISK-0005).

Legenda de status: **mesclado no main** = correção publicada em `origin/main` com SHA citado;
**corrigido na branch** = corrigido e testado em `fix/resolve-all-2026-10-29`, aguardando review/merge humano;
**aberto** = nenhuma correção aplicada ainda; **parcial** = parte já tratada. Esta triagem preserva a estimativa
de risco original da frente WT5, sujeita à confirmação do dono.

| Ledger | Task | Achado | Status | Risco | Dono / próximo passo |
|---|---|---|---|---|---|
| 66 | C2 | `WalHeader::new` emite cabeçalho que `verify` rejeita (nomenclatura de endianness do checksum). Parcialmente tratado: `src/sync/wal_chain.rs` ganhou `checksum_words_le(magic)` e `stored_checksum()` e a lição está em `ERRORS_AND_LESSONS.md` (2026-10-05), que mantém o campo legado `is_little_endian_checksum` "por compatibilidade". `WalHeader::new` (só usado em `tests/wal_*`) ainda emite com o campo legado; não foi reproduzido por esta frente. | parcial | baixo (helper usado em testes) | lane R: teste de ida e volta `serialize → parse → verify_checksum` com BE e LE; renomear só com decisão de compatibilidade |
| 97 | C1 | `resources/subscribe` não considera o principal (oráculo de atividade entre workspaces) | **mesclado no main** (`14f44449` + `f53b75a0`, PRs #247 / #254 -> `2e84a594`, `tests/resource_subscription_auth_tests.rs`) | médio | aguarda receipt humano pós-gate |
| 98 | C1 | Sem teste e2e gRPC com o `EngramHandler` real. `tests/grpc_transport.rs` existe, mas roda sob exclusão documentada de sandbox ([`2026-05-31-grpc-transport-port-bind.md`](./2026-05-31-grpc-transport-port-bind.md)) | **mesclado no main** (`83e3816c`, PR #255 -> `2e84a594`, `tests/grpc_workspace_auth_tests.rs` com cliente/servidor tonic reais) | médio | aguarda receipt humano pós-gate |
| 107 | C1 | (a) `permission_denial_for_mode` público ainda fail-open para tool desconhecida; (b) `permission_mode_status_report` ignora escalada por flag de escrita | **mesclado no main** ((a) `4d4cdc8e`, (b) `a677c10e`, PR #255 -> `2e84a594`) | médio | aguarda receipt humano pós-gate |
| 235 | Q2F | `invalidate_for_workspace` do cache de resultados ignora `workspaces`/global, e a chave do cache não inclui o principal (resultado com escopo de principal pode ser servido a outro) | **mesclado no main (não reproduzido + teste fixado)** (`15b984ab`, PR #254 -> `2e84a594`) | baixo | reabrir só com reprodução |
| 245 | C6 | (i) afirmação de isolamento de panic qualificada no FINALFIX; (ii) `reset_interrupted_sync` concatenava `last_error` sem limite em loop de crash (`src/sync/worker.rs`); (iii) guard de Pull do worker só canonicalizava (sem hard-link/sidecar/fd alias de `active_artifacts`); (iv) zombie em `pid_alive` já substituído por `still_running_after_grace` + `is_zombie`; (v) retry de `EINTR` no `waitid` (`src/multimodal/process.rs`) e cleanup RAII de `.download-*.tmp` em cancelamento (`src/sync/cloud.rs`) | **corrigido na branch** (`fix/resolve-all-2026-10-29`, task S1, com testes unitários de crash-loop dedup/cap, active-artifact guard, `EINTR` e download guard) | baixo | review e merge humano no main |
| 254 | Q2F | Erros legados `{"error":string}` em caminhos fora de faixa de `memory_create`/`memory_boost`/`context_record_artifact` (ainda presentes em `src/mcp/handlers/memory_crud/`); `ignored_date_filter` sem consumidor; `write_unique_backup` casa por prefixo | aberto | baixo (contrato de wire) | normalizar exige coordenação: manter a forma atual até haver flag + anúncio (`development-workflow`: nunca mudar o wire format dentro de um refactor) |
| 308 | H5 | `base_references()`, `protected_changes()` e `workflow_blob()` em `docs/harness/bin/merge_gate_ci.py` agora falham fechado (`CiRefusal` `ci_base_unverifiable` / `ci_workflow_unverifiable`); `required_jobs` depende de indentação e o conjunto protegido amplo (`scripts/**`, `.github/**`) permanece deliberadamente conservador | **corrigido na branch (fail-closed)** (`fix/resolve-all-2026-10-29`, task H2, 3 testes herméticos em `test_merge_gate.py`) | baixo | exige review humano independente de `docs/harness/bin/*` antes do merge |

## Regras desta triagem

- Itens "em andamento" só viram "corrigido" com SHA de merge e teste de regressão citados; quem
  atualizar este arquivo cita o SHA. O integrador faz isso após o merge de WT1.
- Os demais minors do ledger (E0, Q1, H1-H6, O1-O4, Q2-Q7, C3-C7, G1, P1) **não** foram triados aqui;
  continuam apenas no ledger. Ausência neste arquivo não significa resolvido.
- Itens de wire format (254) mantêm a forma atual até haver flag e anúncio; itens de autorização
  (97, 107, 235) só mudam de status com a correção mesclada e testada, nunca por texto.
