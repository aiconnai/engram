# Lane R — execução do plano de melhorias (2026-10-05)

Worktree `engram-improvement-plan-edb43d`, branch
`claude/engram-improvement-plan-edb43d`. Somente commits locais; sem push nem PR.

## C2 — Atomicidade, WAL, recovery e migrações

Commits (em ordem de sub-PR):

1. `1398258` fix(sync): bound WAL replay by byte budget before touching target
   - `ReplayLimits` (padrão e teto aprovado: 64 GiB, injetável); `preflight_replay`
     puro valida page_size, tamanho do frame, `page_number`/`db_size_pages`
     (0/MAX/MAX+1/u32::MAX) e orçamento em bytes com aritmética checked antes de
     criar/abrir o destino ou o diretório pai.
2. `ba07ff3` feat(sync): verify SQLite WAL checksum chain before replay
   - `WalChainContext` nos packs; cadeia de checksum SQLite, salts, ordem,
     contiguidade e page_size consistente validados antes do replay. Leitor de WAL
     verifica o checksum do header e para no primeiro frame inválido (semântica
     SQLite). Integridade, não autenticidade.
3. `d1a9840` fix(sync): stage WAL recovery and replace target atomically
   - Replay em arquivo de staging no diretório do destino, fsync, integrity_check e
     rename; falha tardia (I/O ou integridade) preserva destino, base e origem.
     O caminho PITR sem WAL agora roda integrity_check de verdade. Destino com
     `-wal`/`-journal` não vazio é recusado.
4. `5c113f0` fix(storage): serialize migrations and wait for concurrent writers
   - Cada migração em `BEGIN IMMEDIATE` com releitura da versão; v45 com FK
     desligada fora da transação; `with_transaction` IMMEDIATE; retry limitado da
     troca de journal_mode em SQLITE_BUSY. Guardas: provider lento não segura a
     conexão; second open sem efeito; IDs não reutilizados.
5. test(snapshot) + lições + este registro (commit final da tarefa).

Testes executados (features obrigatórias de `scripts/ci-required-features.env`,
`--no-default-features --features "$CI_REQUIRED_FEATURES"`):

- `cargo test ... --tests` (todos os alvos): exit 0, 1979 passed, 0 failed,
  1 ignored (antes do retry de journal_mode e dos testes de snapshot).
- Depois disso: `--lib` 1560 passed; `wal_replay_hardening_tests` 30,
  `wal_recovery_staging_tests` 7, `storage_concurrency_regression_tests` 7
  (20 execuções seguidas verdes), `wal_replication_tests` 10,
  `snapshot_attestation` 11.
- Pre-commit (`cargo fmt --check` + `cargo clippy --all-targets --all-features
  -D warnings`) passou em todos os commits.

Somente macOS (darwin/arm64). Linux não foi executado localmente (fica com o CI).

Relatório completo: `.superpowers/sdd/2026-10-02-engram-comprehensive-improvement-plan/task-C2-report.md`.

### C2 — rodada de correções 1 (review)

- `c74f1c0` fix(sync): owner-only staging and running decode budget
  - staging criado com modo 0600 (Unix): o destino recuperado não amplia
    permissões;
  - soma corrente checked dos bytes decodificados dos packs, com recusa assim
    que o total passa do budget;
  - a checagem de `-wal`/`-journal` falha fechada em qualquer erro de stat
    diferente de NotFound.
- `a9c3bd8` fix(storage): roll back failed migration commits, check v45 FKs
  - ROLLBACK quando o COMMIT falha;
  - `foreign_key_check` antes/depois nas migrações com FK desligada;
  - teste do v45 com `foreign_keys=ON` (caminho de produção). Falha se a
    salvaguarda for removida.
- `7a7809f` docs(sync): INVARIANTS #25 e CHANGELOG (campos públicos novos;
  packs antigos recusados).
- Suíte completa `--tests` com as features obrigatórias: exit 0,
  1989 passed, 0 failed, 1 ignored, 52 binários.
- `cargo clippy --all-targets --all-features -D warnings`: exit 0.
- Follow-ups registrados (não corrigidos):
  - bug de nomenclatura da endianness do checksum;
  - reset do `last_replicated_frame` no streamer quando `nCkpt == 0`.

## Q2 — Inventário confiável de qualidade Rust (2026-10-05)

- Commit: `docs(harness): add Rust risk inventory and classifier` (HEAD base `46fc3e2`).
- Entregas: `docs/quality/rust-risk-inventory.md` (método, limites, contagens,
  auditoria de `unsafe`/SAFETY, prints CLI/protocolo vs biblioteca, backlog
  priorizado), `scripts/rust_risk_inventory.py` (classificador, stdlib) e
  `scripts/test_rust_risk_inventory.py` (30 testes de fixtures).
- Resultado: `.unwrap()` textual 1601 → 32 em produção (nenhum candidato a
  panic por input); `unsafe` textual 28 → 3 blocos reais (1 só em bench, sem
  rótulo `SAFETY:`). O risco real está em fatiamento por byte de texto/hex do
  usuário e em erros virando sucesso. Reproduzidos 9 achados (5 abortam o
  `engram-server`: `search.rs:732`, `context_grouper.rs:118`,
  `gardening.rs:316`, `compression.rs:269`, `attestation.rs:235`; CLI
  `mcp install` sobrescreve config não-JSON-estrito e o `.bak`;
  `delete_crossref` e `list_memories` engolem erro). O restante é hipótese.
- Nenhum código de `src/`, `benches/` ou `tests/` foi alterado; itens de
  `src/sync`, `connection.rs`, `migrations` ficam só no backlog (C2 em revisão).
- Testes: `python3 -m unittest scripts/test_rust_risk_inventory.py` (30 OK);
  7 mutações do classificador derrubam de 1 a 8 testes cada. Não executado:
  binário release (abort), transporte HTTP, Linux, caminho do handler de
  snapshot.
- Relatório: `.superpowers/sdd/2026-10-02-engram-comprehensive-improvement-plan/task-Q2-report.md`.

## C1 — Autorização por workspace no caminho real (2026-10-05)

- Commit: `fix(mcp): authorize memory IDs by persisted workspace` (base `a87a3ce`).
- Achados reproduzidos (RED): o principal autenticado pelo HTTP/gRPC nunca chegava
  ao `dispatch` (`principal: None` em `server.rs`); com `{id: B, workspace: A}`
  `memory_get`/`memory_get_public` devolviam conteúdo e reforçavam `stability`,
  `memory_update` sobrescrevia, `cascade_chain` apagava membro estrangeiro,
  `memory_export_graph` exportava todos os workspaces e `resources/read
  engram://memory/{id}` vazava para anonymous loopback.
- Correção: `McpHandler::handle_request_as` leva o principal ao contexto;
  `src/mcp/workspace_guard.rs` autoriza IDs pelo workspace persistido (estrangeiro
  e inexistente → mesmo `not_found`), reverificado na mesma transação dos
  mutadores; claim só escopa tools que declaram `workspace` ou estão em
  `ID_SCOPED_TOOLS` (demais → negação conservadora); `read_resource_as`.
