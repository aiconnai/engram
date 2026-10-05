# Engram — Comprehensive Improvement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Autorização do owner (2026-10-05, chat, Ronaldo):** execução autorizada de todas as ondas (0–4), incluindo aceite do ADR `docs/decisions/2026-07-21-agent-harness-hardening-v1.md` para Onda 4 (sandbox/runner somente com fake writer, offline). Budget WAL aprovado: **64 GiB, configurável**. Contrato `defer_embedding=true` aprovado: **enfileirar para background** (código passa a cumprir o catálogo). Entrega: **commits locais na branch da worktree, sem push/PR**. Merge, publicação, produção e limpeza destrutiva de histórico continuam fora de escopo.

**Goal:** Tornar o Engram mais seguro, verificável, retomável e útil, reduzindo falsos verdes e regressões sem reescrever o produto ou ampliar autoridade de agentes.

**Architecture:** Evolução incremental do core Rust/SQLite, MCP e SDKs, preservando contratos. O harness existente continua canônico; seu endurecimento é uma trilha separada, com verificação mecânica, evidência externa ao writer e review independente. Segurança, qualidade de retrieval e performance têm testes e evidências próprios: nenhuma delas é inferida de um score de modelo.

**Tech Stack:** Rust, SQLite/WAL, Tokio, MCP, SDKs Python/TypeScript, scripts Bash/Python, GitHub Actions; toolchains e dependências já registrados no repositório.

**Spec:** `docs/harness/SPEC.md`, `docs/harness/INVARIANTS.md`, `docs/harness/WHAT_WE_DONT_DO.md`, `docs/harness/GATES.md`, `docs/decisions/2026-07-21-agent-harness-hardening-v1.md`, root `INVARIANTS.md`, `STANDARDS.md` e `docs/quality/retrieval-performance-policy.md`. O blueprint do operador e a auditoria fornecida são inputs, não política aceita.

**Status:** PROPOSTA PARA REVISÃO (ver nota de autorização acima, que a supersede para esta execução). Não autoriza alteração de branch protection, publicação, merge ou produção.

**Baseline inspecionada:** `1952f3b461fe7a8f8a6b5c106ac8c4f5929bae91`, em 2026-10-02. Todos os caminhos deste documento são relativos à raiz do repositório. Novos arquivos são explicitamente identificados. `count_lines.py` é sujeira preexistente e está fora do plano.

**Revisão 2026-10-04:** seis lanes GPT‑6 Luna Max, readonly, sobre a mesma baseline local. Consulta GitHub: `main=949c9634be28badea43ba2a2b5bcdf5d4c0dd358`; comparação dos objetos locais: **14 commits somente locais, zero somente remotos**. A baseline local não é prova de publicação. Inventário: zero issues abertas; amostra das 40 fechadas mais recentes, não todo o histórico. Preservar os números originais das issues: sem inferir renumeração ou conclusão pelo status Closed. Esta revisão modifica apenas a proposta, não o código nem issues remotas.

## Global Constraints

- Revalidar HEAD, estado local, instruções e aceites antes de executar qualquer tarefa; estes paths são um mapa do snapshot, não garantia de estado futuro.
- Preservar bootstrap → leitura obrigatória → pre-review → testes/sensores → post-review independente. Dois FAILs consecutivos no mesmo task exigem escalonamento humano.
- O ADR de hardening ainda diz Proposed. Código existente não prova que sua ativação foi autorizada; conciliar implementação, docs e autorização antes de promover novos gates.
- Harness-only não modifica storage, MCP, SDKs, hooks ou dependências. Abrir tarefas/branches separadas para produto, CI e política.
- Não criar um segundo `.harness/`, router, swarm ou cron de writers. Reutilizar `docs/harness/` e os checks existentes.
- Worktree isola revisões, não é sandbox. Execução autônoma exige ADR aceito, sandbox forte, egress explícito e nenhum mount de credenciais. Sem fallback para host.
- Zero produção, dados reais de clientes, publicação de pacotes ou alterações de cloud neste programa. Fixtures sintéticas e diretórios temporários caller-owned.
- Não baixar cobertura, enfraquecer assertions, retirar testes ou aceitar skips para produzir verde. Mudanças de orçamento/fixture/política exigem review independente.
- Não remover dependências ou refatorar 98 arquivos grandes só por contagem. Nenhuma remoção baseada exclusivamente em auditoria estática.
- Nunca `cargo clean` sem confirmação explícita. Preservar cache incremental; medir perfis antes de alterá-los.
- `Connection` não atravessa `.await`; nenhuma chamada externa dentro de transação. Lookups por ID verificam workspace. Replay WAL e descompressão mantêm limites.
- MCP/schema/SDK públicos exigem compatibilidade ou migração explicitamente aprovada. Schema change exige `SCHEMA_VERSION` e testes de upgrade correspondentes.
- Prefixar comandos por `rtk`; usar `rtk proxy` para logs/evidência exatos. Commits só com arquivos explicitamente listados; não `git add -A`.
- Merge humano permanece obrigatório. Configurações de modelo são parâmetros registrados por execução, nunca recomendações permanentes copiadas de anúncio.

## Review Focus

1. Um PASS antigo sobrevive a uma mudança de byte, feature, política ou PR head: H2–H5 precisam recusar esse reaproveitamento.
2. Rename, staged-only, symlink, submodule ou nome incomum escapam do escopo: H1/H4 testam os dois lados e representação NUL-safe.
3. Testes diretos passam, mas dispatcher/HTTP/SDK usa principal diferente: C1/Q4 precisam testar o caminho real e ausência de side effects.
4. Timeout/cancelamento/retry deixa dados parciais ou lock pendurado: C2/C3/C6 precisam inspecionar estado persistido e tarefas/processos remanescentes.
5. Uma limpeza ou compaction apaga evidência recuperável: H6/O2 testam migração, clone limpo e recuperação antes de qualquer descarte.

## 1. Diagnóstico: fatos, hipóteses e correções

| Sinal | Estado no snapshot | Consequência para o plano |
|---|---|---|
| Quality gate CI sem `--criterion` | Confirmado e chamada reproduzida: exit 2 antes da avaliação | Q1 primeiro; não declarar CI inteiro executado |
| Post-review sem artefato pode sair 0 | Confirmado por fonte; impacto no consumidor ainda precisa de regressão | H1, sem inventar incidente/escape |
| Diff padrão não cobre staged | Confirmado por fonte | H1, range explícito e testes do index |
| WAL commit size sem cap no replay | Fonte confirmada: `db_size_pages` alimenta `set_len` sem limite análogo ao de page_number | C2 testa e valida antes de tocar destino; exploração/impacto real não executados |
| Mandatory docs = 210.931 bytes | Medição anterior; não é contagem exata de tokens | H6 mede tokenizer e contexto carregado |
| 73 `.raw`, 4.249.732 bytes | Prompts e diffs, não transcrições | H6/O2 política de retenção antes de remover |
| 490 arquivos Rust / 98 >500 linhas | Indicador de tamanho, não prova de defeito | C5 refatora só um hotspot justificado |
| "22 unsafe" / "206 prints na biblioteca" | Auditoria original refutada | Q2 classifica sintaticamente/contextualmente |
| "170 unwrap em produção" | Não estabelecido por parser confiável | Q2 inventário, sem promessa numérica |
| Fixture security gate morta / sem modos leves | Refutados: fixture usada; modos já existem | Q5/H6 preservam wiring |
| Fuzz nightly efetivo | Não: sem targets rastreados e falhas mascaradas | Q3 |
| Remover async-trait / atualizar root elimina duplicates | Não sustentado; há uso dyn e dependências transitivas | Q6 |
| Abrir SQLite atomicamente via descriptor já resolvido | Progress registra candidatos revertidos; follow-up deferido | C4 é ADR/spike separado |
| Reduzir `.git` de 44 para 12 MB | Estimativa não demonstrada; objetos locais mutáveis | O2 não tem meta de bytes inventada |

Não transformar ausência de testes inline em ausência de cobertura. Não usar snapshots, .sensors-last ou .raw como aprovação do candidato atual.

## 2. Estratégia e priorização

Alternativas: (A) reescrita ampla — alto risco e difícil atribuir resultados; (B) apenas limpar docs/dependências — baixo risco aparente, mas deixa falsos verdes; (C) hardening incremental + regressões + medidas — recomendado.

