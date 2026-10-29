# Engram — levantamento e plano de execução, 2026-10-27

Status: **plano proposto, sem iniciar as novas frentes**. Solicitado pelo owner no chat:
“levante as docs do repo, mais issues e tudo o que falta fazer e crie um plano de execucao”.
Este levantamento amplia o mapa de trabalho; não amplia o GO de implementação ou operação.
O GO anterior continua limitado a diagnóstico do Nightly, defeitos confirmados, testes e
preparação de release/aviso, com evidência e recomendação antes de publicar ou operar.

Base inspecionada: candidato local `21cf80e9148b3f0273494cd3bec73c9c525bca6f`, branch
`fix/nightly-release-readiness-2026-10-27`; `main` remoto `2e84a59422b3659aaaf25c69dbe01b01ab321332`.
Os fixes locais ainda não são entrega no `main`. Conferir refs e árvore novamente ao executar.
Fonte detalhada, inventários e limites: [levantamento](../audits/2026-10-27-repository-backlog.md).
O [log ativo](../progress/2026-10-26-remediation.md) continua canônico; este plano não troca a sprint.

## Diagnóstico e resultado recomendado

Há **zero issues abertas e zero PRs abertos**, mas isso não equivale a ausência de trabalho.
O histórico contém 88 issues fechadas, incluindo duplicações, propostas e aceites incompletos.
O maior gargalo imediato é produzir um candidato final revisável e publicável: os fixes locais
têm evidência útil, porém restam quatro timeouts de mutação, confiança de assinatura,
resolução de pacotes Cargo, contratos de SDK instalado e receipts humanos.

Em paralelo, há dívida concreta de contrato e governança: importação Markdown aplica por
padrão na CLI/API Rust em desacordo com o RFC, o merge gate perde referências dinâmicas se
Git falhar e o registro de riscos ainda trata correções integradas como pendentes.
Não reabrir a pilha de remediação nem executar os checkboxes históricos como uma nova sprint.

Prioridade abaixo é ordem de execução, não classificação formal de vulnerabilidade.
Esforço P/M/G é relativo; incerteza e disponibilidade de revisão impedem estimar datas úteis.
Responsáveis são funções propostas, ainda não designações pessoais.

| Frente | Resultado verificável | Ordem / esforço | Dependência e responsável proposto |
|---|---|---|---|
| R — candidato de release | Core 0.24.0 / SDKs 0.7.0 com evidência no SHA final e decisão de publicação separada | Primeiro; M/G | Integrador de release; owner para chave, receipts e operação |
| C — contratos seguros para consumidores | Importação com revisão explícita e contratos MCP/SDK coerentes | Triagem antes do corte; M | Responsável de contratos; compatibilidade decidida antes de escritores |
| H — verificação e governança | Gate fail-closed, riscos reconciliados, controles remotos propostos com evidência atual | H1/H5 paralelas a R/C; H2 em lote próprio; M | Responsável de harness; revisor independente de scripts |
| S — confiabilidade e privacidade | Pendências de sync, logs e minors reproduzidas e encerradas com regressões | Após triagem de risco; M/G | Responsável Rust/runtime; não tratar hipóteses como defeitos |
| Q — qualidade mensurável | Relatório de qualidade aceito no anchor correto; decisões sobre performance apoiadas em medidas | Diagnóstico paralelo; medição após estabilizar fonte; M/G | Responsável de retrieval e avaliador independente |
| P — completar produto selecionado | Fechar critérios de contexto/Dream ou registrar adiamento; proposta Hermes com contrato honesto | Depois de decidir prioridade; G | Owner de produto + contratos; sem implantação implícita |

## R — fechar a prontidão de release