- Matriz: `docs/security/workspace-operation-matrix.md`.
- Testes: `workspace_auth_enforcement_tests` 11/11, `http_transport_security`
  16/16, `permission_modes_tests` 7/7, `mcp_protocol_tests` 56/56,
  `grpc_transport` 25/25; suíte completa `--tests` com features obrigatórias:
  exit 0, 2003 passed, 0 failed, 1 ignored, 52 binários; clippy
  `--all-targets --all-features -D warnings` limpo.

### C1 — rodada 1 de correções da revisão (2026-10-05)

- Commit: `fix(mcp): close context alias and permission-mode bypasses`.
- `context_*` (5 tools) usam `workspace` como alias de `workspace_path_hash`:
  negados para principal restrito (`WORKSPACE_ALIAS_TOOLS`); RED mostrava o
  lookup do artefato ("Context artifact not found") = oráculo de existência.
- Aliases do dispatcher (`graph_predict_links`, `graph_cluster_concepts`)
  canonicalizados; `required_mode_for_call` eleva para `scoped_write` com
  `auto_apply`/`update_version` e exige `admin` para nome desconhecido. RED:
  `read_only` + `graph_predict_links {auto_apply:true}` gravava crossref.
- `discover_tools`/`permission_mode_status` liberados sem claim (decisão do
  controller); controles positivos de ID próprio; matriz atualizada.
- Testes: suíte completa `--tests` com features obrigatórias: exit 0, 2008
  passed, 0 failed, 1 ignored, 52 binários; clippy `-D warnings` limpo;
  `scripts/test-canonical-journey.sh` exit 0 (8 passed);
  `verify-sdk-artifacts.sh` NÃO EXECUTADO (build/instalação exigem registry
  PyPI/npm sem cache local; só `--self-test-version-mismatch` PASS).

### C3 — Unicode e parsers adversos (2026-10-05)

- Commit: `fix(search): make text truncation and hex parsing char-boundary safe`.
- Causa: fatiamento por byte (`&s[..N]`) de texto de usuario e offsets de texto
  lowercased aplicados ao original; com `panic = "abort"` uma memoria armazenada
  derrubava o servidor. Helper unico novo `src/text_util.rs`
  (`floor/ceil_char_boundary`, `truncate_bytes`, `suffix_bytes`); `safe_truncate`
  de `context/mod.rs` agora delega a ele.
- Corrigidos: Q2-B01 (`memory_search_compact`), B02 (`memory_prepare_context`),
  B03 (`memory_garden`, inclusive dry-run), B04 (`context_budget_check`), B05/B06
  (hex via `hex::decode`: impar, nao-ASCII e sinais `+` viram erro tipado),
  B08 (`truncate_with_marker` sem underflow, contrato `<= max_chars` bytes;
  `ttl_days` limitado a 0..=36500 com erro tipado), B14, B17, B18, e
  `bm25::generate_highlights` (mapa lowercase->original por char; snippet cortado
  no texto original). Mesma classe achada fora do inventario:
  `smart_retrieve::strip_intent_markers` e `dream::worker::extract_procedural_lesson`
  (offset de `to_lowercase()` no original; agora `to_ascii_lowercase()`).
- Testes: `tests/unicode_adverse_parsers_tests.rs` (12, via `dispatch` real),
  property `unicode_adverse_tests` em `tests/property_tests.rs` (seed fixa,
  regressoes em `tests/property_tests.proptest-regressions`), casos Unicode em
  `aaak_compression_tests`, unitarios em bm25/memory_blocks/tokens/auto_capture/
  smart_retrieve/dream worker. Suite completa `--tests` com features obrigatorias:
  exit 0, 2056 passed, 0 failed, 1 ignored, 53 binarios; clippy `-D warnings` limpo.

### Q4 — mapa de cobertura por risco + contratos MCP/SDK (2026-10-05)

- Commit: `test(mcp): add risk-based coverage map and MCP/SDK contract tests`.
- `docs/quality/test-coverage-map.md`: invariante/contrato -> teste -> features -> lane
  (R/A/O/M/L) -> tipo (unit/mock, in-process, protocol, real, static, live). Sem
  percentual de arquivos com `#[test]`; cobertura executada so existe no job
  `coverage` (lane O), nao executado aqui.
- Rust: `tests/mcp_protocol_tests/contract_matrix.rs` (14 testes: tool desconhecida com e
  sem principal, envelopes `isError`, params invalidos, snake_case/camelCase, paginacao,
  ciclo de vida de job dream) e `tests/canonical_journey/contract.rs` (3 testes reais:
  rejeicoes em stdio+HTTP sobre o mesmo banco sem mutacao, modo de permissao por env,
  job dream persistido entre processos). Prova de nao-mutacao le o SQLite so depois que
  o servidor sai e tem controle positivo.
- SDKs: `scripts/check-sdk-contract-alignment.py` (+ baseline
  `docs/quality/sdk-contract-drift-baseline.json`, catraca), `sdks/python/tests/test_contract.py`
  (24), `sdks/typescript/src/contract.test.ts` (14). 161 divergencias SDK x registro
  registradas, nao corrigidas (decisao de contrato publico pendente).
- ACHADO CRITICO G-1: `restrict_sqlite_artifact_permissions` abre/fecha db/-wal/-shm apos
  cada chamada e solta os locks POSIX do SQLite; um segundo processo que abre e fecha o
  banco apaga o WAL vivo e as escritas seguintes do servidor se perdem. Tambem enfraquece
  os checks de linha crua em `tests/http_transport_security/workspace_auth.rs`.
- Testes: suite completa `--tests` com features obrigatorias: exit 0, 2059 passed,
  0 failed, 1 ignored, 53 binarios; clippy `-D warnings` (features obrigatorias) limpo;
  pytest sdks/python 213 passed/3 skipped (live); vitest 98 passed; type-check ok;
  `generate-mcp-reference.sh --check` ok. Scripts live NAO executados (exigem registry);
  equivalentes manuais offline (wheel/tarball instalados) passaram.

### Q4 — rodada 1 de correcoes da revisao (2026-10-05)

- Commit: `fix(qa): make SDK drift baseline shrink-only and per call site`.
- `--update-baseline` agora so encolhe: recusa adicionar entradas sem `--allow-new-drift --reason`
  (razao gravada no `changelog` do baseline); entradas por call site (metodo Python /
  arquivo:metodo TS), entao novo metodo chamando tool ja baselinada e drift novo. Baseline,
  checker e teste entram no CODEOWNERS. Baseline regenerado via o guard (161 -> 161,
  mesma divergencia por tool/chave, so re-chaveada por call site).
- `scripts/test-canonical-journey.sh`: piso de 11 testes + 4 jornadas obrigatorias por nome;
  jornadas limpam `ENGRAM_PERMISSION_MODE`. Teste `#[ignore]` G-3 documenta o bug do cache.
- Testes: checker 28 OK; pytest sdks/python 213 passed/3 skipped; vitest 98 passed;
  mcp_protocol_tests 70 passed/1 ignored; canonical_journey 11 passed; clippy limpo.

### G1 — permissoes SQLite sem soltar locks POSIX (2026-10-05)

