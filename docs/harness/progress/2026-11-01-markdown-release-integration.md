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
