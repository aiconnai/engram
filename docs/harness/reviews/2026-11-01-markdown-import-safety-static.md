# Revisão estática delimitada — RISK-0011

Reviewer nativo `/root/review_projection_plan_v2`, via já autorizada. Registro pelo integrador das
respostas recebidas; mesma família, sem alegação de independência entre modelos. Sem execução de
alvo/build/test/rede, sem edição ou credenciais. Não é max/post-gate/receipt humano/aprovação de merge.

## Rodada 1 — dois achados bloqueantes

Base 9bca4efaeb07d9deff722ab50c407dfe77f02f63. Capture /private/tmp/engram-import-review-bxz5c058.
Target 73ef357922375f266d77ce3bdda295b463823604aebe01adbd228b86d2d802c8.
Manifest b1be35910a84c399bfc5bb052d2b16a6db52d8a8be5b6b3b802312326d2544ab.
17 paths reconciliados: 16 hashes regulares e ausência intencional de import/lookup.rs.

- HIGH: applied0 em todo erro nega writes que Storage pode ter commitado antes de falhar no
  hardening de permissões posterior. Trigger ABORT só cobria erro anterior ao commit.
- MED: parser limitado a colon+space perdia IDs/versões separados por tab; campo inválido virava
  ausente e podia criar memória. Pediu testar review/confirm e ausência de mutação.

Nos demais fluxos, sustentou a autorização pelo workspace persistido antes de hash/versão,
equivalência foreign/missing, classificação integral antes de writes, BEGIN IMMEDIATE, ausência
de IO de input/rede/embedding em TX, quotas, reader ancorado e proteção de SQLite antes de open.

## Rodada 2 — achados resolvidos na fonte

Capture /private/tmp/engram-import-review-v2-idgg8snm.
Target 3df5fe646d44cc3face560eb53d1a1b4aad6dc3285b898a91d4e2a50f1ac6939.
Manifest a336a488219aec7005d16640fb93475a16b946f0d015158f077606fedab865f5.
17 paths conferidos novamente (16 regulares + lookup ausente). Oito alterados: parser, apply,
handler, integração, changelog, referência MCP, migração e catálogo; demais bytes/instruções iguais.

Resposta: "os dois achados estão resolvidos na fonte; não identifiquei novo defeito concreto no delta".
Parser preserva scalar fields com space/tab/EOL e chaves quoted/unquoted para validação. ImportFailure
marca erro pós-callback como unknown/applied null/attempted N e orienta reconciliação antes de retry,
sem expor erro FS original. A regressão pós-commit inspeciona conteúdo e autocommit após restaurar
somente o alias de pai configurado; trigger continua cobrindo rollback anterior ao commit.

Reviewer não atesta execução RED/GREEN. Evidência executada pertence ao log próprio, não a este
parecer. Os hashes serão conferidos contra o commit; os documentos vivos do harness/risco estão
fora destes 17 paths e não recebem aprovação implícita. Formal post-gate/receipt continuam pendentes.