- Commit: `fix(storage): restrict SQLite file modes without dropping locks`.
- Causa: `restrict_sqlite_artifact_permissions` (apos toda chamada) e `prepare_database_file`
  (banco ja existente) abriam e fechavam descritores do db/-wal/-shm; fechar qualquer descritor
  solta os locks fcntl do processo, outro processo apagava o WAL vivo e commits posteriores se
  perdiam. Agora: lstat + `fchmodat(AT_SYMLINK_NOFOLLOW)` com fallback por dev+inode, symlink
  recusado, modo so estreitado, banco novo criado 0600 com O_EXCL. TOCTOU residual documentado.
- Teste novo `tests/storage_posix_lock_regression_tests.rs` (segundo processo via re-exec do
  binario de teste): RED antes (fresh process via 1 linha em vez de 2) em macOS e Linux
  (container local, glibc 2.36), GREEN depois. Controle positivo no C1
  (`workspace_auth.rs`): falhava antes do fix (checks "inalterado" eram vacuos), passa agora.
- Auditoria: riscos abertos documentados no codigo (seed de `replication_recover` a partir do
  banco ativo, `CloudStorage::upload/download`, ATTACH DuckDB); INVARIANTS #27 (itens seguintes
  renumerados) e ERRORS_AND_LESSONS.
- Testes: suite completa `--tests` com features obrigatorias: exit 0, 2065 passed, 0 failed,
  2 ignored, 54 binarios; clippy `-D warnings` (features obrigatorias) limpo; fmt ok.

### Q3 — Nightly efetivo: fuzz, mutation e Miri (2026-10-05)

- Commit: `ci(infra): make nightly fuzz, mutation and Miri lanes effective`.
- Antes: fuzz com `|| true` + `continue-on-error`, `cargo fuzz list 2>/dev/null || echo ""`
  e sem crate `fuzz/` (lane vazia = verde sintetico); mutants com `if` em
  `github.event.schedule == '0 3 * * 0'` inalcancavel (so existia cron diario);
  Miri com filtros `workspace:: tier::` + `|| true`.
- Agora: `fuzz/` (workspace proprio, lock proprio; raiz inalterada) com 3 targets
  (`entity_extraction`, `workspace_normalization` — nao ha `normalize_workspace`
  publico, o alvo cobre `MemoryScope`/`allows_workspace`/extracao de escopos —,
  `text_util_boundaries`) e seeds das entradas C3. `scripts/run-fuzz-smoke.sh`:
  inventario declarado == `cargo fuzz list` == compilados == executados, 60 s por
  alvo com `-rss_limit_mb`/teto de walltime, status pass/fail/not-run/unsupported,
  contagens por alvo e reproducers preservados. `scripts/run-miri-smoke.sh` com
  inventario nominal SQLite-free e contagem > 0. `nightly.yml`: job `plan` mapeia
  `github.event.schedule` (daily Seg-Sab, weekly Dom, manual), mutants gated por
  ele, ferramentas pinadas (`cargo-fuzz 0.13.2`, `cargo-mutants 27.1.0`, `--locked`),
  nenhum `continue-on-error`/`|| true` nas lanes Q3. Gates obrigatorios (`ci.yml`) intactos.
- Testes: `python3 -m unittest scripts/test_optional_lane_contracts.py` (30 OK);
  smoke real local 3 alvos x 60 s: pass (exit 0); Miri local 13 testes: pass.
  cargo-mutants NAO EXECUTADO localmente (nao instalado; so no CI pinado).

### G1 — rodada 1 de correcoes da revisao (2026-10-05)

- Commit: `fix(storage): keep MCP file paths off the live SQLite files`.
- `replication_recover` com a origem padrao (banco ativo) agora semeia via `VACUUM INTO`
  (`WalRecoveryEngine::recover_active_database`, so estado mais recente); PITR por
  frame/tempo do banco ativo e alvo = banco ativo sao recusados. Ferramentas DuckDB anexam
  uma copia `VACUUM INTO` privada (`Storage::snapshot_copy`), nao o arquivo vivo.
- Guarda `Storage::refuse_active_sqlite_artifact` (dev+inode e nome resolvido de
  db/-wal/-shm/-journal) em ingestao de documento, multimodal (5 ferramentas), snapshot
  create/load/inspect, upload de imagem, saida do palace e `replication_recover`.
- `child_storage_helper` agora `#[ignore]` (executado com `--ignored --exact`); lock de
  processo em prepare+open do banco; aviso quando um symlink aparece apos o chmod.
- RED: 4 testes MCP novos falharam antes em Linux (fresh process 2 em vez de 3); o teste
  DuckDB falhou antes em macOS (mesma perda). GREEN em todos.
- Testes (arvore isolada HEAD + estas mudancas, porque C7 estava com o worktree em
  andamento): suite `--tests` com features obrigatorias exit 0, 2072 passed, 0 failed,
  3 ignored, 54 binarios; clippy `-D warnings` obrigatorias, obrigatorias+duckdb-graph+
  snapshot+attestation e `--all-features` limpos; duckdb_graph_tests 6 passed.

### C7 — Embeddings, fila e health coerentes end-to-end (2026-10-05)

- Commit: `fix(embedding): make defer_embedding enqueue and persist atomically`.
- Decisao do dono: `defer_embedding=true` = enfileirar para background (promessa do catalogo
  MCP); `false` = embedding imediato apos commit. Antes: `true` nao enfileirava nem embeddava
  (memoria sem embedding para sempre) e `false` deixava o job `pending` apos embeddar.
- Agora: `memory_create`/`memory_create_batch` enfileiram (idempotente) na mesma transacao do
  insert quando `true`; no caminho imediato o provider roda fora de transacao e
  `persist_computed_embedding` grava row + `has_embedding` + completion em UMA transacao
  (guarda: conteudo inalterado, job ainda `pending/processing`); HNSW so apos commit (Q2-B11).
  Drain/worker usam o mesmo helper (uma transacao por memoria; lote curto do provider ->
  `failed`, nao `processing` eterno). `run_embedding_drain_cycle` recupera leases `processing`
  expirados (15 min, retry budget 3) na partida do server e a cada ciclo; `failed` segue
  so por manutencao explicita (runbook em `docs/OPERATIONS.md`). Health ganhou
  `missing_embedding_unqueued` e `complete_job_without_embedding_row` (degraded): backlog
  vazio nao significa saudavel.
- Sem mudanca de schema/`SCHEMA_VERSION`; formato de resposta MCP preservado.
- Testes: `test_deferred_create_enqueues_and_drains` (MCP real), falhas injetadas por trigger
  entre row/flag/completion, crash/reopen em disco (diretorio temporario do chamador) com
  health apos reopen. Suite `--tests` com features obrigatorias: exit 0, 2090 passed,
  0 failed, 3 ignored, 54 binarios; clippy `-D warnings` e fmt limpos.

### Q3 — rodada 1 de correcoes da revisao (2026-10-05)

