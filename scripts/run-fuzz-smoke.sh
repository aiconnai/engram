#!/usr/bin/env bash
# Bounded fuzz smoke runner for the nightly lane (task Q3).
#
# Contract (see scripts/test_optional_lane_contracts.py):
#   * the declared inventory (fuzz/targets.inventory) must be non-empty and
#     equal `cargo fuzz list`, the targets that compile, and the targets that
#     execute; any drift is a lane failure;
#   * each target runs for FUZZ_SMOKE_MAX_TOTAL_TIME seconds (default 60) with an
#     RSS limit and a hard wall-time cap;
#   * every target ends as pass | fail | not-run | unsupported with counts
#     (executed units, corpus files, seed files); a failing target's reproducers
#     are copied to <out>/reproducers/<target>/ and listed in the report;
#   * nothing is masked: no `|| true`, and the exit code mirrors the overall
#     status (pass 0, fail 1, not-run 2, unsupported 3).
#
# Environment (all optional):
#   FUZZ_DIR                      fuzz workspace (default: <repo>/fuzz)
#   FUZZ_SMOKE_OUT                report directory (default: <repo>/target/fuzz-smoke)
#   FUZZ_SMOKE_MAX_TOTAL_TIME     seconds per target (default 60)
#   FUZZ_SMOKE_RSS_LIMIT_MB       libFuzzer -rss_limit_mb (default 2048)
#   FUZZ_SMOKE_UNIT_TIMEOUT       libFuzzer -timeout, seconds per input (default 25)
#   FUZZ_SMOKE_WALL_GRACE         seconds of slack beyond max-total-time (default
#                                 120). A target whose measured run time exceeds
#                                 max-total-time + grace FAILS even with exit 0;
#                                 a hard kill follows 30s later.
#   FUZZ_SMOKE_TOOLCHAIN          rustup toolchain for `cargo +<toolchain>` (default
#                                 `nightly`; CI passes the pinned dated nightly)
#   FUZZ_SMOKE_OS                 override `uname -s` (tests)
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FUZZ_DIR="${FUZZ_DIR:-$ROOT/fuzz}"
OUT="${FUZZ_SMOKE_OUT:-$ROOT/target/fuzz-smoke}"
MAX_TOTAL_TIME="${FUZZ_SMOKE_MAX_TOTAL_TIME:-60}"
RSS_LIMIT_MB="${FUZZ_SMOKE_RSS_LIMIT_MB:-2048}"
UNIT_TIMEOUT="${FUZZ_SMOKE_UNIT_TIMEOUT:-25}"
WALL_GRACE="${FUZZ_SMOKE_WALL_GRACE:-120}"
TOOLCHAIN="${FUZZ_SMOKE_TOOLCHAIN:-nightly}"
KILL_SLACK=30
OS_NAME="${FUZZ_SMOKE_OS:-$(uname -s)}"
REPORTER="$ROOT/scripts/optional_lane_report.py"
INVENTORY="$FUZZ_DIR/targets.inventory"

rm -rf "$OUT"
mkdir -p "$OUT/logs" "$OUT/reproducers"
ROWS="$OUT/rows.tsv"
: >"$ROWS"
LANE_ERRORS=()
LANE_STATUS=""

clean() { printf '%s' "$1" | tr '\t\r\n' '   ' | cut -c1-300; }

# row <target> <status> <stage> <exit> <elapsed_s> <executed> <corpus> <seeds> <reproducer> <note>
row() {
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$1" "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$9" "$(clean "${10}")" >>"$ROWS"
}

finish() {
  local args=(--lane fuzz --rows "$ROWS" --out "$OUT"
    --setting "max_total_time=$MAX_TOTAL_TIME" --setting "rss_limit_mb=$RSS_LIMIT_MB"
    --setting "unit_timeout=$UNIT_TIMEOUT" --setting "wall_grace=$WALL_GRACE"
    --setting "toolchain=$TOOLCHAIN")
  local error
  for error in ${LANE_ERRORS[@]+"${LANE_ERRORS[@]}"}; do
    args+=(--lane-error "$error")
  done
  if [ -n "$LANE_STATUS" ]; then
    args+=(--lane-status "$LANE_STATUS")
  fi
  python3 "$REPORTER" "${args[@]}"
  exit $?
}

