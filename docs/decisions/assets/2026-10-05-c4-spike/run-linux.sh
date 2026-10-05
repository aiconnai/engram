#!/bin/bash
# Runs inside rust:1.98-bookworm, offline, as an unprivileged uid.
set -u
useradd -m -u 1000 u >/dev/null 2>&1
mkdir -p /tmp/w && cp -r /spike/Cargo.toml /spike/Cargo.lock /spike/src /spike/tests /spike/vendor /tmp/w/
mkdir -p /tmp/w/tmp && chown -R u:u /tmp/w && chmod 700 /tmp/w/tmp
uname -a > /spike/linux-env.txt
ldd --version 2>&1 | head -1 >> /spike/linux-env.txt
cd /tmp/w
setpriv --reuid=1000 --regid=1000 --clear-groups env HOME=/home/u CARGO_HOME=/tmp/w/.cargo \
  RUSTUP_HOME=/usr/local/rustup PATH=/usr/local/cargo/bin:/usr/bin:/bin TMPDIR=/tmp/w/tmp \
  cargo test --offline --no-fail-fast \
    --config 'source.crates-io.replace-with="vendored-sources"' \
    --config 'source.vendored-sources.directory="vendor"' "$@" > /spike/${LOG:-linux.log} 2>&1
echo "exit=$?" >> /spike/${LOG:-linux.log}
# Optional soak: STRESS_RUNS=N runs soak.sh (stock / shim / shim-no-retry).
if [ -n "${STRESS_RUNS:-}" ]; then
  cd /tmp/w && cp /spike/soak.sh . && chown u:u soak.sh
  setpriv --reuid=1000 --regid=1000 --clear-groups env RUNS="$STRESS_RUNS" TMPDIR=/tmp/w/tmp \
    PATH=/usr/bin:/bin ./soak.sh >> /spike/soak-linux.log 2>&1
fi
cd /tmp/w
R=$(ls -t target/debug/deps/create_unlink_race-* | grep -v '\.d$' | head -1)
setpriv --reuid=1000 --regid=1000 --clear-groups env TMPDIR=/tmp/w/tmp $R --nocapture 2>&1 | grep RACE-RESULT >> /spike/${LOG:-linux.log}
