#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
artifact_dir=""
expected_sha=""
run_id=""
run_live=false
channel="all"
mode="verify"
manifest=""
tmp=""
server_pid=""

die() { echo "verify-sdk-artifacts: $*" >&2; exit 1; }
cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  [[ -z "$tmp" ]] || rm -rf "$tmp"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir) artifact_dir=${2:-}; shift 2 ;;
    --expected-sha) expected_sha=${2:-}; shift 2 ;;
    --manifest) manifest=${2:-}; shift 2 ;;
    --run-id) run_id=${2:-}; shift 2 ;;
    --channel) channel=${2:-}; shift 2 ;;
    --write-metadata) mode="write"; channel=${2:-}; shift 2 ;;
    --live) run_live=true; shift ;;
    --self-test-version-mismatch)
      tmp="$(mktemp -d "${TMPDIR:-/tmp}/engram-sdk-mismatch.XXXXXX")"
      python3 -c '
import io,json,pathlib,tarfile,zipfile,sys
root=pathlib.Path(sys.argv[1]); sha="0"*40
(root/"python").mkdir(); (root/"npm").mkdir()
with zipfile.ZipFile(root/"python/engram_client-0.5.0-py3-none-any.whl","w") as z: z.writestr("engram_client-0.5.0.dist-info/METADATA","Name: engram-client\nVersion: 0.5.0\n")
with tarfile.open(root/"python/engram_client-0.5.0.tar.gz","w:gz") as t:
 data=b"[project]\nname=\"engram-client\"\nversion=\"0.5.0\"\n"; info=tarfile.TarInfo("engram_client-0.5.0/pyproject.toml"); info.size=len(data); t.addfile(info,io.BytesIO(data))
with tarfile.open(root/"npm/engram-client-0.5.0.tgz","w:gz") as t:
 for name,data in [("package/package.json",b"{\"name\":\"engram-client\",\"version\":\"0.5.0\"}"),("package/dist/index.js",b"export {}"),("package/dist/index.d.ts",b"export {}")]:
  info=tarfile.TarInfo(name); info.size=len(data); t.addfile(info,io.BytesIO(data))
base={"repository":"aiconnai/engram","package":"engram-client","sha":sha,"compatible_core_min":"0.20.0","compatible_core_max":"0.22.x","files":[]}
(root/"python/python-sdk-metadata.json").write_text(json.dumps(base|{"channel":"pypi","version":"9.9.9"}))
(root/"npm/npm-sdk-metadata.json").write_text(json.dumps(base|{"channel":"npm","version":"0.5.0"}))
' "$tmp"
      set +e
      "$0" --artifact-dir "$tmp" --expected-sha "$(printf '0%.0s' {1..40})" >/dev/null 2>&1
      status=$?
      set -e
      [[ $status -ne 0 ]] || die "version mismatch fixture was accepted"
      echo "verify-sdk-artifacts version mismatch self-test: PASS"
      rm -rf "$tmp"
      exit 0
      ;;
    *) die "unknown argument: $1" ;;
  esac
done

if [[ -n "$manifest" ]]; then
  [[ -f "$manifest" ]] || die "manifest does not exist: $manifest"
  manifest_sha="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["sha"])' "$manifest")"
  [[ -z "$expected_sha" || "$expected_sha" == "$manifest_sha" ]] || die "manifest SHA conflicts with --expected-sha"
  expected_sha="$manifest_sha"
fi

if [[ "$mode" == write ]]; then
  [[ "$channel" =~ ^(python|npm)$ ]] || die "--write-metadata requires python or npm"
  [[ -n "$artifact_dir" && -d "$artifact_dir" ]] || die "--artifact-dir is required"
  [[ "$expected_sha" =~ ^[0-9a-f]{40}$ ]] || die "--expected-sha must be a full lowercase SHA"
  python3 -c '