# --- declared inventory -------------------------------------------------------
DECLARED=()
if [ -f "$INVENTORY" ]; then
  while IFS= read -r line; do
    line="${line%%#*}"
    line="$(printf '%s' "$line" | tr -d '[:space:]')"
    [ -n "$line" ] && DECLARED+=("$line")
  done <"$INVENTORY"
fi
if [ "${#DECLARED[@]}" -eq 0 ]; then
  LANE_ERRORS+=("declared target inventory is empty or missing: $INVENTORY")
  finish
fi

mark_all() { # mark_all <status> <stage> <note>
  local target
  for target in "${DECLARED[@]}"; do
    row "$target" "$1" "$2" "" "" "" "" "" "" "$3"
  done
}

# --- platform / infrastructure -----------------------------------------------
case "$OS_NAME" in
  Linux | Darwin) ;;
  *)
    LANE_STATUS="unsupported"
    mark_all unsupported platform "libFuzzer + AddressSanitizer smoke is not supported on $OS_NAME"
    finish
    ;;
esac

if ! cargo +"$TOOLCHAIN" fuzz --version >"$OUT/logs/cargo-fuzz-version.log" 2>&1; then
  LANE_STATUS="not-run"
  mark_all not-run infrastructure "cargo-fuzz or the nightly toolchain is unavailable (see logs/cargo-fuzz-version.log)"
  finish
fi

# --- list: declared inventory must equal `cargo fuzz list` -------------------
LIST_LOG="$OUT/logs/cargo-fuzz-list.log"
if ! (cd "$FUZZ_DIR" && cargo +"$TOOLCHAIN" fuzz list) >"$LIST_LOG" 2>"$OUT/logs/cargo-fuzz-list.stderr"; then
  LANE_ERRORS+=("cargo fuzz list failed (see logs/cargo-fuzz-list.stderr)")
  mark_all not-run list "cargo fuzz list failed"
  finish
fi
LISTED=()
while IFS= read -r line; do
  line="$(printf '%s' "$line" | tr -d '[:space:]')"
  [ -n "$line" ] && LISTED+=("$line")
done <"$LIST_LOG"
if [ "${#LISTED[@]}" -eq 0 ]; then
  LANE_ERRORS+=("cargo fuzz list returned no targets")
  mark_all not-run list "cargo fuzz list returned no targets"
  finish
fi

sorted() { printf '%s\n' "$@" | LC_ALL=C sort -u; }
if [ "$(sorted "${DECLARED[@]}")" != "$(sorted "${LISTED[@]}")" ]; then
  LANE_ERRORS+=("declared inventory ($(sorted "${DECLARED[@]}" | tr '\n' ' ')) != cargo fuzz list ($(sorted "${LISTED[@]}" | tr '\n' ' '))")
  mark_all not-run inventory "inventory mismatch; no target executed"
  finish
fi

# --- build + run each target --------------------------------------------------
# Hard wall-time cap: coreutils `timeout` when present, otherwise a portable
# watchdog (macOS has neither timeout nor gtimeout by default).
BUDGET=$((MAX_TOTAL_TIME + WALL_GRACE))
WALL_CAP=$((BUDGET + KILL_SLACK))
run_capped() {
  if command -v timeout >/dev/null 2>&1; then
    timeout "$WALL_CAP" "$@"
    return $?
  fi
  if command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$WALL_CAP" "$@"
    return $?
  fi
  local flag="$OUT/.watchdog-fired" pid watchdog status
  rm -f "$flag"
  "$@" &
  pid=$!
  (
    sleep "$WALL_CAP"
    : >"$flag"
    pkill -TERM -P "$pid" 2>/dev/null
    kill -TERM "$pid" 2>/dev/null
  ) &
  watchdog=$!
  wait "$pid"
  status=$?
  kill "$watchdog" 2>/dev/null
  wait "$watchdog" 2>/dev/null
  [ -e "$flag" ] && return 124
  return "$status"
}

