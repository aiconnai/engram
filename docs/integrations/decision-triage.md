# Decision triage — CLI compartilhada e skill global

> Ferramenta compartilhada de triagem estruturada (d1/Jev), instalada globalmente
> em `~/.agents/skills/decision-triage` e acessível a partir de qualquer repo.
> Este repositório está configurado em `decision-triage.json` (sem credenciais)
> e registrado no allowlist central. Preview é o padrão; chamadas pagas exigem
> autorização explícita por tarefa, dados e orçamento.

O mecanismo foi extraído para uma ferramenta compartilhada com configuração
explícita por repositório, preview sem custo e chamadas pagas limitadas por
autorização. A instalação fica em `~/.agents/skills/decision-triage`; cada
projeto tem um `decision-triage.json` sem credenciais, e o orçamento e o
histórico ficam centralizados na instalação global (não neste repo).

A execução histórica da issue #652 foi preservada como evidência, sem alteração
dos arquivos congelados. A CLI está generalizada para triagem de issues e
validações, sem depender da issue #652.

## Usar pelo Zed

Abra uma **nova conversa** e invoque:

```text
/decision-triage
```

Depois peça, por exemplo:

> Faça preview da triagem da issue 123 deste repo, sem inferência paga.

Ou:

> Prepare a triagem dos testes que falharam, separando fatos, hipóteses e evidências ausentes.

A skill usa **preview por padrão**. Chamadas pagas exigem autorização específica
para a tarefa, dados e orçamento.

## Testar neste repo — sem custo

Existe um exemplo sintético em `tmp/shared-triage-example/input.json` (diretório
ignorado pelo Git). Se ele não existir, recrie-o a partir de
`~/.agents/skills/decision-triage/templates/input.json` com
`"repository": "aiconnai/engram"`.

```sh
rtk proxy python3 -B ~/.agents/skills/decision-triage/scripts/triage.py preview \
  --repo . \
  --input tmp/shared-triage-example/input.json
```

Para consultar a contabilidade compartilhada:

```sh
rtk proxy python3 -B ~/.agents/skills/decision-triage/scripts/triage.py status
```

## Configuração deste repo

`decision-triage.json` na raiz (versionado, sem segredos):

- identidade `aiconnai/engram`;
- tarefas `issue-triage` e `validation-triage`;
- campos `observations`, `proposed_next_step`, `constraints`, `evidence_gaps`;
- até **duas chamadas e US$ 0,01** por operação; corte fixo **0,90**;
- d1 principal; Jev somente por incerteza válida abaixo de 0,90.

O root e o SHA exato da config estão fixados no allowlist central via `register`.
Editar `decision-triage.json` exige novo registro; grants anteriores ficam
inválidos. **Registrar não autoriza gastos.**

## Adicionar outro repo

Na raiz do outro projeto, substitua `OWNER/REPO` pela identidade correta:

```sh
rtk proxy python3 -B ~/.agents/skills/decision-triage/scripts/triage.py init \
  --repo . --repository OWNER/REPO --write

rtk proxy python3 -B ~/.agents/skills/decision-triage/scripts/triage.py register --repo .
```

Se `decision-triage.json` já existir, inspecione-o em vez de tentar recriá-lo.

## Estado compartilhado preservado

- Histórico centralizado: **194 tentativas**, subtotal conhecido
  **US$ 0,46017200482**, sem zerar custos por repo.
- Pendência Drex e reserva de **US$ 0,00167088** intactas
  (tentativa `86f7a96a-6958-405e-9f2b-3ae1422bfbb1`). A exceção antiga da
  issue 652 **não** se aplica a novas tarefas.
- Teto acumulado: US$ 5 / 217 tentativas.
- Nenhuma inferência paga foi feita na instalação nem nesta configuração; nenhum
  hook ou alteração automática de produção.
- `paid_dispatch_allowed: false` até uma autorização explícita e escopada.

## Validação registrada

- 77 testes compartilhados e 118 testes legados da skill passaram, além dos
  checks de documentação e do preview real.
- A gravação da suíte específica de inputs foi recusada pela ferramenta e não foi
  repetida nem contornada; essa limitação está registrada na skill.
- Neste repo: `preview` executado com `network_calls: 0`, `registered: true` e
  `paid_dispatch_allowed: false` (evidência gerada em 2026-10-23).

Referências da instalação global (fora deste repo):

- Skill: `~/.agents/skills/decision-triage/SKILL.md`
- Guia completo: `~/.agents/skills/decision-triage/references/usage.md`