P0 = bloqueio demonstrável do fluxo de verificação; P1 = fronteira de confiança/dados; P2 = eficiência e manutenção. Prioridade não é probabilidade de incidente. Não foi medida a taxa de falhas do Engram em produção.

| Onda | IDs | Entrega independente | Condição de saída |
|---|---|---|---|
| 0 — verdade operacional | E0, Q1 | Estado/evidência reconciliados; contrato CI reparável definido | Autorizações e baseline explícitas; parsing CI correto |
| 1 — prevenir falsos verdes | H1, H2, Q2, Q5, C1 | Gate de review, validação e segurança verificáveis | Fixtures negativas recusadas; nenhuma aprovação stale |
| 2 — regressões do produto | C2, C3, C7, Q3, Q4 | WAL, embeddings, Unicode, protocolos e SDKs protegidos | Caminhos reais cobertos; timeout/denial não mutam dados |
| 3 — eficiência mensurável | H6, Q6, Q7, C5, C6, O1, O2 | Contexto, dependências, retrieval, observabilidade e retenção | Ganhos medidos; rollback e histórico preservados |
| 4 — execução governada | H3, H4, H5, O4 | Fake writer isolado, evidence recorder e avaliador read-only | ADRs aceitos; sandbox e exact-SHA comprovados |
| Pesquisa separada | C4, O3 | SQLite nativo e avaliação probabilística | Decisão documentada, não falso PASS |

Estimativa inicial, não compromisso: ondas 0–1 em 1–2 semanas, 2–3 em 3–6 semanas, onda 4 em 2–4 semanas adicionais após aceites. Considera dois implementadores e review dedicado; instalação, CI Linux, fixtures, segurança e disponibilidade do owner podem ampliar prazo. C4 tem timebox inicial de cinco dias de investigação, não promessa de remediação.

## 3. Contrato de execução para cada PR

Cada item abaixo gera um PR independente ou sub-PRs quando houver interfaces diferentes. Owner é uma função a nomear, não uma atribuição inventada a uma pessoa.

- [ ] Confirmar escopo autorizado, base limpa em worktree próprio e canvas quando exigido; registrar os paths permitidos e o task-id.
- [ ] Escrever primeiro os testes especificados no item; executar e registrar a falha relevante (não uma falha por ambiente ausente).
- [ ] Implementar a menor mudança e executar testes focados; preservar casos negativos e aprovação anterior como histórico, não aprovação atual.
- [ ] Executar `rtk make ci`, `rtk proxy bash docs/harness/bin/sensors.sh`, e `rtk proxy git diff --check` sem exclusões antes de handoff/merge; adicionar lanes específicas descritas no item. Reexecução pós-commit é necessária para futura evidência confiável.
- [ ] Atualizar memória canônica na tarefa autorizada, review independente do mesmo candidato, commit com paths explícitos, decisão humana. Não marcar este plano Done porque uma lane quick passou.

Matriz de features: usar `scripts/ci-required-features.env` nos comandos focados que exigem a suíte required, e `scripts/ci-features.env` para a matriz ampliada. Um comando `cargo test --test X` abaixo é foco inicial, não prova de paridade. Em implementação, registrar argv completo resolvido, toolchain, exit, log e SHA. Evitar filtros que executam zero testes: confirmar contagem >0.

## 4. Tarefas

### E0 — Reconciliar estado, autorização e baseline [P1; plataforma/owner]

**Files:** ler SPEC, progress, ADR, schemas, validate-evidence.py e `scripts/ci.sh`; criar `docs/harness/audits/2026-10-02-improvement-baseline.md` no PR futuro. Alterações a SPEC/progress/ADR só em governança dedicada.
**Interfaces:** consome snapshot e docs; produz inventário `capability → implementation → policy status → evidence SHA → owner`, sem promover autorização.

- [ ] Separar capacidades implementadas de ativadas e aprovadas; localizar por histórico a introdução de schemas/validator já existentes. A presença desses arquivos não permite recriá-los nem usá-los como TCB automaticamente.
- [ ] Fixar repo/base de execução, SHA remoto, merge-base, divergência e status de revisão/publicação dos 14 commits locais. Owner decide a baseline adequada antes de execução; não publicar, descartar ou incorporar esses commits implicitamente. Congelar manifest, diff e árvore do escopo autorizado.
- [ ] Reconciliar a seção antiga de audit/deny advisory em GATES com o Security Gate agregado obrigatório transitivo. Conferir branch protection live somente por consulta read-only autorizada; não modificar a proteção.
- [ ] Executar bootstrap, doctor e, na futura baseline aprovada, full sensors. Registrar falhas reais separadas de limitações; não atualizar datas/SHAs para fabricar verde.

**Aceite:** discrepâncias têm resolução/owner; ADR permanece Proposed até aceite humano verificável. **Rollback:** revert do PR documental preservando receipts.

### Q1 — Reparar o contrato CI/local de qualidade [P0; CI]

**Files:** modificar `.github/workflows/ci.yml`, `scripts/ci.sh`, `scripts/ci-parity-check.sh`; criar `scripts/test_check_quality_ci_contract.py`; revisar `scripts/check-quality-budgets.py` e `docs/quality/retrieval-performance-policy.md`.
**Interfaces:** separar validação de baseline histórica de avaliação de resultados do candidato; input de runtime tem SHA/toolchain/features e arquivo Criterion atual, não só uma fixture de main.

**Etapas independentes:** Q1a entrega contrato CI/local e integridade histórica, sem depender de Q7; todas as dependências `Q1` neste plano significam **Q1a**. Q1b é sub-PR posterior que integra resultados atuais aceitos de Q7. Não esperar Q1b para iniciar Q7 nem chamar Q1a de performance verificada.

- [ ] Testar via subprocess que `--criterion` ausente retorna 2; arquivo ausente, unidade inválida e regressão 116% retornam não-zero; a chamada completa e as self-tests usam arquivos válidos. Testar argv do workflow e paridade required sem depender de strings legadas de Makefile.
- [ ] Corrigir a chamada com argumento obrigatório. A validação do arquivo histórico pode verificar integridade, mas não provar desempenho atual. Integrar medição atual em Q7, sem baixar floors ou fazer argumento optional.
- [ ] Integrar o mesmo contrato do checker em `scripts/ci.sh`/`make ci` e testar argv completos das lanes local e CI, não somente procurar strings em `ci-parity-check.sh`. Enquanto Q7 não fornecer resultado atual, nomear a lane como integridade de baseline histórica; nenhuma alegação de performance do candidato. Ambiente/ferramenta ausente é não-pass.
- [ ] Rodar `rtk proxy python3 -m unittest discover -s scripts -p 'test_check_quality_ci_contract.py'`; `rtk proxy python3 scripts/check-quality-budgets.py --budgets docs/quality/budgets.json --retrieval tests/fixtures/retrieval_quality/baseline.json --criterion benches/results/benchmark_baseline.txt --self-test-degraded`; paridade e CI Linux completos.

**Aceite:** erro reproduzido tem regressão; comandos local/CI concordam; uso histórico aparece como histórico. **Rollback:** revert coordenado de wiring; suspensão explícita não equivale a sucesso.

### H1 — Review gate fail-closed e diff completo [P1; harness; depende E0 e escopo aprovado]

**Files:** modificar `docs/harness/bin/review-gate.sh`; criar `docs/harness/bin/test-review-gate.sh`; atualizar GATES/POLICY em PR dedicado.
**Interfaces:** enquanto vigente, preservar marcador legado exigido; `post` zero somente com verdict válido e escopo verificado. `pre` advisory e geração de prompt não aprovam merge.

