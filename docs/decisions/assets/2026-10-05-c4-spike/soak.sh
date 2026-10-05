#!/bin/bash
# Stress soak for F7. Run from the spike root after `cargo test --no-run`.
# Usage: RUNS=50 ./soak.sh > soak-<os>.log
# Modes: stock (stock VFS control), shim (with ENOENT retry),
#        shim-no-retry (C4_DISABLE_CREATE_RETRY=1, reproduces F7).
set -u
RUNS=${RUNS:-50}
S=$(ls -t target/debug/deps/shim-* | grep -v '\.d$' | head -1)
C=$(ls -t target/debug/deps/stress_control-* | grep -v '\.d$' | head -1)
: "${TMPDIR:?set TMPDIR to a private directory}"
run_mode() { # $1 mode
  local fail=0 enoent=0 rofb=0 out
  for i in $(seq 1 "$RUNS"); do
    case "$1" in
      stock) out=$(C4_TRACE=1 "$C" --nocapture 2>&1) ;;
      shim) out=$(C4_TRACE=1 "$S" shim_concurrent --nocapture 2>&1) ;;
      shim-no-retry) out=$(C4_TRACE=1 C4_DISABLE_CREATE_RETRY=1 "$S" shim_concurrent --nocapture 2>&1) ;;
    esac
    echo "$out" | grep -q 'test result: ok' || fail=$((fail+1))
    enoent=$((enoent + $(echo "$out" | grep -cE '(-shm|-wal).*-> -1 No such file|ENOENT after|create retried')))
    rofb=$((rofb + $(echo "$out" | grep -cE '(-shm|-wal).*rdonly=1 -> [0-9]')))
  done
  echo "$(uname -s) mode=$1 runs=$RUNS failed=$fail sidecar_create_enoent_or_retry_events=$enoent ro_fallback_opens=$rofb"
}
run_mode stock; run_mode shim; run_mode shim-no-retry
