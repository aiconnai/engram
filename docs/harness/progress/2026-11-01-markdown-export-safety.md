# P0 — Markdown export safety, 2026-11-01

Owner autorizou seguir no chat após o plano v3: implementar/verificar localmente o P0 independente;
tag 0.24.0 aguarda resultado. Não autoriza push/PR/merge/tag/publicação, dados reais, configuração de
clientes, adoção da RFC0004a ou programa inteiro de evolução.

Base: `2e84a59422b3659aaaf25c69dbe01b01ab321332`, origin/main revalidada por fetch em 2026-11-01.
Branch/worktree: `fix/markdown-export-safety`, `.worktrees/markdown-export-safety`. Checkout principal
permanece em `646636f6`; sujeira documental atribuída ao planejamento e preservada. Bootstrap executado
em ambos antes da edição; instruções/policy do main são a base confiável.

## Contrato e execução

P0 cobre contenção de todos os destinos/links/TOCTOU, autorização/raiz, expiração, ordenação total,
erros explícitos, direção das relações e escrita atômica por arquivo. Não inclui manifest/projeção P1,
filenames estáveis novos, setup, Candle ou schema. Integrador único edita export/auth/catalog/docs;
builds seriais com cache existente, sem cargo clean. Fixtures próprias, sem dados reais ou o banco
temporário do parecer externo; SQLite/WAL segue canônico e arquivos ativos não são abertos fora do SQLite.

| Etapa | Estado | Evidência |
|---|---|---|
| PH0 mandato/base/worktree | completed | Fetch e worktree autorizados pelo mecanismo de permissões |
| Regressões RED | completed | 9/9 na base, depois 3/3 do review; primeiras falhas de fixture preservadas |
| Correção e documentação | completed | P0, compatibilidade e referência gerada; sem P1/schema/deps |
| Focados/clippy/full | completed | 78 Markdown, 23 integrações, clippy e full final PASS/515 s |
| Revisão/post-gate | partial | Dois pareceres estáticos, achados corrigidos; formal post-gate/receipt humano pendentes |

Skills mcp-implementation-security-review e code-review-rm orientam inspeção do export/dispatcher;
não é auditoria global ou revisão independente do próprio patch. Execução de implementação autorizada
não comprova isolamento de reviewer. Bloqueio anterior de créditos não foi contornado com outra conta
ou launcher; sem inferência paga externa. Rever candidato congelado antes de alegar aprovação.

Histórico: child.pid ausente no full do planejamento tem causa desconhecida; outra tentativa falhou
em binds locais no sandbox. Não excluir testes/repetir até verde; conservar primeira falha e pedir
execução fora do sandbox quando exigido pela política de permissões.


## RED/GREEN e verificações focadas

- Primeiro RED: 7 defeitos reproduzidos e 2 falhas de fixture (crossrefs.score ausente);
  `target/markdown-export-safety/red.log` preservado no checkout principal. Relato inicial de
  nove reproduções foi corrigido no chat. Fixtures corrigidas rodaram em worktree detached da mesma
  base (`markdown-export-red`), apenas dois arquivos de teste modificados: 9/9 falhas funcionais,
  `red-corrected-fixtures.log`, sem ConstraintViolation. Não apagar a primeira tentativa.
- Primeiro compile GREEN falhou no tipo variádico de openat no macOS (`green-first.log`); corrigido
  para c_uint. Segunda tentativa apontou as duas fixtures (`green-second.log`), corrigidas conforme
  acima. `green-third.log`: 13/13 PASS, incluindo fault injection de escrita/rename e corrida de symlink.
- `export-suite.log`: 75/75 PASS em paralelo (1,34 s), incluindo subprocessos limpos/limitados que
  comparam quatro árvores byte a byte, principals reais, raiz/configuração e permissões, expiração
  NULL/futuro/passado/igualdade, erros de row/metadata/relações e proteção de aliases SQLite ativos.
- Clippy com CI_REQUIRED_FEATURES/all-targets/-D warnings: PASS, 44,43 s, `clippy.log`.
- Referência MCP regenerada da fonte. Testes de integração e full gate ainda em andamento/pendentes.

Paths de logs acima são relativos ao target do checkout principal, cache serial compartilhado
explicitamente via CARGO_TARGET_DIR. Isso não os vincula automaticamente a outro SHA/build.


## Revisão estática independente e correções

