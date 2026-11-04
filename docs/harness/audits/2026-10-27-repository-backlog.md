# Engram — evidência do levantamento de docs e backlog

Data: 2026-10-27. Fonte local congelada para análise:
`21cf80e9148b3f0273494cd3bec73c9c525bca6f`; consulta remota reconfirmou `main`
`2e84a59422b3659aaaf25c69dbe01b01ab321332`. A árvore estava limpa ao iniciar.
Resultado: [plano de execução](../plans/2026-10-27-repository-execution-plan.md).
Este arquivo documenta descoberta; não é code review formal, post-gate ou autorização.

## Cobertura e método

- Bootstrap do harness executado; leitura das fontes canônicas da sessão, estado ativo,
  handoff, planos, known-issues e risco, com buscas dirigidas e confronto de código/testes.
- Inventário completo de **471 paths rastreados sob docs/**: 335 do harness, 35 de decisões,
  12 de RFCs, 11 de qualidade e 10 de superpowers, além dos demais grupos. Inclui scripts,
  dados e assets; não são 471 documentos de prosa lidos integralmente.
- GitHub REST `issues?state=all&per_page=100`, paginação concluída: **3 páginas, 262 objetos**,
  dos quais **88 issues (todas fechadas)** e 174 PRs históricos. Consulta de abertos separada:
  **0 issues e 0 PRs**. Inventário preserva números originais e títulos; não infere renumeração.
- Snapshot de 30 runs recentes, proteção de `main`, rulesets, settings de segurança e contagem
  de alertas; consultas somente leitura. Conector usado para repo/abertos; CLI `gh` para
  paginação e superfícies não cobertas pelo conector. Comentários selecionados de #170,
  #176 e #235–#241 foram usados para resolver discrepâncias, sem copiar links de acesso.
- Worktrees: **30 entradas registradas**. Nenhuma análise completa de sujeira/commits exclusivos
  das outras 29; o número não autoriza cleanup.
- Extraídas **68 entradas históricas minor/follow-up** do ledger. Algumas já resolvidas;
  o inventário não representa 68 defeitos atuais. A triagem integral ainda é tarefa H5.

O [inventário reproduzível](2026-10-27-repository-inventory.json) contém paths, issues,
entradas do ledger com linha e texto, settings e runs, com observação de API às 23:37Z.
Não contém credenciais, corpos brutos de issues nem conteúdo de alertas de segredos.

## Fontes materiais e classificação

| Evidência | Conclusão / disposição | Plano |
|---|---|---|
| [Readiness](../../releases/v0.24.0-readiness.md) e [auditoria Nightly](2026-10-27-nightly-release-readiness.md) | Main tem CI/Standing Checks verdes, Nightly falhou; candidato tem fixes locais e evidência parcial. Não está publicado ou integrado. | R1–R7 |
| [Mutação focada](2026-10-27-nightly-release-evidence/mutation-recheck.json) | 22 mutantes aplicáveis do recorte original: 18 caught, zero missed, quatro timeouts a 60s. Campanha completa remota usa outro escopo/cap; não declarar PASS ou UB. | R1/R6 |
| [RFC 0004](../../rfcs/0004-markdown-obsidian-portability.md), `src/bin/cli/portability.rs:35–49`, `src/portability/markdown.rs:85–91,183–193` | Default de revisão/confirm exigido pelo RFC diverge da CLI/API Rust que aplicam por padrão; MCP usa confirm=false. Falta decidir compatibilidade e preview. | C1 |
| `docs/harness/bin/merge_gate_ci.py:121–128` | GitError retorna conjunto vazio de referências dinâmicas. Confirmado por fonte; merge humano mitiga, não elimina a perda do controle. | H2 |
| `src/sync/worker.rs:405–416` | Concatena last_error a cada crash/reinício sem limite. Outros itens do mesmo ledger precisam reprodução. | S1 |
| [Triagem de minors](../known-issues/2026-10-26-deferred-review-minors.md) e [risk register](../risk-register.yaml) | Status “não publicado”/“só integração” envelheceu após #244–#262. Ledger 235 cache não reproduzido é distinto de issue #235 mídia. | H1/H5 |
| [Floors](../../quality/candidate-floors.json), [issue #240](https://github.com/aiconnai/engram/issues/240), commits `e17fb5e`/`dd3b6ae` | Aceite humano de labels/floors v2 já registrado. Política ainda os descreve como não revisados. Report-only permanece; aceitação efetiva depende do anchor/relatório do run. | H1/Q1 |
| [ADR C4](../../decisions/2026-10-05-c4-descriptor-bound-sqlite-open.md) e `src/storage/connection.rs` | ADR continua Proposed e janela TOCTOU residual. #238 shm tem retry/probe e regressões; não é o mesmo problema nem pendência confirmada de disponibilidade. | S4 |
| [RFC 0005](../../rfcs/0005-operational-context.md) e código `src/context/` | Políticas, reducers, métricas, search/bundles já existem; falta auditoria de fechamento requisito por requisito, não reimplementação integral. | P1 |
| [RFC 0007](../../rfcs/0007-dream-snapshot-review-pipeline.md) e [issue #170](https://github.com/aiconnai/engram/issues/170) | Fase 5 de integração harness/bundle ausente no status atual, embora issue esteja fechada. Candidatos exigem opt-in/rotulagem de proposta; apply explícito. | P2 |
| [Proposta Hermes](../../integrations/engram_shared_memory_architecture.md) e [threat model](../../security/THREAT_MODEL.md) | Persistência de metadata.scope_path já existe. Bearer compartilhado não prova isolamento adversarial; decidir modelo e contrato antes de uso operacional. | P3 |
| [ROADMAP](../../ROADMAP.md), [oportunidades](../../strategy/product-opportunity-roadmap.md), RFCs 0002/0003 | Histórico/propostas, não novas entregas autorizadas. HNSW já opt-in/reconciliado; default e neural compression requerem avaliação/decisão próprias. | Q3/P4 |
| [ADR cleanup](../../adr/cleanup/2026-07-08-remaining-oversized-files-inspect.md) e [RFC de tipos](../../adr/cleanup/2026-07-09-types-api-rfc.md) | Splits majoritariamente entregues, Turso aposentado e tipos extraídos. Proveniência de inspect/API pode precisar evidência; tamanho não é defeito ou licença para refazer. | H5 |

O integrador confrontou diretamente os defaults Markdown, os dois trechos de código de gate/sync,
floors aceitos, resultados de mutação, v49 de mídia/caps, wiring do servidor e retries shm.
Os demais itens são fontes de investigação delimitadas, com classificação de confiança no plano.

## Estado remoto observado

- Main: CI `40496307840` (schedule) e `40479948259` (dispatch) success;
  Standing Checks `40483891790` success; Nightly `40479914277` failure.
  CodeQL/Semgrep/Gitleaks/AgentShield de push em `2e84a594` success. Resultado verde de
  ferramenta não é prova de inexistência de defeitos.
- Proteção de main: strict=true; required contexts Format, Clippy, Documentation,
  Test (ubuntu-latest), Security Audit, Cargo Deny. Sem required_pull_request_reviews;
  enforce_admins=false, required signatures=false, conversation resolution=false;
  force pushes/deletions=false; rulesets=[]; auto-merge=false; delete_branch_on_merge=false.
- Dependabot security updates e secret scanning/push protection estão disabled.
  API de Dependabot alerts: 403 por desabilitação; secret-scanning alerts: 404 por
  desabilitação. Não reportar zero nesses dois casos. Code scanning: **0 alertas abertos**
  na consulta paginada. Nenhuma configuração alterada.

## Pendências históricas já resolvidas ou que não devem virar retrabalho

- [#235](https://github.com/aiconnai/engram/issues/235): migração v49, índice
  `(workspace,file_hash)` e upsert corrigidos. É independente de ledger 235 sobre cache.
- [#236](https://github.com/aiconnai/engram/issues/236): leitura bounded de mídia e limite
  de 50 MiB existem; eventuais gaps de edge cases são nova investigação, não cap ausente.
- [#237](https://github.com/aiconnai/engram/issues/237): hooks e SyncWorker têm wiring
  no servidor; a alegação “library-only” foi superada. Paths de testes foram reorganizados.
- [#238](https://github.com/aiconnai/engram/issues/238): mitigação de abertura shm e testes
  registrados em `src/storage/connection.rs` / `connection_tests.rs`.
- [#240](https://github.com/aiconnai/engram/issues/240): revisão humana aceita; não pedir
  novamente porque a política de qualidade contém prosa antiga.
- [#241](https://github.com/aiconnai/engram/issues/241): imagens pinadas e ledger
  `docs/security/supply-chain-pins.toml` vazio; não há prazo pendente desses três pins.
- Programa comprehensive encerrado em 14/10: as caixas antigas não são fila executável.
  Bug de IDs octais/Psych do risk-register também já está corrigido.

## Reconciliação das 88 issues, sem renumerar duplicatas

Todos os IDs abaixo estavam fechados na consulta. “Aparentemente entregue” indica
correspondência em código/docs, não certificação de todos os aceites ou execução atual.
Cada issue aparece em exatamente um grupo; pares preservam IDs das duplicações históricas.

| Grupo / quantidade | IDs originais | Disposição e próxima ação |
|---|---|---|
| I01 / 18 | #9/#138, #10/#139, #11/#135, #12/#136, #13/#137, #14/#134, #15/#141, #16/#140, #17/#142 | Entregas/propostas históricas, sem defeito atual confirmado nesta frente. #11 mudou de contrato para não gerar memórias sintéticas. #12 tem função, mas benchmark de aceite não localizado; ONNX #17 tem design implementado, porém smoke/download/benchmark manual não foram verificados. #13 foi superada pelo fluxo Dream posterior; ver I09. Conferir evidência seletiva em H5/Q, não reimplementar automaticamente. |
| I02 / 14 | #21/#143, #22/#144, #23/#145, #24/#146, #25/#147, #26/#148, #27/#149 | CLI maintenance, compact/rebuild, singleton lock e fila têm implementação em `src/bin/cli/maintenance`, `src/storage/queries/maintenance.rs`, `src/storage/lock.rs` e `src/embedding/queue`. Health base existe; health mais rico para índices externos/ANN é condicional. Gerador/parity de MCP já existem; contagem literal de README é ponto de manutenção, sem defeito demonstrado. |
| I03 / 22 | #28/#150, #29/#151, #30/#152, #31/#153, #32/#154, #33/#155, #34/#157, #35/#158, #36/#159, #37/#156, #43/#160 | Decisões/RFCs e funcionalidades correspondentes existem. REST vs MCP tem decisão registrada; chunking em caracteres foi confirmado em 14/10. **Ressalva #43/#160:** import/export implementado não significa cumprimento integral; default CLI/Rust diverge do RFC, task C1. Contratos de harness estão nos handlers próprios. |
| I04 / 10 | #72/#172, #73/#173, #74/#174, #75/#175, #76/#176 | Exemplos Claude/OpenAI/FastMCP/LangGraph existem em `examples/`; execução externa não foi certificada. [Registro de listagens](../../awesome-list-submissions.md) relata um PR aberto, uma branch com falha de criação de PR e um destino manual. Estados externos atuais não consultados; completar evidência/envio só se P4 for selecionada e autorizado. |
| I05 / 2 | #106/#177 | Advisories antigos tratados no histórico. Main atual tem audit/deny verdes; esta tarefa não refez auditoria de dependências. Monitoramento Q4 continua separado. |
| I06 / 1 | #122 | Falha histórica de schema em session_land; evidência insuficiente nesta revisão seletiva para certificar o aceite individual. Nenhum defeito atual confirmado; H5 fecha a rastreabilidade se ainda relevante. |
| I07 / 3 | #161, #162, #163 | Rate limiting, observabilidade e runbook de auth têm código/docs; execução externa Fly/staging de #163 não verificada. Ver `docs/MCP_AUTH.md` e `progress-history.md`, sem novo deploy para descobrir o estado. |
| I08 / 1 | #164 | Documentação atual distingue cosine de produção e sqlite-vec fora do hot path; drift original aparentemente resolvido. Não construir backend para satisfazer texto antigo. |
| I09 / 7 | #165, #166, #167, #168, #169, #170, #171 | RFC 0007 registra fases 1–4 e 6 implementadas. **#170 tem fase 5 ausente** no status e busca dirigida atuais: task P2 condicionada à prioridade, com opt-in/rotulagem de candidatos e apply explícito. Não presumir que issue fechada concluiu essa integração. |
| I10 / 2 | #178, #179 | Seguimentos de rate limit/observabilidade com correspondência em código/docs; nenhum defeito atual confirmado nesta inspeção seletiva. |
| I11 / 1 | #183 | Duplicação de campos de harness_status tem correção/teste correspondente em handlers de harness; sem pendência de código confirmada. |
| I12 / 7 | #235, #236, #237, #238, #239, #240, #241 | Correções com evidência atual: v49 mídia/caps, hooks/worker, retries shm, aws-sdk-s3 1.144 com BehaviorVersion v2026_01_12, aceite de floors e pins de imagem. Limites e exceções detalhados acima. Não confundir com números do ledger. |

O inventário JSON preserva título/URL/data/state_reason de cada issue; os grupos acima são
a interpretação humana/assistida desse snapshot. Especificações superpowers: ONNX e enrichment
marcados implementados; spacing-effect antigo é Superseded; lifecycle/handoff têm planos e
implementação relacionados. Nenhum checkbox histórico desses specs virou obrigação nova.

## Validação desta entrega documental

Doctor PASS, com o warning preexistente de ausência de review da task ativa; não houve
post-gate novo. Checker de links offline PASS nos quatro Markdown alterados/novos.
JSON parseável, 88 IDs distribuídos em 12 grupos sem duplicação/omissão, inventário de 471
paths e 68 entradas do ledger conferidos por código. `progress.md` continua com 149 linhas.
`git diff --check` e verificação de whitespace dos novos artefatos passaram.
Nenhum teste de produto foi rodado para este levantamento documental.

## Limites de cobertura e uso de agentes

Inventariar todos os paths/issues não equivale a auditar cada implementação e aceitação.
Foram lidos índices, estados e trechos pertinentes; nenhum claim de leitura integral dos
471 arquivos. Nem todas as discussões de issues ou os 68 registros deferred foram encerrados.
Não foram consultados projetos Linear/Huly: skill/API apropriada e associação atual não
estabelecidas; comentários de integração não comprovam backlog do outro tracker.
Specs de superpowers e contratos SDK receberam inspeção seletiva. Não foram lidos dados de
clientes, `.raw` ou segredos; não houve builds, testes de produto, writes remotos ou publicação.

| IA / via | Tarefa | Modelo solicitado → reportado | Esforço configurado → reportado | Agentes / disparos de tarefa | Entrada/saída | Cache/raciocínio |
|---|---|---|---|---|---|---|
| Nativa, sessão atual | Produto/RFCs e conferência do plano | gpt-6.1-terra → não exposto | xhigh → não exposto | 1 / 2 | Não observados | Não observados |
| Nativa, sessão atual | Governança/release e complementos | gpt-6.1-terra → não exposto | xhigh → não exposto | 1 / 2 | Não observados | Não observados |
| Nativa, sessão atual | Issues/specs/contratos; “luba” interpretado como Luna após aviso | gpt-6-luna → não exposto | max → não exposto | 1 / 1 | Não observados | Não observados |
| Integrador, sessão atual | Consultas remotas, validação seletiva, artefatos | Não exposto pela interface | Não exposto | 1 / turno atual | Não observados | Não observados |

Fonte/cobertura: respostas de `collaboration.spawn_agent` e `followup_task` desta execução.
São três delegados únicos, cinco disparos de tarefa; isso **não** é contagem de chamadas de
inferência internas. Modelos/esforços foram parâmetros enviados à ferramenta nativa;
identidade efetiva, tokens e custo não são expostos para autenticação independente.
Não registrar zero como uso desconhecido. Não houve inferência externa paga. Registry temporário:
`/private/tmp/engram-repo-discovery-20261027/registry.json`; os resultados úteis estão nestes
artefatos do repo, sem depender da preservação desse diretório temporário.
