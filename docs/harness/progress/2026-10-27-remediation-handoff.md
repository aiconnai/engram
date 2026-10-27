# Handoff — sessão de remediação Engram (2026-10-26 → 2026-10-27)

Handoff para a próxima IA (Claude, Codex, Gemini ou outra) assumir esta frente sem reler a conversa.
Escrito pelo integrador da sessão (Claude Code, perfil `claude-consultores`, Opus 5.5). Fatos com SHA
foram verificados no repositório; o que não foi verificado está marcado como tal.

> **Atualização 2026-10-27:** a pilha foi publicada e mesclada no `main` (#244–#262, opção A). As seções
> 1, 3 e 7 descrevem o estado anterior. Registro: [Publicação](./2026-10-26-remediation.md#publicação-2026-10-27).

## 1. Estado em 30 segundos

- **Branch de trabalho:** `remediation/integration` (worktree `.worktrees/rem-int`). Contém tudo: os 4
  commits locais antigos do `main`, o turso-sunset e as 14 frentes de remediação, todas mescladas.
- **Verde:** `make ci` completo PASS em `317f3cba` (2344/2344 testes, offline lane 12/536, docs e WASM
  ok); `cargo audit` 0 vulnerabilidades; `cargo deny` ok; `check-live-state.sh` estrito PASS. O commit
  que adiciona este arquivo é só de docs.
- **Nada foi publicado.** Push, PR, merge, tag, release, publicação em registries, remoção de
  branches/worktrees e settings do GitHub **não estão autorizados**. A próxima ação depende do owner
  (seção 7).
- **No GitHub (`aiconnai/engram`, repo público):** `origin/main` = `bd3cee13`. O CI agendado do `main`
  está vermelho desde 2026-10-17 (Security Audit e Cargo Deny, que são checks obrigatórios) e o Nightly
  também. As duas causas estão corrigidas na integração, mas só valem depois do merge.

## 2. Como retomar (comandos)

```bash
cd /Users/ronaldo/Projects/_aiconnai/engram/.worktrees/rem-int   # NÃO trabalhe no checkout principal
rtk git status && rtk git log --oneline -5                       # esperado: limpo, HEAD = commit deste handoff
bash docs/harness/bin/bootstrap.sh                               # leitura obrigatória (ordem em CLAUDE.md)
bash docs/harness/bin/doctor.sh                                  # esperado: OK + 1 WARN (sprint sem review)
bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md   # esperado: PASS estrito
make ci                                                          # ~6-10 min; `just` não está instalado
cargo audit && cargo deny check advisories bans licenses sources # make ci não roda os dois
```

- O checkout principal (`/Users/ronaldo/Projects/_aiconnai/engram`) está no `main` local `b29cd16c`, com
  estado do harness antigo. Ele **não** tem este trabalho.
- Leia primeiro: este arquivo, `docs/harness/progress.md` ("Retomada rápida"), o log da sprint
  [`2026-10-26-remediation.md`](./2026-10-26-remediation.md), o
  [plano](../plans/2026-10-26-remediation-plan.md), `CLAUDE.md` (Key Decisions e Active Risks) e
  `CHANGELOG.md` `[Unreleased]`.

## 3. Autoridade e limites (o que o owner disse)

| Quando | Owner disse | Efeito |
|---|---|---|
| 2026-10-26 | "status" e depois "levante tudo … com múltiplos subagents haiku 5.5 e sonnet 5.5" | Levantamento só leitura |
| 2026-10-26 | "resolva tudo o mais paralelo possível" | Implementação **local** em branches/worktrees; commits locais |
| 2026-10-26 | Recusou a pergunta sobre publicação, release, deleções e settings | Nada externo autorizado; parar e esperar |
| 2026-10-26 | "quero, planeje uma janela de 22 min" | Integração local com os padrões propostos (seção 5) |
| 2026-10-26 | "avance uma janela de 40 minutos com máximo paralelismo" e "resolva" | Mais frentes locais e fechamento verde |
| 2026-10-27 | "crie um handoff completo dessa sessão para outra IA" | Este arquivo |

Regras do owner e do projeto que valem sempre:
- **Commits:** conventional commits (`type(scope): descrição`, < 72 chars), **sem `Co-Authored-By`
  nem trailer de IA**. É regra global do owner e INVARIANTS #5, mesmo que o harness do cliente sugira o
  trailer. Nunca `--no-verify`.
- **Bash com RTK:** `rtk git ...`, `rtk grep ...`; use `rtk proxy <cmd>` para saída exata (diffs, JSON,
  hashes).
- **Merge humano obrigatório** (política do harness). Ausência de revisão de IA não bloqueia. Um parecer
  de IA não é receipt nem autoriza merge.
- **Wire format** (status HTTP, formato JSON) não muda sem coordenação. Ver
  `~/.claude/rules/development-workflow.md`.
- **Worktrees:** no máximo 4–6 escritores ao mesmo tempo, uma frente por worktree, ownership exclusivo
  de arquivos.
- **Linear:** só pela skill `linear-graphql-cli`, nunca MCP (não usado nesta sessão).
- **Idioma:** o owner escreve em PT-BR e prefere respostas em PT-BR.

## 4. Topologia

### SHAs de referência
| Ref | SHA | Observação |
|---|---|---|
| `origin/main` | `bd3cee13` | v0.24.0 preparada, sem tag (última tag/release: v0.21.2) |
| `main` local | `b29cd16c` | 4 commits não enviados: `d922a60e`, `e04d3307`, `c9de7136`, `b29cd16c` |
| base da remediação | `08cb17a4` | `main` local + `refactor(storage): complete turso-sunset` |
| `remediation/integration` | `317f3cba` (+ commit do handoff) | 120 commits à frente de `origin/main` |

### Branches mescladas na integração (todas locais, todas com worktree `.worktrees/<wt>`)
| Frente | Branch (worktree) | Modelo | O que faz | SHA da branch |
|---|---|---|---|---|
| WT1 | `fix/workspace-batch-auth` (rem-auth) | Sonnet | P0: autorização por item em `memory_create_batch` e `memory_ingest_fact_batch` (regressão de `e04d3307`); subscribe por principal; fail-closed para tool desconhecida; `memory_search_suggest` limitado ao workspace; `cache_clear` com formato único; SDK com limit 20, offset e include_archived | `ecf8c366` |
| WT2 | `fix/supply-chain-2026-10` (rem-supply) | Sonnet | zip 9.0.5, duckdb 1.10506.1, tokio 1.53.3, rustls-webpki, webpki-roots, ctutils; remove `sigv4a` (crypto-bigint); sqlite-vec vira dev-dep; fixa cargo-audit e cargo-outdated; Node 24; ADRs Turso | `0516204d` |
| WT3 | `fix/release-pipeline` (rem-release) | Sonnet | `sdk-release.yml` deriva tag e versões dos manifests; `bump-version.py --release-plan`; package-lock 0.7.0; docs de versão | `772743e2` |
| WT4 | `fix/mcp-surface-honesty` (rem-surface) | Sonnet | Gating de tools com fonte única (`catalog::required_features`); rerank placeholder retorna erro; contagens (301/254); claims públicos; checkers `check-public-claims.py` e advertised-contracts | `3c58730a` |
| WT5 | `chore/harness-live-state` (rem-harness) | Sonnet | Sprint "Remediação 2026-10-26", `check-live-state.sh` parametrizado, contador do bootstrap, provenance, `CLAUDE.md`, risk register | `ffc326b2` |
| WT6 | `docs/product-status` (rem-docs) | Haiku | Status de RFCs, ADRs e planos com evidência; nomes de tools no ROADMAP; 7 alertas de code scanning | `8ea2832a` |
| WT7 | `fix/hnsw-default-propagation` (rem-hnsw) | Sonnet | HNSW opt-in; tombstones na remoção; busca com fetch ampliado | `307bce39` |
| WT8 | `docs/claims-and-triage-trim` (rem-claims) | Haiku | Doc de decision-triage enxugado; contagens de concorrentes removidas; `architecture.html` | `f0752ea6` |
| WT9 | `fix/hnsw-followups` (rem-hnsw2) | Sonnet | `ENGRAM_HNSW_ENABLED`; reconcile com SQLite; compactação viva; warmup sem vetores antigos | `93165594` |
| WT10 | `fix/security-followups` (rem-sec2) | Sonnet | Envelope `tools/call` desembrulhado num ponto só; escalada de write-flag no relatório; e2e gRPC | `83e3816c` |
| WT11 | `docs/followups-and-harness-tools` (rem-docs2) | Haiku | 9 variáveis de ambiente documentadas; claims sobre sqlite-vec; `justfile`; bugs do `harness-risk-register.sh`; inventário de risco | `75a1d87a` |
| WT12 | `fix/log-redaction-sweep` (rem-redact) | Sonnet | RISK-0002: 41 pontos redigidos em sync, hooks, multimodal e watcher | `f7cafc53` |
| integrador | `fix/live-state-review-authority` (rem-live) | Opus | Review mais recente obrigatório quando a tarefa ativa não tem review; slug exato | `383b70a8` |
| integrador | `docs/fix-getting-started-links` (rem-links) | Opus | Links quebrados (já existiam) do GETTING_STARTED | `8ee32e87` |

Commits do integrador direto na integração:
- `f53b75a0`: fiação do `subscribe_as` no servidor.
- `d76bceee`: contador do bootstrap.
- `c8e1884d`: CHANGELOG e checks no job de docs.
- `93fc8566`: advertised-contracts num job não obrigatório.
- `4134d28b`: link intra-doc.
- `14a65a54`: o override por chamada `_permission_mode` volta a gerar 200 com a recusa no resultado.
- Commits de estado do harness: `08ca8288` e `317f3cba`.

### Outras worktrees (não são desta remediação)
- **Removíveis com segurança** (conteúdo no `main`, verificado por patch-id): `adoring-chatelet-29de61`,
  `wave4-todo22`, `23`, `27`, `37`, `38`, `39` e `harness-hardening-v1`.
- **Cópias idênticas a `08cb17a4`:** `turso-sunset`, `turso-sunset-ci` e `turso-sunset-docs` (byte a byte).
- **`wave4-todo24`:** descartar (75a1e16e foi substituído, segundo R2).
- **`wave4-todo26`:** tem um `.omo` não versionado (symlink).
- **Remover depende de autorização do owner.**

## 5. Decisões já tomadas (revisáveis pelo owner)

- **HNSW:** desligado por padrão (RFC 0003 aceita); liga com `ENGRAM_HNSW_ENABLED` / `--hnsw-enabled`.
- **Federação:** marcada nas docs como não implementada (`src/federation/` nunca existiu). Os shims do
  SDK continuam como aliases. A entrada histórica do CHANGELOG 0.12.0 segue errada, sem nota.
- **Decision-triage:** guia enxugado (contabilidade, personas, issue externa e paths locais removidos).
  `decision-triage.json` não foi tocado, porque tem SHA fixado numa allowlist externa.
- **Dependências:** SigV4a removido do `aws-sdk-s3` (MRAP nunca foi suportado); DuckDB 1.4.4 → 1.5.6.
- **`CrossEncoderReranker`:** retorna erro "not implemented" em vez de scores falsos. É API pública, sem
  chamadores no repositório.
- **README:** contagens de tools dos concorrentes, sem fonte, removidas.
- **Advertised-contracts:** o check roda num job **não obrigatório**. Proteger o baseline exige versionar
  a política de CI do merge-gate (seção 7).

## 6. Evidência de verificação

| Alvo | Resultado |
|---|---|
| `make ci` @ `b29cd16c` (antes da remediação) | PASS, 410 s, 2265 testes |
| `make ci` @ `932e9c8d` | FAIL só no `cargo doc` (link intra-doc), corrigido em `4134d28b` |
| `make ci` @ `4134d28b` | PASS, 628 s, 2319/2319, offline lane 12/534 |
| `make ci` @ `08ca8288` | FAIL em 1 teste: `canonical_journey::contract::rejected_requests_never_mutate_shared_state_across_stdio_and_http` (pré-check HTTP devolvia 403), corrigido em `14a65a54` |
| `make ci` @ `317f3cba` | **PASS, 364 s, 2344/2344 (2 skipped), offline lane 12/536** |
| `cargo audit` @ `08ca8288` | 0 vulnerabilidades; 1 aviso governado (ttf-parser, RUSTSEC-2026-0192) |
| `cargo deny` @ `08ca8288` | advisories, bans, licenses e sources ok; `Cargo.*` não mudou depois |
| `check-live-state.sh` estrito @ `317f3cba` | PASS |
| `sensors.sh` | **Não executado**: grava `.sensors-last` e `.sensors-log`, que são versionados. `Last sensors` segue 2026-10-23 |

**Revisões feitas** (pareceres de IA, não receipts):
- WT1: integrador, aprovado.
- R3 (Sonnet): WT2, WT4, WT7 e `bin/*`. Sem crítico nem alto. O achado médio foi corrigido em
  `383b70a8` e os baixos pelo WT9.
- R1 (Sonnet): 7 arquivos `.raw` sinalizados no histórico. Sem segredo real (e-mail e home path do
  owner), sem rotação; não recomenda reescrever o histórico.
- R2 (Sonnet): nada a portar de `claude/engram-improvement-plan-edb43d`.

## 7. Pendências

### Decisões do owner (bloqueiam publicação)
1. **Publicação:** push da integração e PRs. Sugestão: PRs por frente ou em pilha. O merge é do owner.
   Os commits do decision-triage (`c9de7136`, `b29cd16c`) entram já enxugados por WT8.
2. **Release v0.24.0 e SDK 0.7.0:** runbook abaixo.
3. **`verify-tag` no `sdk-release.yml`:** falha no runner por falta de chave confiável. Opções: arquivo
   de allowed-signers (tags SSH), flag `verification.verified` do GitHub, ou remover o check.
4. **Merge-gate:** incluir `docs/contracts/advertised-surfaces.toml` em `CI_PROTECTED_FILES`
   (`docs/harness/bin/merge_gate_ci.py`). Isso muda o fingerprint fixado para `harness-hardening-v1` e
   exige nova versão da política.
5. **Recibos humanos:** post-gate do turso-sunset e da sprint; revisão independente das mudanças em
   `docs/harness/bin/*` (`check-live-state.sh`, `bootstrap.sh`, `quarterly-audit.sh`,
   `run-offline-lane.sh`, `harness-risk-register.sh`). Canvas:
   `docs/harness/canvas/2026-10-26-harness-script-fixes.md`.
6. **Limpeza:** as worktrees da seção 4 e as worktrees `rem-*` depois do merge. Branches: 16 FOLDED e
   10 SUPERSEDED dos `codex/10of10-*`, `wave2-*` e `wave3-*`; `claude/engram-improvement-plan-edb43d`
   (sem nada a portar); 38 branches remotos de PRs mesclados.
7. **Settings do GitHub:** Dependabot e secret scanning estão desligados; o `main` não exige review
   (D8, `enforce_admins=false`).
8. **ADR C4** (open do SQLite por descriptor): continua *Proposed*.

### Backlog técnico (não bloqueia)
- **RISK-0002:** redação de logs no restante do repositório; `session_id` segue em claro nos hooks.
- **HNSW:** o `update_memory` não limpa a linha antiga de `embeddings`; não há reconcile periódico se
  drain e cleanup estiverem com intervalo 0; mutadores avulsos (`project_context`, `auto_tag`, `dream`,
  markdown import, handoff, CLI) só se corrigem no próximo reconcile.
- **Pré-check gRPC:** `permission_denial_for_principal` ignora write flags; o dispatch aplica.
- **Ledger de minors** (`docs/harness/known-issues/2026-10-26-deferred-review-minors.md`): linhas 66,
  245, 254 e 308 abertas ou parciais; ~60 minors não triados.
- **Produto:**
  - Ciclo de deprecação de tools (spec §5) não implementado.
  - `garden_undo` não exposto via MCP.
  - Hook `stop` vazio.
  - Funções sem chamador em sync, cloud e fila de embeddings.
- **Ferramentas:** piso de Python 3.10 no SDK; MSRV não declarada; toolchain nightly fixada em
  2026-04-21; seções 1 e 4–7 de `docs/quality/rust-risk-inventory.md` com números de 2026-10-05.
- **Docs:** `docs/architecture.html` ainda tem mock-ups com tempos ilustrativos; nota de federação na
  entrada 0.12.0 do CHANGELOG.

### Runbook de release (resumo do WT3; detalhes em `docs/releases/SDK_PUBLISHING.md`)
1. PRs mesclados no `main`; rodar `make ci` e `python3 scripts/bump-version.py --release-plan --release-tag v0.24.0`.
2. Observar de novo os registries: `python3 scripts/check-release-channels.py --matrix docs/releases/channel-matrix.toml --read-only`; atualizar `observed_*`.
3. Dry-run do `release.yml` e do `sdk-release.yml`.
4. Resolver o `verify-tag` (pendência 3).
5. **[IRREVERSÍVEL]** `git tag -s v0.24.0 <sha> && git push origin v0.24.0`.
6. **[IRREVERSÍVEL]** `cargo publish -p engram-types` (se faltar no crates.io), depois `engram-core`.
7. **[IRREVERSÍVEL]** `sdk-release.yml` com `publish=true` (PyPI e npm), com aprovação nos environments.
8. Atualizar `latest_released_tag` e `observed_*`.

Antes do release, a cópia do checkout principal de `release/RELEASE_NOTES.md` (gitignored, v0.22.0)
está velha.

## 8. Armadilhas e lições desta sessão

- **`sensors.sh` grava arquivos versionados.** Para verificar sem sujar a árvore, use `doctor.sh` e
  `make ci` (`scripts/ci.sh`). O `make ci` não roda `cargo audit` nem `deny`; rode à parte.
- **Nunca compartilhe `CARGO_TARGET_DIR` entre worktrees.** Os testes executam binários
  (`engram-server`, worker de PDF) pelo caminho fixo em `target/`, então uma worktree pode testar o
  binário de outra. Dê a cada worktree o seu `target/` com clone APFS:
  `cp -cR <outra>/target <wt>/target` (instantâneo, copy-on-write).
- **O pre-commit roda `cargo fmt --check` e `clippy --all-targets --all-features`.** Não faça commit
  numa worktree enquanto `make ci` roda nela: os dois disputam o lock do cargo.
- **Live-state:** "Last commit" precisa ser o HEAD ou o primeiro pai do HEAD. Para atualizar, grave o
  HEAD atual no campo e faça o commit. Depois de qualquer novo commit ou merge, atualize de novo.
- **Autoridade de review** (`check-live-state.sh`): sem review da tarefa ativa, "Last review" precisa ser
  o PASS mais recente de todos.
- **Merge-gate:** um teste de drift exige que todo arquivo usado por um job obrigatório do `ci.yml`
  esteja em `CI_PROTECTED_*`. Mudar essa lista dispara `policy_mismatch`, porque o fingerprint está
  fixado. Não contorne a trava.
- **Depois de mudar docs com variáveis de ambiente ou contagens de tools:**
  `python3 scripts/check-advertised-contracts.py --inventory docs/contracts/advertised-surfaces.toml --write`,
  depois o mesmo comando sem `--write`, e `python3 scripts/check-public-claims.py` (que proíbe
  "sub-millisecond", contagens velhas e "native adapters").
- **Contador do bootstrap:** lê `catalog::required_features` e deve mostrar `254 active / 301 total`.
- **Contrato de wire:** a recusa por `_permission_mode` da própria chamada viaja no resultado (HTTP
  200). O teste `tests/canonical_journey` protege isso; o pré-check HTTP usa
  `permission::check_transport_authorization`.
- **Ownership:** o CHANGELOG fica com o integrador. As frentes entregam fragmentos e o integrador monta
  o `[Unreleased]`.
- **Agents só de leitura** às vezes deixam arquivos em `/tmp` com redirecionamento; ponha "sem
  redirecionar para arquivos" no brief.
- **zsh:** `$VAR:s` é modificador; use `${VAR}`.
- **Gating de features:** um nome de feature desconhecido falha fechado em `gate_feature_compiled`, e a
  drift entre catálogo e dispatch é coberta por testes.

## 9. Arquivos-chave
- Estado: `docs/harness/progress.md`, `docs/harness/progress/2026-10-26-remediation.md`,
  `docs/harness/plans/2026-10-26-remediation-plan.md`, `docs/harness/SPEC.md`,
  `docs/harness/risk-register.yaml`, `docs/harness/known-issues/2026-10-26-deferred-review-minors.md`.
- Autorização: `src/mcp/permission.rs` (`check_tool_authorization`, `check_transport_authorization`,
  `tool_call_parts`, `requested_workspaces`), `src/mcp/workspace_guard.rs`,
  `src/mcp/http_transport/mcp_handler.rs`, `src/mcp/grpc_transport.rs`, `src/mcp/resources.rs`.
- HNSW: `src/search/hnsw.rs`, `src/search/hnsw_sync.rs`, `src/search/hybrid.rs`, `src/bin/server.rs`.
- Superfície: `src/mcp/tools/catalog/mod.rs` (`required_features`), `docs/MCP_TOOLS.md`,
  `docs/contracts/advertised-surfaces.toml`.
- Supply chain: `docs/security/advisory-exceptions.toml`, `.cargo/audit.toml`, `deny.toml`,
  `docs/security/security-gate-evidence.md`.
- Release: `.github/workflows/sdk-release.yml`, `scripts/bump-version.py`,
  `docs/releases/channel-matrix.toml`, `docs/VERSIONING.md`.

## 10. Uso de IA na sessão (subtotal observado)

| Fase | Participantes | Tokens (subagents) |
|---|---|---|
| Levantamento (10 + F2) | 4 Sonnet + 7 Haiku | 1.273.128 |
| Remediação (WT1–WT6 + R1) | 6 Sonnet + 1 Haiku | 1.734.338 |
| Janela de 22 min (WT7, WT8) | 1 Sonnet + 1 Haiku | 174.075 |
| Janela de 40 min (WT9–WT12, R2, R3) | 5 Sonnet + 1 Haiku | 689.428 |
| **Total** | 16 Sonnet + 10 Haiku | **3.870.969** |

- **Integrador:** consumo não medido.
- **Modelos:** os informados são autodeclarados pelos agents (`claude-sonnet-5-5`, `claude-haiku-5-5`)
  e não foram autenticados.
- **Registro efêmero:** `/tmp/engram-survey-20261026/` (`registry.md` e logs de CI). Pode sumir.
