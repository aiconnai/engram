# Review Canvas: markdown-release-integration

## Approaches Considered

| Opção | Decisão | Razão |
|---|---|---|
| Integrar só export/import sobre main | Recusada | Perde remediação79889460 e outras correções |
| Restaurar handler antigo | Recusada | Reabre autorização/IO/atomicidade |
| Remediação + branch import + contratos no engine novo | Adotada | Preserva ambos históricos e resultado seguro |

## Hot Path Complexity

Sem alteração de busca/embeddings/schema. Reader mantém limites de entradas visitadas e ordenação;
pasta oculta conta como uma entrada mas seus descendentes não são lidos. Nenhum IO de input em TX.

## Edge Cases

Import omitindo confirm; flags conflitantes; force_version sem confirm; criação e update; MCP wire
dry_run; SDKs Python/TS; .trash/.git/.obsidian com symlinks; links visíveis continuam recusados;
NotFound restrito vs owner local; outcome unknown; conflito de catálogo e notas de release.

## Breakage Risk

| Risco | Efeito | Verificação |
|---|---|---|
| Merge perde defaults | CLI/SDK escrevem sem pedido | Binário real e testes SDK existentes |
| Merge perde dry_run wire | Cliente recebe contrato incompleto | Assert direto handler/dispatch |
| Pastas ocultas entram na captura | Lixeira vira memória/quotas falham | Testes reader com limites pequenos e symlink oculto |
| Conteúdo seguro perdido | IDOR ou sucesso zero | Suíte Markdown/G1 e review do delta |
| Gate de outra árvore | Falsa prontidão de release | Full novo, manifest e commit conferidos |