COMPILED=()
EXECUTED=()
for target in "${DECLARED[@]}"; do
  log="$OUT/logs/$target.log"
  work="$FUZZ_DIR/corpus/$target"
  seeds="$FUZZ_DIR/seeds/$target"
  artifacts="$FUZZ_DIR/artifacts/$target"
  mkdir -p "$work"
  seed_count=0
  [ -d "$seeds" ] && seed_count="$(find "$seeds" -type f | wc -l | tr -d ' ')"

  if [ "$seed_count" -eq 0 ]; then
    row "$target" fail seeds "" "" "" "" 0 "" "no seed corpus in $seeds"
    continue
  fi

  if ! (cd "$FUZZ_DIR" && cargo +"$TOOLCHAIN" fuzz build "$target") >"$OUT/logs/$target.build.log" 2>&1; then
    row "$target" fail build "" "" "" "" "$seed_count" "" "compile failed (see logs/$target.build.log)"
    continue
  fi
  COMPILED+=("$target")

  mkdir -p "$artifacts"
  marker="$OUT/.started-$target"
  : >"$marker"
  started="$(date +%s)"
  (cd "$FUZZ_DIR" && run_capped cargo +"$TOOLCHAIN" fuzz run "$target" "$work" "$seeds" -- \
    "-max_total_time=$MAX_TOTAL_TIME" "-rss_limit_mb=$RSS_LIMIT_MB" "-timeout=$UNIT_TIMEOUT" \
    -print_final_stats=1) >"$log" 2>&1
  code=$?
  elapsed=$(($(date +%s) - started))
  EXECUTED+=("$target")

  executed="$(sed -n 's/^stat::number_of_executed_units:[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$log" | tail -n 1)"
  corpus_count="$(find "$work" -type f | wc -l | tr -d ' ')"

  reproducer=""
  if [ "$code" -ne 0 ]; then
    mkdir -p "$OUT/reproducers/$target"
    while IFS= read -r file; do
      cp "$file" "$OUT/reproducers/$target/"
      reproducer="${reproducer:+$reproducer,}reproducers/$target/$(basename "$file")"
    done < <(find "$artifacts" -type f -newer "$marker" 2>/dev/null)
    note="exit $code (see logs/$target.log)"
    [ "$code" -eq 124 ] && note="hard wall-time cap exceeded (exit 124)"
    row "$target" fail run "$code" "$elapsed" "${executed:-}" "$corpus_count" "$seed_count" "${reproducer:-none-found}" "$note"
  elif [ "$elapsed" -gt "$BUDGET" ]; then
    row "$target" fail run "$code" "$elapsed" "${executed:-}" "$corpus_count" "$seed_count" "" "elapsed ${elapsed}s exceeds budget ${MAX_TOTAL_TIME}s + grace ${WALL_GRACE}s despite exit 0"
  elif [ -z "$executed" ] || [ "$executed" -eq 0 ]; then
    row "$target" fail run "$code" "$elapsed" "${executed:-}" "$corpus_count" "$seed_count" "" "exit 0 but no executed-unit evidence in log"
  else
    row "$target" pass run "$code" "$elapsed" "$executed" "$corpus_count" "$seed_count" "" "budget ${MAX_TOTAL_TIME}s, elapsed ${elapsed}s"
  fi
done

if [ "$(sorted ${COMPILED[@]+"${COMPILED[@]}"})" != "$(sorted "${DECLARED[@]}")" ]; then
  LANE_ERRORS+=("compiled targets != declared inventory")
fi
if [ "$(sorted ${EXECUTED[@]+"${EXECUTED[@]}"})" != "$(sorted "${DECLARED[@]}")" ]; then
  LANE_ERRORS+=("executed targets != declared inventory")
fi
finish
