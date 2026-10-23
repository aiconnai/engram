# Review Canvas: turso-sunset

Date: 2026-10-23
Owner: Ronaldo / Codex integrador
Scope: Executar remoção Turso/libSQL prevista no ADR aceito para v0.24.0.

## Trigger

Dependência externa e backend removidos, API Rust quebrada deliberadamente e lockfile/exceções reconciliados; nenhuma alteração de schema ou superfície MCP.

## Approaches Considered

| Abordagem | Decisão | Motivo |
|---|---|---|
| Remoção completa conforme ADR | aceita no ADR e autorizada pelo owner | encerra backend e dependências obsoletas |
| Manter feature vazia ou backend deprecated | rejeitada | mascararia breaking change e não cumpre sunset |
| Extrair backend/converter bancos automaticamente | fora de escopo | exige mantenedor/contrato e migração distinta |

## Hot Path Complexity

SQLite CRUD/busca/transações mantêm tempo e espaço anteriores. Apenas um backend opcional e sua resolução de dependências são removidos. Não há I/O, rede ou novas alocações no hot path.

## Edge Cases

| Caso | Verificação |
|---|---|
| Consumidor pede `--features turso` ou importa `TursoBackend` | breaking change explícito; manifesto sem feature/export |
| Réplica Turso registra versão alta com schema incompleto | migração documenta destino novo; não reaproveitar caminho como conversão |
| Advisory também afeta outra dependência | árvore all-features e política comprovam remoção; exceções de PDF/embeddings permanecem |
| Closure de transação rejeitada produz side effect | asserção `!called` preservada em SQLite |

## Breakage Risk

| Risco | Impacto | Mitigação | Evidência prevista |
|---|---|---|---|
| API Rust/feature retirada | consumidores Turso precisam mudar | anúncio e guia de migração | reconciliação + docs |
| SQLite ou outro backend quebra | uso padrão afetado | não alterar implementação | sensores completos |
| CI pede feature removida | builds falham | lista compartilhada e inventário atualizados | paridade + contrato anunciado |
| Exceção retirada indevidamente | audit/deny falham | confirmar grafo sobrevivente | árvore, audit e deny |
| Migração perde IDs/proveniência | dados de usuário | não executar conversão; limites explícitos | revisão documental |

## Decision

Implementar receita aceita em branch isolada e submeter candidato integrado à revisão independente. Sensores/testes e parecer de IA não substituem receipt humano ou autorizam merge/publicação.
