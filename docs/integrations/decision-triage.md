# Decision triage — CLI compartilhada

> Ferramenta compartilhada de triagem estruturada, instalada separadamente deste
> repositório e acessível a partir de qualquer projeto. Este repositório tem um
> `decision-triage.json` sem credenciais e registrado no registro central da
> ferramenta. Preview é o padrão; chamadas pagas exigem autorização explícita por
> tarefa, dados e orçamento.

A CLI é generalizada para triagem de issues e validações. Orçamento e histórico de
uso ficam centralizados na instalação da ferramenta, não neste repositório. Cada
projeto mantém apenas a sua configuração.

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
ignorado pelo Git). Se ele não existir, crie-o a partir do template fornecido com a
instalação da CLI, definindo `"repository": "aiconnai/engram"`.

Em todos os comandos abaixo, `<decision-triage>` é o diretório onde a ferramenta
compartilhada está instalada.

```sh
python3 -B <decision-triage>/scripts/triage.py preview \
  --repo . \
  --input tmp/shared-triage-example/input.json
```

Para consultar o status da instalação compartilhada:

```sh
python3 -B <decision-triage>/scripts/triage.py status
```

## Configuração deste repo

`decision-triage.json` na raiz (versionado, sem segredos):

- identidade `aiconnai/engram`;
- tarefas `issue-triage` e `validation-triage`;
- campos `observations`, `proposed_next_step`, `constraints`, `evidence_gaps`;
- até **duas chamadas por operação**;
- o modelo principal é o padrão; o modelo secundário só é acionado com incerteza
  válida abaixo do corte fixo de **0,90**.

A raiz do repositório e o SHA exato da configuração ficam registrados no registro
central por meio de `register`. Editar `decision-triage.json` exige novo registro,
e autorizações anteriores ficam inválidas. **Registrar não autoriza gastos.**

## Rotas de inferência

A rota é congelada no momento da autorização (`authorize`). O padrão é `systemone`.

| Rota | Endpoint | Chamadas | Credencial | Custo registrado |
|---|---|---|---|---|
| `systemone` | OpenRouter `api/v1/systemone` (modelo principal, com escalonamento condicional para o modelo secundário) | até 2 | `OPENROUTER_API_KEY` | recibo do provider |
| `muse` | Meta Model API `api.meta.ai/v1/chat/completions` (`muse-spark-1.3`, Standard mode) | 1 | `MUSE_API_KEY` | calculado por tabela pinada (`cost_basis: computed_standard_price_table`) |

Cada rota usa somente a variável de ambiente indicada na tabela, e somente durante
`execute`. Na rota `muse`, o custo é estimativa por tabela, não recibo do provider.
Contributor mode (que autoriza a Meta a treinar com o conteúdo enviado) **não** é
usado.

## Adicionar outro repo

Na raiz do outro projeto, substitua `OWNER/REPO` pela identidade correta:

```sh
python3 -B <decision-triage>/scripts/triage.py init \
  --repo . --repository OWNER/REPO --write

python3 -B <decision-triage>/scripts/triage.py register --repo .
```

Se `decision-triage.json` já existir, inspecione-o em vez de tentar recriá-lo.

## Comportamento padrão

Por padrão, `preview` não faz chamadas de rede (`network_calls: 0`), e
`paid_dispatch_allowed` fica `false` até uma autorização explícita e escopada.