- [ ] Fixtures em repo temporário: review ausente → nonzero; prose PASS sem marcador → nonzero; FAIL → nonzero; somente staged → path incluído; rename/delete → ambos os paths; range explícito cobre todos os commits. Testar docs-only skip allowlisted sem ampliá-lo e alterações a scripts sempre exigindo reviewer.
- [ ] Tornar pending distinguível de PASS; especificar a fonte do diff (base/candidate para final, index/working tree para preparação) em vez de adivinhar último commit. Não tratar untracked relevante como invisível.
- [ ] Fixtures `test_stale_review_rejected`, `test_invalid_range_rejected`, `test_missing_commit_rejected`: reaproveitar PASS após mudar um byte → recusa; falha de `git diff`/`git show` → nonzero antes de consultar verdict. Vincular task/base/head/tree/diff e identidade de captura externa ao writer. Marcador legado sozinho só é histórico, nunca evidência atual; não converter texto de erro de diff em escopo revisado.
- [ ] Antes de H4/H5, procedência usa procedimento manual aprovado: operador humano autenticado confere task/base/head/tree/diff e origem do review, registra aceite vinculado em serviço/receipt externo aprovado fora da autoridade do writer; gate compara com valores esperados fornecidos por esse operador. Nenhum JSON ou marcador do writer se autentica sozinho. Sem receipt/identidade externa verificável → pending/nonzero, inclusive para PASS textual. Fixtures simulam trusted/untrusted receipt; H4/H5 automatizam depois esse boundary, não são pré-requisito para implementar a recusa de H1. A aprovação manual continua sujeita à política canônica, não é substituída por estes seis pareceres.
- [ ] Rodar `rtk proxy bash docs/harness/bin/test-review-gate.sh`, self-test existente e doctor. Revisor independente inspeciona fonte/fixtures e não deixa o script alterado autorizar a própria alteração.

**Aceite:** consumidores não aceitam pending como PASS; staged-only demonstrado. **Rollback:** manter guard externo recusando pending; nunca restaurar falso verde silenciosamente.

### H2 — Endurecer schemas e semântica de evidência existentes [P1; harness; depende E0/H1]

**Files:** modificar `docs/harness/bin/validate-evidence.py`, `docs/harness/bin/test-fixtures.sh`, schemas e fixtures existentes; criar `docs/harness/tests/test_validate_evidence.py`; wiring autorizado em `.github/workflows/ci.yml`, `docs/harness/bin/sensors.sh` e `docs/harness/bin/doctor.sh`. Não editar workflow compartilhado em paralelo com Q1/Q5.
**Interfaces:** `validate_file(...)` valida estrutura, não autenticidade. Versões novas não reinterpretam artefatos v1 históricos como confiança nova. Wrapper/avaliador recebe candidate SHA e policy version esperados fora do payload do writer.

- [ ] Testes: duplicate JSON keys, NaN/Infinity, bool usado como inteiro, SHA malformado/errado, unknown fields, timeout/zero check faltante, caps ausentes, traversal, glob amplo não autorizado, data calendário impossível, PASS com finding blocking. Rodar com e sem jsonschema instalado; resultados equivalentes, keyword não suportada falha fechado.
- [ ] Corrigir lacunas comprovadas sem fallback parcial permissivo. Adicionar verificação externa de expected SHA/policy, catálogo de check IDs e hashes de logs; não confundir comparação de dois campos fornecidos pelo autor com origem confiável.
- [ ] Rodar `rtk proxy python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py'`, `rtk proxy python3 docs/harness/bin/validate-evidence.py --self-test`, `rtk proxy bash docs/harness/bin/test-fixtures.sh`.
- [ ] Tornar as regressões uma lane offline obrigatória do CI/sensors, com checagem de wiring no doctor; fixtures/manual runs não bastam. Missing lane/zero testes/failure → nonzero. H3 não inicia enquanto a lane não estiver persistente e aprovada.

**Aceite:** parser estrito e semanticamente fail-closed; nenhuma fixture agent-authored vira trusted evidence. **Rollback:** versão anterior continua histórica; desabilitar novo consumidor, não downgrade automático.

### Q2 — Inventário confiável de qualidade Rust [P1; core; depende E0]

**Files:** criar `docs/quality/rust-risk-inventory.md`; revisar `src/storage/connection.rs`, `src/storage/lock.rs`, `benches/search.rs` e módulos apontados por análise contextual. Tool de classificação, se necessário, entra em PR próprio com fixtures.

- [ ] Classificar unwrap/expect/unsafe/prints por cfg, produção/teste, binário/biblioteca e path alcançável; fixtures do classificador incluem raw strings, módulos aninhados, macros e comentários. Resultado desconhecido permanece unknown.
- [ ] Auditar SAFETY dos blocos reais e priorizar apenas caminhos que podem panic por input ou silenciar erro operacional. Saída CLI/protocolo não vira tracing por substituição em massa; teardown best-effort é distinto de erro de negócio.
- [ ] Para cada fix subsequente: input que reproduz a falha, assertion de erro tipado/ausência de panic, teste focado e Clippy required. Inventário sozinho não autoriza remoção/refactor.

**Aceite:** contagens reproduzíveis com método/limites; backlog de falhas concretas, não meta artificial de zero ocorrências textuais. **Rollback:** cada patch isolado.

### Q5 — Segurança agregada, findings e supply chain [P1; segurança/CI; depende E0/Q1]

**Files:** `.github/workflows/ci.yml`, `.github/workflows/codeql.yml`, `scripts/check-security-gate.py`, `tests/fixtures/security_gate_matrix.json`, configs dos scanners e docs de segurança.

- [ ] Exercitar constituent failure, missing, cancelled e unauthorized skip → recusa; skip explicitamente permitido → neutral, não PASS genérico. Executar self-tests já existentes `--self-test-failure` e `--self-test-unrequired` com a matrix.
- [ ] Definir separadamente execução de scanner, publicação SARIF e política de findings bloqueantes. Deduplicar só após provar cobertura do aggregate e retenção SARIF. Separar job de medição read-only do job de comentário/publicação; PR não confiável não recebe credenciais write.
- [ ] Adicionar fixtures e decisão mecânica para finding high bloqueante mesmo com scanner exit0, exceção explicitamente aprovada com owner/expiry, SARIF ausente/stale/malformado e scanner não executado. Identidade/SHA do relatório vêm do supervisor confiável; findings policy exige aceite próprio, não autodeclaração do payload. Não confundir upload SARIF bem-sucedido com ausência de findings.
- [ ] Revisar imagens por digest e actions já pinadas por SHA; atualizações mantêm provenance e smoke. Workflow/matrix checker devem rejeitar remoção do required-context dependency.

**Aceite:** nenhum scan ausente conta como limpo; identidade/digest/finding policy documentados. **Rollback:** workflow e matrix revertidos juntos sem bypass.

### C1 — Autorização por workspace no caminho real [P1; core/segurança; depende E0 e escopo aprovado]

**Files:** `tests/workspace_auth_enforcement_tests.rs`, `tests/permission_modes_tests.rs`, `tests/http_transport_security.rs`, `src/mcp/handlers/`, dispatcher e principal existentes; criar `docs/security/workspace-operation-matrix.md`.

- [ ] Mapear operações read/update/delete/link/export por ID para check de workspace e permission mode. Testar ID de A acessível por A com permissão correta; A tentando acessar ID de B por dispatcher e HTTP → denial estruturado e estado de B inalterado. ID inexistente e ID estrangeiro não revelam conteúdo/existência. Incluir listas/search/link traversal e metadados reservados case-insensitive.
- [ ] Completar lacunas realmente identificadas, mantendo auth no boundary correto; não presumir que unit test do handler comprova transporte. Cobrir principal ausente conforme contrato de cada transporte. O guard atual pode negar ID-only por workspace ausente: distinguir negação conservadora de IDOR, nunca permitir por falta de workspace. Lookups mutantes validam o workspace efetivo na mesma transação da alteração.
- [ ] `test_claimed_workspace_does_not_authorize_foreign_id`: principal autorizado em A informa `{id: id_de_B, workspace: A}`; dispatcher/HTTP e variantes `memory_get`/`memory_get_public` devem recusar antes de retornar conteúdo ou registrar reinforcement. Cobrir anonymous loopback/default, principal autenticado e ID legítimo positivo. Preservar principal até o contexto/lookup real; autorizar pelo workspace persistido, nunca apenas pelo argumento. Fonte local: precheck em `http_transport/mcp_handler.rs`, `principal: None` em `src/bin/server.rs` e leitura por ID em `memory_crud/read_update_delete.rs`; cenário ainda não executado. Q2 não bloqueia a matriz/regressões de C1; somente patch ligado a achado específico de Q2 depende dele.
- [ ] Rodar `rtk cargo test --test workspace_auth_enforcement_tests`, `rtk cargo test --test permission_modes_tests`, `rtk cargo test --test http_transport_security` e journey/SDKs relevantes.

**Aceite:** matriz com teste executado por operação crítica; zero vazamento/side effect nos negativos. **Rollback:** auth regression não permite fallback permissivo; revert coordenado de contrato.

### C2 — Atomicidade, WAL, recovery e migrações [P1; storage; depende E0 e escopo aprovado]