- Commit: `ci(infra): scope mutants lane and harden fuzz/Miri nightly caps`.
- mutants: escopo por `.cargo/mutants.toml` (`examine_globs`: text_util, scoping,
  entity_extraction, transport_principal; dono Ronaldo, como ampliar documentado),
  timeout 240 min, continua fail-closed. Contrato testado (escopo != whole-lib).
- fuzz/Miri: grace 120 s, `elapsed_secs` por alvo, falha se elapsed > budget+grace mesmo
  com exit 0; nightly datado `nightly-2026-04-21` via `NIGHTLY_TOOLCHAIN`; `cargo audit
  --file fuzz/Cargo.lock` no job fuzz; job `plan` roda os testes de contrato.
- Testes: `unittest scripts/test_optional_lane_contracts.py` 37 OK; smoke fuzz real
  3x20 s pass; Miri 13 testes pass. cargo-mutants NAO EXECUTADO localmente.

### C7 — correcoes de review, rodada 1 (2026-10-05)

- Commit: `fix(embedding): enqueue jobs for CLI, snapshot and dream creates`.
- CLI `create`, `interactive create`, `SnapshotLoader::load` e promocao dream passavam
  `defer_embedding: true` ao `create_memory` de storage (que so enfileira para `false`):
  memorias nunca embeddadas e health Degraded para sempre. Agora cada site chama
  `enqueue_embedding_job` na propria transacao. RED por site (4 testes), GREEN depois.
- Menores: health `missing_embedding_unqueued` exige `has_embedding = 0` (sem dupla contagem
  com `flagged_without_embedding_row`; `rebuild --embeddings` repara ambos); warn quando
  persist imediato retorna `Ok(false)`; docs de `jobs.rs`/`drain.rs`/runbook corrigidos
  (comando, `update_memory` -> pending, `retry_count` incrementado 2x); testes de health
  movidos para `health/tests.rs` (arquivo 854 -> 420 linhas).
- Testes: suite `--tests --no-fail-fast` com features obrigatorias: 2101 passed, 1 falha
  transitoria em `storage_posix_lock_regression_tests` (G1 concorrente; passa isolado, 7/7);
  clippy `-D warnings` e fmt limpos.

### G1 — rodada 2 de correcoes da revisao (2026-10-05)

- Commit: `fix(storage): refuse descriptor aliases of the live SQLite files`.
- Guarda recusa caminhos sob `/dev` e `/proc` (`/dev/fd/N` no macOS escapava do dev+inode
  e soltava os locks) e compara so inode quando o dispositivo e devfs/fdesc/procfs; nova
  `refuse_lock_bearing_sqlite_file` (db e -shm) nas leituras de `{db_path}-wal` de
  `replication_status`/`replication_sync_now`; import de markdown recusa `*.md` que
  apontam para o banco ativo; origem explicita de `replication_recover` tambem guardada.
- DuckDB: ordem de drop corrigida (grafo antes da copia). Contrato de `replication_recover`
  documentado no catalogo e em `docs/MCP_TOOLS.md` (regenerado, check ok).
- RED: teste MCP `/dev/fd/N` falhou antes no macOS (2 em vez de 3). GREEN macOS e Linux
  (container local, inclui `/proc/self/fd/N`).
- Testes (arvore isolada HEAD + estas mudancas): suite `--tests` com features
  obrigatorias exit 0, 2093 passed, 0 failed, 3 ignored, 54 binarios; clippy `-D warnings`
  obrigatorias e `--all-features` limpos; duckdb_graph_tests 6 passed.

### C7 — correcoes de review, rodada 2 (2026-10-05)

- Commit: `fix(storage): make embedding rebuild repair flag-without-row cases`.
- O runbook prometia que `maintenance rebuild --embeddings --apply` reparava os tres
  diagnosticos degraded; o codigo so reenfileirava `has_embedding = 0`. Agora
  `rebuild_derived_indexes` tambem trata memorias vivas com flag=1 sem row em `embeddings`
  (reseta o flag e reenfileira `pending`, inclusive job `complete` sem row), na transacao do
  chamador. Teste end-to-end dos tres casos -> rebuild -> drain -> Healthy; runbook ajustado ao codigo.
- Testes: suite `--tests --no-fail-fast` com features obrigatorias: 2103 passed, 0 failed,
  3 ignored, 54 binarios; clippy `-D warnings` e fmt limpos.

### C6a — hooks sob falha/replay (2026-10-05)

- Commit: `fix(hooks): bound and de-duplicate session injections and reinforcement`.
- `pending_injections`: `drain_for_workspace` era SELECT + DELETE em duas instrucoes: duas
  conexoes recebiam as mesmas linhas (RED: 1600 entregas para 400 linhas). Agora um unico
  `DELETE ... RETURNING` com limite `MAX_DRAIN_ROWS`; TTL grande demais derrubava o processo
  (`TimeDelta::days` panic) e agora e limitado a `MAX_TTL_DAYS`; payload > 64 KiB rejeitado.
- `SessionEnd`: replay da mesma sessao nao duplica enquanto a linha esta na fila
  (`enqueue_once_for_session`); `notes` > 8 KiB truncadas em fronteira de char com
  `notes_truncated`. `SessionStart` informa `remaining` (entrega parcial visivel).
- `HookManager::trigger` isola panic de handler so com `panic = "unwind"` (no perfil release
  `panic = "abort"` o processo encerra; ver correcao C6 abaixo); `PostToolUse` limita 50 ids por evento e
  ignora memorias de outro workspace quando o contexto traz workspace.
- Semantica documentada: fila at-most-once (sem exactly-once); replay so deduplica enquanto
  a linha esta enfileirada.
- Testes: `--lib pending_injections` 14 passed, `--lib hooks` 24 passed (features obrigatorias).

### C6b — multimodal sob falha/cancelamento (2026-10-05)

- Commit: `fix(multimodal): bound ffmpeg/vision work and own keyframe dirs`.
- Novo `multimodal/process.rs::run_bounded`: timeout, kill do process group + wait em todo
  caminho (inclusive drop), saida capturada com teto. `ffprobe`/`ffmpeg`/`-version` passam por
  ele (antes `.output()` sem prazo: filho travado bloqueava o handler para sempre).
- `engram_frames_*` agora tem dono (`FramesDir`, RAII, 0700): erro de spawn/ffmpeg/vision,
  timeout, cancelamento do future e fim do ultimo consumidor removem o diretorio (antes vazava
  em spawn error, falha de visao e em TODO sucesso de `memory_process_video`). ffmpeg com 0
  frames agora e erro; visao tem timeout por frame com indice no erro; hash de arquivo em
  streaming; clientes HTTP de visao/audio com timeout (reqwest::Client::new nao tem).
- `memory_ingest_media`: retry do mesmo arquivo/workspace devolve a memoria existente
  (`deduplicated: true`), nao revive deletada, nao cruza workspace; `asset_id` real.
- pdf_worker: 5 testes de caracterizacao (sinal, lixo, saida antecipada, spawn, pid sumiu);
  implementacao ja estava correta.
- Semantica: deduplicacao por hash de conteudo, nao exactly-once; falha de visao no meio do
  video devolve erro (sem resultado parcial silencioso), frames removidos.
