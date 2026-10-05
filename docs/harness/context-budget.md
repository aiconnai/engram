# Orçamento de contexto, retenção e retomada (task H6)

> Objetivo: um agente sem memória do chat retoma o trabalho lendo pouco e chega a escopo,
> limites e última evidência, **sem perder histórico e sem reduzir autoridade obrigatória**.
> Testes: `docs/harness/tests/test_context_budget.py` (componente `context_budget` da lane
> offline). Medição: `docs/harness/bin/measure-context.py`.

## 1. Orçamentos (enforçados)

| Item | Orçamento | Onde é enforçado |
|---|---|---|
| `progress.md` (resumo vivo) | ≤150 linhas e ≤24.000 bytes | `doctor.sh` (`live_summary:budget`) e `test_context_budget.py` |
| Saída do `bootstrap.sh` | ≤50 linhas (duro em qualquer ambiente); <500 ms por mediana de 5 execuções, configurável por `HARNESS_BOOTSTRAP_MAX_SECONDS` (padrão 0,5 s local; 2,0 s quando `CI=true`, porque runners compartilhados são lentos e ruidosos) | linhas: `doctor.sh` (`bootstrap_contract:output_size`) e `test_context_budget.py`; tempo: só `test_context_budget.py` |
| Ordem de leitura obrigatória | idêntica em bootstrap, `AGENTS.md`, `CLAUDE.md` e `INVARIANTS.md` do harness | `test_context_budget.py` |
| Histórico | nunca apagado; `progress-history.md` byte a byte; SHA-256 fixado `6c3fc88f85a183a8bd5821c4046905f840c0d46945da82f750c57a442b2d8dd6` (87.559 bytes, origem `docs/harness/progress.md` em `4d341a0`) | `test_context_budget.py` (constante fixada no teste, comparada ao marcador e ao corpo; confere também com `git show 4d341a0:docs/harness/progress.md` quando o objeto existe) |
| Links e âncoras (`progress.md`, `progress-history.md`, este arquivo) | todos resolvem | `test_context_budget.py` + `check-doc-links.py` |

O conjunto de leitura obrigatória inteiro **não** tem teto duro: ele só é medido e reportado
(seção 2). Um teto duro derrubaria o doctor de tasks alheias a cada entrada de log; o que cresce
sem limite é o log do active plan, tratado na seção 4. Subir um orçamento da tabela exige review,
como os pisos da lane offline.

## 2. Medição

