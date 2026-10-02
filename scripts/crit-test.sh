#!/bin/sh
# usage: scripts/crit-test.sh <cargo test filter>
# Passes only when the filter ran at least one test, none failed, and the run is ok.
export CARGO_TARGET_DIR=/Users/kamal/Desktop/solana/cwf/zcash/.cargo-target
out=$(cd "$(dirname "$0")/.." && cargo test --lib "$1" 2>&1) || { echo "$out" | tail -15; exit 1; }
echo "$out" | grep -q 'test result: ok' || exit 1
echo "$out" | grep -qE '[1-9][0-9]* failed' && exit 1
echo "$out" | grep -qE '[1-9][0-9]* passed' || { echo "filter '$1' matched no tests"; exit 1; }