import hashlib,json,pathlib,sys,tomllib
root=pathlib.Path(sys.argv[1]); out=pathlib.Path(sys.argv[2]); sha=sys.argv[3]; sdk=sys.argv[4]
matrix=tomllib.loads((root/"docs/releases/channel-matrix.toml").read_text())
channel_id="pypi" if sdk=="python" else "npm"
channel=next(item for item in matrix["channels"] if item["id"]==channel_id)
compat=next(item for item in matrix["sdk_compatibility"] if item["sdk"]==sdk)
files=[{"name":path.name,"sha256":hashlib.sha256(path.read_bytes()).hexdigest()} for path in sorted(out.iterdir())]
metadata={"repository":"aiconnai/engram","channel":channel_id,"package":channel["package"],"version":channel["local_version"],"sha":sha,"compatible_core_min":compat["compatible_core_min"],"compatible_core_max":compat["compatible_core_max"],"files":files}
(out/("python-sdk-metadata.json" if sdk=="python" else "npm-sdk-metadata.json")).write_text(json.dumps(metadata,sort_keys=True,indent=2)+"\n")
' "$repo_root" "$artifact_dir" "$expected_sha" "$channel"
  echo "verify-sdk-artifacts metadata: PASS ($channel)"
  exit 0
fi

if [[ -n "$run_id" ]]; then
  [[ -z "$artifact_dir" ]] || die "use either --run-id or --artifact-dir"
  command -v gh >/dev/null || die "gh is required for --run-id"
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/engram-sdk-artifacts.XXXXXX")"
  gh run download "$run_id" --dir "$tmp"
  artifact_dir="$tmp"
  if [[ -z "$expected_sha" ]]; then
    expected_sha="$(gh run view "$run_id" --json headSha --jq .headSha)"
  fi
fi

[[ -n "$artifact_dir" && -d "$artifact_dir" ]] || die "--artifact-dir or --run-id is required"
[[ -n "$expected_sha" ]] || expected_sha="$(git -C "$repo_root" rev-parse HEAD)"
[[ "$expected_sha" =~ ^[0-9a-f]{40}$ ]] || die "expected SHA must be a full lowercase commit SHA"
[[ "$channel" =~ ^(all|python|npm)$ ]] || die "--channel must be all, python, or npm"

[[ -n "$tmp" ]] || tmp="$(mktemp -d "${TMPDIR:-/tmp}/engram-sdk-install.XXXXXX")"
trap cleanup EXIT
python3 -c '
import hashlib
import json
import pathlib
import re
import sys
import tarfile
import tomllib
import zipfile

root = pathlib.Path(sys.argv[1])
artifacts = pathlib.Path(sys.argv[2])
expected_sha = sys.argv[3]
selected = sys.argv[4]
matrix = tomllib.loads((root / "docs/releases/channel-matrix.toml").read_text())
channels = {item["id"]: item for item in matrix["channels"]}
compat = {item["sdk"]: item for item in matrix["sdk_compatibility"]}

def exactly(pattern: str) -> pathlib.Path:
    matches = sorted(artifacts.rglob(pattern))
    if len(matches) != 1:
        raise SystemExit(f"expected exactly one {pattern}, found {len(matches)}")
    return matches[0]