**Conjunto medido** (o mesmo que `AGENTS.md`/`CLAUDE.md`/`bootstrap.sh` mandam ler, nessa ordem,
com o active plan resolvido a partir de `progress.md`): `SPEC.md`, `INVARIANTS.md` (harness),
`WHAT_WE_DONT_DO.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `security/anthropic-reference-harness.md`,
`README.md` (harness), `progress.md`, active plan, `AGENTS.md`, `CLAUDE.md`, `INVARIANTS.md` (raiz),
`STANDARDS.md`, `ERRORS_AND_LESSONS.md` — 14 arquivos.

**Tokenizer identificado.** Os números abaixo usam **`cl100k_base` via `tiktoken-rs 0.5.9`**: a
mesma crate e versão de `Cargo.lock`, a mesma codificação que `src/intelligence/token_counter.rs`
(`TiktokenCounter`) usa para os orçamentos do Engram, com o vocabulário embutido na crate (sem
rede). Foi rodada por uma ferramenta descartável fora do repositório (crate com
`tiktoken-rs = "=0.5.9"`, `cargo build --offline`, `encode_with_special_tokens` por arquivo). É um
**proxy**: `cl100k_base` não é o tokenizer de nenhum modelo de contexto específico, só serve para
comparar antes/depois com a mesma régua.

**Ferramenta reproduzível no repositório** (`measure-context.py`): usa `tiktoken` (Python,
`cl100k_base`) **somente** se estiver importável e com o vocabulário já em cache local (a
ferramenta bloqueia sockets; nunca baixa nada). Caso contrário usa a aproximação rotulada
**`approx:utf8-bytes/4`** (= ⌈bytes UTF-8 / 4⌉), imprime o motivo do fallback e marca
`exact=false`. A aproximação **não** é tokenizer; neste conjunto (Markdown PT/EN) ela subestima
cerca de 6% (66.436 contra 70.842 antes). `--tokenizer tiktoken` falha em vez de aproximar em
silêncio. Bytes e linhas são exatos em qualquer modo.

### Antes e depois (commit de origem `4d341a0` contra o commit `c7a8d71` de H6)

Colunas: bytes UTF-8 (exatos), tokens `cl100k_base` (exatos, ver procedimento abaixo) e a
aproximação `approx:utf8-bytes/4` (= ⌈bytes/4⌉, reproduzível só com este repositório).

| Arquivo | Antes: bytes / tokens / approx | Depois: bytes / tokens / approx |
|---|---|---|
| `docs/harness/progress.md` | 87.559 / 23.331 / 21.890 | 9.718 / 2.987 / 2.430 |
| active plan (log da lane P) | 33.466 / 10.025 / 8.367 | 39.685 / 11.887 / 9.922 (+ D7–D12 e entrada H6) |
| `docs/harness/GATES.md` | 40.281 / 11.034 / 10.071 | 42.129 / 11.563 / 10.533 |
| `docs/harness/README.md` | 22.594 / 6.134 / 5.649 | 23.518 / 6.396 / 5.880 |
| demais 10 arquivos (inalterados) | 81.825 / 20.318 / 20.459 | idem |
| **Total (14 arquivos)** | **265.725 / 70.842 / 66.436** | **196.875 / 53.151 / 49.224** |

**Re-derivar os números.** Bytes e `approx`: `git worktree add --detach <dir> 4d341a0` (antes) e
`... c7a8d71` (depois), depois `python3 docs/harness/bin/measure-context.py --root <dir> --tokenizer approx`
em cada um (o script existe só a partir de H6: use o do commit de H6 com `--root` apontando para a árvore
antiga). Tokens exatos: crate descartável fora do repositório com `tiktoken-rs = "=0.5.9"`,
`cargo build --release --offline`, e um `main` que lê cada arquivo e imprime
`tiktoken_rs::cl100k_base()?.encode_with_special_tokens(&texto).len()`; rodar sobre os 14 arquivos de
cada árvore, na ordem da seção 2. Com `tiktoken` Python em cache local, `measure-context.py` (modo
`auto`) dá a mesma contagem exata sem a crate.

Variação: **−25,9% em bytes e −25,0% em tokens** no conjunto obrigatório. O `progress.md`
caiu **−88,9% em bytes e −87,2% em tokens** (era 33% do conjunto, agora ≈5%). O ganho líquido é
menor que o do `progress.md` porque o log do active plan, `GATES.md` e `README.md` cresceram
(+6,2 KB, +1,8 KB e +0,9 KB) com a entrada H6 e a documentação deste orçamento. O log do active
plan cresce por task: o número "depois" envelhece, rode `measure-context.py` para o valor atual.

**Repetição.** `bootstrap.sh`, 20 execuções sequenciais por árvore: antes mediana 154 ms
(mín. 148, máx. 241), 40 linhas. Medição intercalada (antes/depois alternados, máquina mais
carregada): antes 192 ms (184–203), depois 198 ms (188–207), 42 linhas (duas linhas novas: `Last
review` e o ponteiro de retomada). O bootstrap **já cumpria** a meta proposta (≤50 linhas, <500
ms) antes de H6; H6 só a transforma em contrato testado, ao custo de ≈6 ms. Não se afirma ganho
de tempo de bootstrap.

**Contexto útil e tempo de retomada.** O custo de retomar é o que se lê até chegar a escopo,
limites e última evidência. Antes: a leitura obrigatória até `progress.md` somava 51.220 tokens
(`progress.md` sozinho 23.331) e a evidência recente ficava num arquivo de 1.522 linhas. Depois:
até `progress.md` são 31.667 tokens (`progress.md` 2.987), com a seção "Retomada rápida" nas
primeiras ~40 linhas apontando para os três itens com links testados; uma seção histórica sai
inteira por `measure-context.py --section`. Não se exige percentual de economia no contrato (não
havia baseline medida antes de H6); os números acima são a baseline daqui em diante.

## 3. Retenção

- **Nada é apagado.** O `progress.md` anterior foi movido, byte a byte, para
  `progress-history.md` (cabeçalho com índice numerado das 68 seções H2 e âncoras estáveis; o
  marcador `BEGIN-VERBATIM` guarda tamanho e SHA-256, verificados por teste). Links relativos
  continuam válidos porque o arquivo mora no mesmo diretório do original.
- **Histórico por task** vive em `progress/` (um log por sprint/lane, uma entrada `###` por task,
  âncora estável derivada do título). O resumo vivo só aponta para eles.
