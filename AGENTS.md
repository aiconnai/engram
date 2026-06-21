# Engram - AI Agent Context

## Visão Geral
Engram é um sistema de memória persistente para agentes de IA e times que trabalham sobre contexto proprietário, construído em Rust. Ele organiza entrevistas, reuniões, documentos, transcrições e decisões, e expõe isso por busca híbrida (BM25 + vetores + fuzzy), grafos de conhecimento e protocolo MCP (Model Context Protocol) para consulta direta na fonte.

**Repositório**: https://github.com/aiconnai/engram  
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
git clone https://github.com/aiconnai/engram.git
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
3. **TypeScript Syntax**: Verifique cuidadosamente o fechamento de tipos em métodos que retornam objetos complexos (ex: `Promise<{...}>`).

## Ferramentas MCP Disponíveis
O Engram expõe as ferramentas MCP documentadas em [`docs/MCP_TOOLS.md`](docs/MCP_TOOLS.md). Elas existem para transformar contexto disperso em memória acionável, com leitura e escrita pela mesma superfície. Principais:
- `memory_create`, `memory_search`, `memory_list`
- `memory_create_daily` (memórias efêmeras)
- `identity_create`, `identity_resolve`
- `memory_temporal_contradictions`, `memory_scope_set`

## Huly
Se o skill `huly` estiver disponível no seu ambiente (`.claude/skills/huly/SKILL.md`, não versionado), leia-o primeiro. Caso contrário, as regras abaixo são auto-suficientes.

- Prefira a Huly Platform API para automação com token, especialmente quando o login humano usa GitHub/Google/OIDC.
- Variáveis mínimas: `HULY_URL`, `HULY_WORKSPACE`, `HULY_PROJECT`, `HULY_API_TOKEN`.
- Fallbacks aceitos: `HULY_TOKEN` e `HULY_APY_TOKEN`.
- Nunca imprima tokens, headers, cookies ou dumps de ambiente com credenciais.
- Antes de qualquer escrita, faça um lookup read-only para confirmar workspace, token e projeto.

## Onde Encontrar O Que
- **Hooks de ciclo de vida**: `src/hooks/`
- **Implementação MCP**: `src/mcp/`
- **Cliente Python**: `sdks/python/engram_client/client.py`
- **Cliente TypeScript**: `sdks/typescript/src/index.ts`
- **Testes de integração**: `tests/*.rs`
- **Documentação da API**: `docs/REFERENCE.md` (Engram Cloud), `INVARIANTS.md` (regras do projeto)
- **Tese do produto**: `README.md`, `docs/README.md`, `docs/AI_GUIDE.md`, `docs/USING_ENGRAM_IN_A_REPO.md`
- **AI Operating Guide / lazycodex-ai**: `docs/AI_OPERATING_GUIDE.md`

## Harness de Desenvolvimento (Obrigatório no Início de Toda Sessão)

Antes de qualquer planejamento ou edição, rode:

```bash
bash docs/harness/bin/bootstrap.sh
```

Em seguida leia (em ordem):
- `docs/harness/SPEC.md`
- `docs/harness/INVARIANTS.md`
- `docs/harness/WHAT_WE_DONT_DO.md`
- `docs/harness/GATES.md`
- `docs/harness/CODE_REVIEW_POLICY.md`
- `docs/harness/security/anthropic-reference-harness.md`
- `docs/harness/README.md`
- `docs/harness/progress.md`
- active plan apontado em `docs/harness/progress.md`
- `AGENTS.md` e `CLAUDE.md`
- `INVARIANTS.md` (raiz)
- `STANDARDS.md` e `ERRORS_AND_LESSONS.md`

O harness garante que o trabalho seja retomável entre agentes (Claude Code CLI, Grok Build TUI, etc.) e entre sessões. Ele implementa as camadas de Context Engine, Planner, Memory Manager e Verifier diretamente no repositório.

## AI Operating Guide (`lazycodex-ai`)

Para decidir quando delegar trabalho de repositório ao `lazycodex-ai`, leia
[`docs/AI_OPERATING_GUIDE.md`](docs/AI_OPERATING_GUIDE.md) depois da leitura
obrigatória do harness. Use esse guia para escolher entre resposta direta,
`lazycodex-ai doctor` e `lazycodex-ai run`, sempre preservando os gates locais,
o boundary de segurança do harness e a validação com evidência.

## Comandos Úteis para Agentes
```bash
# Harness bootstrap + gates (obrigatório)
bash docs/harness/bin/bootstrap.sh
bash docs/harness/bin/doctor.sh
bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
bash docs/harness/bin/review-gate.sh pre harness-bootstrap

# Verificar código Rust
cargo clippy && cargo fmt --check

# Rodar os gates obrigatórios de CI localmente (equivalente ao que deve estar
# verde no PR): formato, lint, testes core (Linux) e documentação.
make ci
# ou
just ci
# Para ignorar falhas de validações opcionais localmente (ex.: dependências ausentes):
# CI_ALLOW_OPTIONAL_TEST_FAILURES=1 make ci
# Opcional: instale o hook local de pre-commit
git config core.hooksPath .githooks

# Rodar testes Rust
cargo test

# lazycodex-ai operational delegation
rtk npx lazycodex-ai doctor
rtk npx lazycodex-ai run "Task summary with acceptance criteria"

# Verificar tipos TypeScript (se node disponível)
cd sdks/typescript && npm run type-check

# Testar cliente Python (se ambiente configurado)
cd sdks/python && pytest tests/
```
