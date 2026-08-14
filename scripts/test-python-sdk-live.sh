#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

scenario="happy"
case "${1:-}" in
  "") ;;
  --self-test-wrong-bearer) scenario="wrong_bearer" ;;
  --self-test-killed-server) scenario="killed_server" ;;
  *) printf 'usage: %s [--self-test-wrong-bearer|--self-test-killed-server]\n' "$0" >&2; exit 2 ;;
esac

tmp="$(mktemp -d "${TMPDIR:-/tmp}/engram-python-sdk-live.XXXXXX")"
venv="$tmp/venv"
server_pid=""
port=""
token=""

stop_server() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
    server_pid=""
  fi
}

redact_log() {
  if [[ -f "$tmp/server.log" ]]; then
    sed "s/${token:-unused}/<redacted-api-key>/g" "$tmp/server.log" >&2
  fi
}

cleanup() {
  stop_server
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

python3 -m venv "$venv"
python="$venv/bin/python"
"$python" -m pip install --quiet --upgrade pip
"$python" -m pip install --quiet build pytest pytest-asyncio

mkdir -p "$tmp/dist"
cp -R sdks/python "$tmp/python-sdk-source"
(
  cd "$tmp/python-sdk-source"
  "$python" -m build --wheel --outdir "$tmp/dist"
)
wheel=("$tmp"/dist/*.whl)
if [[ ${#wheel[@]} -ne 1 || ! -f "${wheel[0]}" ]]; then
  printf 'expected exactly one Python SDK wheel\n' >&2
  exit 1
fi
"$python" -m pip install --quiet "${wheel[0]}"

cargo build --quiet --bin engram-server
port="$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"
token="$(python3 - <<'PY'
import secrets
print(secrets.token_hex(24))
PY
)"

ENGRAM_DB_PATH="$tmp/live.db" \
ENGRAM_EMBEDDING_MODEL=tfidf \
ENGRAM_HTTP_API_KEY="$token" \
target/debug/engram-server \
  --transport http \
  --http-port "$port" \
  --embedding-model tfidf \
  --cleanup-interval-seconds 0 \
  --embedding-drain-interval-seconds 0 \
  --compression-interval-seconds 0 \
  --ws-port 0 >"$tmp/server.log" 2>&1 &
server_pid=$!

ready=0
for _ in $(seq 1 300); do
  if "$python" - "$port" <<'PY' >/dev/null 2>&1
import sys
import urllib.request
with urllib.request.urlopen(f"http://127.0.0.1:{sys.argv[1]}/health", timeout=0.2) as response:
    assert response.status == 200
PY
  then
    ready=1
    break
  fi
  kill -0 "$server_pid" 2>/dev/null || break
  sleep 0.1
done
if [[ $ready -ne 1 ]]; then
  printf 'real Engram HTTP server did not become ready\n' >&2
  redact_log
  exit 1
fi

client_token="$token"
if [[ "$scenario" == "wrong_bearer" ]]; then
  client_token="${token}wrong"
elif [[ "$scenario" == "killed_server" ]]; then
  stop_server
fi

ENGRAM_LIVE_BASE_URL="http://127.0.0.1:$port" \
ENGRAM_LIVE_API_KEY="$client_token" \
ENGRAM_LIVE_TENANT="python-sdk-live" \
ENGRAM_LIVE_VENV="$venv" \
ENGRAM_LIVE_SCENARIO="$scenario" \
"$python" -I -m pytest --import-mode=importlib sdks/python/tests/test_live_client.py -q

stop_server
"$python" - "$port" <<'PY'
import socket
import sys
with socket.socket() as sock:
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("127.0.0.1", int(sys.argv[1])))
PY

printf 'Python SDK installed-wheel live contract passed (%s).\n' "$scenario"
