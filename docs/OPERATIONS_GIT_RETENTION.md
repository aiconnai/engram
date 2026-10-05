# Higiene Git e retenção recuperável (task O2)

> Escopo: o que fica versionado, o que fica fora do git, por quanto tempo, onde, como achar e como
> restaurar. **Untracking e `.gitignore` não são limpeza de histórico.** Nada neste documento apaga
> objeto, ref, reflog ou arquivo versionado. Ferramenta:
> [`docs/harness/bin/retention-manifest.py`](./harness/bin/retention-manifest.py); testes:
> `docs/harness/tests/test_retention.py` (hermético, usa clones descartáveis); manifest da única
> classe migrável: [`docs/harness/retention/review-raw.manifest.json`](./harness/retention/review-raw.manifest.json).
> Orçamento de contexto e retenção do `progress.md`: [`harness/context-budget.md`](./harness/context-budget.md).

## 1. Estado desta política

- **Proposta técnica pronta e provada; migração real não aplicada.** A classe `review-raw` foi
  migrada **somente em clones descartáveis** (seção 6). No repositório real, nenhum arquivo foi
  destracked, nenhuma regra nova entrou no `.gitignore` e nenhum objeto foi tocado. Aplicar a migração
  na branch real é decisão do owner (seção 5).
- Durações abaixo são **propostas** a ratificar pelo owner; nenhuma é imposta por código hoje.
- **Não prometemos redução de MB.** Só as medições da seção 3 e 6 valem, com data. O ganho de
  untracking é de *checkout* (tamanho do working tree de um clone novo); o object store não encolhe,
  porque o histórico continua alcançando os blobs.

## 2. Política por classe

Colunas: **Owner** (quem responde pela classe), **Duração**, **Armazenamento** (onde vive e como se
prova integridade), **Consulta** (como achar), **Restauração** (como voltar ao estado anterior).

| Classe | Owner | Duração | Armazenamento / hash | Consulta | Restauração |
|---|---|---|---|---|---|
| `review-raw`: dumps `.raw` (prompt+diff) de `review-gate.sh`, `docs/harness/reviews/*.raw`; 73 arquivos | harness (lane P; gerador `review-gate.sh`) | Histórico git: **sem expiração** neste programa. Cópia no working tree: até o owner aprovar o untracking. Backup externo: **mínimo 12 meses** e até autorização separada de limpeza (proposta) | Hoje: versionada em git. Após migração: ignorada + manifest versionado (`path`, `size`, `sha256`, `git_blob`, commit de origem, flags de sensibilidade) + backup endereçado por `sha256` fora do repo (`objects/<2 hex>/<sha256>`) | `jq '.entries[]' docs/harness/retention/review-raw.manifest.json`; `git log --diff-filter=A -- <path>`; commit de origem no manifest | `python3 docs/harness/bin/retention-manifest.py restore --manifest M --backup DIR` ou `--from-git` (usa o `git_blob` gravado); depois `verify` |
| `review-verdicts`: `docs/harness/reviews/*.md` (veredictos `REVIEW_VERDICT:`) | harness | **Permanente, versionada**; `doctor.sh`, `bootstrap.sh` e `check-live-state.sh` dependem deles | git; hash = id do blob git | `docs/harness/reviews/`; "Last review" em `progress.md` | `git show <rev>:<path>`; nunca untrackear |
| `progress-logs`: `progress.md`, `progress-history.md`, `progress/*.md` | harness (cada lane edita só o seu) | Permanente, versionada (regras H6 em `context-budget.md`) | git; `progress-history.md` com SHA-256 fixado por teste | índice por âncora, `measure-context.py --section` | `git show`; H6 já define rollback |
| `harness-audits` e `harness-canvas` | harness | Permanente, versionada | git | `docs/harness/audits/`, `docs/harness/canvas/` | `git show` |
| `benchmark-results`: `benches/results/*` (3 arquivos) e histórico `dev/bench*` (gh-pages, só em refs) | CI de benchmarks / quem altera `benches/` | Resultados no tree: versionados. Histórico em `dev/bench*`: **só histórico**, sem poda neste programa | git; ver medição na seção 3 | `git log --all -- dev/bench-nightly/data.js` | `git show <rev>:<path>` |
| Logs locais e saída de build ignorados: `*.log`, `logs/`, `target/`, `__pycache__/` | cada desenvolvedor / CI efêmero | Locais, **descartáveis**; não são evidência. `cargo clean` só com confirmação explícita (restrição global) | fora do git (`.gitignore`); sem backup | `git ls-files --others --ignored --exclude-standard --directory` | regenerar (build/teste); evidência durável vai para review/progress, não para log local |
| Logs operacionais e diagnósticos da observabilidade (O1) | **pending O1 implementation** (a definir por O1) | **pending O1 implementation** | **pending O1 implementation**; contrato já fixado no brief de O1: sem conteúdo proprietário por default (credencial, texto privado, query, path, erro de provider) | **pending O1 implementation** | **pending O1 implementation**. Quando O1 fechar, esta linha passa a apontar para `docs/OPERATIONS.md` e ganha duração/rotação medidas |
| Evidência do runner (H4): receipts/logs sob o runs root do chamador | harness (runner, `record-evidence.py`) | Fora do repositório, escolhido pelo chamador; regra em `GATES.md`. **Não coberta por este manifest** | diretório do chamador (`0700`), hashes no receipt validados por `validate-evidence.py` | `validate-evidence.py` | conforme `GATES.md`; não há cópia no git |
| Object store, refs, reflog e worktrees do git | quem mantém o clone; **compartilhado** entre todas as worktrees e lanes | Sem expiração aqui | `.git` comum às worktrees | `git count-objects -vH`; `git for-each-ref`; `git worktree list` | n/a (somente leitura) |