**Files:** `src/storage/migrations/mod.rs`, `src/storage/migrations/tests.rs`, `src/storage/queries/tests/migrations.rs`, `src/sync/wal_replication.rs`, `src/snapshot/loader.rs`, `tests/wal_replication_tests.rs`, `tests/snapshot_attestation.rs`; novos testes focados apenas para lacunas. Instruções antigas citam migrations.rs: localizar o módulo atual antes de mudar versão.

- [ ] Adicionar casos de falha antes/depois de commit, writer concorrente, cancelamento, checkpoint/recovery, upgrade de predecessor e versão antiga suportada, migration reexecutada e second open. WAL page 0/MAX/MAX+1/u32::MAX é testada em validação pura/mock de I/O: não criar arquivo esparso de centenas de GB para o caso MAX. Frame inválido após frame válido deve ser recusado sem alterar destino sentinel. Snapshot excedente é recusado antes de alocação ilimitada.
- [ ] Fazer primeiro sub-PR para o tamanho declarado no commit: frame com página pequena válida e `db_size_pages=MAX+1/u32::MAX` deve ser recusado antes de `write_all/set_len`. O limite de `page_number` já existe, mas `db_size_pages` não recebe o mesmo guard em `src/sync/wal_replication.rs:912–929`. Validar também page_size suportado e tamanho do frame antes de replay. Usar fixture/mock/limite injetável seguro; não executar payload expansivo no filesystem. Trata-se de gap de validação observado, não incidente/exploit comprovado.
- [ ] Aprovar um budget máximo em bytes antes do patch; parâmetros ausentes/acima do teto aprovado bloqueiam replay. Multiplicação checked de página/commit × page_size, offset+data, total descomprimido e tamanho final obedecem esse budget. Validar todos os frames antes de abrir/alterar destino. Cap de 100 milhões de páginas com page_size65536 ainda admite cerca de 6,55 TB; não basta repetir cap em páginas. Testes usam limites pequenos injetados, inclusive fronteiras/overflow, nunca arquivos gigantes.
- [ ] Validar checksum-chain SQLite, salts, ordenação e consistência de page_size entre packs, distinguindo integridade SHA do pacote de autenticidade. Fixture altera checksum de frame e recalcula SHA externo → recusa antes de tocar destino; definir header/seed necessário no contrato de replay, sem alegar que checksum autentica emissor.
- [ ] `test_recovery_preserves_target_on_late_failure`: I/O tardio e integrity-check inválido preservam sentinel pela API de replay e por `point_in_time_recovery` (hoje copia base antes do replay). Usar staging caller-owned/commit atômico compatível com plataforma, cleanup e falha antes da substituição; source/base e target preexistente permanecem íntegros. Testar caminho sem WAL e documentar limites de atomicidade/crash durabilidade.
- [ ] Confirmar que provider lento não mantém transaction/Connection; usar mock bloqueado e segundo writer com bounded wait, sem fixar timings de microssegundos frágeis. Contagens/hashes verificam rollback e IDs não reutilizados conforme invariant.
- [ ] Executar testes WAL/snapshot e unit storage com features pertinentes em Linux/macOS. Schema muda somente se o caso provar necessidade; fazer backup/restore antes de migração destrutiva.

**Aceite:** limites existentes protegidos por regressões e recovery demonstra integridade. **Rollback:** código pode reverter; schema/data requer plano de restauração e compatibilidade, não promessa de down-migration.

### C7 — Embeddings, fila e health coerentes end-to-end [P1; storage/search; depende C2/Q4-offline]

**Files:** `src/embedding/queue/tests.rs`, `src/storage/sqlite_backend/health.rs`, `src/mcp/handlers/memory_crud/create.rs` e queries existentes; novo teste de integração somente se os módulos atuais não exercitarem a jornada completa.
**Interfaces:** manter enqueue no fluxo storage atual e embedding imediato após commit; health derivado de rows/flags/backlog existentes, não novo contador paralelo.

- [ ] Testar create deferred/immediate → worker → retry/drain/reopen com embedder determinístico. Em cada etapa conferir row de embeddings, `has_embedding`, queue, orphan count e backlog; falha após cálculo ou após enqueue não pode produzir sucesso inconsistente.
- [ ] Resolver contrato público `defer_embedding`: catálogo promete background queue, mas enqueue e embedding imediato são guardados por `!defer_embedding`. `test_deferred_create_enqueues_and_drains` exerce MCP real → job pending/health → drain → embedding/flag coerentes. Se intenção é nunca gerar embedding, mudança de contrato exige aprovação e compatibilidade, não silently redefinir defer.
- [ ] Injetar falha entre escrita de embedding, flag e completion; crash/reopen com job processing stale. Definir transação das escritas locais e retry idempotente; provider fora da transação. Recuperação automática ou manutenção explícita têm owner/runbook e health pendente visível; não chamar memória sem embedding/sem fila de saudável por ausência de backlog.
- [ ] Corrigir só discrepâncias demonstradas. Definir transição/idempotência de job por memória antes de alterar query; provider externo continua fora da transação. Missing row, row sem flag e órfãos são diagnósticos distintos.
- [ ] Rodar `rtk cargo test --lib embedding::queue::tests`, unit tests de health e protocolo create com matriz de features. Registrar casos executados e health após reopen, não só retorno do worker.

**Aceite:** caminhos de sucesso/falha convergem ou reportam pendência explícita; retry não duplica estado; API pública preservada. **Rollback:** revert de código; fila persistida exige compatibilidade com jobs já enfileirados.

### C3 — Unicode e parsers adversos [P1; search/intelligence; depende Q2]

**Files:** `tests/property_tests.rs`, `tests/aaak_compression_tests.rs`, `src/search/`, módulos de normalização/compressão identificados pelo inventário.

- [ ] Casos explícitos `ß/ẞ`, İ, combining marks, emoji/ZWJ, bidi/control, empty e payload no limite. Property strategy baseada em chars inclui categoria C; verificar nunca panic e preservação do contrato, não só validade UTF-8.
- [ ] Regressão explícita de `src/search/bm25.rs::generate_highlights` e caller BM25: query `target`, conteúdo `"x".repeat(100) + "ẞ" + "a".repeat(30) + "target"`. Hoje `find` usa texto lowercased e slices usam original; teste exige ausência de panic e snippet original correto, incluindo limites UTF-8. Observação de fonte, não execução nesta revisão.
- [ ] Corrigir apenas slices cuja origem de offset é incompatível; teste compara resultado e limites antes de qualquer split de módulo.
- [ ] Rodar suites property/compression; lane ampliada tem seed e reproducers persistidos, sem rede/modelos.

**Aceite:** regressões determinísticas e property cases efetivamente executados; strings originais não fatiadas por offsets normalizados. **Rollback:** revert patch sem descartar o reproducer.

### Q3 — Nightly efetivo: fuzz, mutation e Miri [P1; QA/CI; depende C3/Q5 e autorização de execução]

**Files:** `.github/workflows/nightly.yml`; criar `fuzz/Cargo.toml`, `fuzz/fuzz_targets/entity_extraction.rs`, `fuzz/fuzz_targets/workspace_normalization.rs`, `scripts/run-fuzz-smoke.sh`, `scripts/test_optional_lane_contracts.py`.

- [ ] Fixtures do runner: targets vazios, list failure, crash e infraestrutura indisponível → status explícito não-pass. Escolher APIs públicas reais antes de compilar targets; corpus com entradas C3.
- [ ] Instalar ferramentas apenas em imagem/ambiente aprovado e pinado; listar targets e executar 60s por target com limites de memória/walltime. Não usar `|| true`/continue-on-error para chamar execução falha de bem-sucedida.
- [ ] Corrigir schedule semanal mutants, hoje não alcançado pelo cron diário; testar daily/weekly/manual. Miri usa alvo compatível, limitações SQLite/FFI explícitas, não claims de cobertura total. Rodar unittest do contrato e fuzz bounded em sandbox.
- [ ] Inventory declarado de targets fuzz precisa ser não-vazio e corresponder aos targets listados/compilados/executados; relatório registra contagens por alvo e preserva falhas. Miri tem lista nominal compatível e contagem >0; filtro vazio/target não encontrado → não-pass. Testar mapeamento de `github.event.schedule`, não apenas existência de cron; nenhuma falha desaparece em `continue-on-error`.