- Testes: `--lib multimodal handlers::multimodal pdf_worker` 103 passed (3 rodadas).

### C6c — sync sob interrupcao/restart/retry (2026-10-05)

- Commit: `fix(sync): keep WAL streaming and cloud pulls safe across restarts`.
- Streamer WAL (follow-up C2 pendente, agora corrigido): com `checkpoint_seq == 0` na primeira
  geracao, o primeiro checkpoint+reset nunca era detectado e as frames da nova geracao eram
  puladas (RED: pack ausente). Agora `baseline_seen` marca a linha de base. WAL truncado
  (menor que o cabecalho) devolve `Ok(None)` em vez de `UnexpectedEof` + `last_error`.
- `CloudStorage::download` grava em temp irmao + fsync + rename (antes `fs::write` in place:
  leitor com o arquivo aberto via o conteudo mudar; falha deixava banco truncado). Sem temp
  restante em falha; arquivo novo 0600, existente preserva permissoes.
- `SyncWorker`: `start_with_cloud`; restart limpa `is_syncing` preso e registra
  `last_error` (RED: flag sobrevivia); push debounced falho antes limpava o dirty e nunca
  tentava de novo -> `DirtyTracker` com backoff exponencial (cap 300 s) e no maximo 5
  tentativas seguidas; `Sync` explicito nunca e repetido sozinho.
- Contrato documentado: `flush_delta` e at-least-once com dedupe no consumidor (restart sem
  estado reemite do frame 1; `reset_offset` retoma); upload idempotente; wrong-key ja
  coberto (cloud_tests_security) e continua nao-destrutivo.
- Testes: `--lib sync::` (+cloud) 71 passed; `tests/wal_streamer_restart_tests` 4 passed.

## C5 — Refatoração seletiva da persistência no handler de create

- Commit: `refactor(mcp): commit memory before updating fuzzy vocabulary in create`
- Achado: o SQL de embedding já havia sido extraído por C7 (`persist_computed_embedding`); o handler não tem SQL cru. Restava o
  vocabulário fuzzy ser atualizado dentro da transação (vazava estado em memória num commit falho e segurava o mutex na tx).
- Mudança: `create.rs` — `insert_memory_transaction`, `mirror_existing_embedding`, `merge_into_existing`; vocabulário após o commit.
- Testes: 6 `c5_*` em `tests/mcp_protocol_tests.rs` (caminho MCP real); RED = vazamento de vocabulário; suíte completa
  required-features 2151 passed/0 failed; clippy -D warnings limpo. Bench mcp_dispatch memory_create ruidoso (carga ~7), sem regressão distinguível.

### Q7 — retrieval e performance do candidato (2026-10-05)

- Commits: `feat(search): add candidate retrieval runner, corpus and mode benches` (12d2c3b) e
  `docs(search): record Q7 floors justification and progress`.
- `scripts/run-quality-candidate.py` (+ pacote `scripts/quality_candidate/`): argv fixo
  (`cargo test --locked --test retrieval_quality ... candidate_retrieval_metrics`), HEAD/tree ==
  `--candidate-sha` com checkout limpo, corpus SHA256 ligado a `docs/quality/candidate-floors.json`
  (digest por entrada + ancora opcional shrink-only + reconciliacao com budgets.json), metricas
  ausentes/NaN/fora de [0,1] e abaixo do floor rejeitadas, Criterion so aceito se capturado por
  `capture-criterion-candidate.py` (marcador com SHA/tree/features/toolchain/supervisor/hash do corpo,
  max 24 h). Historico (`benches/results/*`) e so contexto no report. Ceiling 1.15 e floors de
  `budgets.json` intactos.
- Corpus v2 sintetico (50 memorias / 49 consultas, PT/EN, typos, negacao, entidades ambiguas,
  duplicatas, daily/transcript, isolamento). Rotulos escritos por LLM, NAO revisados por humano
  (registrado no proprio JSON e no doc). Floors v2 = medicao deterministica arredondada para baixo,
  status `proposed-pending-independent-review`; `anchor_revision` fica nulo ate a revisao.
- Benches novos: `storage_modes/*` (memoria vs disco WAL vs disco cloud-safe),
  `storage_concurrency/*`, `latency_percentiles` (opt-in) e `mcp_dispatch_memory_search_uncached`.
- Resultado real: candidato 12d2c3b reprovado no ceiling: `entity_extraction/extract_mixed`
  ~213-300 us contra baseline 17.144 us (razao 12.4); causa provavel `find_case_insensitive_match`
  (bf45c0b). Nao foi mascarado nem rebaselinado; decisao fica com o controlador/dono.
- Testes: `unittest scripts/test_run_quality_candidate.py` 49 OK (15 mutacoes do runner mortas);
  `cargo test --test retrieval_quality` 8 passed (features obrigatorias); clippy `-D warnings` limpo.

## Q2F / Q2-B07 — `mcp install` nao destroi config nem sobrescreve backup

- `engram-cli mcp install` agora recusa (erro claro + exit != 0) config existente que nao e JSON
  estrito/objeto (ou com `mcpServers` nao-objeto), a menos que `--force` (que antes era ignorado);
  backup nunca sobrescrito (`.bak`, depois `.bak.<ts>[.<n>]`, `create_new`), falha de backup aborta;
  falha por caminho agora produz exit != 0 em vez de "✅".
- Testes: `tests/cli_mcp_install_tests.rs` (3, binario real, HOME/cwd sandbox; 2 RED antes do fix) e
  unit `mcp::tests` (3) com features obrigatorias.

## Q2F / Q2-B09 — `StorageBackend::delete_crossref` propaga erros

- Apenas `NotFound` (aresta inexistente para o tipo) e ignorado; qualquer outro erro derruba a
  transacao e retorna `Err`. Idempotencia para aresta ausente preservada.
- Testes: `sqlite_backend::tests::delete_crossref_*` (2; o de erro RED antes do fix: `Ok(())` com
  `crossrefs` removida), features obrigatorias.

## Q2F / G-3 — chave do cache de `memory_search` inclui `limit` e demais opcoes

- `CacheFilterParams` ganhou `limit`, `min_score_bits`, `strategy`, `scope`, `workspaces`,
  `scope_path`, `filter` (todos `#[serde(default)]`); `memory_search` preenche a partir de
  `SearchOptions`. Antes: limit 3,1,2 devolvia 3 linhas nas tres; `workspaces`/`min_score`
  distintos tambem recebiam a resposta cacheada da primeira chamada.
- Testes: `contract_matrix::memory_search_limit_survives_the_result_cache` (un-ignored; RED
  `left: 3 right: 1` com o campo neutralizado), `memory_search_cache_distinguishes_workspaces_filter_and_min_score`
  (RED idem), unit `result_cache::tests::cache_key_distinguishes_*`; `semantic_cache_integration` 4 ok.

## Q2F / Q2-B10 — `list_memories` nao descarta mais linha indecodificavel

- Contrato: nenhum doc promete resultado parcial, entao erro tipado (`EngramError::Storage`
  "list_memories: undecodable memory row: ..."; via MCP vira envelope normalizado `internal_error`,
  codigo ja existente). `load_tags` tambem propaga erro (antes `unwrap_or_default`/`filter_map(ok)`).
  Outros sites `filter_map(ok)` (snapshot/builder, hybrid, compact) seguem HYPOTHESIS e fora do escopo.