def digest(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def validate_metadata(path: pathlib.Path, channel: str, version: str, expected_files: set[str]) -> None:
    value = json.loads(path.read_text())
    expected = {
        "repository": "aiconnai/engram",
        "channel": channel,
        "package": "engram-client",
        "version": version,
        "sha": expected_sha,
        "compatible_core_min": compat["python" if channel == "pypi" else "typescript"]["compatible_core_min"],
        "compatible_core_max": compat["python" if channel == "pypi" else "typescript"]["compatible_core_max"],
    }
    for key, expected_value in expected.items():
        if value.get(key) != expected_value:
            raise SystemExit(f"{channel} metadata mismatch for {key}: {value.get(key)!r} != {expected_value!r}")
    items = value.get("files")
    if not isinstance(items, list) or len(items) != len(expected_files):
        raise SystemExit(f"{channel} metadata must bind exactly {sorted(expected_files)}")
    names = [item.get("name") for item in items if isinstance(item, dict)]
    if len(names) != len(set(names)) or set(names) != expected_files:
        raise SystemExit(f"{channel} metadata file set mismatch: {names}")
    for item in items:
        if not isinstance(item.get("sha256"), str) or re.fullmatch(r"[0-9a-f]{64}", item["sha256"]) is None:
            raise SystemExit(f"{channel} invalid SHA-256 metadata")
        candidate = path.parent / item["name"]
        if not candidate.is_file() or digest(candidate) != item["sha256"]:
            raise SystemExit(f"{channel} checksum mismatch: {candidate.name}")

verified = []
if selected in {"all", "python"}:
    python_version = channels["pypi"]["local_version"]
    wheel = exactly("*.whl")
    sdist = exactly("*.tar.gz")
    python_meta = exactly("python-sdk-metadata.json")
    validate_metadata(python_meta, "pypi", python_version, {wheel.name, sdist.name})
    with zipfile.ZipFile(wheel) as archive:
        metadata_names = [name for name in archive.namelist() if name.endswith(".dist-info/METADATA")]
        if len(metadata_names) != 1:
            raise SystemExit("wheel must contain exactly one METADATA file")
        metadata = archive.read(metadata_names[0]).decode()
        if f"Name: engram-client\n" not in metadata or f"Version: {python_version}\n" not in metadata:
            raise SystemExit("wheel metadata does not match channel matrix")
    with tarfile.open(sdist, "r:gz") as archive:
        pyprojects = [member for member in archive.getmembers() if member.name.endswith("/pyproject.toml")]
        if len(pyprojects) != 1:
            raise SystemExit("sdist must contain exactly one pyproject.toml")
        project = tomllib.loads(archive.extractfile(pyprojects[0]).read().decode())["project"]
        if project.get("name") != "engram-client" or project.get("version") != python_version:
            raise SystemExit("sdist metadata does not match channel matrix")
    verified.append(f"Python {python_version}")
if selected in {"all", "npm"}:
    npm_version = channels["npm"]["local_version"]
    npm = exactly("*.tgz")
    npm_meta = exactly("npm-sdk-metadata.json")
    validate_metadata(npm_meta, "npm", npm_version, {npm.name})
    with tarfile.open(npm, "r:gz") as archive:
        package_member = archive.getmember("package/package.json")
        package = json.loads(archive.extractfile(package_member).read())
        if package.get("name") != "engram-client" or package.get("version") != npm_version:
            raise SystemExit("npm tarball metadata does not match channel matrix")
        files = {member.name for member in archive.getmembers()}
        if "package/dist/index.js" not in files or "package/dist/index.d.ts" not in files:
            raise SystemExit("npm tarball is missing compiled entry points")
    verified.append(f"npm {npm_version}")
print("SDK artifacts verified for {}: {}".format(expected_sha, ", ".join(verified)))
' "$repo_root" "$artifact_dir" "$expected_sha" "$channel"

if [[ "$channel" == all || "$channel" == python ]]; then
  wheel="$(find "$artifact_dir" -type f -name '*.whl' -print -quit)"
  python3 -m venv "$tmp/venv"
  "$tmp/venv/bin/python" -m pip install --quiet "$wheel"
  "$tmp/venv/bin/python" -I -c 'import engram_client; assert engram_client.__file__'
fi
if [[ "$channel" == all || "$channel" == npm ]]; then
  tarball="$(find "$artifact_dir" -type f -name '*.tgz' -print -quit)"
  mkdir -p "$tmp/npm-consumer"
  (
    cd "$tmp/npm-consumer"
    npm init --yes >/dev/null
    npm install --ignore-scripts --no-audit --no-fund "$tarball" >/dev/null
    node --input-type=module -e 'import("engram-client").then(m => { if (!m.EngramClient) process.exit(1) })'
  )
fi

if [[ "$run_live" == true ]]; then
  (cd "$repo_root" && cargo build --quiet --bin engram-server --features dream-phase)
  port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
  api_key="$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')"
  ENGRAM_HTTP_API_KEY="$api_key" ENGRAM_EMBEDDING_MODEL=tfidf \
    "$repo_root/target/debug/engram-server" --transport http \
    --http-bind-address 127.0.0.1 --http-port "$port" \
    --db-path "$tmp/sdk-live.db" >"$tmp/server.log" 2>&1 &
  server_pid=$!
  ready=false
  for _ in $(seq 1 200); do
    if curl --fail --silent "http://127.0.0.1:$port/health" >/dev/null; then ready=true; break; fi
    kill -0 "$server_pid" 2>/dev/null || break
    sleep 0.1
  done
  [[ "$ready" == true ]] || die "artifact-backed server did not become ready"

  if [[ "$channel" == all || "$channel" == python ]]; then
    "$tmp/venv/bin/python" -m pip install --quiet pytest pytest-asyncio
    ENGRAM_LIVE_BASE_URL="http://127.0.0.1:$port" \
    ENGRAM_LIVE_API_KEY="$api_key" ENGRAM_LIVE_TENANT="python-sdk-artifact" \
    ENGRAM_LIVE_VENV="$tmp/venv" ENGRAM_LIVE_SCENARIO=happy \
      "$tmp/venv/bin/python" -I -m pytest --import-mode=importlib \
      "$repo_root/sdks/python/tests/test_live_client.py" -q
  fi

  if [[ "$channel" == all || "$channel" == npm ]]; then
    (
      cd "$tmp/npm-consumer"
      ENGRAM_LIVE_BASE_URL="http://127.0.0.1:$port" \
      ENGRAM_LIVE_API_KEY="$api_key" node --input-type=module -e '
      import { EngramClient, EngramError } from "engram-client";
      const payload = result => JSON.parse(result.content[0].text);
      const client = new EngramClient({baseUrl: process.env.ENGRAM_LIVE_BASE_URL, apiKey: process.env.ENGRAM_LIVE_API_KEY, tenant: "npm-sdk-artifact", timeout: 5000});
      const created = payload(await client.create("npm artifact live contract", {workspace: "npm-sdk-artifact"}));
      if (payload(await client.get(created.id)).id !== created.id) throw new Error("npm get mismatch");
      if (!JSON.stringify(payload(await client.search("artifact live", {workspace: "npm-sdk-artifact"}))).includes(String(created.id))) throw new Error("npm search mismatch");
      if (payload(await client.update(created.id, {content: "npm artifact updated"})).content !== "npm artifact updated") throw new Error("npm update mismatch");
      if (payload(await client.delete(created.id)).deleted !== created.id) throw new Error("npm delete mismatch");
      const denied = new EngramClient({baseUrl: process.env.ENGRAM_LIVE_BASE_URL, apiKey: process.env.ENGRAM_LIVE_API_KEY + "-wrong", tenant: "npm-sdk-artifact", timeout: 2000});
      try { await denied.stats(); throw new Error("wrong bearer accepted"); } catch (error) { if (!(error instanceof EngramError) || !error.message.includes("HTTP 401")) throw error; }
      '
    )
    echo "verify-sdk-artifacts wrong-auth negative: PASS"
  fi

  for content in "workspace boundary source one" "workspace boundary source two"; do
    curl --fail --silent --show-error -X POST "http://127.0.0.1:$port/mcp" \
      -H "Authorization: Bearer $api_key" \
      -H 'Content-Type: application/json' \
      --data "{\"jsonrpc\":\"2.0\",\"id\":91,\"method\":\"tools/call\",\"params\":{\"name\":\"memory_create\",\"arguments\":{\"content\":\"$content\",\"workspace\":\"default\"}}}" \
      >/dev/null
  done
  curl --fail --silent --show-error -X POST "http://127.0.0.1:$port/mcp" \
    -H "Authorization: Bearer $api_key" \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","id":92,"method":"tools/call","params":{"name":"dream_create","arguments":{"job_id":"sdk-negative-workspace-job","workspace":"default","run":true,"max_candidates":1,"summary_min_memories":2}}}' \
    >"$tmp/dream-create.json"
  dream_candidate_id="$(python3 -c '
import json, pathlib, sys
outer=json.loads(pathlib.Path(sys.argv[1]).read_text())
inner=json.loads(outer["result"]["content"][0]["text"])
ids=inner.get("report",{}).get("candidate_ids",[])
if len(ids) != 1: raise SystemExit(f"expected one dream candidate, got {inner}")
print(ids[0])
' "$tmp/dream-create.json")"
  memory_count_before="$(python3 -c 'import sqlite3,sys; print(sqlite3.connect(sys.argv[1]).execute("SELECT COUNT(*) FROM memories").fetchone()[0])' "$tmp/sdk-live.db")"

  kill "$server_pid"
  wait "$server_pid" 2>/dev/null || true
  server_pid=""
  anonymous_port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
  ENGRAM_EMBEDDING_MODEL=tfidf \
    "$repo_root/target/debug/engram-server" --transport http \
    --http-bind-address 127.0.0.1 --http-port "$anonymous_port" \
    --db-path "$tmp/sdk-live.db" >"$tmp/anonymous-server.log" 2>&1 &
  server_pid=$!
  ready=false
  for _ in $(seq 1 200); do
    if curl --fail --silent "http://127.0.0.1:$anonymous_port/health" >/dev/null; then ready=true; break; fi
    kill -0 "$server_pid" 2>/dev/null || break
    sleep 0.1
  done
  [[ "$ready" == true ]] || die "anonymous workspace-boundary server did not become ready"

  for attempt in get review apply reject; do
    case "$attempt" in
      get) tool="dream_candidate_get"; arguments="{\"id\":\"$dream_candidate_id\",\"workspace\":\"other\"}" ;;
      review) tool="dream_candidate_review"; arguments="{\"id\":\"$dream_candidate_id\",\"review_state\":\"accepted\",\"workspace\":\"other\"}" ;;
      apply) tool="dream_candidate_apply"; arguments="{\"id\":\"$dream_candidate_id\",\"confirm\":true,\"workspace\":\"other\"}" ;;
      reject) tool="dream_candidate_review"; arguments="{\"id\":\"$dream_candidate_id\",\"review_state\":\"rejected\",\"workspace\":\"other\"}" ;;
    esac
    status="$(curl --silent --show-error -o "$tmp/cross-workspace-$attempt.json" -w '%{http_code}' \
      -X POST "http://127.0.0.1:$anonymous_port/mcp" \
      -H 'Content-Type: application/json' \
      --data "{\"jsonrpc\":\"2.0\",\"id\":93,\"method\":\"tools/call\",\"params\":{\"name\":\"$tool\",\"arguments\":$arguments}}")"
    [[ "$status" == 403 ]] || die "cross-workspace dream $attempt returned HTTP $status, expected 403"
    python3 -c '
