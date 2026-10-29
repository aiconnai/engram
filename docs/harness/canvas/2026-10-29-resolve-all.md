# Review Canvas — Resolução consolidada (release readiness, contratos, harness e auditoria de 5 unidades)

Data: 2026-10-29. Base: `21cf80e9` (sobre `origin/main` `2e84a594`). Branch: `fix/resolve-all-2026-10-29`.
Mandato: [plano de execução](../plans/2026-10-27-repository-execution-plan.md) e auditoria de 5 unidades.
Este canvas registra abordagens, complexidade e riscos como evidência técnica; **não concede aprovação nem substitui receipt humano**.

## Abordagens consideradas e decisões

1. **R1 (`src/text_util.rs` — 4 timeouts do `cargo-mutants`)**:
   - *Alternativa descartada*: adicionar exclusões em `.cargo/mutants.toml` ou reduzir o escopo de testes Unicode adversariais (proibido pela política fail-closed da lane Nightly).
   - *Decisão*: delegar `floor_char_boundary` e `ceil_char_boundary` diretamente para `str::floor_char_boundary` / `str::ceil_char_boundary` da stdlib (eliminando os 3 laços `while` mutáveis) e reescrever `LowercaseIndex::find_window` com `remaining.split_first()` (fatiamento estrutural sem aritmética `cursor += 1` que travava sob mutação `+ -> *`), adicionando caso determinístico para spans com byte inicial idêntico (`"a — İ\u{307}\u{307}"` / `"\u{307}\u{307}"`). Resultado: **42/42 mutantes detectados, 0 missed, 0 timeouts**.
2. **R3 (`scripts/verify-release-tag.sh` + `.github/workflows/sdk-release.yml`)**:
   - *Alternativa descartada*: confiar no arquivo `allowed-signers` ou no script verificador contidos na própria tag verificada (permitiria auto-autorização de chave ou substituição do script) ou usar apenas flag `verified` da API GitHub.
   - *Decisão*: helper dedicado `scripts/verify-release-tag.sh` materializado a partir de `origin/main:scripts/verify-release-tag.sh` em `${RUNNER_TEMP}` nos três jobs (`preflight`, `publish-python`, `publish-npm`), carregando `allowed-signers` exclusivamente de `origin/main:docs/releases/allowed-signers` (ou `ENGRAM_RELEASE_ALLOWED_SIGNERS` / `ENGRAM_RELEASE_ALLOWED_SIGNERS_FILE` externo), pinando `-c gpg.ssh.program=ssh-keygen`, falhando fechado se o trust store estiver vazio (`docs/releases/allowed-signers` permanece sem chaves até o owner commitar sua chave SSH pública no `main`) e validando tag anotada + assinatura SSH + igualdade exata com `HEAD` e `expected-sha`.
3. **C1 (Default de importação Markdown — RFC 0004)**:
   - *Alternativa descartada*: manter escrita imediata na CLI/SDKs e documentar a divergência do RFC 0004.
   - *Decisão*: alinhar `ImportOptions::default()`, `engram-cli import markdown`, MCP `memory_import_markdown` (adicionando `"dry_run": !confirm`), Python `client.vault.vault_import()` e TypeScript `client.vault.import()` para modo seguro de revisão (`confirm = false`), exigindo `--confirm` / `confirm = true` para persistir e preservando `--dry-run` na CLI como alias compatível.
4. **H2 (`docs/harness/bin/merge_gate_ci.py` e `merge-gate.py`)**:
   - *Alternativa descartada*: retornar `set()` silencioso em `GitError` durante inspeção de workflows da revisão base.
   - *Decisão*: converter `GitError` em `CiRefusal("ci_base_unverifiable" | "ci_workflow_unverifiable", ...)` fail-closed, cobrindo `git show` e `git ls-tree` com testes herméticos (`FLOOR_MERGE_GATE=50`).
5. **S1 e Unidades 1–5 (Runtime, Storage, Lifecycle/Auth, Consolidação e Document Ingest)**:
   - *Alternativa descartada*: alterar assinaturas públicas Rust (`docs/api/types-baseline.txt` é pinado por hash/contagem de linhas) ou abrir `unchecked_transaction()` dentro de `OfflineConsolidator` (quebraria `dream::run_once_workspace`, que já envolve a chamada em `Storage::with_transaction`).
   - *Decisão*: usar helpers `pub(crate)` (`compress_old_memories_scoped`, `OfflineConsolidator::preview_with_strategy`, `active_artifacts::*`), `SavepointGuard` (`SAVEPOINT sp_consolidate_group`) compatível com autocommit e transações externas, incluir as 9 tools de lifecycle/policy em `ID_SCOPED_TOOLS` com `ensure_memory_access` dentro de cada transação/conexão e sincronia 4-way MCP/SDK (`0` drift), e reescrever `chunk_text` sobre `Vec<char>` com `min_break = (chunk_size / 2).max(effective_overlap)`.

