#!/usr/bin/env bash
set -euo pipefail

ENGRAM_URL="${ENGRAM_URL:-http://localhost:8080/mcp}"
ENGRAM_HTTP_API_KEY="${ENGRAM_HTTP_API_KEY:-dev-engram-token}"
WORKSPACE="${ENGRAM_WORKSPACE:-claude-mcp-example}"

curl -fsS -X POST "$ENGRAM_URL" \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"tools/call\",
    \"params\": {
      \"name\": \"memory_create\",
      \"arguments\": {
        \"content\": \"Claude MCP example: use Engram for durable project decisions.\",
        \"memory_type\": \"decision\",
        \"workspace\": \"$WORKSPACE\",
        \"tags\": [\"claude-code\", \"mcp\", \"example\"]
      }
    }
  }"

printf '\n\n'

curl -fsS -X POST "$ENGRAM_URL" \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 2,
    \"method\": \"tools/call\",
    \"params\": {
      \"name\": \"memory_search\",
      \"arguments\": {
        \"query\": \"durable project decisions\",
        \"workspace\": \"$WORKSPACE\",
        \"limit\": 5
      }
    }
  }"

printf '\n'
