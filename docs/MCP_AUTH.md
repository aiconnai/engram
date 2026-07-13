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

HTTP auth is evaluated before MCP request rate limiting. Unauthorized requests
do not consume rate-limit tokens and should continue to return `401` even when a
bucket for the same client identity is exhausted.

## HTTP resource bounds

MCP request bodies and the time spent establishing MCP/SSE responses are
bounded by default:

- `ENGRAM_HTTP_MAX_BODY_BYTES` (default: `1048576`, or 1 MiB)
- `ENGRAM_HTTP_REQUEST_TIMEOUT_MS` (default: `30000`, or 30 seconds)

Both settings must be positive integers. Invalid values and `0` fail closed to
the documented safe defaults; there is no unlimited sentinel. Requests larger
than the configured body limit return `413 Payload Too Large` before JSON
parsing. Requests that do not complete within the configured lifetime return
`408 Request Timeout`; MCP responses use JSON-RPC code `-32008` and message
`Request Timeout`.

Bearer authentication runs before request-body collection and parsing, so an
unauthenticated oversized request returns `401`, not `413`. The timeout covers
authentication, body collection, JSON extraction, and asynchronous response
setup. Engram's current MCP handler trait is synchronous; once handler dispatch
begins it cannot be safely preempted, so the deadline prevents slow/incomplete
requests from reaching it rather than claiming unsafe cancellation. For
`GET /v1/events`, the timeout covers authentication and SSE setup only; an
established event stream remains long-lived and is governed by its
keepalive/reconnection behavior. A notification that times out before parsing
cannot yet be identified as a notification and receives the same stable
JSON-RPC `-32008` timeout response; parsed notifications complete with `202`.

## Rate limiting (HTTP MCP)

Engram can enforce a token-bucket rate limit for MCP HTTP requests:

- `--http-rate-limit-rps` / `ENGRAM_HTTP_RATE_LIMIT_RPS` (default: `120`)
- `--http-rate-limit-burst` / `ENGRAM_HTTP_RATE_LIMIT_BURST` (default: `240`)
- `--http-rate-limit-key` / `ENGRAM_HTTP_RATE_LIMIT_KEY` (optional identity header)

When the key is unset, bucket keys use the TCP socket peer address. Forwarded
identity is accepted only when that peer matches a CIDR in
`ENGRAM_HTTP_TRUSTED_PROXIES` (comma-separated IPv4/IPv6 CIDRs). Engram then
normalizes `X-Forwarded-For` from right to left across trusted hops. Malformed,
empty, or overlong chains fall back to the socket peer. `X-Real-IP` is never
trusted implicitly.

Example for a loopback reverse proxy and a private proxy tier:

```bash
ENGRAM_HTTP_TRUSTED_PROXIES="127.0.0.0/8,10.0.0.0/8"
```

Leave the variable unset to disable trusted-proxy mode.

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

## Fly.io deployment validation (auth + rate limit)

For each new Fly.io deployment of `engram-server` with HTTP transport enabled,
run this validation sequence before routing production traffic:

Replace `https://your-fly-app.fly.dev` with your own deployment URL. The
placeholder is not a public Engram service endpoint.

1. **Health and protection state**

```bash
curl -sS https://your-fly-app.fly.dev/health | jq '.protection, .transport.http.mcp_requests_total'
```

2. **Unauthorized access must fail**

```bash
curl -sS -o /tmp/mcp-no-auth.json -w "%{http_code}\n" \
  https://your-fly-app.fly.dev/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

Expect:

- HTTP `401`
- `error.code` is `-32001`

3. **Authorized request succeeds**

```bash
curl -sS -o /tmp/mcp-with-auth.json -w "%{http_code}\n" \
  https://your-fly-app.fly.dev/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

Expect HTTP `200`.

4. **Rate limit is enforced**

Set `ENGRAM_HTTP_RATE_LIMIT_RPS=1` and `ENGRAM_HTTP_RATE_LIMIT_BURST=1` in the
deployment for this check. Then run three quick requests with the same bearer:

```bash
for i in 1 2 3; do
  curl -sS -o /tmp/mcp-rl-$i.json -w "%{http_code} %{time_total}\\n" \
    -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
    https://your-fly-app.fly.dev/mcp
done
```

Expect the third request to return:

- HTTP `429`
- `error.code` is `-32005`
- `Retry-After: 1`

5. **SSE guardrail still protected**

```bash
curl -sS -o /tmp/events-unauth.json -w "%{http_code}\n" \
  "https://your-fly-app.fly.dev/v1/events?workspace=default" \
  -H "Accept: text/event-stream"
```

Expect HTTP `401`.

6. **Confirm metrics are exposed**

```bash
curl -sS https://your-fly-app.fly.dev/health | jq '.transport.http'
```

Check that `mcp_requests_total`, `mcp_rate_limited_total`, and
`events_requests_total` advance during the validation run.
