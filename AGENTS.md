# Engram - AI Agent Context

## Visão Geral
Engram é um sistema de memória persistente para agentes de IA, construído em Rust. Ele fornece armazenamento, busca híbrida (BM25 + vetores + fuzzy) e grafos de conhecimento através de uma API REST, WebSocket e protocolo MCP (Model Context Protocol).

**Repositório**: https://github.com/limaronaldo/engram  
**Linguagem Principal**: Rust (com SDKs em Python e TypeScript para integração)  
**Banco de Dados**: SQLite + WAL (local-first), com sincronização opcional S3/R2

## Estrutura do Projeto
```
engram/
├── src/                # Código principal Rust (crate: engram-core)
│   ├── lib.rs          # Ponto de entrada da biblioteca
│   ├── bin/            # Binários (engram-server, engram-cli, ...)
│   ├── hooks/          # Lifecycle hooks (session_end, etc.)
│   ├── storage/        # Camada de banco de dados e migrações
│   ├── search/         # Busca híbrida e embeddings
│   ├── mcp/            # Implementação do protocolo MCP
│   └── ...
├── sdks/
│   ├── python/         # SDK Python assíncrono (EngramClient)
│   └── typescript/     # SDK TypeScript (EngramClient)
├── tests/              # Testes de integração Rust
├── docs/               # Documentação adicional
├── Cargo.toml
└── README.md           # Documentação principal
```

## Como Rodar Localmente
```bash
# Clone o repositório
git clone https://github.com/limaronaldo/engram.git
cd engram

# Compile
cargo build --release

# Rode como servidor MCP (para Claude Code, Cursor, etc.)
./target/release/engram-server --mcp

# Ou como API HTTP
./target/release/engram-server --http --port 8080
```

## Padrões de Código Importantes

### Rust
- **Tratamento de Erros**: Use `Result<T, EngramError>` com `?` para propagação
- **Thread Safety**: Handlers devem implementar `Send + Sync`
- **Testes**: Use `#[cfg(test)] mod tests` com `#[test]` functions
- **Clippy**: Sempre execute `cargo clippy` antes de submeter PRs
- **Migrations**: Ao alterar o schema, atualize `SCHEMA_VERSION` em `storage/migrations.rs` e os testes correspondentes

### Python SDK (`sdks/python/`)
- **Async**: Use `async with EngramClient(...) as client` para gerenciar ciclo de vida
- **Parâmetros**: Evite sombrear built-ins (ex: use `filter_` em vez de `filter`)
- **Type Hints**: Use `dict[str, Any]` (Python 3.9+) em vez de `Dict`/`List` de `typing`
- **Docstrings**: Siga PEP 257, inclua exemplos assíncronos corretos

### TypeScript SDK (`sdks/typescript/`)
- **Tipagem**: Evite `unknown` excessivo, crie interfaces específicas para retornos
- **Nomenclatura**: Use camelCase no TS, convertido para snake_case na API
- **Documentação**: Use TSDoc (`/** */`) para todos os métodos públicos

## Armadilhas Conhecidas (Gotchas)
1. **Schema Version Mismatch**: Testes em `storage/migrations.rs` têm versão hardcoded. Ao atualizar schema, atualize também os testes.
2. **Python Built-ins**: O parâmetro `filter` em `EngramClient.list()` foi renomeado para `filter_` para evitar sombra do built-in.
4. **TypeScript Syntax**: Verifique cuidadosamente o fechamento de tipos em métodos que retornam objetos complexos (ex: `Promise<{...}>`).

## Ferramentas MCP Disponíveis
O Engram expõe 155+ ferramentas via MCP. Principais:
- `memory_create`, `memory_search`, `memory_list`
- `memory_create_daily` (memórias efêmeras)
- `identity_create`, `identity_resolve`
- `memory_temporal_contradictions`, `memory_scope_set`

## Onde Encontrar O Que
- **Hooks de ciclo de vida**: `src/hooks/`
- **Implementação MCP**: `src/mcp/`
- **Cliente Python**: `sdks/python/engram_client/client.py`
- **Cliente TypeScript**: `sdks/typescript/src/index.ts`
- **Testes de integração**: `tests/*.rs`
- **Documentação da API**: `REFERENCE.md` (Engram Cloud), `INVARIANTS.md` (regras do projeto)

## Comandos Úteis para Agentes
```bash
# Verificar código Rust
cargo clippy && cargo fmt --check

# Rodar testes Rust
cargo test

# Verificar tipos TypeScript (se node disponível)
cd sdks/typescript && npm run type-check

# Testar cliente Python (se ambiente configurado)
cd sdks/python && pytest tests/
```