**Aceite:** relatórios distinguem pass/fail/not-run/unsupported e incluem reproducer; required gates permanecem inalterados. **Rollback:** desativar lane explicitamente com owner; nunca gerar verde sintético.

### Q4 — Cobertura por risco + contratos MCP/SDK [P1; QA/SDK; depende C1/Q1]

**Files:** criar `docs/quality/test-coverage-map.md`; `tests/mcp_protocol_tests.rs`, `tests/canonical_journey.rs`, `scripts/test-canonical-journey.sh`, `sdks/python/tests/`, `sdks/typescript/src/index.test.ts`, scripts live e geração de referência MCP existentes.

- [ ] Mapear invariant/contrato → teste → features → lane; distinguir mocks, integração real, protocolos e coverage executada. Não chamar percentual de arquivos com `#[test]` de code coverage.
- [ ] Cobrir bearer inválido, workspace cross-tenant, unknown tool/invalid params, envelopes/erros normalizados, timeout, async lifecycle, snake_case/camelCase e paginação. Usar o mesmo banco isolado na jornada stdio+HTTP; após denial, ler e confirmar que não houve mutação.
- [ ] Rodar `rtk proxy bash scripts/test-canonical-journey.sh`, suites MCP, `rtk npm --prefix sdks/typescript test`, `rtk npm --prefix sdks/typescript run type-check`, `rtk proxy python3 -m pytest sdks/python/tests -q`; scripts live aprovados separadamente. Pacotes wheel/tarball instalados também precisam provar contrato antes de publicação futura.
- [ ] Separar aceite offline (mocks/serialização) de live SDK compatibility. Esta última exige autorização de build/install/start e execução de `scripts/test-python-sdk-live.sh` e `scripts/test-typescript-sdk-live.sh` com wheel/tarball instalados contra HTTP local isolado, sem provider/produção. Se não autorizada/executada, reportar live como pendente, nunca inferir das suites mock.

**Aceite:** regressão de contrato quebra suite real; schema/reference/SDKs alinhados. **Rollback:** mudança pública e clients revertidos coordenadamente, sem publicar.

### H6 — Contexto curto, retomável e retenção explícita [P2; docs/harness; depende E0/H1]

**Files:** `docs/harness/progress.md`, progress logs, AGENTS/CLAUDE, bootstrap/doctor e onboarding; criar `docs/harness/context-budget.md` e testes de routing/leitura no PR autorizado.

- [ ] Medir bytes e tokens com tokenizer identificado, repetição e arquivos realmente carregados. Testar task ativa, seção histórica, link inexistente e clone limpo; novo índice leva ao active plan sem cortar no meio de parágrafo.
- [ ] Reconciliar #152: counter compartilhado já existe; document ingestion continua por chars e `TokenChunker` não tem caller de produção. Owner decide explicitamente se o contrato exige tokenizer/model conhecido nessa ingestão. Se sim, criar sub-PR independente com seleção explícita de tokenizer, fallback identificado, counts/metadados e teste multilíngue de budget; se não, documentar chars como limite distinto de tokens. Não declarar integração completa por existir o tipo, nem adicionar modelo silenciosamente. Essa decisão é input de Q7; implementação não se mistura ao PR de contexto/harness.
- [ ] Separar live summary e histórico por seção/tarefa, com links estáveis. Reduzir duplicação AGENTS/CLAUDE sem contradizer precedência; read order só muda em política aprovada. Não reduzir mandatory authority para economizar contexto.
- [ ] Meta proposta: live summary ≤150 linhas e bootstrap ≤50 linhas/<500ms; comparar contexto útil e tempo de retomada antes/depois. Não exigir % de economia sem baseline medido. Doctor e test-check-live-state precisam passar.

**Aceite:** cenário de retomada encontra escopo/limites/última evidência sem memória do chat; histórico não perdido. **Rollback:** índice/documentação anterior preservados e links testados.

### Q6 — Dependências e tempo de build pelo grafo [P2; core/CI; depende Q2/Q5]

**Files:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `.cargo/audit.toml`, `docs/security/advisory-exceptions.toml`, `governance/exceptions.toml`, toolchain/config existentes.

- [ ] Inventariar duplicates/reverse deps em duas execuções `rtk cargo tree -d --locked --no-default-features --features <lista-resolvida>`: listas de `scripts/ci-required-features.env` e `scripts/ci-features.env`, cada argv/lista/hash anexados. Não usar default `openai` como grafo full/required. Medir compile/link incremental sem cargo clean. Mapear exceções entre manifests e exigir owner/rationale/expiry coerentes.
- [ ] Um upgrade por PR, manifest+lock juntos; verificar release notes atuais antes de selecionar versão. Não remover async-trait com dyn; não prometer que root rand/base64 elimina transitivos. Feature vazia pode ser contrato e não lixo.
- [ ] Rodar required matrix, backend-smoke/full-feature-check quando aplicável, deny/audit em ambiente permitido. Manter perfis já otimizados até comparação mensurável.

**Aceite:** risco real reduzido ou ganho demonstrado sem feature/API regression; duplicatas remanescentes explicadas. **Rollback:** manifest+lock/config juntos.

### Q7 — Retrieval e performance do candidato, não fixture stale [P2; search/QA; depende Q1/C3/Q4]

**Files:** `tests/retrieval_quality.rs`, fixtures corpus/baseline, budgets/schema/policy, `benches/search.rs`, `benches/memory_ops.rs`, `benches/mcp_dispatch.rs`, scripts de baseline/comparison.

**Interface NEW proposed:** `scripts/run-quality-candidate.py --candidate-sha <SHA> --corpus <path> --features <lista> --criterion <resultado-atual> --output <path>` seleciona argv fixo aprovado, verifica checkout/tree e produz report com candidate SHA/tree, toolchain/features, corpus SHA256/seed, métricas calculadas e hash do Criterion atual. Não recebe comando shell livre. Histórico é input comparativo separado; fonte atual do Criterion tem vínculo ao mesmo candidato/supervisor, não só um path fornecido. Resultado incompleto/stale é nonzero. Q1 integra este report somente após Q7 aceito.

- [ ] Testar runner com candidato diferente, metric ausente/NaN, corpus hash divergente, regressão e floor editado; avaliação atual tem seed, SHA e corpus/feature/provider. Histórico não substitui esse output.
- [ ] Expandir corpus sintético versionado com PT/EN, typos, negation, entidades ambíguas, duplicatas, daily/transcript filtering e isolamento workspace; revisão de relevância separada do tuning. Fixtures atuais com métricas 1.0 não provam retrieval de clientes.
- [ ] Rodar `rtk cargo test --test retrieval_quality` e benchmarks equivalentes (in-memory/disco/WAL/concorrência separados). Preservar ceiling existente 1.15, reconciliar thresholds entre lanes sem relaxamento automático. Floors novos só com review do corpus/runner; registrar warmup/hardware/percentis quando medidos.

**Aceite:** resultado do candidato reproduzível; nenhuma promessa de SLO hospedado. **Rollback:** dataset/runner/floors versionados; não mascarar regression rebaselining.

### C5 — Refatoração seletiva da persistência no handler [P2; core; depende C7/Q4]

**Files:** `src/mcp/handlers/memory_crud/create.rs`, `src/storage/queries/` e `tests/mcp_protocol_tests.rs`; inventário Q2 decide eventual split adicional, sempre em outro PR.

- [ ] Caracterizar criação com/sem embeddings, provider failure, duplicate e transaction failure. Assert resposta MCP, estado persistido e flags/index coerentes.
- [ ] Extrair escrita SQL de embedding para função storage tipada usando a conexão/transaction existente, sem fazer chamada de rede dentro dela. A criação principal já usa storage: não vender isso como correção de bypass inexistente.
- [ ] Testes before/after e benchmarks do hotspot; nenhum rename público nem split dos 98 arquivos em lote.

**Aceite:** fronteira mais clara com comportamento compatível e ausência de lock ampliado. **Rollback:** revert normal mantendo regression tests.

### C6 — Hooks, multimodal e sync com falha/cancelamento [P1/P2 conforme alcance; core; depende C2/Q4]

**Files:** `src/hooks/`, `src/storage/pending_injections.rs`, `src/intelligence/pdf_worker.rs`, `src/multimodal/`, `src/sync/`; `tests/pdf_worker.rs`, `tests/multimodal_artifact_indexing_tests.rs`, `tests/watcher_integration.rs`, suites dream/sync existentes. Abrir sub-PRs hooks, multimodal e sync: um pode ser aceito sem aprovar os outros.

