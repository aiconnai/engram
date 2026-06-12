# OpenAI Agents SDK Memory Pattern

Engram does not currently ship a native OpenAI Agents SDK adapter. Use this
pattern when your agent can call an HTTP tool or lifecycle hook.

## Run Engram over HTTP

```bash
ENGRAM_HTTP_API_KEY="change-me" engram-server --transport http --http-port 8080
```

## Runnable Example

Dry-run without an OpenAI key, Python dependencies, or a running Engram server:

```bash
python examples/openai-agents-sdk/agent_memory_tool.py
```

Run the live OpenAI Agents SDK example after starting Engram HTTP transport and
setting `OPENAI_API_KEY`:

```bash
ENGRAM_HTTP_API_KEY=change-me \
ENGRAM_URL=http://localhost:8080/mcp \
uv run examples/openai-agents-sdk/agent_memory_tool.py --live
```

The script exposes two OpenAI Agents SDK function tools:

- `remember_project_decision` calls Engram `memory_create`.
- `search_project_memory` calls Engram `memory_search`.

## What to Adapt

- Change `DEFAULT_WORKSPACE` in `agent_memory_tool.py` for each product, tenant,
  or agent boundary.
- Keep Engram failures visible. The example raises `EngramCallError` instead of
  returning fake success.
- Keep OpenAI Assistants API thread sync separate; that native adapter lives in
  `sdks/python/engram_client/integrations/openai_threads.py`.

## Limitation

Wire these calls into your own OpenAI Agents SDK tool or lifecycle hook. This is
an integration pattern, not native SDK support.

See [OpenAI Agents memory guide](../../docs/openai-agents-memory.md).
