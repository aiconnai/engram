#!/usr/bin/env bash
# docs/harness/bin/harness-stats.sh
#
# Read-only measurement analytics for `.sensors-log` (Wave 0+ artifacts).

set -euo pipefail

SENSORS_LOG_PATH="${HARNESS_SENSORS_LOG_PATH:-docs/harness/.sensors-log}"
WINDOW=30
JSON_MODE=0

usage() {
  cat <<'USAGE'
Usage:
  harness-stats.sh [--file docs/harness/.sensors-log] [--window N]
  harness-stats.sh --json [--file ...] [--window N]

Options:
  --file path    Path to sensors log (default: docs/harness/.sensors-log)
  --window N     Number of latest runs to include (default: 30, 0=all)
  --json         Emit harness JSON v1 object on stdout
  --help         Show this help
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --file)
      SENSORS_LOG_PATH="$2"
      shift 2
      ;;
    --window)
      WINDOW="$2"
      shift 2
      ;;
    --json)
      JSON_MODE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown arg: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if ! [[ "$WINDOW" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --window must be a non-negative integer" >&2
  exit 2
fi

if [ ! -f "$SENSORS_LOG_PATH" ]; then
  if [ "$JSON_MODE" -eq 1 ]; then
    if ! command -v python3 >/dev/null 2>&1; then
      echo "ERROR: python3 required for --json" >&2
      exit 2
    fi
    python3 - "$SENSORS_LOG_PATH" <<'PY'
import json
import sys
from datetime import datetime, timezone

path = sys.argv[1]
payload = {
    "schema_version": "harness-json-v1",
    "tool": "harness-stats",
    "mode": "json",
    "status": "warn",
    "exit_code": 0,
    "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "summary": "harness sensor stats unavailable: .sensors-log not found",
    "warnings": [f"sensors log missing: {path}"],
    "failures": [],
    "checks": [{"id": "sensors_log:exists", "status": "warn", "message": f"sensors log missing: {path}"}],
    "artifacts": [{"path": path, "kind": "sensors_log", "format": "jsonl"}],
    "metrics": {
        "window": 30,
        "total_entries": 0,
        "window_runs": 0,
        "status_counts": {"pass": 0, "pass_with_exclusion": 0, "fail": 0, "other": 0},
    },
}
print(json.dumps(payload, ensure_ascii=False, separators=(",", ":")))
PY
    exit 0
  fi
  echo "WARN: no sensors log at $SENSORS_LOG_PATH"
  echo "Hint: run `bash docs/harness/bin/sensors.sh` once to create it."
  exit 0
fi

if [ "$JSON_MODE" -eq 1 ] && ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: python3 required for --json" >&2
  exit 2
fi

run_json() {
python3 - "$SENSORS_LOG_PATH" "$WINDOW" <<'PY'
import json
import sys
from collections import Counter, defaultdict, deque
from datetime import datetime, timezone
from pathlib import Path

path = Path(sys.argv[1])
window = int(sys.argv[2])

required = {
    "schema_version",
    "timestamp",
    "tool",
    "mode",
    "status",
    "duration_sec",
    "ci_status",
    "doctor_status",
    "ci_command",
    "artifacts",
}

allowed_statuses = {"pass", "pass_with_exclusion", "fail"}

items = []
parse_errors = 0
raw_lines = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]

for index, raw in enumerate(raw_lines, start=1):
    try:
        parsed = json.loads(raw)
    except Exception:
        parse_errors += 1
        continue
    if not isinstance(parsed, dict):
        parse_errors += 1
        continue
    if parsed.get("schema_version") != "sensors-log-v1":
        parse_errors += 1
        continue
    if parsed.get("tool") != "sensors":
        parse_errors += 1
        continue
    status = parsed.get("status")
    if status not in allowed_statuses:
        parse_errors += 1
        continue
    if parsed.get("duration_sec") is None or not isinstance(parsed.get("duration_sec"), int) or parsed.get("duration_sec") < 0:
        parse_errors += 1
        continue
    if not required.issubset(parsed.keys()):
        parse_errors += 1
        continue
    items.append(parsed)

items.sort(key=lambda item: item.get("timestamp", ""))
windowed = items[-window:] if window > 0 else items[:]

status_counts = Counter()
ci_counts = Counter()
doctor_counts = Counter()
mode_stats = defaultdict(lambda: {"runs": 0, "pass": 0, "pass_with_exclusion": 0, "fail": 0, "other": 0, "duration_sum": 0})
mode_recent = defaultdict(lambda: deque(maxlen=2))

for item in windowed:
    mode = item.get("mode") or "unknown"
    status = item.get("status") or "unknown"
    ci_status = item.get("ci_status") or "unknown"
    doctor_status = item.get("doctor_status") or "unknown"
    duration = item.get("duration_sec")

    status_counts[status] += 1
    ci_counts[ci_status] += 1
    doctor_counts[doctor_status] += 1

    bucket = mode_stats[mode]
    bucket["runs"] += 1
    if status == "pass":
        bucket["pass"] += 1
    elif status == "pass_with_exclusion":
        bucket["pass_with_exclusion"] += 1
    elif status == "fail":
        bucket["fail"] += 1
    else:
        bucket["other"] += 1

    if isinstance(duration, int) and duration >= 0:
        bucket["duration_sum"] += duration

    mode_recent[mode].append({"status": status, "timestamp": item.get("timestamp", "")})

for bucket in mode_stats.values():
    bucket["avg_duration_sec"] = bucket.pop("duration_sum") / bucket["runs"] if bucket["runs"] else 0

total_runs = len(windowed)
pass_like = status_counts.get("pass", 0) + status_counts.get("pass_with_exclusion", 0)
pass_like_rate = (pass_like / total_runs) * 100.0 if total_runs else 0.0

last_entry = None
if windowed:
    last = windowed[-1]
    last_entry = {
        "mode": last.get("mode"),
        "status": last.get("status"),
        "timestamp": last.get("timestamp"),
        "ci_status": last.get("ci_status"),
        "doctor_status": last.get("doctor_status"),
        "duration_sec": last.get("duration_sec"),
    }

flaky_modes = {}
for mode, seq in mode_recent.items():
    if len(seq) == 2 and seq[0]["status"] == "pass" and seq[1]["status"] != "pass":
        flaky_modes[mode] = {
            "from": seq[0]["status"],
            "to": seq[1]["status"],
            "previous_status_ts": seq[0]["timestamp"],
            "latest_status_ts": seq[1]["timestamp"],
        }

status = "pass"
if total_runs == 0:
    status = "warn"
if status_counts.get("fail", 0) > 0:
    status = "warn"

payload = {
    "schema_version": "harness-json-v1",
    "tool": "harness-stats",
    "mode": "json",
    "status": status,
    "exit_code": 0,
    "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "summary": "harness sensor metrics computed",
    "warnings": [] if parse_errors == 0 else [f"{parse_errors} malformed .sensors-log line(s) skipped"],
    "failures": [],
    "checks": [
        {"id": "sensors_log:exists", "status": "pass", "message": f"sensors log exists: {path}"},
        {"id": "sensors_log:parse", "status": "warn" if parse_errors else "pass", "message": f"{parse_errors} malformed .sensors-log line(s) skipped"},
    ],
    "artifacts": [{"path": str(path), "kind": "sensors_log", "format": "jsonl"}],
    "metrics": {
        "window": window,
        "total_entries": len(items),
        "window_runs": total_runs,
        "status_counts": {
            "pass": status_counts.get("pass", 0),
            "pass_with_exclusion": status_counts.get("pass_with_exclusion", 0),
            "fail": status_counts.get("fail", 0),
            "other": status_counts.get("other", 0) + status_counts.get("unknown", 0),
        },
        "pass_like_rate": round(pass_like_rate, 2),
        "ci_status_counts": dict(ci_counts),
        "doctor_status_counts": dict(doctor_counts),
        "mode_stats": dict(mode_stats),
        "flaky_modes": flaky_modes,
        "last_entry": last_entry,
    },
}

print(json.dumps(payload, ensure_ascii=False))
PY
}

run_text() {
python3 - "$SENSORS_LOG_PATH" "$WINDOW" <<'PY'
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

path = Path(sys.argv[1])
window = int(sys.argv[2])
required = {
    "schema_version",
    "timestamp",
    "tool",
    "mode",
    "status",
    "duration_sec",
    "ci_status",
    "doctor_status",
}

items = []
parse_errors = 0
for raw in path.read_text(encoding="utf-8").splitlines():
    raw = raw.strip()
    if not raw:
        continue
    try:
        parsed = json.loads(raw)
    except Exception:
        parse_errors += 1
        continue
    if not isinstance(parsed, dict):
        parse_errors += 1
        continue
    if parsed.get("schema_version") != "sensors-log-v1" or parsed.get("tool") != "sensors":
        parse_errors += 1
        continue
    if not required.issubset(parsed.keys()):
        parse_errors += 1
        continue
    if parsed.get("status") not in ("pass", "pass_with_exclusion", "fail"):
        parse_errors += 1
        continue
    items.append(parsed)

items.sort(key=lambda item: item.get("timestamp", ""))
if window > 0:
    items = items[-window:]

total = len(items)
status_counts = Counter()
mode_counts = defaultdict(int)
duration_sum = defaultdict(int)
pass_like_counts = defaultdict(int)

for item in items:
    status = item.get("status", "unknown")
    mode = item.get("mode", "unknown")
    status_counts[status] += 1
    mode_counts[mode] += 1
    if isinstance(item.get("duration_sec"), int):
        duration_sum[mode] += item.get("duration_sec")
    if status in ("pass", "pass_with_exclusion"):
        pass_like_counts[mode] += 1

print("harness-stats")
print(f"source: {path}")
print(f"window: {window if window > 0 else 'all'}")
print(f"runs: {total}")
if parse_errors:
    print(f"malformed lines skipped: {parse_errors}")
if total == 0:
    print("no data in selected window")
    raise SystemExit(0)

pass_like = status_counts.get("pass", 0) + status_counts.get("pass_with_exclusion", 0)
print(f"overall pass-like: {pass_like}/{total} ({pass_like * 100.0 / total:.1f}%)")
for key in ("pass", "pass_with_exclusion", "fail", "unknown"):
    print(f"  {key}: {status_counts.get(key, 0)}")

print("by mode:")
for name in sorted(mode_counts):
    runs = mode_counts[name]
    avg = duration_sum[name] / runs if runs else 0
    rate = (pass_like_counts[name] / runs) * 100.0 if runs else 0
    print(f"  {name}: runs={runs} pass_like_rate={rate:.1f}% avg_duration_sec={avg:.1f}")

last = items[-1]
print("last entry:")
print(f"  mode={last.get('mode')} status={last.get('status')} ci={last.get('ci_status')} doctor={last.get('doctor_status')} timestamp={last.get('timestamp')}")
PY
}

if [ "$JSON_MODE" -eq 1 ]; then
  run_json
else
  run_text
fi