Após o novo GO, reutilizada a mesma via nativa/conta e o reviewer anterior em contexto separado do
autor de código; duas tarefas lógicas (passagem + reavaliação focada), sem agentes novos, CLI/conta
alternativa ou inferência paga externa. Ambas retornaram; a recusa de créditos anterior é histórica,
não prova de disponibilidade geral. Modelo/effort solicitados herdados do agente Sol/high; resolução,
tokens/custo e esforço efetivos não observados, não zero. Limite root + 3 preservado (root + 1 usados).

- Snapshot v1 de produto/contratos: target `9036e4debf3b7f3dba44a13b9ceebc85ae6ddd9e6d5413b2816766c4c527c0eb`,
  manifest `d487efaec1bd5b29955c80622740960e4af358c40a4f2e67dcd80390c843b790`, 14 paths congelados
  fora do repo em `/private/tmp/engram-p0-review-mb_jue_s`. [Resposta crua v1](../reviews/2026-11-01-markdown-export-safety-static-v1.txt).
- Reviewer identificou ACL macOS que invalida confiança só em mode/uid e comparação textual de
  RFC3339. Integrador validou em fixtures: 3/3 RED (`review-red.log`) para ACL permissiva em 0700,
  equivalência de timezone/precisão e expiração inválida. Não houve exploração com outro usuário real.
- Corrigidos: inspeção de ACL por fd (macOS API, Linux atributos conhecidos/lista), recusa de ACLs
  e inspeção indisponível, comparação de DateTime com nanossegundos e erro explícito para data inválida.
  Outras plataformas recusam export antes de criar saída. `review-fix-export-suite.log`: 78/78 PASS,
  1,33 s; `review-fix-clippy.log`: PASS, 9,67 s. Linux nativo não foi executado neste host macOS.
- Snapshot v2: target `26db892e3081efcda47503271aa9e8770a9eecd985ba77733e58bddb9527d608`, manifest
  `215728a53af6b14fa2a205a50a4a7ec5de4a37a6b4f4e70c0c2d304f171de646`, 15 paths em
  `/private/tmp/engram-p0-review-v2-j2tizr6x`. [Resposta crua v2](../reviews/2026-11-01-markdown-export-safety-static-v2.txt):
  dois achados resolvidos na fonte, nenhum novo defeito concreto no delta; hashes/cobertura conferidos.
- Snapshots são file-manifests SHA256 de comparação base→working tree, política confiável do commit
  base e arquivos regulares lidos sem seguir links; conteúdo/modos conferidos antes/depois. Base tree
  e descritor estão nos receipts locais. Harness process-state/artifacts excluídos com razão explícita
  e revisados pelo integrador; não alegar que esses pareceres cobrem todo o branch ou aprovam merge.
- Capability dos reviewers: static_only, sem isolamento verificado para execução; não executaram
  código/testes. Não é revisão cross-model comprovada nem perfil max/post-gate/receipt. Pela skill de
  review, default solicitado/shape autorizado med (uma passagem + validação), risco recomenda max,
  achieved none/status incomplete para aprovação formal; não foi executada/creditada a forma ampliada.

Primeiro full passou em `2026-11-01T08:45:43Z`, **467 s**, sem exclusões, `sensors.log`; era o
snapshot v1 e não encerra os dois achados encontrados. Segundo full do snapshot v2 está rodando em
`sensors-final.log`, fora do sandbox com autorização, após os fixes. Não reutilizar o PASS anterior.
A tentativa dirigida de integração no sandbox falhou com 10 binds locais recusados; reexecução
autorizada fora dele passou: canonical_journey 11, permission_modes 9, portability/routing 3 (23/23).
Ambos logs (`integration-first.log`, `integration-unsandboxed.log`) preservados.

