#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sdk_dir="${repo_root}/sdks/typescript"
mode="happy"

case "${1:-}" in
  "") ;;
  --self-test-wrong-bearer) mode="wrong-bearer" ;;
  --self-test-missing-endpoint) mode="missing-endpoint" ;;
  *)
    echo "usage: $0 [--self-test-wrong-bearer|--self-test-missing-endpoint]" >&2
    exit 2
    ;;
esac

for command_name in cargo node npm python3; do
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    echo "missing required command: ${command_name}" >&2
    exit 2
  fi
done

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/engram-typescript-live.XXXXXX")"
server_pid=""
server_log="${tmp_root}/server.log"
api_key="$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')"

cleanup() {
  local status="$1"
  trap - EXIT INT TERM
  if [[ -n "${server_pid}" ]] && kill -0 "${server_pid}" 2>/dev/null; then
    kill "${server_pid}" 2>/dev/null || true
    wait "${server_pid}" 2>/dev/null || true
  fi
  if [[ -f "${server_log}" ]] && grep -Fq -- "${api_key}" "${server_log}"; then
    echo "server log exposed the live bearer" >&2
    status=1
  fi
  rm -rf "${tmp_root}"
  exit "${status}"
}
trap 'cleanup $?' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

(
  cd "${sdk_dir}"
  npm ci --ignore-scripts --no-audit --no-fund
  npm run type-check
  npm test
  npm run build
  npm pack --dry-run >/dev/null
  npm pack --pack-destination "${tmp_root}" >/dev/null
)

(
  cd "${repo_root}"
  cargo build --bin engram-server
)

port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
db_path="${tmp_root}/engram.db"
ENGRAM_HTTP_API_KEY="${api_key}" \
ENGRAM_TOOL_TIER=all \
ENGRAM_EMBEDDING_MODEL=tfidf \
ENGRAM_CLEANUP_INTERVAL=0 \
ENGRAM_EMBEDDING_DRAIN_INTERVAL=0 \
  "${repo_root}/target/debug/engram-server" \
    --transport http \
    --http-bind-address 127.0.0.1 \
    --http-port "${port}" \
    --db-path "${db_path}" \
    >"${server_log}" 2>&1 &
server_pid=$!

base_url="http://127.0.0.1:${port}"
ready=0
for _ in $(seq 1 100); do
  if ! kill -0 "${server_pid}" 2>/dev/null; then
    echo "engram-server exited before becoming ready" >&2
    sed "s/${api_key}/[REDACTED]/g" "${server_log}" >&2
    exit 1
  fi
  if node -e 'fetch(process.argv[1]).then(r => process.exit(r.ok ? 0 : 1)).catch(() => process.exit(1))' "${base_url}/health"; then
    ready=1
    break
  fi
  sleep 0.1
done
if [[ "${ready}" -ne 1 ]]; then
  echo "engram-server did not become ready" >&2
  exit 1
fi

consumer_dir="${tmp_root}/consumer"
mkdir -p "${consumer_dir}/test"
cp "${sdk_dir}/test/live-client.test.ts" "${consumer_dir}/test/live-client.test.ts"
tarball="$(find "${tmp_root}" -maxdepth 1 -name 'engram-client-*.tgz' -print -quit)"
if [[ -z "${tarball}" ]]; then
  echo "npm pack did not produce an installable tarball" >&2
  exit 1
fi

(
  cd "${consumer_dir}"
  npm init --yes >/dev/null
  npm pkg set type=module >/dev/null
  npm install --ignore-scripts --no-audit --no-fund "${tarball}"
  "${sdk_dir}/node_modules/.bin/tsc" \
    test/live-client.test.ts \
    --target ES2020 \
    --module NodeNext \
    --moduleResolution NodeNext \
    --lib ES2020,DOM \
    --strict \
    --skipLibCheck \
    --outDir compiled

  cat > run-live.mjs <<'EOF'
import { runLiveClientContract } from "./compiled/live-client.test.js";

await runLiveClientContract({
  baseUrl: process.env.ENGRAM_LIVE_BASE_URL,
  apiKey: process.env.ENGRAM_LIVE_CLIENT_KEY,
  tenant: "typescript-live",
  mode: process.env.ENGRAM_LIVE_MODE,
  marker: process.env.ENGRAM_LIVE_MARKER,
});
EOF

  client_key="${api_key}"
  client_url="${base_url}"
  if [[ "${mode}" == "wrong-bearer" ]]; then
    client_key="${api_key}-wrong"
  elif [[ "${mode}" == "missing-endpoint" ]]; then
    client_url="${base_url}/missing"
  fi

  ENGRAM_LIVE_BASE_URL="${client_url}" \
  ENGRAM_LIVE_CLIENT_KEY="${client_key}" \
  ENGRAM_LIVE_MODE="${mode}" \
  ENGRAM_LIVE_MARKER="$(python3 -c 'import secrets; print(secrets.token_hex(8))')" \
    node run-live.mjs
)

echo "TypeScript SDK packed live contract (${mode}): PASS"