- Testes: `queries::tests::list::list_memories_errors_*` (2, RED `Ok([1])` antes) e
  `contract_matrix::memory_list_reports_an_undecodable_row_instead_of_dropping_it` (RED `isError must be true`);
  `--lib` 1646 passed.

### C6 fix rodada 1 — hooks (2026-10-05)

- Commit: `fix(hooks): filter by workspace before the cap and report unknown backlog`.
- Doc de `HookManager` corrigido: isolamento de panic so com panic=unwind; o perfil release usa
  `panic = "abort"` e la um panic de handler encerra o processo (perfil nao alterado).
- Filtro de workspace antes do teto de 50 (RED: ids estrangeiros menores consumiam o teto,
  memoria propria nao reforcada); lookups limitados a 1000. `remaining` vira `null` (com log)
  quando a contagem falha, nunca `0`.
- Testes: `--lib hooks` 26 passed.

### C6 fix rodada 1 — multimodal e sync (2026-10-05)

- Commits: `fix(multimodal): normalize workspace in ingest dedup and close PGID window` e
  `fix(sync): guard live-db pulls and record interrupted restarts`.
- Ingest: dedup normaliza o workspace antes do lookup ("WS-A" depois "ws-a"/" ws-a " criava
  duplicata; RED); workspace invalido -> erro tipado; resposta dedup devolve `perceptual_hash`
  e o mesmo formato; semantica de retry documentada no catalogo e em `docs/MCP_TOOLS.md`.
- `process.rs`: grupo morto ANTES de reaper o lider (`waitid(WNOWAIT)`), sem janela de
  reciclagem de PGID; nota de Windows; teste de retardatarios. Teste de hash renomeado.
- Sync: `download` recusa alvo com `-wal` nao vazio/`-shm`/`-journal` e faz fsync do diretorio;
  `download_checked` recusa o banco vivo; pull do worker recusa o proprio banco;
  interrupcao registrada mesmo com `last_error` anterior; `Sync` explicito bem sucedido
  reseta o backoff; contrato de `reset_offset` corrigido (salts nao sao restauraveis).
- Testes: `--lib sync::` (+cloud) 76, multimodal/handlers/pdf/hooks 133 passed.

### Q7 — rodada 1 de correcoes da revisao (2026-10-05)

- Commit: `fix(search): trust supervisor anchor and require supervisor id in candidate runner`.
- Ancora de floors agora vem do supervisor (`--floors-anchor <sha>`): estrito ancestral do
  candidato, com entrada para o mesmo hash de corpus e `review.status` aceito; `anchor_revision`
  saiu do arquivo de floors (forjado e ignorado). Novo `floors.accepted` no report.
- `--require-supervisor` (runner e capture), `--candidate-dir` (verificador roda de ref confiavel
  e nao importa codigo do candidato), `--output` fora dos checkouts, checkout reverificado depois
  do cargo, `build_env` + sha de `Cargo.lock` no report. Politica documenta o modelo de confianca e
  a regra de consumo do Q1. Bench: sem `latency_percentiles/noop`; `storage_modes/create` usa
  banco novo a cada 1000 linhas.
- Testes: `unittest scripts/test_run_quality_candidate.py` 63 OK (10 mutacoes novas mortas);
  `cargo test --test retrieval_quality` 8 passed.

### P1 — regressao de performance medida pelo Q7 (2026-10-05)

- Commit: `perf(intelligence): lowercase entity text once per extraction` (772e84a).
- Causa confirmada: `find_case_insensitive_match` (bf45c0b) alocava um `to_lowercase` por
  posicao de char para cada termo conhecido (~57). Novo `text_util::LowercaseIndex`: texto
  minusculo uma vez, mapa de offsets por char, `str::find` + alinhamento de janela; `Σ`
  (sigma final, dependente de contexto) comparado exatamente. Semantica identica ao matcher
  anterior (mantido como oraculo em testes de equivalencia + fuzz com seed fixa).
- Medido (M5 Pro, sequencial, Criterion padrao, load 5-10): `extract_mixed` 229-246 us ->
  5.0-5.6 us. Runner Q7 no worktree destacado limpo de 772e84a (load ~25): `status: pass`,
  ratio 0.496 (8.50 us / 17.144 us), teto 1.15 intocado.
