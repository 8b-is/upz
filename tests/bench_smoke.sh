#!/usr/bin/env bash
# Exercise the comparison contract without compiling or timing fake kernels.
set -euo pipefail
cd "$(dirname "$0")/.."
fixture=$(mktemp -d "${TMPDIR:-/tmp}/wupz-bench-test.XXXXXX")
trap 'rm -rf "$fixture"' EXIT

cat > "$fixture/cargo" <<'SH'
#!/usr/bin/env bash
set -eu
backend=zig-simd
sink='sink: fold=abc diff=1 pop=2'
if [[ ${WUPZ_NO_ZIG+x} ]]; then backend=rust-scalar; fi
case "$BENCH_TEST_CASE" in
  scalar-everywhere) backend=rust-scalar ;;
  zig-everywhere) backend=zig-simd ;;
  mismatch) if [[ $backend == rust-scalar ]]; then sink='sink: fold=def diff=1 pop=2'; fi ;;
  native-failure) if [[ $backend == zig-simd ]]; then exit 7; fi ;;
  scalar-failure) if [[ $backend == rust-scalar ]]; then exit 7; fi ;;
esac
printf 'wupz-bench · backend = %s\nfixture output, not a measurement\n' "$backend"
if [[ $BENCH_TEST_CASE != missing-sink ]]; then printf '%s\n' "$sink"; fi
if [[ $BENCH_TEST_CASE == duplicate-sink ]]; then printf '%s\n' "$sink"; fi
SH
chmod +x "$fixture/cargo"

run_case() {
  env -u WUPZ_NO_ZIG BENCH_TEST_CASE="$1" PATH="$fixture:$PATH" bash scripts/bench.sh
}

run_case valid > "$fixture/valid.log" 2>&1
for scenario in scalar-everywhere zig-everywhere mismatch missing-sink duplicate-sink native-failure scalar-failure; do
  if run_case "$scenario" > "$fixture/$scenario.log" 2>&1; then
    printf 'FAIL: benchmark comparison accepted %s\n' "$scenario" >&2
    cat "$fixture/$scenario.log" >&2
    exit 1
  fi
done
if env WUPZ_NO_ZIG=1 BENCH_TEST_CASE=valid PATH="$fixture:$PATH" bash scripts/bench.sh > "$fixture/opt-out.log" 2>&1; then
  printf 'FAIL: benchmark comparison ignored the explicit Zig opt-out\n' >&2
  exit 1
fi
printf 'benchmark comparison: 9 contract checks passed\n'
