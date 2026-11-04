# Review Canvas — Nightly e release

Data: 2026-10-27. Base: `2e84a594`. Mandato: [plano](../plans/2026-10-27-nightly-release-readiness.md).
Este canvas registra decisões; não concede aprovação.

## Alternativas e decisão

- Árvore de escopos: corrigir a ordem antes de `pop()`; manter o algoritmo existente e a
  semântica de órfãos. Reescrita recursiva ou criação de pais ausentes ampliaria o contrato.
  Remover o caso especial de Global: `parent() == None` já produz a mesma raiz, logo esse
  ramo era redundante. O efeito do mutante `==` → `!=` era mascarado pela árvore achatada;
  a nova cobertura exige a hierarquia e não depende desse caso especial.
- Mutantes: fortalecer asserções comportamentais e persistência; não remover módulos do
  inventário e não aceitar resultado vazio como cobertura. Reproduzir com ferramenta pinada.
- Miri: usar 256 entradas do mesmo seed sob `cfg!(miri)`; manter 20.000 entradas nativas e
  todos os casos Unicode explícitos. Aumentar o timeout não resolve o custo de interpretar
  o oráculo alocador; ignorar o teste perderia toda sua cobertura no interpretador.
- Assinaturas: manter verify-tag e propor confiança explícita em chave pública dedicada.
  Nenhuma chave de release foi fornecida e nenhuma chave local foi promovida a autorizada.
  O workflow atual fica intacto; proposta e requisitos ficam nos docs de release.

- SDK metadata: separar canal de registry (`npm`) do nome do SDK (`typescript`)
  e ignorar só o próprio arquivo de saída ao montar hashes. A regressão executa o
  helper real duas vezes em cada canal; antes do fix, npm falha com StopIteration
  e Python inclui o próprio metadata na segunda geração. Guards de publicação intactos.

## Custo e limites

`build_tree` mantém O(n²) em tempo e O(n) em memória, sem I/O novo. A mudança só ordena os
mesmos nós. Os novos testes SQLite usam bancos sintéticos em memória. O smoke Miri deixa
de interpretar a campanha nativa inteira; seu resultado não prova ausência geral de UB.

## Casos de borda e risco

| Caso/risco | Mitigação e verificação |
|---|---|
| Cinco níveis e irmãos sob um mesmo pai | Teste de árvore completa; caminhos e contagens exatos |
| Pai direto ausente, Global presente | Órfão continua raiz; não anexar a ancestral indireto |
| Credencial vazia só de um lado | Ambos os casos devem ser `MalformedBearer`; ambos vazios também |
| Extrator com tipos desligados ou orçamento já cheio | Contagem total, identidade e ordem dos resultados, não só `len <= max` |
| Retorno de auto-link correto sem efeito persistido | Assert de links SQLite e aliases em duas memórias distintas |
| Amostra Miri menor reduz variedade aleatória interpretada | Mesmo seed, matriz adversarial integral e 20.000 casos nativos; revisão independente pendente |
| Notas extraídas omitem a remediação em Unreleased | Mover conteúdo para 0.24.0 e executar o extrator real |
| Mutantes que travam continuam vermelhos | Registrar timeout como timeout; não modificar o runner para produzir verde |

## Validação e aprovação

Comandos, resultados e limites estão na [auditoria](../audits/2026-10-27-nightly-release-readiness.md).
Verificação objetiva é separada da decisão humana de publicar. Parecer independente e receipt
continuam pendentes; nenhum resultado local aprova publicação.