- Storage: re-assert de permissao por operacao NAO e o gargalo do nao-escalonamento do Q7 —
  `get_memory` faz `UPDATE` de access tracking, entao os "readers" do bench sao escritores
  serializados pelo lock de escrita do SQLite. Custo do re-assert ~3.5 us/op (relevante so em
  leituras puras). Sem mudanca em `src/storage` (G1/#27 preservado); proposta no report.
- Testes: `entity_extraction_alloc_tests` RED (149k-193k alocacoes) -> GREEN (3/3); suite
  `--tests` com features requeridas: 57 binarios, 2182 passed, 0 failed; clippy -D warnings ok.

## C4 — abertura SQLite descriptor-bound: investigacao + ADR (2026-10-05)

- Commit: `docs(storage): propose descriptor-bound SQLite open ADR`. Somente docs; `src/`
  intocado. ADR `docs/decisions/2026-10-05-c4-descriptor-bound-sqlite-open.md` (Proposed,
  owner Ronaldo) + notas do spike `...-spike-notes.md`.
- Spike descartavel fora do repo/crate (rusqlite 0.31.0 / SQLite 3.45.0 bundled, offline),
  macOS arm64 + Linux `rust:1.98-bookworm` (`--network none`, uid 1000): 29/29 testes em
  cada plataforma; soak final shim/stock 0/50 nas duas. BSD NAO RODADO.
- Achados: `/dev/fd/N` liga so o arquivo principal no macOS (writes/WAL falham) e no Linux e
  recusado por NOFOLLOW ou resolve de volta ao pathname (explica o CI de julho); NOFOLLOW
  deixa janela entre o `lstat` walk e `open(2)` (reproduzida); `unix-dotfile`/`unix-none`
  nao interoperam com peers do VFS padrao. Shim de syscalls com dirfd fixado e viavel
  (12 testes, pool/peer/checkpoint/DELETE/VACUUM INTO), mas depende de interface de teste
  do SQLite, e process-global e exigiu retry de ENOENT em `O_CREAT` no macOS (READONLY em
  18/95 runs antes do fix).
- Proposta: cadeia de diretorios confiavel (E) como fronteira; shim (B) viavel mas adiado;
  A/A'/C/D rejeitados. Abertura atomica NAO declarada resolvida; risco residual registrado.

## Q6 — rustls 0.23.45 (RUSTSEC-2026-0285), lock-only (2026-10-05)

- Commit: `fix(infra): update rustls to 0.23.45 for RUSTSEC-2026-0285`. Somente `Cargo.lock`
  (`cargo update -p rustls@0.23.36 --precise 0.23.45`; arrasta aws-lc-rs 1.17.0->1.18.1,
  aws-lc-sys 0.41->0.45, rustls-webpki 0.103.13->0.103.15). Manifest nao muda (rustls e transitivo).
- Notas lidas: rustls 0.23.45 (fix GHSA-2mjx-qc3c-rqvc), 0.23.44 (ML-DSA default, KeyLogFile 0600),
  aws-lc-rs 1.18.x (FIPS 4.x so afeta feature `fips`, nao usada), webpki 0.103.14/15 (ML-DSA, docs).
- Verificacao em worktree limpo (HEAD + lock): `--tests` requeridas 57 binarios, 2171 passed;
  1 flake de timing sob carga 20+ (`hanging_ffmpeg_times_out...`) passou isolado 2x; clippy
  `-D warnings` requeridas, `--all-features` e `CI_FEATURES` ok; `cargo audit` (DB fresco): RUSTSEC-2026-0285
  some, exit 0; `cargo deny check advisories bans licenses sources`: ok (antes: advisories FAILED).

## Q6 — rust_decimal 1.43.0 remove rkyv 0.7 do lock (RUSTSEC-2026-0235), lock-only (2026-10-05)

- Commit: `fix(infra): update rust_decimal to 1.43.0 to drop rkyv 0.7`. Somente `Cargo.lock`
  (`cargo update -p rust_decimal --precise 1.43.0`; transitivo via duckdb 1.4.4, feature
  `duckdb-graph`). Notas 1.43.0: "Remove rkyv 0.7 from feature bridge" (#819) + backports de fixes
  e performance. Saem do lock: rkyv 0.7.46, rkyv_derive, rend, bytecheck 0.6, ptr_meta, ahash 0.7,
  bitvec/funty/radium/tap/wyz, seahash, simdutf8, syn 1.0.109. rkyv ausente do lock (grep 0), a excecao
  RUSTSEC-2026-0235 em `.cargo/audit.toml` fica sem objeto (remocao fica com a lane P / merge).
- Verificacao em worktree limpo: `--tests` requeridas 57 binarios, 2171 passed, 1 flake de
  timing sob carga (`multimodal::process::timeout_kills_child_and_descendants`) passou isolado 2x;
  clippy `-D warnings` requeridas/`--all-features`/`CI_FEATURES` ok; audit exit 0; deny ok.

## Q6 — event-listener 5.4.2 (RUSTSEC-2026-0221), lock-only (2026-10-05)

- Commit: `fix(infra): update event-listener to 5.4.2 for RUSTSEC-2026-0221`. Somente `Cargo.lock`
  (transitivo: async-channel 2.5.0 -> event-listener-strategy 0.5.4 -> event-listener). Notas 5.4.2:
  "Fix unbounded Send/Sync implementations on StackSlot" (#163), remove implementacao slab, spinlock
  em intrusive.rs.
- Verificacao em worktree limpo: `--tests` requeridas 57 binarios, 2172 passed, 0 failed; clippy
  `-D warnings` requeridas/`--all-features`/`CI_FEATURES` ok; `cargo audit` (DB fresco) exit 0, avisos
  restantes: ttf-parser 0192 (governado) e lru 0253; `cargo deny check` ok.

## O1 — observabilidade sem conteudo proprietario (2026-10-05)

- Commit: o trabalho da O1 entrou no commit `35a1592` (assunto "update event-listener to
  5.4.2"), porque outro lane commitou com o indice compartilhado enquanto o commit da O1
  rodava o hook; a mensagem pretendida era `feat(mcp): redact logs and instrument critical
  paths`. Historico nao reescrito (lanes concorrentes). Novo modulo
  `src/observability/` (redact, operation, counters, alerts); logs dos caminhos criticos
  passam a registrar so a classe do erro (nunca texto de memoria, query, credencial, path,
  corpo de provider nem payload de panic); payload MCP, stdout do CLI e a coluna `error` do
  job de embedding ficam intactos. Opt-in explicito: `ENGRAM_LOG_ERROR_DETAIL=1`.
- Transporte HTTP: `x-request-id`/correlation id em toda resposta, outcome por classe
  (unauthorized, forbidden, rate_limited, parse_error, body_too_large, timeout,
  protocol_error, tool_error, handler_panic), histograma de latencia por outcome (sem dado =
  `null`, nunca zero), gauge de SSE ativo, gauge de in-flight sem vazamento em timeout/disconnect.
  Contadores criticos em `/health` (`observability`): permission_denied por razao C1,
  SQLITE_BUSY, timeout/falha de provider de embedding, recovery; rollback do export extra:
  `ENGRAM_OBSERVABILITY_EXPORT=off`.
- `docs/OPERATIONS.md`: contrato de logs, reconciliacao #161/#178 e #162/#179 (nao-goals
  explicitos), catalogo de alertas com owner/runbook/rollback, exercicio local de incidente
  (sem mensagens externas; disponibilidade local nao e SLO). RISK-0002 registrado.
- Testes: required-features `--tests` 58 binarios, 2226 passed, 0 failed, 2 ignored;
  clippy `-D warnings` (requeridas e `--all-features`) limpo. Residual: sites de log fora dos
  caminhos criticos (ex.: `storage/connection.rs`, `sync/wal_recovery_staging.rs`) nao varridos.

## Q6 — dirs 6.0 (dedup dirs-sys) + inventario de dependencias (2026-10-05)

- Commit: `build(infra): update dirs to 6.0 to dedupe dirs-sys` (572495b). `Cargo.toml` + `Cargo.lock`
  juntos. dirs 6.0.0 so sobe `dirs-sys` 0.5; API usada (home_dir/config_dir/data_dir/data_local_dir)
  intocada; `shellexpand` ja puxava dirs 6. Saem do lock dirs 5.0.1, dirs-sys 0.4.1, redox_users 0.4.6.
  `cargo tree -d` (CI_REQUIRED_FEATURES): 24 -> 22 crates duplicadas; (CI_FEATURES): 72 -> 70.
  `--tests` requeridas 2172 passed/0 failed; clippy requeridas/`--all-features`/CI_FEATURES ok; audit exit 0; deny ok.
- Q6 completo: 4 commits (rustls 0.23.45, rust_decimal 1.43.0, event-listener 5.4.2, dirs 6.0). RUSTSEC-2026-0285
  (vulnerabilidade) e 0221 resolvidos; rkyv 0.7 (0235) fora do lock. Sem `cargo clean`; sem mudanca de perfis.
  Nao feito de proposito: aws-sdk-s3 1.152 (limpa lru 0253) — `BehaviorVersion::latest()` + 32 releases do SDK
  sem canario S3/R2 real; excecoes (lane P) so reportadas. Detalhes: `.superpowers/sdd/.../task-Q6-report.md`.
- Este commit tambem carrega a nota da O1 (`35a1592`) que ja estava staged: o commit `9a6ed82`
  ("docs(harness): note where the O1 changes were committed") foi desanexado do branch por um
  `git reset --soft` meu acidental; o diff staged era byte-identico ao dele (md5 conferido), conteudo preservado.

### C4 fix rodada 1 (2026-10-05)

- Commit: `docs(storage): commit C4 spike and tighten descriptor-bound ADR`. Spike versionado
  em `docs/decisions/assets/2026-10-05-c4-spike/` (pacote Cargo isolado com `[workspace]`
  proprio, fora do workspace; `.semgrepignore` exclui o diretorio; README com comandos;
  evidencias em `evidence/*.txt` porque `*.log`/`logs/` sao ignorados).
- ADR: E passa a exigir tambem artefatos (db/-wal/-shm/-journal) regulares, do euid, modo
  `& 077 == 0`, re-checados por fstat; residuais (fd aberto antes) e custos de
  compatibilidade enumerados; "explica" -> "consistente com" o CI de julho (novo teste do
  formato alias verbatim + NOFOLLOW removido -> CANTOPEN no Linux); F10 "por inspecao";
  F7 reconciliado so com evidencia retida (stock 0/140; shim sem retry 12/50 no macOS) e
  follow-up 3 com receita de reproducao (bug potencial de disponibilidade, owner Ronaldo).
- Testes do spike: 35/35 macOS e Linux; soak 3 modos x 50 nas duas plataformas.

## O1 — fix round 1 (2026-10-05)

- Regras de alerta passam a depender de denominador (zero falhas sem tentativas = `no_data`);
  exercicio local afirma `no_data` em TODAS as regras num router novo. Teste de recovery agora usa
  banco em arquivo (recovery real); logs de path em `storage/connection.rs`, `sync/wal_recovery_staging.rs`
  e `sync/cloud.rs` redigidos com RED/GREEN. `replication_recover` classifica `rejected` (entrada) vs
  `failed` (engine); corrigido bug em que relatorio com `"error": null` contaria sucesso como falha.
  Timeout antes do handler mantem o invariante dos contadores; evento `abandoned` em desconexao.
  Testes: required-features `--tests` 2232 passed, 0 failed; clippy `-D warnings` limpo.

## Q2F / C3 follow-up — `ttl_seconds` com overflow vira erro tipado

- `chrono::Duration::seconds` e `DateTime + Duration` entram em panic em valores enormes (abort em
  release). Novo `storage/queries/core/ttl.rs::expiry_after` (aritmetica checada, limite
  `MAX_TTL_SECONDS` = 100 anos, `InvalidInput` fora da faixa) usado por `create_memory`,
  `update_memory`, `set_memory_expiration` e `acquire_dream_lock` (`u64` -> `i64` checado).
  TTL <= 0 em create/update segue significando "default"/"remover" (contrato inalterado).
- Testes: `contract_matrix::huge_ttl_seconds_is_rejected_on_every_mcp_entry_point` (memory_create,
  memory_create_daily, memory_update, memory_set_expiration com `i64::MAX`, `i64::MAX/1000`,
  100 anos + 1; RED = panic `chrono ... lib.rs:717` com o helper neutralizado),
  `queries::tests::expiration_ttl::out_of_range_ttl_is_a_typed_error_not_a_chrono_panic`, unit `ttl::tests`.

## Q2F / C3 follow-up — auditoria de fatias de bytes (content_utils, bundle)

- `intelligence/content_utils.rs::soft_trim` (REAL): com `preserve_words`, `first_space + 1` assumia
  espaco de 1 byte; NBSP/U+3000/U+2003 (whitespace multibyte) cortavam o char e o `content[tail_start..]`
  entrava em panic (RED: `start byte index 74 is not a char boundary; it is inside '\u{a0}'`). Corrigido
  somando `len_utf8()` do espaco. Demais fatias (143, 179, 190, 231, 239) derivam de `char_indices`/
  `rfind` de espaco ASCII ou do inicio do char: seguras por construcao (teste `compact_preview_is_char_boundary_safe`).
- `context/bundle.rs::truncate` (~499): ja era seguro (laco ate `is_char_boundary`); migrado para
  `text_util::truncate_bytes` + teste exaustivo `truncate_never_splits_a_multibyte_char`.

## Q2F / C3 follow-up — "last N days" no parser de linguagem natural

- Mesma classe do TTL: `NaturalLanguageParser` (API publica, sem handler MCP) chamava
  `Duration::days(N)` e `num * 7`/`* 30` sem checagem; "last 9223372036854775807 days" entrava em
  panic. Agora `try_days` + `checked_sub_signed`/`checked_mul`; fora da faixa = sem filtro de data.
- Teste: `natural_language::tests::huge_lookback_does_not_panic` (RED `TimeDelta::days out of bounds`).

## Q2F — inventario Q2 atualizado

- `docs/quality/rust-risk-inventory.md`: status + commit para B01-B11, B14, B17, B18 (C3/C7/Q2F)
  e tabela "Fixed outside the numbered backlog" (G-3, TTL, soft_trim, NL lookback). Sem reescrever
  contagens nem o resumo; B11 marcado PARTIAL (`build_related_map` segue aberto).
- Verificacao: suite `--tests` com features obrigatorias (arvore compartilhada com trabalho em
  andamento de C6/Unicode): 57 binarios, 2182 passed, 0 failed, 2 ignored; clippy `-D warnings`
  rodado pelo hook de pre-commit em worktree isolado (commits de Q2F).

## Q2F — rodada 1 de correcoes de review

- Mesma classe do TTL fechada em mais sites alcancaveis: `context_record_artifact`
  (`ttl_seconds`/`stale_after_seconds`), `memory_boost` (`duration_seconds`), `memory_archive_old`
  (`max_age_days`), `memory_get_working_memory` (`since_minutes`, antes `u64::MAX as i64` = -1),
  `cleanup_sync_data`, retencao (`auto_delete_after_days`, `compress_old_memories`), `create_api_key`
  e os dois pontos do backend Meilisearch. Helpers compartilhados reexportados de
  `storage::queries`: `expiry_after`, `cutoff_days_ago`, `MAX_TTL_SECONDS`, `MAX_OFFSET_DAYS`.
  Nao tocados (config-driven/nao-default): `gardening`, `auto_consolidate`, `graph::coactivation`, backend turso.
- NL parser: lookback fora da faixa nao aplica filtro e agora sinaliza em
  `ParsedCommand.params["ignored_date_filter"]` (contrato existente de `params`).
- `memory_set_expiration` usa `ToolError` normalizado; `list_memories` nomeia o id da linha corrompida e o
  catalogo/`MCP_TOOLS.md` documentam; `mcp install` grava atomico (tmp+rename), reaproveita backup
  identico em vez de duplicar e `--force` esta documentado em `docs/GETTING_STARTED.md`.
- Testes: `huge_time_offsets_are_rejected_with_normalized_errors_on_covered_tools` (RED por site:
  panic chrono `lib.rs:717` com cada correcao revertida; `since_minutes` passava sem erro),
  `soft_trim_cuts_at_multibyte_whitespace_with_exact_output`, `compact_preview_exact_output_on_multibyte_text`.
- Suite `--tests` (features obrigatorias, `--no-fail-fast`): 56 binarios, 2171 passed, 1 failed, 2 ignored; a falha
  e um teste de processo do C6 (`multimodal::{process,video::failure_tests}`) que reprova sob carga ~30 e passa isolado
  (93 passed em `--lib multimodal`).