- [ ] Inventariar side effects e testar cancelamento, payload excedente, subprocess crash, retry duplicado, provider lento e interromper/reiniciar sync. Hooks incluem replay/FIFO/TTL e notas excedentes; multimodal inclui create/index/read/delete/cleanup e path ownership; sync inclui wrong-key non-destructive e restart/conflict. Distinguir operação de negócio de kill/wait best-effort.
- [ ] Video: fake ffmpeg que trava/falha, spawn error e VisionProvider lento/falho após extração. `extract_keyframes` e `create_video_memory` têm timeout/kill/wait e ownership explícito de `engram_frames_*`; erros/cancelamento não deixam filho ou diretório órfão. Sucesso com paths retornados define transferência de ownership e cleanup após último consumidor; não apagar frames ainda usados.
- [ ] Corrigir gaps comprovados com limites/timeout explícitos e deduplicação definida. Descrever resultado parcial em vez de retornar sucesso silencioso; não afirmar exactly-once universal.
- [ ] Executar suites específicas com stubs offline; verificar ausência de subprocess/task órfão, retry infinito e mutação não autorizada. Testes de serviço externo são separados, opcionais e autorizados.

**Aceite:** falha pode ser reproduzida/diagnosticada e não cria estado corrupto; policy de retry conhecida. **Rollback:** feature nova desabilitada explicitamente, sem provider fallback silencioso.

### O1 — Observabilidade sem conteúdo proprietário [P2; operação; depende C1/C6/H2]

**Files:** `docs/OPERATIONS.md`, normalização de erros, tracing/metrics já existentes, stats/risk-register do harness; novos testes de redaction no módulo dono do log.

- [ ] Definir operação/correlação/resultado/duração/retry; teste com credencial sentinela, texto privado, query, filesystem path e erro de provider. Logs não podem conter esses payloads por default; CLI stdout/protocolo continuam corretos.
- [ ] Instrumentar poucos caminhos críticos: permission_denied, SQLITE_BUSY, provider timeout, recovery, gate mismatch. Agregar contagens sem ID/tenant/query como labels de alta cardinalidade.
- [ ] Reconciliar #178/#179 com tracing/counters/rate limiter existentes antes de nova instrumentação. Cobrir parse-error fora do handler, classificação MCP-error vs sucesso HTTP, correlação/method, SSE ativo e distribuição de latência conforme contrato aprovado; items excluídos são não-goals explícitos, não green implícito. 429/Retry-After têm regressão; nenhum dashboard de produção presumido.
- [ ] Exercício local dispara falha e demonstra alerta/owner/runbook/rollback, sem enviar mensagens externas. Separar disponibilidade local de SLO real de deployment.

**Aceite:** diagnósticos úteis sem vazamento; dashboards não chamam falta de amostra de zero erro. **Rollback:** desligar export extra sem perder mensagens de erro estruturadas.

### O2 — Higiene Git e retenção recuperável [P2; operação; depende H6/O1]

**Files:** `.gitignore`, `docs/harness/reviews/`, artefatos/logs e histórico benchmark; criar `docs/OPERATIONS_GIT_RETENTION.md`.

- [ ] Inventário read-only separa tracked/raw, logs, artifacts, git object store e refs compartilhadas; métricas têm timestamp. Verificar sensibilidade sem publicar conteúdo privado.
- [ ] Definir policy por classe: owner, duração, storage/hash, consulta e restauração. Migrar uma classe de artefato em clone descartável; testar links e gates sem caches versionados. Backup/restore comprovado antes de expirar qualquer objeto.
- [ ] `gitignore`/untracking não são history cleanup. `reflog expire --expire=now --all`/`gc --prune=now` ficam fora do plano executivo; qualquer limpeza destrutiva exige autorização separada e exclusão de writers concorrentes.

**Aceite:** evidência ainda localizável/restaurável; nenhum ganho de MB prometido sem medição. **Rollback:** restaurar índice/artefatos via manifest/backup sem reescrever history por padrão.

### H3 — Registry e sandbox com fake writer [P1; plataforma/segurança; depende H2 e ADRs aceitos]

**Files NEW:** `docs/harness/checks/registry.json`, `docs/harness/bin/sandbox-adapter.py`, `docs/harness/tests/fake_writer.py`, `docs/harness/tests/test_sandbox_adapter.py`. Atualizar segurança/ADR em PR próprio antes do código autoritativo.
**Interfaces proposed:** `run_isolated(manifest_path: Path, worktree_path: Path, run_dir: Path) -> RunOutcome`; outcome contém `status`, `exit_code`, `argv`, `started_at`, `finished_at`, `log_paths` e `limits_enforced`, registrados pelo supervisor. Registry seleciona argv fixo de checks humanos, nunca shell gerado. TCB e run_dir fora do writer-writable mount.

- [ ] Testar falta de runtime/imagem, egress, escrita fora do worktree, TCB/manifest readonly, HOME/SSH/askpass ausentes, timeout/process cap e subprocess sobrevivente → recusa/termination completa. Fake writer tenta adulterar evidência.
- [ ] Implementar uma imagem por digest, capabilities dropped, no-new-privileges, user não privilegiado, network disabled e limites. Credential brokerage de agente real não pertence ao MVP.
- [ ] Rodar unittest isolada e smoke sandbox com fake writer; nenhuma fallback host. Instalação/check cache e execução de código não confiável nunca compartilham secrets.

**Aceite:** boundary comprovada negativamente; fake writer apenas. **Rollback:** disable execution; static review permanece disponível.

### H4 — Runner e evidência externa pós-commit [P1; plataforma; depende H3/H1/H2]

**Files NEW:** `docs/harness/bin/run-task.py`, `check-scope.py`, `record-evidence.py`, `docs/harness/tests/test_runner.py`, `test_scope.py`, `test_evidence_integrity.py`.
**Interfaces proposed:** scope compara base/candidate paths; recorder recebe outcome do supervisor, hashes calculados por ele e trusted identity, não JSON de sucesso do writer. Expor somente fake writer até task posterior autorizada.

- [ ] Testar dirty base, wrong repo/ref, protected rename/delete, symlink escape, submodule, mode change, filenames com newline/dash, lockfile e existing-test weakening. Wall/turn/attempt/retry cap e check timeout → nonzero, logs sobreviverão.
- [ ] Criar candidate commit só depois de scope check; executar gate em checkout limpo desse commit, separado do writer; recusar tracked/untracked output que altere input relevante. Review/evidence incluem base SHA, candidate SHA, tree, política/registry, argv, ferramentas/imagem e hashes completos; writer harness/CLI version, modelo/effort solicitado e identidade reportada (ou unavailable), limites/aprovação/changed paths. Capturar log bruto trusted sem RTK filtrado como única evidência.
- [ ] Model/tool budgets só são marcados enforced quando o adapter realmente pode limitar. Defaults propostos para piloto: wall45min, attempts2, repair1, writer concurrency1; turn cap20 só com adapter observável. Se provider cost cap não é enforceable, registrar limitação e negar task que o exige.
- [ ] Rodar unittest runner/scope/integrity; mudar um byte após gate, apagar log, adulterar hash e tentativa writer PASS → tudo recusado. Cleanup nunca apaga a única ref do candidato nem worktree de outro task.

**Aceite:** evidence aceita apenas candidato imutável verificado; candidato/ref e logs preservados no failure path. **Rollback:** desabilitar adapter/runner sem apagar receipts.

### H5 — Review SHA-bound e merge-policy read-only [P1; plataforma/segurança; depende H4/Q5]

**Files:** schemas/review existentes; NEW `docs/harness/bin/merge-gate.py`, `docs/harness/tests/test_merge_gate.py`, workflow separado `agent-evidence.yml` somente após desenho TCB aceito.
**Interfaces:** consome aprovação externa, PR head atual, CI artifacts e policy version esperados; produz decisão structured `eligible/refused` com razões, nunca merge/deploy.

