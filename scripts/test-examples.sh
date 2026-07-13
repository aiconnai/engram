#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PYTHONDONTWRITEBYTECODE=1

if [[ "${1:-}" == "--self-test-broken-example" ]]; then
  output="$(EXAMPLES_SELF_TEST_BROKEN=langgraph-tool bash "$0" 2>&1)"
  status=$?
  if [[ $status -eq 0 || "$output" != *"langgraph-tool"* ]]; then
    printf 'self-test failed: aggregate did not report the broken langgraph-tool family\n' >&2
    exit 1
  fi
  printf 'self-test passed: broken family was named and rejected\n'
  exit 0
fi

tmp="$(mktemp -d "${TMPDIR:-/tmp}/engram-examples.XXXXXX")"
server_pid=""
cleanup() {
  [[ -z "$server_pid" ]] || kill "$server_pid" 2>/dev/null || true
  [[ -z "$server_pid" ]] || wait "$server_pid" 2>/dev/null || true
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

failures=()
run_family() {
  local family="$1"
  shift
  printf '[examples] %-22s ' "$family"
  if [[ "${EXAMPLES_SELF_TEST_BROKEN:-}" == "$family" ]]; then
    false
  elif "$@" >"$tmp/$family.log" 2>&1; then
    printf 'PASS\n'
    return
  fi
  printf 'FAIL\n' >&2
  sed 's/^/  /' "$tmp/$family.log" >&2 2>/dev/null || true
  failures+=("$family")
}

python_payload_examples() {
  python3 - <<'PY'
from pathlib import Path
for name in (
    "examples/openai-agents-sdk/agent_memory_tool.py",
    "examples/fastmcp-server/server.py",
    "examples/langgraph-tool/memory_graph.py",
):
    compile(Path(name).read_text(), name, "exec")
PY
  python3 examples/openai-agents-sdk/agent_memory_tool.py >/dev/null
  python3 examples/fastmcp-server/server.py >/dev/null
  python3 examples/langgraph-tool/memory_graph.py >/dev/null
}

cursor_config() {
  python3 - <<'PY'
import json, pathlib, re
text = pathlib.Path("examples/cursor-mcp/README.md").read_text()
blocks = re.findall(r"```json\n(.*?)\n```", text, re.S)
assert blocks, "missing Cursor JSON configuration"
config = json.loads(blocks[0])
assert config["mcpServers"]["engram"]["args"] == ["--transport", "stdio"]
PY
}

crewai_adapter() {
  python3 - <<'PY'
from pathlib import Path
name = "sdks/python/engram_client/integrations/crewai.py"
compile(Path(name).read_text(), name, "exec")
PY
  grep -q 'class EngramShortTermMemory' sdks/python/engram_client/integrations/crewai.py
  grep -q 'class EngramLongTermMemory' sdks/python/engram_client/integrations/crewai.py
  grep -q 'class EngramEntityMemory' sdks/python/engram_client/integrations/crewai.py
}

run_family rust-library-demos cargo check --quiet --examples
run_family cursor-mcp cursor_config
run_family crewai-memory crewai_adapter
run_family python-patterns python_payload_examples

# The network examples share one real, loopback-only server and disposable DB.
if cargo build --quiet --bin engram-server; then
  port="$(python3 - <<'PY'
import socket
s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()
PY
)"
  token="examples-smoke-token"
  ENGRAM_DB_PATH="$tmp/examples.db" ENGRAM_EMBEDDING_MODEL=tfidf \
    ENGRAM_HTTP_API_KEY="$token" target/debug/engram-server \
    --transport http --http-port "$port" >"$tmp/server.log" 2>&1 &
  server_pid=$!
  ready=0
  for _ in $(seq 1 100); do
    if curl -fsS "http://127.0.0.1:$port/health" >/dev/null 2>&1; then ready=1; break; fi
    kill -0 "$server_pid" 2>/dev/null || break
    sleep 0.1
  done
  if [[ $ready -eq 1 ]]; then
    export ENGRAM_URL="http://127.0.0.1:$port/mcp" ENGRAM_HTTP_API_KEY="$token"
    run_family claude-mcp bash examples/claude-mcp/seed-and-search.sh
    for family in openai-agents-sdk fastmcp-server langgraph-tool; do
      file="examples/$family/$(case "$family" in openai-agents-sdk) echo agent_memory_tool.py;; fastmcp-server) echo server.py;; *) echo memory_graph.py;; esac)"
      run_family "$family" python3 scripts/smoke-example-module.py "$file"
    done
  else
    failures+=("real-http-server")
    printf '[examples] real-http-server       FAIL\n' >&2
    sed 's/^/  /' "$tmp/server.log" >&2
  fi
else
  failures+=("real-http-server")
fi

if ((${#failures[@]})); then
  printf 'example smoke failures: %s\n' "${failures[*]}" >&2
  exit 1
fi
printf 'All example families passed without external credentials or services.\n'
