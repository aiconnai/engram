# MCP HTTP Authentication

Engram's HTTP MCP transport accepts JSON-RPC 2.0 requests at:

- `POST /mcp` (canonical local endpoint)
- `POST /v1/mcp` (compatibility alias)
- `GET /v1/events` (SSE event stream)

## Server Configuration

Start the HTTP transport with a bearer token:

```bash
engram-server \
  --transport http \
  --http-port 3000 \
  --http-api-key "$ENGRAM_HTTP_API_KEY"
```

The same setting is available through the `ENGRAM_HTTP_API_KEY` environment
variable. When no HTTP API key is configured, local HTTP MCP access is open.

For gRPC, use `--grpc-api-key` or `ENGRAM_GRPC_API_KEY`; the token is sent in
gRPC metadata as `authorization: Bearer <token>`.

## Client Requests

HTTP clients must include:

```text
Authorization: Bearer <ENGRAM_HTTP_API_KEY>
Content-Type: application/json
```

Example:

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

If a configured token is missing or wrong, `POST /mcp` and `POST /v1/mcp`
return `401 Unauthorized` with a JSON-RPC error response (`code=-32001`) and
message `Unauthorized`. `GET /v1/events` returns `401 Unauthorized`.

## Rate limiting (HTTP MCP)

Engram can enforce a token-bucket rate limit for MCP HTTP requests:

- `--http-rate-limit-rps` / `ENGRAM_HTTP_RATE_LIMIT_RPS` (default: `120`)
- `--http-rate-limit-burst` / `ENGRAM_HTTP_RATE_LIMIT_BURST` (default: `240`)
- `--http-rate-limit-key` / `ENGRAM_HTTP_RATE_LIMIT_KEY` (optional identity header)

When the key is unset, bucket keys are derived from `x-forwarded-for`,
then `x-real-ip`, then `ip:unknown`.

When a key header is set, its value is used as the bucket identity key.

If the limit is exceeded, the server returns `429 Too Many Requests` with
`Retry-After: 1` and a JSON-RPC error response (`code=-32005`) and message
`Too Many Requests`.

## Browser Clients

By default, CORS allows localhost origins only. Set `ENGRAM_CORS_ORIGINS` to a
comma-separated allowlist for browser clients:

```bash
ENGRAM_CORS_ORIGINS="https://app.example.com,https://admin.example.com"
```

Use `ENGRAM_CORS_ORIGINS="*"` only for explicitly open deployments.
