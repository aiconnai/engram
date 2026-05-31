# Fase 2.1 — Decisão #28: REST API local vs MCP-only

**Data:** 2026-05-31  
**Issue:** #28 (`Decision: local REST API vs MCP-only`)  
**Decisão do operador:** **MCP-only como superfície local de protocolo de aplicação**.  

## Evidência do estado atual de main

- `src/bin/server.rs` define `TransportMode` com `stdio`, `http`, `both`, e `grpc` (feature-gated).  
- `src/mcp/http_transport.rs` expõe `POST /mcp` e `GET /v1/events` (SSE), sem rotas de domínio REST como `POST /v1/memories` no código atual.  
- Não há rotas locais `src/bin/server.rs`/`src/mcp` para contrato REST de CRUD de memória; a exposição HTTP é transporte JSON-RPC MCP.  
- `README.md` e `docs/USING_ENGRAM_IN_A_REPO.md` tratam o “HTTP” como forma de acesso ao MCP (`/mcp`) e, em alguns trechos, mencionam “REST” em linguagem de uso; isso precisa alinhamento para não induzir confusão.

## Tomada de decisão

1. **Manter MCP como superfície de produto canônica no OSS local**:
   - MCP fica como interface funcional principal para automação (agent-first).
   - HTTP disponível como transporte streamable para MCP em `/v1/mcp` + SSE `GET /v1/events`.
2. **Não prometer API REST CRUD local de domínio no OSS até existir plano e implementação dedicados** para memória/sessão/busca.
3. **Alinhar SDKs/documentação para não depender de um contrato REST local inexistente** (passo do próprio issue #28).
4. **Condição de aceitação pós-decisão**: qualquer caminho HTTP/SDK fora de MCP deve ser explicitamente marcado como não suportado/localmente indisponível no scope atual.

## Ação obrigatória derivada para issues seguintes

- Ajustar documentação de onboarding e integração para esclarecer:
  - o que é endpoint de transporte (`/mcp`, `--transport http`)
  - o que ainda não é REST CRUD local
- Manter rastreabilidade com este documento e com `docs/harness/decisions/` antes de codificar mudança de contratos.

## Resultado

- Decisão registrada.  
- Próximo: Fase 2.2 com decisões #29/26/31/32, e só então iniciar implementação de contratos/hardening (#34–#37 etc.).
