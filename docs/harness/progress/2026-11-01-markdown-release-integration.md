# Candidato integrado Markdown + remediação — 2026-11-01

Mandato vigente: corrigir bloqueantes antes da tag, preparar candidato local revisável. Bootstrap no
checkout principal e no WT novo limpo. Bases congeladas: remediação646636f6 e import1faa5c6f;
79889460 é ancestral da remediação e preservado. Só import entra no merge: contém o produto do export.
Worktree principal suja com planejamento permanece intacta. Sem push/PR/merge de main/tag/release,
configurações reais, dados reais, inferência externa paga ou remoção de branches/worktrees.

## Contrato e ownership

- Root: resolver merge neste WT, manter novo handler + resposta dry_run no engine transacional,
  testes reais CLI/revisão-default, pular entradas ocultas no reader sem seguir links, docs/gates.
- /root/plan_quality_evidence: leitura estática de todos10 paths do79889460; sem edição/build/rede.
- Review posterior read-only por agente que não implementou; scope/manifest congelados antes.
- Cargo pesado serial, target compartilhado; nenhum cargo clean. Políticas/gates imutáveis.
- Erro pós-commit continua unknown/applied null; não restaurar antigo handler/applied0.

## Resolução semântica do merge

Conflitos em handler, CHANGELOG, SPEC, progress, log e risk register. Handler do import seguro;
review defaults CLI/Rust/Python/TS da remediação; campo wire dry_run precisa mudar em apply::run.
CHANGELOG mantém seção0.24.0 preparada da remediação, recebe só4 bullets Markdown, sem duplicar
release antiga. SPEC/log preservam as duas histórias. Progress usa autorização local mais recente;
riscos0001–0009 mantêm remediação,0010/0011 entram do import (abertos). Sem usar receipt anterior
como teste da combinação. SDK metadados unknown completos em exceção não são contrato novo aqui;
falha/mensagem continuam propagadas, sem sucesso zero. Captura e ensaios novos pendentes.

## RED e ajuste

integration-red.log:100 PASS/3 FAIL/1 probe manual ignorado,2.27s. Dois testes reproduzem leitura
indevida de metadados/lixeira e consumo de quotas ocultas; terceiro comprova ausência do campo
wire dry_run. Sem erro de build/fixture. Depois: skip de nomes ocultos antes de stat/open, mantendo
contagem da entrada visitada; apply::run responde dry_run=!confirm, sem alterar o erro unknown.
79889460 preservado nos oito paths não envolvidos no conflito/fachada. Teste real CLI em preparação.

## Checks focais do combinado

integration-green.log:103 PASS/1 probe manual ignorado,2.37s, features obrigatórias. integration-cli.log:
binário real3 PASS (7 invocações), G1 8 PASS/1 helper invocado pelos pais, vault6 PASS. Defaults,
force semconfirm, confirm aplica create+update e conflitoflags antes de abrir DB conferidos.
Python2 PASS; TypeScript4 VaultResource PASS (100 outros filtrados) + type-check PASS. SDKs idênticos
à remediação; dependências TS já instaladas reutilizadas por link temporário novo (nenhum path existente
substituído, sem npm install/rede); removido após os testes, sem alterar node_modules original. Modelo/effort dos agentes nativos herdados; identidade efetiva,
tokens e custo não expostos. Participantes root(integração/gates), quality(testeCLI+contratos) e
reviewer independente da implementação(consulta posterior). Não alegar independência entre modelos.

## Achado documental na revisão

Snapshotv1: a resolução por regex DOTALL gulosa perdeu seções da base em SPEC e active log.
Achado aceito; reconstrução a partir de646 INTEGRAL + seção P0/tail exclusivas de1faa contra2e84,
sem interpretar marcadores por regex. Assert: retirar apenas os dois acréscimos do SPEC recupera
literalmente o SPEC-base; active log inicia com base inteira; todas as headings base e âncora do
provisionamento30/10 presentes. Snapshotv1 permanece imutável; nova revisão necessária. Rust não mudou.

## Commit e revisão da combinação

Merge local9040153b985bef83ad2c99ffa13409caae42ce76 com pais646636f6 e1faa5c6f. Confirmados por
ancestralidade também79889460 e9bca4efa. Hook fmt+clippy all-targets/all-features PASS. Todos56
paths do snapshotv2 conferidos nos blobs do commit (55regulares+lookup ausente), sem omissões.
Reviewerfechou bloqueante documental na v2; rawsv1/v2 preservados. Revisão estática de merge/delta,
não auditoria global nova, max, post-gate ou receipt humano. Codegate está executando sobre esse
commit; ambiente managedexigiu liberação técnica para gravar.git e binds locais do gate. Essa
liberação não é assinatura/receipt de review nem autoriza main/tag/publish.

14 comandos complementares de CI/release-workflow PASS (incluindo14 testes de workflowSDK);
log integration-ci-complement.log. Input Linux congelado por git archive desse commit, SHA256
ece2170bf1ad1c87a8a7dd72eab51194e7a07b90738a76eafe2a00eb486b1557, ainda não executado.

## Resultado integrado e limites

Full PASS timestamp2026-11-02T00:19:12Z duration422s, sem exclusões, executado depois do merge9040153b.
Macpin1.96.0, hooks fmt+clippy all-targets/all-features mantidos. ComplementoCI14comandosPASS.
LinuxDocker arm64/Rust1.97.1 exit0/oomfalse:103MarkdownPASS/1probe manual ignorado, CLI3PASS,
permissões9PASS, routing3PASS, G1 8PASS/1helper ignorado mas invocado pelos pais, vault6PASS.
Total132PASS no recorte. Logs integration-full.log, integration-cli.log, integration-linux.log,
integration-ci-complement.log e SDKs em target/markdown-export-followup no checkout principal.

Linux usa gitarchive9040153b, imagem/preparação/cache já registrados em linux-environment.json,
rootfsread-only, networknone, capdropALL, nonewprivileges,4CPU/8GiB/512PIDs e só scratch da tarefa;
sem socket/credenciais montados. Managedsandbox recusou leitura do socket local; coleta permitida
fora dela apenas para esse container. Nenhuma alegação de toolchain pin1.96 Linux ou CI Linux inteiro.

NotFound restrito vs ownernew está documentado na migraçãoIDs e agora também na descrição MCP.
Migração integral do writerFFI paraRustix continua fora desta fatia; não ampliada durante integração.
Não adicionei probes HTTP novos com principals reais: o recorte novo exercita handler/dispatch eCLI;
o full inclui os testes HTTP existentes. Não é certificação global MCP nem formal max/post-gate.

Worktree principal de planejamento e branches originais preservados. Sem push/PR/main/tag/release
ou dados reais. RISK0010/0011 continuamabertos no publicado. Handoff pós-gate muda só documentos,
receipts e artefatos de review; 32paths não-harness do código/contratos conferidos contra o snapshot e9040153b.