| ID | Entrega / caminhos principais | Aceite e dependência |
|---|---|---|
| R1 | Diagnóstico dos quatro timeouts: `src/text_util.rs`, `.github/workflows/nightly.yml`, evidência de mutação | Reproduzir cada caso com argv/toolchain/perfil e distinguir não terminação do mutante, custo do teste e defeito nativo. Baseline termina; manter Unicode adversarial, política e caps. Não converter timeout em “caught”. Mudança de disposição/política exige decisão e revisão explícitas. |
| R2 | Revisão do candidato local: scoping, auth, extração de entidades, Miri e helper de metadados | Revisão independente do diff exato `main..candidato`; revalidar regressões, make ci e lanes específicas depois de qualquer mudança. Miri local 5 filtros PASS e replay focado 18 caught/0 missed/4 timeouts são evidências parciais, não Nightly completo. |
| R3 | `verify-tag`: `.github/workflows/sdk-release.yml`, política de confiança de release | Recomendação: preservar verificação criptográfica e provisionar confiança explícita, preferindo chave SSH dedicada; GPG isolado é alternativa se já houver chave aprovada. Owner escolhe identidade/chave pública e fonte de confiança independente do tag. Aplicar nos três pontos, testar signer autorizado/desconhecido, store ausente, unsigned/lightweight e SHA errado. Não eliminar o check. Depende dessa decisão; preparação de fixtures pode avançar. |
| R4 | Empacotamento Rust: `Cargo.toml`, `crates/engram-types`, pipeline de release | Resolver/validar erro `no hash listed` no staging Cargo 1.96.0. Archives `--no-verify` e build extraído com patch local não fecham o aceite. Validar manifests/arquivos/checksums no ambiente da release e resolução real de dependências; planejar `engram-types` disponível antes de `engram-core`. Nenhum upload como teste. |
| R5 | Contratos/artifacts SDK 0.7.0 e nota Core 0.24.0 | Refazer build, package, instalação limpa e contratos dos SDKs instalados contra servidor local do SHA final, com fixtures sintéticas. Vincular versões, metadados e hashes ao candidato final; CI usa Node 24. Os builds/instalações atuais são anteriores a essa vinculação final. |
| R6 | Evidência remota e post-gate | Depois da integração autorizada, CI completo e Nightly no SHA final, não reaproveitar verde do main antigo. Registrar Miri e campanha de mutação completos, disposição de todos os resultados e receipts humanos conforme política. Parecer de agente e PASS de CI não são receipt. Disparo/push/PR/merge dependem da decisão operacional. |
| R7 | Pacote final de decisão e aviso | Entregar readiness, changelog, matriz de canais e [migração #247](../../migrations/v0.24.0-cache-clear.md), destinatários/canal/timing propostos pelo owner. Escopo mudou de `{cleared,scope,workspace}` para `{success,entries_cleared}`; variante global e `search_cache_clear` têm contratos distintos. Reobservar registries antes de publicação e registrar resultado; acompanhamento até aproximadamente 14/11 tem responsável a nomear, sem agendamento criado. |

R1/R3/R4 podem investigar em paralelo. Integrador serializa mudanças em `CHANGELOG.md`,
workflows, manifests, matriz de canais e docs de release. R5 final espera o congelamento do
candidato; R6 espera R1–R5 e autorização operacional; R7 consolida a recomendação de GO/NO-GO.
A [readiness existente](../../releases/v0.24.0-readiness.md) contém a proposta detalhada.
R7 também revisa os claims públicos: qualificar “100% offline” no README como capacidade
local/offline, explicando opções remotas de embeddings e backup já descritas ali, sem inferir
que dados são enviados por padrão.

## C — tornar os contratos de consumo previsíveis

| ID | Entrega / limite | Aceite e decisão |
|---|---|---|
| C1 | Importação Markdown: RFC 0004, `src/bin/cli/portability.rs`, `src/portability/markdown.rs`, handler MCP e wrappers SDK | Confirmada divergência de default: MCP revisa por padrão, CLI/Rust aplicam salvo `dry_run`. Definir transição compatível e preview de mudanças suficiente para decisão, preservar version conflicts/force e exigir ação explícita para aplicar. Testar ausência de escrita por padrão e confirmação pelas interfaces reais. Antes do corte, decidir correção nesta release ou adiamento justificado e documentado; não mudar flags silenciosamente. |
| C2 | Restante da dívida de erro MCP/SDK, ledger 254 e RFC 0006 | Mapear caminhos legados `{"error":string}`, `ignored_date_filter` e backup por prefixo; reproduzir impacto e decidir contrato/versionamento. Atualizar characterization tests apenas com contrato aprovado e aviso. **É separado da migração de cache do PR #247**; não normalizar todo o wire format incidentalmente. |
| C3 | Catálogo, SDK drift e exemplos de integração | Confrontar schema gerado, dispatcher e chamadas SDK para os caminhos alterados. Exercitar exemplo em ambiente sintético quando virar entrega. Registrar limites de integrações; exemplos OpenAI/LangGraph/FastMCP existentes não provam adapter nativo nem execução externa. Listagens públicas exigem evidência própria e autorização para envio. |

Não reconstruir export/import, harness memory ou todos os SDKs: as implementações já existem.
C1 e C2 compartilham contratos públicos; aprovar interfaces antes de implementar em paralelo.

## H — eliminar falsas pendências e falhas de verificação

| ID | Entrega / caminhos | Aceite |
|---|---|---|
| H1 | Reconciliar `risk-register.yaml`, known-issues, `CLAUDE.md` e prosa de qualidade | Vincular correções ao merge `2e84a594` e testes; preservar histórico e riscos residuais. Batch auth, subscribe/gRPC e permission modes não continuam “só locais”. Ledger 235 de cache foi não reproduzido; **issue #235 de mídia é outro item**, corrigido na migração v49. Advisory verde no main atual não elimina acompanhamento de exceções. |
| H2 | `merge_gate_ci.py` e seus testes, ledger 308 | Falha de `git show`/`ls-tree` deve recusar elegibilidade com erro explícito; regressões herméticas para base inválida, GitError e referências dinâmicas protegidas. Reavaliar parser de jobs sem reescrever o gate. Review Canvas + revisão independente de script process-critical; script alterado não aprova a si próprio. |
| H3 | Proposta de controles GitHub baseada na consulta atual | Definir required PR review/admin enforcement/conversation resolution, Harness Contract, Dependabot e secret scanning segundo necessidade e plano do GitHub; justificar cada controle e impacto. Hoje required contexts são Format, Clippy, Documentation, Test Ubuntu, Security Audit, Cargo Deny; strict=true, sem required PR review, enforce_admins=false e rulesets vazios. Dependabot/secret scanning desabilitados não significam zero alertas. Owner aprova configuração concreta antes de qualquer escrita. |
| H4 | Receipts, retenção e higiene operacional | Separar receipts faltantes do sunset/remediação/candidato; não retrofabricar review. Inventariar branches/worktrees por merge, commits exclusivos, sujeira e consumidores; foram observadas 30 worktrees, sem autorização de remoção. Sete `.raw` potencialmente sensíveis no histórico exigem decisão sobre risco/reescrita; não abrir conteúdo ou reescrever histórico neste levantamento. |
| H5 | Fechar triagem do ledger histórico | Percorrer **todos** os registros minor/follow-up, relacionar correções posteriores e produzir disposição individual com evidência. Primeiro auth/TOCTOU/handler principal e testes e2e; depois evidência/gates, WAL/embedding/processos, SDKs/observabilidade. Inventário no levantamento evita esquecer itens fora da triagem de 26/10. Checkboxes antigos e contagem de linhas não autorizam refactor. |

H1 é documental e pode avançar com R. H2 altera o boundary de verificação e deve ficar em
task/branch/review próprios. H3/H4 entregam primeiro proposta e inventário; aplicar settings,
apagar refs ou reescrever histórico são operações separadas. H5 pode elevar um achado para
triagem antes da release se confirmar risco no caminho entregue.
H5 também fecha lacunas de aceites históricos: benchmark de #12/#136, smoke/modelo ONNX
de #17/#142, health externo opcional de #26/#148, rastreabilidade de #122 e evidência staging
de #163. Primeiro confirmar relevância e evidência existente; nada disso é defeito atual
confirmado ou autorização para baixar modelos, chamar provedores ou operar Fly.

## S — confiabilidade e privacidade com reprodução

| ID | Entrega | Aceite |
|---|---|---|
| S1 | Sync e subprocessos, ledger 245: `src/sync/worker.rs`, cloud/process helpers | Limitar/deduplicar `last_error` em crash loop, com teste de reinícios repetidos. Verificar guard de Pull/download_checked, estado de processo zombie e EINTR em waitid; só corrigir os demais após reprodução. Confirmar timeout/cancelamento sem tarefa/processo ou arquivo parcial remanescente. Não tratar símbolo antigo ausente como defeito atual. |
| S2 | Completar inventário de logs e métricas, RISK-0002 / ledger 295 | Cobrir sync/hooks/multimodal/watcher e classificar dados expostos; os 41 pontos tratados no #256 não fecham toda a superfície. Regressões capturam/redigem exemplos sintéticos, com mensagens estáticas úteis ao operador e denominadores corretos. Contagem de chamadas de log não é número de vulnerabilidades. |
| S3 | WAL, embeddings e criação em lote após H5 | Triar checksum endianness/streamer, lease multi-processo, retomada de fila/HNSW, partial drains e atualização de vocabulário de batch contra implementação atual. Cada defeito precisa de reprodução, assert de estado persistido e teste pelo consumidor real. Compatibilidade pública/schema só muda com migração/decisão apropriada. |
| S4 | ADR C4: decisão sobre abertura SQLite e fronteira de filesystem | Owner decide adotar/alterar/rejeitar opção proposta e premissas de diretório confiável. Janela TOCTOU residual não foi resolvida por retries shm. #238 já tem mitigação e regressões: não refazer como bug confirmado. Nova implementação depende de decisão, prova de locks/WAL/sidecars e matriz POSIX, sem experimentar em dados reais. |

## Q — qualidade e performance como evidência do candidato

| ID | Entrega | Aceite |
|---|---|---|
| Q1 | Reconciliar e consumir qualidade candidata | Floors/labels v2 já registram aceite independente de Ronaldo em 14/10 (`e17fb5e`), com anchors endurecidos em `dd3b6ae`. Corrigir prosa desatualizada; checar relatório e ancestor anchor exatos de cada run. Workflow continua report-only. Antes de propor bloqueio, revisar baseline Criterion por classe de runner e demais condições da política. Não pedir de novo a revisão já registrada nem elevar lane silenciosamente. |
| Q2 | Medir contenção e hotspots antes de otimizar | Separar leitura real de `get_memory` que atualiza access_count; avaliar custo de chmod, alocação Unicode e indexação com flags desabilitadas. Comparar qualidade/latência/memória com mesma fonte/toolchain/runner; só implementar ganho demonstrável. Não criar read pool ou reescrever extração por hipótese. |
| Q3 | Decidir HNSW/default e compressão neural | HNSW já é opt-in e tem reconciliação/compactação após #253/#259. Default exige protocolo RFC 0003: recall, latency, delete/update, rebuild e disk; baseline SQLite não é benchmark de HNSW. Compressão neural continua opcional/pesquisa segundo RFC 0002, sem nova integração obrigatória. |
| Q4 | Supply-chain recorrente | Revalidar as duas exceções de advisory antes de 31/12/2026, sem renovação automática; atualizar dependências corrigíveis. #241 pins de imagem foi resolvida e ledger vazio não constitui prazo pendente de 05/11. Novos avisos/falhas viram tarefas por evidência atual. |

## P — completar produto por resultado, após decisão de prioridade

| ID | Entrega condicionada | Corte útil e aceite |
|---|---|---|
| P1 | Auditar fechamento do MVP de contexto operacional, RFC 0005 | Matriz de sete blockers → código/teste/evidência: redação, retenção, isolamento, redutores, acesso a artefatos, freshness, métricas. Há políticas, reducers, busca/bundles e teste de raw excluído; não implementar tudo novamente. Resolver separadamente a relação de `memory_archive_tool_output` com políticas de raw. Corrigir apenas lacunas confirmadas; ainda não há evidência completa de fechamento nesta descoberta. |
| P2 | Dream: fechar fase 5 da issue fechada #170 / RFC 0007 ou registrar adiamento | Geração a partir de eventos harness/contexto; candidatos, inclusive aceitos ainda não aplicados, só aparecem em ferramentas Dream explícitas ou bundle opt-in que os identifique como propostas, com proveniência/freshness. Memórias aplicadas seguem o fluxo canônico. Decidir accepted vs applied antes do contrato. Raw e candidatos continuam fora do bundle por padrão; apply exige confirmação. Schema, gerador, lifecycle, review tools, eval e SDKs existentes não precisam ser refeitos. |
| P3 | Hermes: validar proposta de memória compartilhada antes de usá-la | Atualizar reality check: storage já persiste metadata.scope_path; search tem filtro interno, mas schema público/semântica herdada não estão fechados. Mesmo Bearer process-wide não comprova isolamento adversarial entre agentes. Owner escolhe separação cooperativa ou fronteira autenticada; esta última exige principal/enforcement/testes negativos por ID, workspace e scope omitido/forjado. Sem deploy/VPS/Tailscale/replicação nesta entrega de planejamento. |
| P4 | Adoção e oportunidades sem compromisso vigente | Validar evidência de submissões a listas e exemplos; selecionar onboarding/setup/doctor/guia de time se forem prioridade. Dashboard, novos conectores, federação multi-host, backends e compressão neural ficam como propostas explícitas; não transformar ROADMAP histórico em escopo aprovado. |

## Ordem de ataque, ownership e integração

1. **Congelar o ponto de partida.** Repetir bootstrap, refs/status e instruções; preservar
   fixes locais e evidências. Converter os IDs selecionados em tasks do harness existente.
2. **Começar juntas R1/R3/R4 e a triagem C1/H5**, com H1 documental. Decisões de chave e
   compatibilidade correm cedo; investigações independentes não precisam esperá-las.
3. **Fechar o corte da release.** Decidir a disposição da divergência confirmada C1 e dos
   achados relevantes que H5 confirmar: incluir com escopo e regressão próprios ou documentar
   a disposição do owner; não ampliar o lote
   com todos os projetos de produto. Integrador decide a ordem de commits compartilhados.
4. **Integrar, revisar e verificar R2/R5/R6**, mediante autorização para operações remotas.
   Rodar checks adequados uma vez no candidato congelado; qualquer nova mudança invalida
   evidência/receipt afetados. Um resultado parcial não substitui lane completa.
5. **Entregar R7 para decisão final.** Publicação, anúncio, settings e limpeza têm propostas
   concretas e decisões separadas. A recomendação atual permanece NO-GO para publicação.
6. **Depois**, implementar H2/S e as otimizações/promoções de Q em lotes pequenos e verificáveis;
   o diagnóstico documental de Q pode avançar antes. Selecionar P1/P2/P3/P4
   conforme objetivo de produto. Não bloquear confiabilidade por oportunidades não escolhidas.

Durante implementação, aplicar `parallel-model-orchestration` para dimensionar agentes e
atribuir paths/worktrees exclusivos. Um único integrador escreve memória canônica, changelog,
manifests/workflows compartilhados. Workers não editam essas superfícies simultaneamente.
Review de scripts de governança fica fora da autoridade do writer; não usar agentes de
levantamento como receipt humano. Não iniciar inferência externa paga por consequência do plano.

## Verificação e fechamento

Cada task executada registra baseline/head, paths, decisão aplicável, comando completo,
toolchain/features, quantidade de testes, exit e evidência. Seguir gates vigentes; rodar clippy
antes de PR e CI/sensores apropriados ao escopo. Não enfraquecer testes/cobertura/floors nem
usar exclusão histórica de ambiente para ocultar defeito. Nunca `cargo clean` sem autorização.
Mudanças de schema exigem versão e teste de upgrade; wire format exige migração coordenada.

O levantamento atual só produz documentação e consultas read-only. Sua validação é de links,
estrutura, estado Git e doctor; nenhum teste de produto foi reexecutado para este plano.
O aceite do plano é mapa rastreável e executável, não PASS do produto ou autorização operacional.

Prompt de retomada sugerido (sem precisar redescobrir o backlog):

> Leia AGENTS e execute bootstrap; leia o harness e este plano. Carregue
> `parallel-model-orchestration`. Revalide as bases e execute somente os IDs explicitamente
> selecionados pelo owner, com ownership exclusivo e integração única. Preserve os fixes
> locais de `21cf80e9`. Reproduza antes de corrigir; entregue evidência e revisão do candidato
> final. Não publique, envie avisos, altere settings, crie operações remotas ou remova
> branches/worktrees sem a decisão operacional correspondente. O plano não concede esse GO.
