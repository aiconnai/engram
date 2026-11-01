# Review Canvas: markdown-export-safety

Data: 2026-11-01. Escopo autorizado: P0 local sobre origin/main `2e84a594`.

## Approaches Considered

| Abordagem | Decisão | Motivo |
|---|---|---|
| Canonicalize da raiz seguido de fs::write | Rejeitada | Tag escapa, symlink redireciona e check/open tem corrida |
| Incorporar projeção P1/manifest | Rejeitada | Hotfix do produto existente, sem política/RFC nova |
| Validar lote e escrever por diretórios ancorados sem seguir links | Selecionada | Conter destinos/efeitos; filesystem não é workspace |
| Ignorar row/relação inválida | Rejeitada | Oculta perda de dados |

## Hot Path Complexity

Consultar/renderizar antes de escrever. Ordenar O(n log n + e log e); buffers proporcionais ao lote.
Sem rede/LLM/busca vetorial ou transação SQLite durante I/O de arquivos. Atomicidade por arquivo.

## Edge Cases

Traversal/separadores de outra plataforma; symlink de diretório/arquivo; alias SQLite; read-only e
workspace alheio; expiradas referenciadas; row/relacionamento inválido; empate no índice; erro de
escrita/rename; corrida em entradas de diretório e limites da raiz administrativa confiável.

## Breakage Risk

| Risco | Impacto | Mitigação/verificação |
|---|---|---|
| Permissão sobe de read-only | Cliente antes permissivo é recusado | Nota/dispatcher negativo antes dos efeitos e referência coerente |
| Plataforma sem primitiva segura | Fallback reabre escape | Falhar fechado sem escrita insegura |
| Falha parcial de lote | Falso export completo | Preflight e erro explícito; atomicidade só por arquivo |
| Link falso/fora de escopo/TTL | Histórico incorreto ou vazamento | IDs elegíveis do lote, direção explícita e negativos |
| Review/gate de outro SHA | Falsa aprovação | Candidato identificado; revisão/receipt próprios, risco não fecha por baseline |