## 3. Medições (somente leitura, 2026-10-05T15:07:31Z, HEAD `f96b200`)

Geradas por `python3 docs/harness/bin/retention-manifest.py inventory` (repetir para atualizar; os
números envelhecem).

| Classe | Arquivos em HEAD | Bytes em HEAD | Blobs distintos no histórico | Bytes no histórico (sem compressão) | Bytes em disco (packed) |
|---|---:|---:|---:|---:|---:|
| `review-raw` | 73 | 4.249.732 | 57 | 4.154.850 | 739.705 |
| `review-verdicts` | 82 | 525.665 | 80 | 841.063 | 200.449 |
| `progress-logs` | 7 | 279.832 | 217 | 9.607.211 | 1.383.401 |
| `harness-audits` | 3 | 97.585 | 4 | 123.913 | 39.539 |
| `harness-canvas` | 34 | 114.907 | 35 | 118.115 | 52.415 |
| `benchmark-results` | 3 | 80.203 | 3 | 80.203 | 12.211 |
| `dev/bench*` (só histórico) | 0 | 0 | 235 | 145.088.903 | 1.633.991 |
| Total rastreado em HEAD | 1.219 | 31.366.068 | | | |

- `git count-objects -vH` do repositório compartilhado: `count: 5862` (soltos), `size: 36.06 MiB`,
  `in-pack: 15529`, `packs: 9`, `size-pack: 13.47 MiB`, `prune-packable: 4464`, `garbage: 1`
  (64 bytes, em `.git/worktrees/engram-improvement-lane-p/refs`; só registrado).
- Refs compartilhadas: 59 heads, 59 remotes, 10 tags, 76 outras; 1.640 entradas de reflog; 0 stashes;
  13 worktrees registradas (lanes P e R, `wave4-todo*`, `harness-hardening-v1`, duas temporárias).
  Qualquer limpeza destrutiva afetaria todas elas: ver seção 7.
- Ignorados em disco (esta worktree): `target/` 4.987.044 KiB (build, não é log), `__pycache__/` ~316 KiB.
  Nenhum `*.log` presente; nenhum log rastreado (único `.jsonl` rastreado é um exemplo de 870 bytes).
- `review-raw` é ~5% do pacote (`739.705` bytes de ~13,47 MiB) e ~14% dos bytes de HEAD; é o maior bloco
  de artefato gerado e versionado *no tree*, mas o histórico de benchmark (`dev/bench*`) pesa mais no
  histórico sem compressão e comprime bem (1,6 MB em disco).
- **Sensibilidade** (flags por nome; conteúdo nunca impresso): 7 dos 73 `.raw` têm algum flag
  (`sensitive: yes, path only`): 4 com `email-address` (endereço de e-mail em diff/prompt), 3 com
  `abs-home-path` (caminho absoluto do home do dev em prompts do reviewer) e 1 com
  `credential-assignment` (heurística; pode ser texto de exemplo). Nenhum com token de provedor,
  chave AWS, bloco de chave privada ou bearer. A lista fica no manifest (`sensitive_flags`). Antes de
  qualquer publicação externa do repositório, revisar esses 7 caminhos por humano.

## 4. Verificação e comandos

```bash
T=docs/harness/bin/retention-manifest.py
M=docs/harness/retention/review-raw.manifest.json
python3 $T inventory                                  # snapshot somente leitura, com timestamp
python3 $T generate --class review-raw --out $M       # recusa arquivo diferente do índice
python3 $T verify --manifest $M [--strict] [--allow-missing]   # 0 ok, 1 diferença, 4 ausente
python3 $T backup --manifest $M --dest <DIR-fora-do-repo>      # endereçado por sha256, idempotente
python3 $T verify-backup --manifest $M --backup <DIR>
python3 $T restore --manifest $M --backup <DIR>        # ou --from-git; nunca sobrescreve arquivo diferente
python3 $T untrack --manifest $M --backup <DIR>        # dry run; --apply exige backup verificado
```