## Hot-path complexity

- `text_util::{floor_char_boundary, ceil_char_boundary}`: $O(1)$ (máximo 4 bytes UTF-8), zero alocações.
- `LowercaseIndex::find_window`: $O(N \cdot M)$ no pior caso de substring sobre fatia lowercased já indexada, zero alocações adicionais no loop de busca.
- `ensure_memory_access` nas 9 tools `ID_SCOPED_TOOLS`: 1 lookup SQLite por chave primária `INTEGER PRIMARY KEY` ($O(\log N)$, ~15 µs) na mesma conexão/transação do handler; zero I/O de rede.
- `OfflineConsolidator::{preview_with_strategy, consolidate_with_strategy}`: executado apenas em background/sleep-time (`batch_size = 100`), $O(B^2)$ comparações Jaccard em memória + 1 `SAVEPOINT` por grupo válido.
- `DocumentIngestor::chunk_text`: $O(C)$ em tempo e memória para $C$ caracteres da seção, avanço estrito garantido ($\ge 1$ caractere por iteração).

## Edge cases verificados

1. **Expansão multibyte no lowercase (`İ` → `i\u{307}`) seguida de caractere combinante idêntico (`"a — İ\u{307}\u{307}"`)**: múltiplos caracteres de destino compartilham o mesmo byte inicial de origem; `LowercaseIndex::find_window` valida fronteiras de caractere sem loop infinito nem fatia inválida.
2. **Bancos SQLite `:memory:` em `StoragePool::new(":memory:", N)` e `Storage::compact`**: pool clampado para `1` conexão (evitando conexões isoladas sem schema) e `available_disk_bytes(":memory:")` ignorado com `saturating_mul` contra overflow em `u64`.
3. **Crash loop repetido do `SyncWorker` e tentativa de `Pull`/`Push` contra o banco ativo**: `format_interrupted_last_error` deduplica o sufixo `[interrupted sync reset on startup]` e limita `last_error` em `1024` bytes UTF-8; `is_live_database` / `push_database` recusam aliases de inode/descritor (`/dev/fd`, hardlinks, `-wal`/`-shm`) via `active_artifacts`.
4. **Cross-workspace IDOR via `memory_lifecycle_update(action="decay", id=foreign_id, workspace=own_ws)` e chamadas diretas aos handlers**: bloqueado por `ensure_memory_access` antes de resolver workspace ou mutar `memory_policy`, mantendo `row_state` e `memory_policy` idênticos.
5. **Chunking UTF-8 com `overlap >= chunk_size / 2` e escrita PDF com codepoint UTF-8 dividido entre chamadas `Write::write`**: `min_break = (chunk_size / 2).max(effective_overlap)` garante `actual_end - start > effective_overlap` (preservando overlap exato de caracteres sem estagnação); `BoundedPdfText` acumula `Vec<u8>` limitado por `max_bytes` e decodifica UTF-8 ao fechar a página.

## Breakage risk

| Superfície alterada | Risco de quebra para consumidores | Mitigação e evidência |
|---|---|---|
| Default de importação Markdown (`confirm=false` em CLI, Rust, Python e TS SDKs) | **Médio (breaking intencional — RFC 0004)**: scripts que chamavam `engram-cli import markdown` ou `vault_import` / `vault.import` sem `confirm` passam a receber preview sem escrita | Documentado em `CHANGELOG.md`, `docs/releases/v0.7.0-sdk-notes.md` e `v0.24.0-readiness.md`; `--confirm` / `confirm=true` explícito e `--dry-run` mantido na CLI como alias |
| Schema MCP `memory_lifecycle_update` (remoção de `"purged"` em `state`, adição de `workspace` e `dry_run`) | **Baixo**: `"purged"` já falhava em runtime (`LifecycleState::from_str`); novos campos são opcionais | `generate-mcp-reference.sh` + `check-sdk-contract-alignment.py` (`0` drift) + testes Python/TS/Rust |
| Baseline público de tipos Rust (`docs/api/types-baseline.txt`) | **Zero**: nenhuma assinatura `pub` alterada (novos helpers são `pub(crate)`) | Verificado por `scripts/check-advertised-contracts.py` (`line_count` e `sha256` intactos) |
| Verificação de tag de release (`scripts/verify-release-tag.sh`) | **Médio operacional (fail-closed intencional)**: `sdk-release.yml` recusará tags até o owner commitar sua chave SSH pública em `docs/releases/allowed-signers` no `main` | Documentado explicitamente no checklist de release e validado por 14 testes em `scripts/test_sdk_release_workflow.py` |
| `docs/harness/bin/merge_gate_ci.py` (`GitError` → `CiRefusal`) | **Baixo (process-critical)**: repositórios com `base_rev` ilegível falham fechados em vez de ignorar referências dinâmicas | 9 testes herméticos `CiPolicyPaths` + lane offline (`FLOOR_MERGE_GATE=50`) + revisão independente R2 |
