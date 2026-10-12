#!/usr/bin/env bash
# wupz — the benchmark lane: both backends, one table.
set -euo pipefail
cd "$(dirname "$0")/.."

run_backend() {
  local expected="$1"
  shift
  local output
  if ! output=$("$@"); then
    printf 'Benchmark command failed for %s\n%s\n' "$expected" "$output" >&2
    return 1
  fi
  if [[ ${output%%$'\n'*} != "wupz-bench · backend = $expected" ]]; then
    printf 'Expected %s; comparison rejected. Check Zig availability and WUPZ_NO_ZIG.\n%s\n' "$expected" "$output" >&2
    return 1
  fi
  printf '%s\n' "$output"
}

# Validate the actual backend: a successful Cargo build can use the fallback.
zig_output=$(run_backend zig-simd cargo run --release --quiet --bin wupz-bench)
scalar_output=$(run_backend rust-scalar env WUPZ_NO_ZIG=1 cargo run --release --quiet --bin wupz-bench)
zig_sink=$(printf '%s\n' "$zig_output" | sed -n '/^sink: /p')
scalar_sink=$(printf '%s\n' "$scalar_output" | sed -n '/^sink: /p')
if [[ -z $zig_sink || $zig_sink == *$'\n'* || $zig_sink != "$scalar_sink" ]]; then
  printf 'Benchmark comparison rejected: expected one matching result sink per backend.\n' >&2
  exit 1
fi

printf '== zig-simd ==\n%s\n\n' "$zig_output"
printf '== rust-scalar (WUPZ_NO_ZIG=1) ==\n%s\n\n' "$scalar_output"
printf 'Validated backend identities and matching result sinks.\n'
