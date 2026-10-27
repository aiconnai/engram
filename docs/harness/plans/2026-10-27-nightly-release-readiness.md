# Nightly e preparação da release — 2026-10-27

Base: `2e84a59422b3659aaaf25c69dbe01b01ab321332` (pilha #244–#262 mesclada).
Owner, chat de 2026-10-27: GO para diagnosticar o Nightly, corrigir defeitos confirmados e
lacunas relevantes de testes, resolver a proposta de verify-tag e preparar a release e o
aviso de migração. Entregar evidências e recomendação antes de publicar ou executar
mudanças operacionais. Este mandato prevalece sobre o GO histórico encerrado às 09:35Z.

## Entregas e aceite

1. Consultar o run `40479914277` e seus artefatos; registrar causa de Miri, inventário de
   mutantes sobreviventes e timeouts. Fonte e SHA explícitos; timeout não é prova de UB.
2. `src/storage/scoping.rs`: teste deve reproduzir a árvore achatada antes do fix; corrigir
   a ordem de processamento e verificar descendentes, irmãos e órfãos sem alterar schema.
3. `src/auth/transport_principal.rs` e `src/intelligence/entity_extraction.rs`: testar
   credenciais vazias, flags/limites/contadores, filtros comuns e links persistidos.
   Reexecutar mutações com cargo-mutants 27.1.0; não mascarar timeouts ou sobreviventes.
4. `src/text_util.rs`: preservar os 20.000 casos nativos, limitar o prefixo determinístico
   do smoke Miri a 256; preservar a matriz Unicode explícita, inventário e cap de 1800 s.
   Executar o inventário inteiro com a nightly local e registrar plataforma/toolchain.
5. `scripts/verify-sdk-artifacts.sh` e `scripts/test_sdk_release_workflow.py`: corrigir
   o mapeamento do canal npm para SDK typescript e excluir o próprio metadata do hash
   em retries. Regressão deve falhar antes do fix e passar para ambos os canais.
6. `CHANGELOG.md`, `docs/releases/` e `docs/migrations/`: incorporar a remediação à seção
   0.24.0 efetivamente extraída pelo workflow; preparar SDK 0.7.0 e aviso para #247.
   Proposta de trust store para verify-tag, sem escolher identidade/chave pelo owner.
7. Verificar `--release-plan`, extração de notas, contratos de release, claims, links,
   doctor e `make ci`; preparar revisão independente e registrar receipt como pendente.

## Limites

Só alterações locais, checks e consultas read-only. Sem push/PR/merge, tags de release,
publicação, avisos enviados, deleções de branches/worktrees, settings, produção ou inferência
paga. Fixtures de assinatura podem criar tags apenas em repositórios sintéticos temporários.
Sem receita humana fabricada, exclusões de sensores ou mudança da política de timeouts.
Não há agendamento de monitoramento até novembro.

## Rastreabilidade

Log ativo: [remediação](../progress/2026-10-26-remediation.md).
Canvas: [nightly e release](../canvas/2026-10-27-nightly-release-readiness.md).
Evidência: [auditoria](../audits/2026-10-27-nightly-release-readiness.md).
Decisão de release: [readiness](../../releases/v0.24.0-readiness.md).
