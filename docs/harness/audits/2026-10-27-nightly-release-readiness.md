# Nightly e readiness — evidência de 2026-10-27

Base congelada: `2e84a59422b3659aaaf25c69dbe01b01ab321332`.
Mandato: [plano](../plans/2026-10-27-nightly-release-readiness.md).
Este relatório distingue evidência de execução de autorização para publicar.

## Nightly original

[Run 40479914277](https://github.com/aiconnai/engram/actions/runs/40479914277), no SHA
acima: concluído com falha. Cinco jobs passaram; Miri e Mutation Testing falharam.

- Miri: `text_util::tests::`, exit 124, 1800 s, cinco testes selecionados. Log do filtro
  termina em `find_window_matches_reference_on_seeded_random_inputs`, após os testes
  de mapeamento e Unicode adversarial passarem. Os quatro filtros de escopos passaram
  em 34–35 s cada. Não há diagnóstico de UB nesse log; o teste interrompido não concluiu.
- Mutation Testing: 142 mutantes, 106 detectados, 19 sobreviventes, 13 inviáveis e quatro
  timeouts. Baseline passou. Os quatro timeouts são operações que podem não avançar nos
  loops de limites UTF-8 ou de busca (`-= → /=`, `>= → <`, `+= → *=`, `+ → *`).
- Artefatos originais: `miri-smoke-report` (ID 12724366081) e `mutation-report`, baixados
  via `gh run download`. Cópia local efêmera: `/private/tmp/engram-nightly-40479914277/`.

## Diagnóstico dos sobreviventes

| Grupo original | Quantidade | Diagnóstico / tratamento |
|---|---:|---|
| Bearer vazio (`|| → &&`) | 1 | Lacuna de teste do erro `MalformedBearer`; comparação constante ainda recusa segredos diferentes. Casos vazios em ambos os lados adicionados; não se demonstrou bypass de autenticação |
| Display de erro vazio | 1 | Testes das quatro mensagens distintas adicionados |
| Flags, caps e contadores de extração (linhas 204–283) | 10 | Cobrir estágios separados, orçamento cheio, duplicatas e contagens exatas |
| Domínios/frases comuns | 2 | Casos positivos e negativos pelos resultados públicos da extração |
| Auto-link retorna constante ou contador incorreto | 3 | Verificar retorno e links/aliases persistidos, com criação ligada/desligada |
| Árvore: teste de Global | 1 | Ramo redundante: `parent()` já retorna None para Global. Removido ao corrigir a construção, sem exclusão de mutantes |
| Árvore: comparação do pai | 1 | Defeito confirmado: ordenar em ordem reversa e fazer pop processava pais primeiro. Corrigir ordenação e testar toda a hierarquia |

Total: 19 sobreviventes, com os nomes exatos preservados em `mutation-report/missed.txt`.

## Verificações locais

- Bootstrap: árvore limpa em `2e84a594`; doctor passa com aviso de review da sprint ausente.
- Regressão da árvore antes da correção: dois testes falharam, retornando 3 raízes em vez
  de 1, e 7 em vez de 2. Após o fix: 22 testes de escopos passaram.
- Auth: nove testes passaram; extração: 12 passaram, incluindo persistência SQLite.
- Miri: inventário completo PASS, cinco filtros / 16 testes, sem not-run ou unsupported.
  `text_util::tests::` 80 s; filtros de escopos 28/19/18/21 s. Comando:
  `MIRI_SMOKE_TOOLCHAIN=nightly CARGO_NET_OFFLINE=true bash scripts/run-miri-smoke.sh`.
  Miri local é `nightly` de 2026-04-20 (distribuição do pin `nightly-2026-04-21`),
  em aarch64 macOS, não o runner Linux original. Report em `target/miri-smoke/`.
- Reexecução focada de cargo-mutants: 22 mutantes em 8 min, **18 caught, 0 missed,
  4 timeouts**. O 19º sobrevivente original não existe após remover o ramo redundante.
  Config temporária seleciona os 23 nomes originais, mapeando só a linha da comparação
  do pai (348 → 346); 22 continuam aplicáveis. Sem modificar `.cargo/mutants.toml`.
  Baseline focado passa (48 testes dos quatro módulos; sem testes de rede). Isso não
  é um rerun completo dos 141 mutantes atuais nem substitui o Nightly no candidato final.
  Comando reproduzível e nomes seguem no artefato local abaixo.
- CI: run inicial no sandbox falhou em hard links Git temporários. O primeiro rerun
  externo pegou um erro real na atualização de progresso: a última evidência precisava
  linkar uma âncora no log ativo. Corrigido o link; os 50 testes de contexto passaram
  fora do sandbox. **Run completo final PASS, exit 0**: 2351 testes Rust passaram, dois
  skips preexistentes; lane offline 12 componentes / 536 checks, Clippy obrigatório,
  WASM, rustdoc, contratos Python e referência MCP passaram. Log completo:
  `/private/tmp/engram-ci-candidate-final.log`. Sensores versionados não foram regravados.
- Primeira tentativa de mutação: baseline abortado por oito testes WebSocket com
  `Operation not permitted` ao abrir sockets no sandbox. Nenhum mutante foi testado
  nessa tentativa. Rerun autorizado fora do sandbox, sem excluir testes.
- A campanha local usa cargo-mutants 27.1.0, profile ci, dois jobs, cap de 60 s por
  mutante; quatro filtros de testes são passados ao libtest. O job GitHub usa profile
  default, toda a lib e 300 s; não são medições idênticas. A tentativa ampla foi interrompida
  para priorizar os achados; não é evidência de campanha completa concluída.

```bash
CARGO_NET_OFFLINE=true /private/tmp/engram-nightly-tools/bin/cargo-mutants mutants \
  --config docs/harness/audits/2026-10-27-nightly-release-evidence/mutation-recheck.toml --profile ci --copy-target true \
  --timeout 60 --jobs 2 --output /private/tmp/engram-nightly-recheck \
  -- --lib -- auth::transport_principal::tests storage::scoping::tests \
  intelligence::entity_extraction::tests text_util::tests
```

Config exata e relatórios compactos com hashes dos quatro arquivos Rust estão versionados em
[evidência CI](2026-10-27-nightly-release-evidence/ci.json),
[evidência Miri](2026-10-27-nightly-release-evidence/miri.json),
[evidência de mutações](2026-10-27-nightly-release-evidence/mutation-recheck.json) e
[config de reexecução](2026-10-27-nightly-release-evidence/mutation-recheck.toml).
São evidências produzidas pelo implementador, não aprovação independente. Logs completos efêmeros: `/private/tmp/engram-nightly-recheck.toml` e
`/private/tmp/engram-nightly-recheck/mutants.out/`. Os quatro timeouts reaparecem
nos mesmos operadores/funções do Nightly original; não foram excluídos nem tratados
como PASS. O resultado do comando é não-zero por esses timeouts.

## Defeito adicional confirmado na preparação de SDK

O helper real `--write-metadata npm` falhou com StopIteration: procurava `sdk=npm`
na tabela que usa `sdk=typescript`. Uma segunda geração Python também incluía o
próprio metadata entre os hashes. Corrigidos ambos em `scripts/verify-sdk-artifacts.sh`.
Teste novo executa os dois canais duas vezes, verifica compatibilidade, SHA e lista
exata de hashes. Antes do fix: dois subcasos FAIL; depois: os 12 testes do arquivo PASS.
`bash -n` e shellcheck passam. Isso altera o gerador local, sem mudar workflow,
proteções ou publicar. O CI completo foi repetido após esse fix: exit 0, 2351 testes
Rust e lane 12/536; os 12 testes do workflow/helper foram executados separadamente.

## Release e SDKs

- `--release-plan --release-tag v0.24.0`: core 0.24.0, Python 0.7.0, npm 0.7.0;
  `--check` e os 12 testes de `test_sdk_release_workflow.py` passam.
- Extrator real de notas gera a seção 0.24.0 com batch auth, wire format, árvore de
  escopos e Turso. Notas locais em `/private/tmp/engram-core-v0.24.0-notes.md`.
- Python wheel/sdist construídos com Python 3.10 / setuptools 84.0.0 via backend
  `setuptools.build_meta`; ambos passam no `twine check`. O frontend `python3 -m build
  --no-isolation` em Python 3.14 falhou por falta de setuptools nesse ambiente; não foi
  tratado como sucesso nem exige mudar o projeto. Artefatos em `/private/tmp/engram-sdk-candidate/python/`.
- TypeScript: type-check PASS, 103 testes PASS, build e npm pack PASS; Node 22.22.3
  local (workflow usa Node 24). Testes HTTP locais precisaram rodar fora do sandbox.
  Tarball em `/private/tmp/engram-sdk-candidate/npm/`. Não houve npm publish.
- Instalação limpa dos dois SDKs via `verify-sdk-artifacts.sh`: **PASS** após o fix
  do helper. Fonte dos SDKs idêntica a `2e84a594` (`git diff HEAD -- sdks` vazio);
  metadata registra esse SHA de origem dos SDKs, não o candidato core final. Os
  artefatos de publicação devem ser refeitos e vinculados ao SHA final integrado.
  Log: `/private/tmp/engram-sdk-install-live.log`. Não executado `--live` contra servidor;
  o dry-run de release final ainda exige os contratos instalados contra o core final.
- Fixture de confiança SSH em repo temporário: seis verificações PASS (assinante
  autorizado, desconhecido, trust file ausente, tag anotada sem assinatura, tag leve,
  SHA diferente). Nenhuma chave do owner foi lida e nenhuma configuração real mudou.
- Registries: checker read-only completo PASS em `2026-10-27T18:14:43Z`; matrix timestamp
  atualizado a partir dessa observação. Tag/release 0.24.0 ausentes, última declarada
  0.21.2 presente; crates.io 0.21.1, PyPI 0.4.0, npm 0.3.0, Homebrew 0.21.2.
  Log efêmero: `/private/tmp/engram-release-channels-20261027-live.log`.
  O checker compara os alvos/versões da matriz; não é inventário de todas as versões.
- Claims públicos e advertised-contracts passam; links Markdown e diff verificados.
  Sem agendamento de monitoramento futuro.

## Preparação dos crates Rust

- `cargo package -p engram-types --allow-dirty --no-verify --offline`: PASS.
- Core isolado, inclusive com consulta online ao índice crates.io: falha porque
  `engram-types` ainda não está disponível no registry; publicar types antes do core.
- `cargo package -p engram-types -p engram-core --registry crates-io --allow-dirty
  --no-verify --offline`: PASS; arquivos `.crate` em `target/package/` (13,3 KiB / 2,6 MiB).
  O empacotamento conjunto simula a presença da dependência sem fazer upload.
- O mesmo comando com verificação ativada falha em Cargo 1.96.0 com erro interno
  `no hash listed for engram-types v0.24.0`, no `target/package/tmp-registry`.
  Não é PASS e não foi alterado o toolchain ou configuração real para contorná-lo.
- Verificação alternativa dos fontes empacotados **PASS** (16 s, profile ci, bins): extrair ambos os `.crate`
  num diretório temporário e compilar o core com `[patch.crates-io]` apontando apenas
  para o types extraído. Isso valida a compilação do conteúdo dos arquivos, mas não
  substitui a resolução real no registry nem o dry-run final de publicação.
  Comando: `cargo build --offline --profile ci --bins --manifest-path <core-extraído>/Cargo.toml
  --config <patch-temporário>.toml --target-dir target`. Log:
  `/private/tmp/engram-packaged-source-build.log`. Nenhum teste rodou em paralelo com
  essa compilação; esses binários não foram usados como evidência de CI workspace.

## Liberação e pendências

[Readiness](../../releases/v0.24.0-readiness.md) contém a recomendação e o checklist.
Sem parecer independente/receipt humano e sem confiança provisionada para verify-tag,
o pacote não está autorizado para publicação. Esta sessão não publicou nem enviou avisos.

## Atualização de fechamento local (2026-10-29)

Execução local na branch `fix/resolve-all-2026-10-29` (a partir do candidato `21cf80e9` sobre `origin/main` `2e84a594`):

- **R1 (`src/text_util.rs` — eliminação dos 4 timeouts de mutação)**: `floor_char_boundary` e
  `ceil_char_boundary` delegados aos métodos stdlib de `str` (sem laços `while` mutáveis) e
  `LowercaseIndex::find_window` reescrito com `remaining.split_first()` + teste determinístico
  `"a — İ\u{307}\u{307}"` / `"\u{307}\u{307}"`. Campanha completa do módulo
  (`cargo-mutants mutants --no-config --file src/text_util.rs --profile ci --copy-target true --timeout 60 --jobs 2 -- --lib -- text_util::tests`):
  **42/42 mutantes detectados (`42 caught, 0 missed, 0 timeouts`, exit 0)**.
- **R3 (`scripts/verify-release-tag.sh` + `docs/releases/allowed-signers`)**: verificação SSH
  fail-closed integrada nos 3 jobs de `.github/workflows/sdk-release.yml` com anti-auto-autorização
  via `origin/main:docs/releases/allowed-signers` (ou `ENGRAM_ALLOWED_SIGNERS_FILE`).
  `scripts/test_sdk_release_workflow.py` (13 testes) e `scripts/check-workflow-supply-chain.py`: **PASS**.
  `docs/releases/allowed-signers` permanece vazio (comentários apenas) aguardando chave pública SSH do owner no `main`.
- **R4 (diagnóstico Cargo 1.96 `no hash listed`)**: causa raiz identificada — executar
  `cargo package --no-verify` grava entradas sem hash de checksum em `target/package/tmp-registry`,
  envenenando execuções subsequentes com verificação no mesmo `target-dir`. Em `target-dir` limpo,
  `cargo package -p engram-types -p engram-core --registry crates-io --offline --target-dir <dir>`
  passa com verificação completa (**exit 0**).
- **C1, H1, H2, Q1, S1 e auditoria profunda de 5 unidades**: todos os itens locais implementados,
  com contratos MCP/SDK 4-way alinhados (`0` drift) e `verify-sdk-artifacts.sh --live` validado no SHA candidato final.
  Operações externas (push, PR/merge, chave SSH do owner, Nightly remoto, receipts humanos e publicação) continuam pendentes do owner.
