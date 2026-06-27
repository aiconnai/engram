# Claude MCP Memory Example

This example configures Engram as a local MCP memory server for Claude Code.

## Run Engram

```bash
cargo install engram-core
mkdir -p .engram
ENGRAM_DB_PATH="$PWD/.engram/memories.db" engram-server --transport stdio
```

For the smoke-test script below, run HTTP transport in another terminal:

```bash
ENGRAM_HTTP_API_KEY=dev-engram-token \
ENGRAM_DB_PATH="$PWD/.engram/memories.db" \
engram-server --transport http --http-port 8080
```

## Claude MCP Config

Use an absolute path for global configs:

```bash
cp examples/claude-mcp/claude-code-mcp.json /tmp/engram-claude-mcp.json
```

Edit `/absolute/path/to/your/repo/.engram/memories.db` in the copied file before
placing it in your Claude Code MCP config.

## Seed and Search

After the HTTP server is running:

```bash
examples/claude-mcp/seed-and-search.sh
```

The script writes one decision with `memory_create`, then searches it with
`memory_search`.

## Suggested Agent Instruction

```markdown
Before major work, search Engram for prior decisions with `memory_search`.
When a durable decision, convention, or recurring gotcha is discovered, store it
with `memory_create`. Do not store secrets or raw command logs.
```

## Limitation

Claude must be configured to use the MCP server. Engram does not force memory
reads or writes by itself.

See also:

- [Claude Code MCP memory guide](../../docs/integrations/claude-code-mcp-memory.md)
- [MCP tools reference](../../docs/MCP_TOOLS.md)