- **Atualização do resumo**: cada task altera apenas os campos da tabela superior, a linha da
  task na tabela do programa e, se mudou, "Última evidência". Detalhe novo vai para o log.
- **Ler uma seção histórica sem cortar parágrafo**:
  `python3 docs/harness/bin/measure-context.py --section docs/harness/progress-history.md#<âncora>`
  (heading até a linha anterior ao próximo heading do mesmo nível ou superior; falha com exit 1 se
  a âncora não existir).
- **Links**: `python3 docs/harness/bin/check-doc-links.py [--allow-missing PATH] ARQUIVO.md...`
  (regras de âncora do GitHub, incluindo sufixos `-1` para duplicatas; código em fence ou inline
  é ignorado; alvo ignorado pelo git conta como quebrado porque não existe num clone limpo). O
  único alvo pendente tolerado é o log da lane R, criado na worktree da outra lane; ele aparece
  como `ALLOWED` na saída em vez de passar em silêncio.
- **Retenção de artefatos gerados (task O2)**: política por classe (owner, duração, armazenamento/hash,
  consulta, restauração), manifest com hashes e prova de restauração em clone descartável em
  [`../OPERATIONS_GIT_RETENTION.md`](../OPERATIONS_GIT_RETENTION.md). Untracking não é limpeza de histórico.
- **Rollback**: reverter o commit de H6 devolve o `progress.md` anterior (também preservado em
  `progress-history.md` até lá). Nada mais depende da divisão.
- **Pendente de propósito**: o log do active plan (≈10.000 tokens e crescendo por task) é o
  maior arquivo não-normativo do conjunto. Rotacionar entradas de tasks concluídas para um arquivo
  por task (mantendo índice e âncoras) exige mexer num arquivo que outras tasks e outra edição
  concorrente anexam; fica como follow-up em vez de arriscar conflito. O teste não impõe teto a ele.

## 4. Enforcement do live state (regra)

Problema deixado por H2: `test-check-live-state.sh` ficou hermético (usa cópias e repositórios
sintéticos), então **nada** passou a rodar o checker contra o `progress.md` real. Regra desta task:

1. `check-live-state.sh --structural` roda no `doctor.sh` sobre o `progress.md` real e é
   fail-closed para drift real: campo obrigatório ausente, active plan inexistente, review
   autoritativo ausente/sem PASS, linha de reconciliação required/advisory removida e
   **Last commit malformado** (não é um id de 7-40 dígitos hexadecimais).
2. **Não** falha por churn nem por reescrita de SHA: Last commit atrás de HEAD é aceito e o
   timestamp de `.sensors-last` não é comparado. Um id bem formado que HEAD **não alcança**
   (`ancestor_check=unreachable`) é só **WARN**, porque squash e rebase merges reescrevem SHAs
   legitimamente (reproduzido: depois de um squash merge a checagem de ancestralidade falhava o
   job required `Test (ubuntu-latest)`, que roda a lane com o `progress.md` real).
3. Em clone raso (checkout típico de CI) a ancestralidade não é provável:
   `ancestor_check=skipped-shallow` com linha `WARN ancestry not verified (shallow clone)`. O
   `doctor.sh` transforma `skipped-shallow` e `unreachable` em **warnings** do doctor
   (`live_state:structural`), nunca em "estruturalmente consistente" silencioso.
4. `--require-ancestor` (junto de `--structural`) torna o `unreachable` uma falha dura
   (`ancestor_check=not-ancestor`). Só os fixtures herméticos o usam; nem o doctor nem a lane
   obrigatória asseveram ancestralidade do `progress.md` real.
5. A forma **estrita** (`check-live-state.sh --progress docs/harness/progress.md`, sem flag:
   Last commit = HEAD/pai ou baseline aprovada, Last sensors = `.sensors-last`) continua sendo a
   que fecha uma task, exatamente como antes.