import json,pathlib,sys
value=json.loads(pathlib.Path(sys.argv[1]).read_text())
outer=value.get("error",{})
try: denial=json.loads(outer.get("message",""))["error"]
except (KeyError,TypeError,json.JSONDecodeError): denial={}
if outer.get("code") != -32003 or denial.get("code") != "permission_denied":
    raise SystemExit(f"cross-workspace dream denial was not stable: {value}")
if sys.argv[2] == "get" and denial.get("current_mode") != denial.get("required_mode"):
    raise SystemExit(f"candidate get was denied for mode rather than workspace: {value}")
' "$tmp/cross-workspace-$attempt.json" "$attempt"
  done
  kill "$server_pid"
  wait "$server_pid" 2>/dev/null || true
  server_pid=""
  python3 -c '
import sqlite3,sys
conn=sqlite3.connect(sys.argv[1]); candidate=sys.argv[2]; before=int(sys.argv[3])
row=conn.execute("SELECT review_state, application_result_json, applied_at FROM dream_candidates WHERE id = ?", (candidate,)).fetchone()
if row != ("pending", None, None): raise SystemExit(f"cross-workspace attempt mutated candidate: {row}")
after=conn.execute("SELECT COUNT(*) FROM memories").fetchone()[0]
if after != before: raise SystemExit(f"cross-workspace attempt mutated memories: {before} -> {after}")
' "$tmp/sdk-live.db" "$dream_candidate_id" "$memory_count_before"
  echo "verify-sdk-artifacts invalid-dream-workspace negative: PASS"
fi

echo "verify-sdk-artifacts: PASS"
