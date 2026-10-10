#!/usr/bin/env bash
# wupz — the benchmark lane: both backends, one table.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== zig-simd =="
cargo run --release --quiet --bin wupz-bench

echo
echo "== rust-scalar (WUPZ_NO_ZIG=1) =="
WUPZ_NO_ZIG=1 cargo run --release --quiet --bin wupz-bench