6. O mesmo `test-check-live-state.sh` (componente `live_state` da lane) cobre os dois modos,
   incluindo o `progress.md` real em modo estrutural (sem asseverar ancestralidade), squash merge, rebase/divergência e id malformado; `test_context_budget.py` roda o `doctor.sh` real em clones raso e completo e exige o warning.

## 5. Ordem de leitura e autoridade obrigatória: inalteradas

- A ordem de leitura obrigatória não mudou: bootstrap → `SPEC.md` → `INVARIANTS.md` →
  `WHAT_WE_DONT_DO.md` → `GATES.md` → `CODE_REVIEW_POLICY.md` →
  `security/anthropic-reference-harness.md` → `README.md` → `progress.md` → active plan →
  `AGENTS.md`/`CLAUDE.md` → `INVARIANTS.md` (raiz) → `STANDARDS.md` → `ERRORS_AND_LESSONS.md`.
  Mudá-la exige política aprovada; o teste compara as quatro fontes e falha se divergirem.
- Nenhuma autoridade obrigatória foi reduzida para economizar contexto: nenhum arquivo da lista
  saiu dela, nenhum texto normativo foi removido de `AGENTS.md`, `CLAUDE.md`, `INVARIANTS.md` raiz
  ou `STANDARDS.md` (D10 da auditoria E0: **não consolidados** nesta task).
- **Duplicação AGENTS/CLAUDE medida**: a lista de leitura repete-se em `AGENTS.md` (864 B, 254
  tokens), `CLAUDE.md` (771 B, 192 tokens) e na saída do bootstrap, ≈0,6% do conjunto. Decisão:
  **manter** a duplicação, porque `CLAUDE.md` é carregado automaticamente pelo Claude Code e
  `AGENTS.md` serve os demais agentes; cada um precisa ser autossuficiente. O risco real da
  duplicação (divergência de ordem) passa a ser coberto por teste, não por remoção. O ganho de
  contexto estava no `progress.md`, não aqui.

## 6. Decisão #152 — chunks por caracteres, não por tokens

**Ruling do controller (2026-10-05), pendente de confirmação do owner**: na ingestão de
documentos, os limites de chunk são em **caracteres** e permanecem distintos de tokens. Nenhum
tokenizer ou modelo foi adicionado, nem será adicionado em silêncio. Não é decisão do owner até
ele confirmar.

Fatos verificados em 2026-10-05 (HEAD `4d341a0`):

- `src/intelligence/document_ingest.rs`: `DEFAULT_CHUNK_SIZE = 1200` e `DEFAULT_OVERLAP = 200`
  "in characters"; `chunk_text` fatia por `chars()`.
- O catálogo MCP `ingest_document` descreve `chunk_size` como "Maximum characters per chunk" e
  `overlap` "in characters" (`src/mcp/tools/catalog/context.rs`).
- O contador compartilhado de tokens existe (`TiktokenCounter`, `cl100k_base`/`o200k_base`), mas
  `TokenChunker` só é referenciado em `token_counter.rs` e reexportado em
  `src/intelligence/mod.rs`: **não há chamador de produção**. Existir o tipo não é integração.

Consequências documentadas: (a) `chunk_size`/`overlap` na ingestão não limitam tokens; um chunk de
1.200 caracteres pode ter contagem de tokens muito diferente conforme idioma/script; (b) nenhum
orçamento de contexto do harness depende disso; (c) quem precisar de limite em tokens deve abrir
uma mudança própria, com seleção explícita de tokenizer, fallback identificado, contagens/metadados
e teste multilíngue de budget, em sub-PR independente do harness. **Entrada para Q7**: o contrato
de ingestão fica "chars" por esse ruling; o owner confirma ou reabre, e só Q7 pode exigir
tokenizer/modelo conhecido. Esta
decisão não é implementação e não mistura código de produto com o PR de contexto/harness.

## 7. Verificação em clone limpo

```bash
git worktree add --detach "$(mktemp -d)/h6-clean" HEAD   # clone limpo do commit
cd <esse diretório>
python3 -m unittest discover -s docs/harness/tests -p test_context_budget.py -v
bash docs/harness/bin/test-check-live-state.sh
cd - && git worktree remove --force <esse diretório>
```

Os testes só leem arquivos rastreados pelo git (e o checker rejeita alvos ignorados), então o
resultado em clone limpo deve igualar o da worktree de trabalho.
