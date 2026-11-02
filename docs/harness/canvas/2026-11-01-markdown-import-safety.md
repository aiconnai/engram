# Review Canvas: markdown-import-safety

RISK-0011, candidato local separado, empilhado em export `9bca4efa`. Sem publicação/dados reais.

## Approaches Considered

| Abordagem | Decisão | Motivo |
|---|---|---|
| Canonicalize seguido de read_to_string | Rejeitada | Descendentes seguem links e há check/open |
| Autorizar só workspace do argumento | Rejeitada | ID persistido pode pertencer a outro workspace |
| Leitura ancorada/limitada e aplicação DB transacional | Selecionada | Valida origem antes do open e workspace/versão no mesmo snapshot |

## Hot Path Complexity

Walk ordenado e limitado: profundidade 32, 10000 Markdown, 20000 entradas, 4 MiB por arquivo e
64 MiB no total. Buffers limitados; nenhum IO de arquivo/rede/embedding dentro da transação DB.
Descendentes privados segundo a política ACL/ownership do export; mesma identidade OS fora do sandbox.

## Edge Cases

Symlink/ciclo/hardlink/FIFO, artefatos SQLite antes de abrir descritor regular, UTF-8 inválido,
ID explícito inválido, ID estrangeiro/inexistente indistinguíveis para principal restrito, criação
por workspace efetivo, versão concorrente, force_version, tags/Obsidian e dry-run legados.

## Breakage Risk

Com raiz administrada, scoped_write; sem raiz, maintenance. Negar chamadas diretas também. Pré-ler
arquivos e validar lote antes de aplicar. Confirm usa uma transação SQLite para autorizar/classificar
antes de escrever e aplicar o lote; erro antes do commit desfaz o lote DB; erro após callback concluído tem outcome unknown/applied null, sem afirmar rollback. Dry-run usa snapshot de leitura. Erros
não viram sucesso zero no wrapper. Limites/ACL e política de IDs restritos exigem nota de compatibilidade.


| Risco | Impacto | Mitigação e evidência |
|---|---|---|
| Mode/raiz/workspace mais estritos | Clientes permissivos passam a falhar | Nota de migração, testes handler+dispatch com principals e controle positivo |
| Erro após callback/commit | Retry duplica ou sobrescreve contexto | outcome unknown/applied null, teste real pós-commit, reconciliação antes de retry |
| Erro antes do commit | Parte do lote persistida | Transação única, preflight, trigger ABORT confirma rollback com tags/versões |
| Walker abre alias SQLite | Perda de locks e commits WAL | Metadata antes do open, no-follow, G1 por processo independente |
| YAML escalar/quotas | Campos viram ausência ou consumo sem limite | Testes tab/quoted/empty ID, reader com limites e erros explícitos |