Garantias: `restore` confere `sha256` e tamanho **antes** de escrever e escreve de forma atômica;
`untrack --apply` só remove do índice (`git rm --cached`), não apaga arquivo, não cria commit e não
mexe em histórico, refs nem object store; caminhos do manifest absolutos, com `..` ou via symlink são
recusados. `verify` com tudo ausente sai com 4 (nunca "passa" em silêncio).

## 5. Migração de uma classe (`review-raw`) e decisão pendente do owner

Procedimento recomendado, nesta ordem, **numa worktree/branch dedicada, com writers concorrentes
excluídos** (`review-gate.sh` grava novos `.raw` a cada execução):

1. `generate` → commit do manifest; `backup` para diretório **fora** do repo; `verify-backup`.
2. Prova em clone descartável (`git clone --no-hardlinks`): `untrack --apply`, commit, clone novo do
   resultado, `doctor.sh`, `check-doc-links.py` e `run-offline-lane.sh` sem os `.raw`, depois
   `restore` por `--backup` e por `--from-git` e `verify` (resultado na seção 6; reexecutável pelos
   testes hermético em `test_retention.py`).
3. Só então, com autorização do owner: `untrack --apply` na branch real + commit explícito de
   `.gitignore` e remoção do índice. Rollback: `git revert` desse commit e/ou `restore`.

Não feito nesta task de propósito: a regra `docs/harness/reviews/*.raw` no `.gitignore` altera o que
`review-gate.sh` deixa para commit e pertence à mesma decisão (H1/H5 estão em andamento).

## 6. Prova em clone descartável (2026-10-05)

Clones `git clone --no-hardlinks` do HEAD `f96b200` mais os arquivos desta task, em diretório temporário:

1. `verify --strict`: `VERIFY: OK files=73`; `backup`: 63 objetos distintos escritos (10 arquivos
   têm conteúdo idêntico a outro) e `verify-backup: OK ok=73`.
2. `untrack --apply` + commit no clone A: 0 `.raw` rastreados; clone novo B de A sem nenhum `.raw`.
3. Em B, sem os caches versionados: `verify` = `MISSING` ×73 e exit 4 (não passa em silêncio);
   `doctor.sh` = `OK harness doctor`; `check-doc-links.py` = `LINK_CHECK: PASS`
   (alvo pendente da lane R permitido, como em H6); `run-offline-lane.sh` = `OFFLINE_LANE: PASS
   components=9 checks=401` (a lane antes da ligação do componente `retention`).
4. `restore --backup` em B: `restored=73`, `verify` OK, `diff -r docs/harness/reviews` contra o original
   **idêntico**, `git status` limpo (os arquivos restaurados são ignorados). Segundo clone B2:
   `restore --from-git` (blobs gravados no manifest) idem.
5. Medição (clones `--no-local --single-branch`, antes x depois do untracking): arquivos rastreados
   1.219 → 1.148 (−73 `.raw`, +2 da ferramenta); bytes de checkout 31.366.068 → 27.175.476
   (−4.190.592, = −4.249.732 dos `.raw` + ferramenta e manifest). `size-pack` 6.290 → 6.178 KiB: a
   diferença de 112 KiB **não** é ganho atribuível (os blobs seguem alcançáveis pelo histórico e a
   restauração `--from-git` o prova; é variação de empacotamento) e **não** é prometida.

## 7. Proibido sem autorização separada (e sem excluir writers concorrentes)

Estas operações estão **fora** deste programa e a ferramenta não as expõe. Se um dia forem
necessárias, exigem autorização explícita do owner, backup/restore já provado, e parada de qualquer
writer concorrente (outras worktrees, lanes, CI) porque o object store e as refs são compartilhados:

- `git gc --prune=now`, `git reflog expire --expire=now --all`, `git prune` agressivo;
- reescrita de histórico (`git filter-repo`, `git filter-branch`, rebase de branch publicada,
  `git push --force`);
- apagar arquivo versionado "para economizar espaço", `git stash`/`reset`/`clean` para esconder
  mudança de outra lane, ou `cargo clean` sem confirmação.

`git count-objects` mostra `prune-packable` e objetos soltos, mas empacotar/podar é manutenção do
clone, não parte desta política. Expirar qualquer objeto só depois de backup e restauração
comprovados (seção 6) e de nova medição.

## 8. Rollback desta task

Reverter o commit remove ferramenta, manifest, testes e este documento; nenhum arquivo versionado,
ref ou objeto foi alterado. A migração da seção 6 aconteceu apenas em clones temporários.
