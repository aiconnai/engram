#!/usr/bin/env bash
# Miri smoke runner for the nightly lane (task Q3).
#
# Runs each filter in scripts/miri-inventory.txt under `cargo +"$TOOLCHAIN" miri
# test --lib`. A filter that matches zero tests, an empty inventory, a test
# failure, or a missing Miri toolchain yields an explicit non-pass status
# (fail / not-run); the exit code mirrors the overall status
# (pass 0, fail 1, not-run 2, unsupported 3). Nothing is masked.
#
# Environment (optional): MIRI_SMOKE_OUT (report dir), MIRI_INVENTORY (file),
# MIRI_SMOKE_WALL_SECS (per-filter hard cap, default 1800), MIRI_SMOKE_OS,
# MIRI_SMOKE_TOOLCHAIN (rustup toolchain, default `nightly`; CI passes the pinned dated nightly).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${MIRI_SMOKE_OUT:-$ROOT/target/miri-smoke}"
INVENTORY="${MIRI_INVENTORY:-$ROOT/scripts/miri-inventory.txt}"
WALL_SECS="${MIRI_SMOKE_WALL_SECS:-1800}"
OS_NAME="${MIRI_SMOKE_OS:-$(uname -s)}"
TOOLCHAIN="${MIRI_SMOKE_TOOLCHAIN:-nightly}"
REPORTER="$ROOT/scripts/optional_lane_report.py"

rm -rf "$OUT"
mkdir -p "$OUT/logs"
ROWS="$OUT/rows.tsv"
: >"$ROWS"
LANE_ERRORS=()
LANE_STATUS=""

clean() { printf '%s' "$1" | tr '\t\r\n' '   ' | cut -c1-300; }
row() { # row <filter> <status> <stage> <exit> <elapsed_s> <executed> <note>
  printf '%s\t%s\t%s\t%s\t%s\t%s\t\t\t\t%s\n' "$1" "$2" "$3" "$4" "$5" "$6" "$(clean "$7")" >>"$ROWS"
}

finish() {
  local args=(--lane miri --rows "$ROWS" --out "$OUT" --setting "wall_secs=$WALL_SECS" --setting "toolchain=$TOOLCHAIN")
  local error
  for error in ${LANE_ERRORS[@]+"${LANE_ERRORS[@]}"}; do
    args+=(--lane-error "$error")
  done
  [ -n "$LANE_STATUS" ] && args+=(--lane-status "$LANE_STATUS")
  python3 "$REPORTER" "${args[@]}"
  exit $?
}

FILTERS=()
if [ -f "$INVENTORY" ]; then
  while IFS= read -r line; do
    line="${line%%#*}"
    line="$(printf '%s' "$line" | tr -d '[:space:]')"
    [ -n "$line" ] && FILTERS+=("$line")
  done <"$INVENTORY"
fi
if [ "${#FILTERS[@]}" -eq 0 ]; then
  LANE_ERRORS+=("Miri filter inventory is empty or missing: $INVENTORY")
  finish
fi

case "$OS_NAME" in
  Linux | Darwin) ;;
  *)
    LANE_STATUS="unsupported"
    for filter in "${FILTERS[@]}"; do row "$filter" unsupported platform "" "" "" "Miri smoke is not supported on $OS_NAME"; done
    finish
    ;;
esac

if ! cargo +"$TOOLCHAIN" miri --version >"$OUT/logs/miri-version.log" 2>&1; then
  LANE_STATUS="not-run"
  for filter in "${FILTERS[@]}"; do row "$filter" not-run infrastructure "" "" "" "nightly Miri component unavailable (see logs/miri-version.log)"; done
  finish
fi

run_capped() {
  if command -v timeout >/dev/null 2>&1; then
    timeout "$WALL_SECS" "$@"
    return $?
  fi
  if command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$WALL_SECS" "$@"
    return $?
  fi
  local flag="$OUT/.watchdog-fired" pid watchdog status
  rm -f "$flag"
  "$@" &
  pid=$!
  ( sleep "$WALL_SECS"; : >"$flag"; pkill -TERM -P "$pid" 2>/dev/null; kill -TERM "$pid" 2>/dev/null ) &
  watchdog=$!
  wait "$pid"
  status=$?
  kill "$watchdog" 2>/dev/null
  wait "$watchdog" 2>/dev/null
  [ -e "$flag" ] && return 124
  return "$status"
}

index=0
for filter in "${FILTERS[@]}"; do
  index=$((index + 1))
  log="$OUT/logs/filter-$index.log"
  started="$(date +%s)"
  (cd "$ROOT" && run_capped cargo +"$TOOLCHAIN" miri test --lib -- "$filter") >"$log" 2>&1
  code=$?
  elapsed=$(($(date +%s) - started))
  # Sum "running N tests" across the test binaries cargo ran for this filter.
  running="$(sed -n 's/^running \([0-9][0-9]*\) tests\{0,1\}$/\1/p' "$log" | awk '{s+=$1} END {print s+0}')"
  if [ "$code" -eq 124 ]; then
    row "$filter" fail run "$code" "$elapsed" "$running" "hard wall-time cap exceeded"
  elif [ "$code" -ne 0 ]; then
    row "$filter" fail run "$code" "$elapsed" "$running" "miri exited $code (see logs/filter-$index.log)"
  elif [ "$running" -eq 0 ]; then
    row "$filter" fail filter 0 "$elapsed" 0 "filter matched no tests (target not found)"
  else
    row "$filter" pass run 0 "$elapsed" "$running" "$running test(s) passed under Miri"
  fi
done
finish