- [ ] Fixtures wrong task/SHA/policy, reviewer unavailable, malformed/prose PASS, PASS com blocking, log missing, later FAIL, stale base, head trocado durante avaliação e approval do candidato anterior → refused. Rechecar current head antes de qualquer ação humana posterior.
- [ ] Reviewer readonly/fresh-context recebe task/diff/files/evidence, sem transcript do writer. Migração dos consumidores ocorre conjuntamente; marcador legado continua histórico, não satisfaz a nova decisão.
- [ ] Reviewer identity/provenance vem de supervisor/serviço autorizado fora do writer payload; campo livre `reviewer` não autentica aprovação. Fixtures reviewer forjado/não autorizado, replay de approval e troca de lineage → refused. Política de lineage é validada por identidade confiável ou permanece unavailable, nunca inferida do nome textual.
- [ ] CI supervisor/policy de base protegido não usa script alterado pelo próprio PR para registrar confiança. Candidato é input não confiável, sem secrets; validar provenance do job/artifact fora desse ambiente. Examinar merge queue/synthetic merge: head e tree integrado têm evidências distintas.
- [ ] Rodar unittest do merge-gate e testes de workflow com fixtures trusted/untrusted. Workflow apenas contents-read, sem approve/merge/publish token.

**Aceite:** decisão determinística recusa incompletude; humano ainda decide merge. **Rollback:** desabilitar avaliador novo e preservar gates aceitos; evidência velha não é reetiquetada.

### C4 — Abertura SQLite descriptor-bound: investigação dedicada [P1; segurança/storage; independente após C2]

**Files:** ler progress sobre reversões, `src/storage/connection.rs`, `src/storage/lock.rs`; NEW ADR dedicado em `docs/decisions/` e sandbox spike descartável separado do core.

- [ ] Reproduzir threat model e compatibilidade com locking/pool/WAL; documentar por que pathname aliases e hardlinks rejeitados não resolvem o contrato. Não reutilizar antigo PASS como closure.
- [ ] Comparar shim nativo vs VFS auditado com matriz Linux/macOS/BSD, ABI, múltiplas conexões/processos, checkpoint e symlink/race. Timebox cinco dias; sem viable boundary, registrar risco residual e owner, não implementar workaround vulnerável.
- [ ] Submeter ADR/resultado e testes de viabilidade; implementação futura exige review especialista e plano de dados/portabilidade próprio.

**Aceite desta tarefa:** decisão de viabilidade verificável ou defer explícito; não exige declarar atomic opening resolvido. **Rollback:** descartar spike sem tocar bancos reais.

### O3 — Avaliadores probabilísticos como aconselhamento, não gates [P2; avaliação; depende E0/H2]

**Files NEW:** `docs/quality/probabilistic-evaluation-policy.md` e fixtures/eval isoladas aprovadas; não adicionar Laya/Kev/JEV ao gate default. Fontes locais só após reference intake/licença; nenhum peso/secret versionado.

- [ ] Preservar experimento anterior como diagnóstico: Laya6/8/Brier0.21588291625, Kev7/8/0.06903502125; oito casos curados, não holdout. JEV não executado por falta de credencial; alias local jev-latest não conta como independente. Proveniência local em `/tmp/engram-probability-review-2026-10-02/` precisa ser sanitizada/arquivada antes de virar referência durável.
- [ ] Criar benchmark holdout human-labeled separado do autor da policy, com fatos/refutações/unknown e tarefa/risco. Separar p(true), distribution over actions, ordinal score e confidence; não somar/mediar métricas incompatíveis nem tratar modelos correlacionados como votos independentes.
- [ ] Testar explicit abstention, permutações e duas redações predefinidas, contexto/truncamento e identificação do modelo. Laya mostrou P(defer)62.13–82.70% por ordem; policy fail-closed prevalece. Brier por classe, baseline/base rate, erros e estabilidade são reportados com N e limites; não fit/eval na mesma amostra.
- [ ] Hosted JEV só com credencial existente obtida de forma segura, autorização para conteúdo/budget e runtime oficial; ausência → unavailable. Não provisionar chave, instalar proxy ou enviar conteúdo privado para completar tabela.

**Aceite:** resultados não habilitam merge nem ficam obrigatórios/offline-flaky no CI; qualquer mudança policy tem avaliação independente. **Rollback:** retirar advisor sem afetar verifier.

### O4 — Standing checks read-only com ownership [P2; operação/CI; depende H3/H5/Q3]

**Files NEW:** `docs/harness/goals/registry.json`, `docs/harness/schemas/goal-v1.schema.json`, `docs/harness/tests/test_standing_checks.py`; workflow scheduled separado apenas após autorização de CI.
**Interfaces:** goal contém ID de check aprovado, owner, schedule enum, timeout e `on_failure=alert_only`; nenhum comando shell livre. Registry H3 fornece argv e limites máximos.

- [ ] Testar unknown check, goal com shell/comando extra, timeout acima da policy, owner ausente e concorrência do mesmo goal → recusa. Timeout/failure/missing artifact jamais vira pass; testar dispatched/manual/scheduled.
- [ ] Executar somente check aprovado em sandbox/read-only, com concurrency lock, run SHA/policy/toolchain/log e nenhuma credencial de produção. Nenhuma auto-remediação/commit/PR criada pelo scheduler.
- [ ] Rodar unittest offline com fake check. Handoff de falha para owner exige canal e autorização humana próprios; relatório local pode demonstrar alerta sem enviar mensagem real.

**Aceite:** agendamento não amplia autoridade e tem falha rastreável; scheduler não inicia writer. **Rollback:** pausar schedule e preservar receipts, sem mudar baseline do goal.

## 5. Dependências, ownership e paralelismo

Tabela completa, alinhada aos headers; cada predecessor significa aceite do entregável relevante, não só presença do arquivo. Inventários readonly podem ocorrer antes para preparar escopo; não satisfazem dependência de implementação. Aprovação de ADR/escopo e baseline de E0 são nós reais, não detalhes operacionais.

| Task | Predecessores de implementação | Gate externo adicional |
|---|---|---|
| E0 | nenhum | decisão humana sobre baseline/escopo |
| Q1 | E0 | autorização de CI |
| H1 | E0 | escopo harness aprovado |
| H2 | E0, H1 | política/schema/lane aprovados |
| Q2 | E0 | inventário não autoriza patches |
| Q5 | E0, Q1 | findings policy aprovada |
| C1 | E0 | escopo segurança aprovado; Q2 só para achado específico |
| C2 | E0 | escopo storage e budget bytes aprovados |
| C7 | C2, Q4-offline | entregável offline de protocolo/create, não live SDK; contrato defer aprovado |
| C3 | Q2 | patches bounded; reproducer BM25 pode ser preparado antes |
| Q3 | C3, Q5 | execução nightly autorizada |
| Q4 | C1, Q1 | live lane build/install/start separadamente autorizada |
| H6 | E0, H1 | alterações de read order/policy aprovadas |
| Q6 | Q2, Q5 | upgrades/deps aprovados por PR |
| Q7 | Q1, C3, Q4 | corpus/floors revisados; decisão #152 registrada |
| C5 | C7, Q4 | hotspot caracterizado |
| C6 | C2, Q4 | sub-PRs hooks/multimodal/sync independentes |
| O1 | C1, C6, H2 | nenhum envio/export externo implícito |
| O2 | H6, O1 | descarte exige autorização e restore |
| H3 | H2 | ADRs aceitos; validator lane persistente |
| H4 | H3, H1, H2 | fake writer somente |
| H5 | H4, Q5 | TCB e provenance aprovados |
| C4 | C2 | ADR/spike separado; não solução presumida |
| O3 | E0, H2 | conteúdo/provider/budget autorizados |
| O4 | H3, H5, Q3 | scheduler CI autorizado; alert-only |

Sub-DAG de entregáveis: `E0 → Q1a → Q7 → Q1b`; Q7 também depende C3/Q4 conforme tabela. Q1a entrega o contrato de integridade histórica; Q1b integra resultado atual depois. Q4-offline prova contratos de protocolo/create antes de C7; live SDK tem autorização própria e pode permanecer pendente sem bloquear C7. Testes específicos de embedding são adicionados em C7 sem reabrir toda a execução de Q4 como pré-requisito circular.

- Lane A: harness/governança (H*); lane B: CI/qualidade (Q*); lane C: core/segurança (C*); operação O* após contratos relevantes.
- Até dois writers em worktrees independentes por piloto e um reviewer readonly; depth1. H3/H4 são serializados. Não editar workflow compartilhado, Cargo.lock ou MCP reference em paralelo.
- Uma pessoa/agent integra; cherry-picks/merges seriais; full gate no tree integrado e revisão nova. Passing de cada lane não prova passing do conjunto.
- GPT‑6 Luna Max pode revisar raciocínio/ameaças e GPT‑6.1 Sol Low inventariar/mechanical docs; configuração registrada, não assumption de melhor modelo. Autor não aprova seu próprio patch; revisor de pesquisa não substitui o post-review canônico.

