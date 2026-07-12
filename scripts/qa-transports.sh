#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract="$repo_root/tests/fixtures/canonical_journey/contract.json"

case "${1:-}" in
  --self-test-harness)
    rtk python3 - "$contract" <<'PY'
import json
import sys

contract = json.load(open(sys.argv[1], encoding="utf-8"))
required = {
    "initialize-discover",
    "create-get-list-search-update",
    "export-read-back",
    "workspace-isolation",
    "wrong-bearer",
    "clean-shutdown",
}
assert contract["version"] == 1
assert set(contract["transports"]) == {"stdio", "authenticated-http"}
assert required == set(contract["scenarios"])
assert len(contract["required_tools"]) == len(set(contract["required_tools"]))
print("qa-transports harness: PASS")
PY
    ;;
  *)
    echo "usage: $0 --self-test-harness" >&2
    exit 2
    ;;
esac
