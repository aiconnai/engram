# 2026-10-26 — Triagem dos minors de review adiados (programa de outubro)

**Tipo**: achados de review adiados. **Não** é exclusão de sensor: nenhum sensor é excluído por este
arquivo, e ele não serve como `--known-issue` de `sensors.sh`.
**Origem**: `docs/harness/reviews/2026-10-05-comprehensive-improvement/ledger.md` (linhas citadas abaixo).
**Estado de referência**: `08cb17a4`. Status só vale para esta árvore.
**Registrado em**: [`progress/2026-10-26-remediation.md`](../progress/2026-10-26-remediation.md) e
`risk-register.yaml` (RISK-0003, RISK-0004, RISK-0005).

Legenda de status: **em andamento** = a correção está numa branch local paralela e **não** está nesta
árvore; **aberto** = nenhuma correção conhecida no repositório; **parcial** = parte já tratada.
Nada abaixo é "corrigido". Esta triagem não altera código nem ranqueia por severidade de revisão:
a coluna "Risco" é estimativa da frente WT5 a partir do texto do ledger e deve ser confirmada pelo dono.

| Ledger | Task | Achado | Status | Risco | Dono / próximo passo |
|---|---|---|---|---|---|
| 66 | C2 | `WalHeader::new` emite cabeçalho que `verify` rejeita (nomenclatura de endianness do checksum). Parcialmente tratado: `src/sync/wal_chain.rs` ganhou `checksum_words_le(magic)` e `stored_checksum()` e a lição está em `ERRORS_AND_LESSONS.md` (2026-10-05), que mantém o campo legado `is_little_endian_checksum` "por compatibilidade". `WalHeader::new` (só usado em `tests/wal_*`) ainda emite com o campo legado; não foi reproduzido por esta frente. | parcial | baixo (helper usado em testes) | lane R: teste de ida e volta `serialize → parse → verify_checksum` com BE e LE; renomear só com decisão de compatibilidade |
| 97 | C1 | `resources/subscribe` não considera o principal (oráculo de atividade entre workspaces) | **em andamento** (`fix/workspace-batch-auth`, WT1) | médio | WT1; não marcar como corrigido antes do merge e do teste de regressão |
| 98 | C1 | Sem teste e2e gRPC com o `EngramHandler` real. `tests/grpc_transport.rs` existe, mas roda sob exclusão documentada de sandbox ([`2026-05-31-grpc-transport-port-bind.md`](./2026-05-31-grpc-transport-port-bind.md)); se ele cobre o handler real não foi verificado | aberto | médio (lacuna de teste do caminho de autorização) | sem dono: abrir task de teste e2e; não depende de WT1 |
| 107 | C1 | (a) `permission_denial_for_mode` público ainda fail-open para tool desconhecida; (b) `permission_mode_status_report` ignora escalada por flag de escrita | (a) **em andamento** (WT1); (b) aberto | médio | (a) WT1; (b) sem dono, fora da lista conhecida de WT1 |
| 235 | Q2F | `invalidate_for_workspace` do cache de resultados ignora `workspaces`/global, e a chave do cache não inclui o principal (resultado com escopo de principal pode ser servido a outro) | **em andamento** (WT1, "result-cache-principal") | médio-alto (vazamento entre principais) | WT1; `e04d3307` alterou `src/search/result_cache.rs`, então a correção precisa partir da árvore pós-`e04d3307` |
| 245 | C6 | (i) afirmação não qualificada de isolamento de panic no log da lane R — qualificada no FINALFIX (README de reviews); (ii) `reset_interrupted_sync` concatena `last_error` sem limite em loop de crash (`src/sync/worker.rs`, ainda presente); (iii) guard de Pull do worker só canonicaliza (`download_checked` existe com testes em `cloud_tests_failure.rs`; uso no worker não verificado); (iv) risco de flake com zombie em `pid_alive` (símbolo não encontrado na árvore: renomeado/removido ou não verificado); (v) retry de `EINTR` no `waitid` (`src/multimodal/process.rs`, não verificado) | parcial (i); (ii) aberto; (iii)-(v) não reverificados | baixo | lane R; abrir uma task única para (ii)-(v) com teste primeiro |
| 254 | Q2F | Erros legados `{"error":string}` em caminhos fora de faixa de `memory_create`/`memory_boost`/`context_record_artifact` (ainda presentes em `src/mcp/handlers/memory_crud/`); `ignored_date_filter` sem consumidor; `write_unique_backup` casa por prefixo | aberto | baixo (contrato de wire) | normalizar exige coordenação: manter a forma atual até haver flag + anúncio (`development-workflow`: nunca mudar o wire format dentro de um refactor) |
| 308 | H5 | `base_references()` falha aberto (conjunto vazio) se o `git` falhar (`docs/harness/bin/merge_gate_ci.py`, ainda presente); `required_jobs` depende de indentação; linhas Markdown > 100; conjunto protegido amplo (qualquer PR em `scripts/**` ou `.github/**` vai para humano) | aberto | baixo-médio: o avaliador só aponta elegibilidade, o merge continua humano, mas perde proteção dinâmica se o `git` falhar | harness; é script process-critical (`bin/*`): corrigir exige review independente; sem dono ativo |

## Regras desta triagem

- Itens "em andamento" só viram "corrigido" com SHA de merge e teste de regressão citados; quem
  atualizar este arquivo cita o SHA. O integrador faz isso após o merge de WT1.
- Os demais minors do ledger (E0, Q1, H1-H6, O1-O4, Q2-Q7, C3-C7, G1, P1) **não** foram triados aqui;
  continuam apenas no ledger. Ausência neste arquivo não significa resolvido.
- Itens de wire format (254) mantêm a forma atual até haver flag e anúncio; itens de autorização
  (97, 107, 235) só mudam de status com a correção mesclada e testada, nunca por texto.