## 6. Medição e definição de sucesso

| Medida | Como observar | Alvo/ação |
|---|---|---|
| Falso verde | Machine PASS posteriormente rejeitado por defeito/escopo/evidência stale | Registrar cada caso; qualquer stale/policy bypass interrompe piloto |
| Gate integrity | Fixtures adversas + same candidate SHA/policy/log provenance | 100% fixtures especificadas recusadas corretamente; não estimativa de segurança total |
| Autorização | Matriz por operação/transport/permission mode | Cada operação crítica tem negativo real + ausência de mutação |
| Data integrity | Backup/restore/migration/WAL/cancellation tests | Sem perda/corrupção nos cenários de teste; limites explicados |
| Retrieval | recall@10, MRR, NDCG@10 em corpus versionado | Floors atuais preservados até review; sem transferir para SLO público |
| Performance | Candidato/base comparáveis, features/hardware/corpus | Ceiling atual1.15 aplicado onde contratado; ruído vira investigação, não rebaseline |
| Context | Tokens medidos + bytes + tempo de retomada | Live summary proposto≤150linhas; nenhuma autoridade removida |
| Eficiência | tempo até accepted change, p50/p95, attempts, créditos disponíveis | Comparar baseline; não inventar USD de subscription |
| Operação | rejeições humanas, rework7/30dias, rollback, escaped defects | Escapes high/critical interrompem a classe de automação |

Auto-merge fica desabilitado. Futuro experimento exige aprovação separada, ≥100 tarefas representativas e ≥30dias estáveis, sem escapes high/critical, testes high-signal, branch protection e rollback. Zero falhas em 100 tarefas ainda é compatível com risco relevante: não chamar 99.9% seguro. Routing automático também só depois de baseline/heldout; se não supera configuração simples em qualidade e eficiência, não adotar.

## 7. Próxima sequência recomendada

1. Revisar este plano e nomear responsáveis; escolher um escopo de onda, não autorizar tudo em bloco.
2. E0: reconciliar governança/capacidades/estado, especialmente ADR Proposed e docs de security gate divergentes.
3. Q1: PR pequeno para contrato quebrado CI/local, com medição histórica claramente distinguida.
4. H1: PR de review gate sob governança aprovada e gates anteriores; regressões de pending/staged.
5. C2: antecipar validação preflight WAL com cap de bytes, checksum e preservação do destino; dividir PRs independentes sem chamar preflight de atomicidade completa. C1 + Q2 podem inventariar em paralelo readonly; C1 não depende de inventário Rust genérico, somente de achado Q2 explicitamente relevante a um patch. Não iniciar runner real antes do sandbox.

## 8. Evidência de elaboração e limites

- Bootstrap e doctor executados nesta elaboração: exit0. A chamada CI de quality budgets sem criterion foi reproduzida: exit2 esperado, sem avaliação/build.
- Não foram executados Cargo builds/testes, sensores full, rede/provider, publicação ou produção nesta elaboração. Este arquivo não é post-gate PASS.
- Contribuições de planejamento readonly solicitadas a `plan_harness` (GPT‑6 Luna Max), `plan_product` (GPT‑6 Luna Max) e `plan_quality` (GPT‑6.1 Sol Low). A configuração solicitada não deve ser confundida com atestado independente da identidade do provider.
- Revisão independente do plano por `review_improvement_plan` (GPT‑6.1 Sol Low) encontrou o gap de WAL commit size e a ambiguidade de paralelismo C1/Q2; fonte conferida e plano ajustado. Parecer de planejamento não é post-gate PASS.
- A auditoria original é input secundário com erros documentados; paths/contratos locais são a referência para execução. Fonte externa não confere autoridade ao harness.
- Reference intake: blueprint e auditoria fornecidos pelo operador nesta conversa, 2026-10-02; licença não estabelecida, sem copiar texto/pipeline externa. Adaptação local: trust boundaries, prioridades e casos negativos; excluídos model IDs/prices como policy, scripts shell gerados, self-approval, produção e auto-merge. O3 requer intake específico de implementações Laya/Kev/JEV antes de adoção.
- O plano fica em `docs/harness/plans/` como proposta docs-only permitida por WHAT_WE_DONT_DO. Não altera SPEC/progress/ADR ativos. Aprovação humana e gates do PR futuro permanecem pendentes.

### Revisão adicional de 2026-10-04

- Seis revisores solicitados com modelo `gpt-6-luna`, effort `max`: `oct04_plan`, `oct04_harness`, `oct04_storage`, `oct04_contracts`, `oct04_ci`, `oct04_issues`; ownership separado, readonly, sem recursão. Identidade real do provider e consumo/custo não atestados pelo supervisor; não estimar créditos/tokens.
- Uma rodada de revisão integrada por `oct04_plan` conferiu hash, 25 tarefas e DAG acíclico; suas correções sobre Q1a/Q1b, procedência manual de H1 e wiring/escopo de H2 foram incorporadas pelo integrador. Sem nova rodada de agentes nem alegação de aprovação canônica.
- Achados confrontados com fonte local e incorporados nas tarefas existentes; 25 tarefas, sem novo swarm/router. Principal integra sozinho a proposta. Pareceres não autorizam implementação nem são aprovação canônica de PR.
- Bootstrap dos revisores e doctor local passaram; sem Cargo builds/testes/full sensors/scanners atuais. Fonte sustenta hipóteses de regressão, não exploração comprovada ou runtime/published readiness.
- Consulta read-only GitHub pelo integrador: zero issues abertas e amostra de 40 fechadas. Nenhuma issue criada, reaberta, fechada ou editada. Ledger e snapshots temporários: `/tmp/engram-plan-review-2026-10-04/`; armazenamento temporário não é audit store durável nem trusted evidence. E0 arquiva receipts sanitizados no PR autorizado.

## 9. Issues: reconciliação, não reimplementação

Status abaixo é observação do código local da baseline, não aceite integral da issue nem prova no `main` remoto. IDs antigos do audit de maio e IDs recentes são distintos; correspondências são apenas temáticas. A amostra não explica fechamento de issues fora dela.

| Issue(s) | Evidência/disposição local | Task |
|---|---|---|
| #143; tema antigo #21 | CLI status presente; não recriar feature por status antigo partial | E0/baseline |
| #147/#148; temas #25/#26 | Hygiene/health e testes presentes; defer, writes parciais e reopen ainda precisam regressão end-to-end | C7 |
| #149; tema #27 | Gerador MCP/reference presentes; preservar drift check | Q4 |
| #150/#151; temas #28/#29 | Decisão MCP-only e RFC search presentes; Cloud REST não equivale ao transporte local | E0/Q4/Q7 |
| #152; tema #30 | Counter/TokenChunker existem; ingest por chars, sem configuração tokenizer/model. Decisão de integração explícita, não closure presumida | H6/Q7, eventual sub-PR produto |
| #153/#154/#160; temas #31/#32 | RFCs/compressão/Markdown portability presentes; regressões apenas se gap validado | H6/Q7/Q4 |
| #156–159; temas #34–37 | Record/status/handoff e verification manifest presentes; não equivalem a runner/evidence TCB | H2/H4/H6/Q4 |
| #183 | Duplicação de campo status corrigida localmente; `active_issues` único testado, não reimplementar | E0/baseline |
| #177 | Lock `quinn-proto0.11.15`/`lopdf0.42.0` atende floors da issue; prose antiga de blockers precisa reconciliação. Scanner atual não executado: sem afirmar vulnerável nem audit clean | E0/Q5/Q6 |
| #161/#178 e #162/#179 | Rate limiter, 429 e telemetry presentes; validar critérios residuais, parse/MCP errors e SSE/latency | C1/Q4/O1 |
| #164 | Docs atuais BLOB/cosine coerentes com implementação local; sem gap residual identificado | Q7/baseline |
| #163/#165–176 | Paths dream/examples presentes; Fly/cloud/listing e execução externa não demonstrados nem autorizados | Q4/C6 quando local; deploy fora de escopo |

Novos achados deste plano só viram issues mediante ação autorizada separadamente: título/escopo/reproducer/aceite e owner, com verificação contra histórico para evitar duplicatas. Closed é índice de gestão, não underlying fact.