Fontes de API: SDK local macOS `sys/acl.h` e `acl_get_entry(3)` (interfaces/constantes/convenções, sem
copiar implementação/texto); [Linux listxattr(2)](https://man7.org/linux/man-pages/man2/listxattr.2.html)
e [acl(5)](https://man7.org/linux/man-pages/man5/acl.5.html), fontes primárias consultadas em 2026-11-01.
O Linux pode omitir nomes de atributos inacessíveis na listagem, por isso há probes diretos de namespaces
ACL conhecidos com erro de permissão explícito. Limite de inspeção 64 KiB e mudanças na lista falham
fechado. Adaptação em helper de produto, sem importar prompts/scripts/política externa ao harness.


## Verificação final do candidato atualizado

Full final exit 0, `2026-11-01T09:02:00Z`, **515 s**, sem exclusões, fora do sandbox com permissão.
Inclui fmt, clippy, lib/integração, binários, WASM, rustdoc, referência MCP, offline lane, policy de
título e doctor. WARN antigo de post-review da remediação continua; casos negativos de título falham
como esperado. Fonte de produto permaneceu igual aos 15 hashes revisados durante o gate.

| Log local em target/markdown-export-safety do checkout principal | SHA256 |
|---|---|
| red.log (inclui 2 falhas de fixture) | e463cecfb71973118d901c6891d9d063b4834af202ea41834e65b3e9a8b62452 |
| red-corrected-fixtures.log | 24488b139da074f35e623cdd71d52e09086124d66e64b1f685ca69cc827176a0 |
| export-suite.log | 007c13b3ffb2f287c9309c65c9f624ca4100d326deffff1652f164d489eb3d70 |
| integration-first.log (bind negado) | 247d268a979d249c3f8237fc8e8ae15a8b5dfda22524450b07a7f037de8a2429 |
| integration-unsandboxed.log | 3b9711b3466f62ab256ea50365f5d2a95c51f70f3ea78948d5436f3c12ed4cac |
| review-red.log | 48e8a21bb03c33964eeb963d29a84db2e1c6ca34ad1cd7c97670f200fca75dd6 |
| review-fix-export-suite.log | f2054cb712a3ab0f30129b9c6b2aafd7dfc7c82b04aa691333617521cc9fb455 |
| sensors.log (snapshot v1) | 8a7da35db4a0f691e6e4fce5a28467b88c49818501a0f3cb6defbe14f49cd0a7 |
| sensors-final.log (snapshot v2) | d2146363fa0fa65dc4ed2803e9a187088b9b4bb1a3aab0efac76b810b9c49ea7 |

RISK-0010 permanece open: correção local verificada não corrige automaticamente main/publicado ou
o candidato da tag. Não houve push/PR/merge/tag/publicação, mudança de config real ou dados reais.
Arquivos de planejamento do checkout principal foram preservados; worktree detached do RED mantida
para evidência (sem autorização de remoção). P1/RFC0004a/B1 não foram implementados/aceitos.

Hook de commit instalado aponta para `.githooks` do checkout principal. Seu pre-commit exige fmt e
clippy all-targets/all-features; foi inspecionado e está sendo verificado offline, sem desabilitá-lo.
A configuração ort-sys consultada reconhece CARGO_NET_OFFLINE; não foram baixados pesos de modelos.


Hook Git instalado: PASS em 27,52 s, `precommit.log`, com fmt e clippy all-targets/all-features
sem desativação de hook ou downloads de pesos. Preparado commit local `fix(mcp)` com arquivos
específicos. O commit não implica push/PR/merge/tag, receipt humano ou fechamento de risco.


Refresh do estado vivo: primeira checagem estrita recusou a descrição sem a invocação literal do CLI
(`Last live-state check does not name the checker CLI invocation`). Campo corrigido sem alterar o
checker/gates; doctor iniciado nesse mesmo refresh também recusou o campo antigo. Checagem estrita
e doctor reexecutados após a correção, antes do commit. Nenhum comportamento de produto mudou.


## Commit e handoff local

Código commitado em `0cd2d2d562bb514ac9870a1e2e8e3f657d9feaa4`, pai `2e84a59422b3659aaaf25c69dbe01b01ab321332`,
branch `fix/markdown-export-safety`. Hook instalado rodou de novo no commit e passou (0,39 s),
sem desativação. Os 15 blobs de produto/contratos no commit coincidem byte a byte com o manifest
revisado do snapshot v2; comprovante local `committed-review-match.json`. Estado vivo estrito PASS
em head `0cd2d2d5`, worktree clean antes deste fechamento documental. Último refresh registra o
commit de código como ancestral; não inventa review do snapshot completo pós-commit.

Entrega de implementação local concluída. Integração em main/candidato da tag, runtime Linux,
post-gate formal e receipt humano permanecem próximos passos próprios; RISK-0010 continua open.
Fonte de produto não mudou após o full final; esta atualização final é apenas memória canônica.
