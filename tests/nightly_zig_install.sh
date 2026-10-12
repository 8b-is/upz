#!/usr/bin/env bash
# Exercise real JSON parsing and failure ordering without running a compiler.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/runner temp"
export RUNNER_TEMP="$fixture/runner temp" GITHUB_PATH="$fixture/github-path"
export TEST_TRACE="$fixture/trace" TEST_INDEX="$fixture/index.json"
export TEST_VERSION='0.18.0-dev.146+35accc06e'
export PATH="$fixture/bin:$PATH"
cat > "$fixture/original.json" <<'JSON'
{"master":{"version":"0.18.0-dev.146+35accc06e","x86_64-linux":{"tarball":"https://ziglang.org/builds/zig-x86_64-linux-0.18.0-dev.146+35accc06e.tar.xz","shasum":"1061dbb921f62b67765810609a80c17fa69eeced9f21e9a7e0d06b4e0d7a3715","size":"16"}}}
JSON
cat > "$fixture/bin/uname" <<'MOCK'
#!/usr/bin/env bash
case "$1" in
  -s) echo "${TEST_OS:-Linux}" ;;
  -m) echo "${TEST_ARCH:-x86_64}" ;;
  *) exit 1 ;;
esac
MOCK
cat > "$fixture/bin/curl" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
url= output=
while (($#)); do
  case "$1" in
    https://*) url=$1 ;;
    --output) output=$2; shift ;;
    --location) exit 1 ;;
  esac
  shift
done
if [[ "$url" == https://ziglang.org/download/index.json ]]; then
  echo index >> "$TEST_TRACE"
  [[ "$TEST_MODE" != index-failure ]] || exit 22
  cp "$TEST_INDEX" "$output"
else
  [[ "$url" == "https://ziglang.org/builds/zig-x86_64-linux-$TEST_VERSION.tar.xz" ]] || exit 1
  echo download >> "$TEST_TRACE"
  [[ "$TEST_MODE" != download-failure ]] || exit 22
  printf '%s\n' 'fixture archive' > "$output"
fi
MOCK
cat > "$fixture/bin/sha256sum" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo checksum >> "$TEST_TRACE"
[[ "$*" == '--check --status' ]] || exit 1
read -r digest archive
[[ "$digest" == 1061dbb921f62b67765810609a80c17fa69eeced9f21e9a7e0d06b4e0d7a3715 && -f "$archive" ]] || exit 1
[[ "$TEST_MODE" != checksum-failure ]]
MOCK
cat > "$fixture/bin/tar" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo extract >> "$TEST_TRACE"
[[ "$1" == -xJf && -f "$2" && "$3" == -C ]] || exit 1
[[ "$TEST_MODE" != extraction-failure ]] || exit 2
mkdir -p "$4/zig-x86_64-linux-$TEST_VERSION"
cat > "$4/zig-x86_64-linux-$TEST_VERSION/zig" <<'ZIG'
#!/usr/bin/env bash
set -euo pipefail
echo version >> "$TEST_TRACE"
[[ "$1" == version ]] || exit 1
if [[ "$TEST_MODE" == version-failure ]]; then
  echo unexpected-version
else
  echo "$TEST_VERSION"
fi
ZIG
chmod +x "$4/zig-x86_64-linux-$TEST_VERSION/zig"
MOCK
chmod +x "$fixture/bin/"*

count=0
run_case() {
  export TEST_MODE="$1" TEST_OS="${5:-Linux}" TEST_ARCH="${6:-x86_64}"
  if [[ "$1" == invalid-json ]]; then
    echo '{' > "$TEST_INDEX"
  else
    jq "${4:-.}" "$fixture/original.json" > "$TEST_INDEX"
  fi
  : > "$TEST_TRACE"
  # A failed installation must preserve any preexisting PATH entries too.
  printf '%s\n' '/existing/path' > "$GITHUB_PATH"
  local status=0
  bash "$repo/scripts/install-nightly-zig.sh" > "$fixture/output" 2>&1 || status=$?
  if [[ "$2" == success ]]; then
    [[ "$status" == 0 ]] || { cat "$fixture/output" >&2; return 1; }
    [[ "$(wc -l < "$GITHUB_PATH" | tr -d ' ')" == 2 ]] || return 1
    [[ -x "$(tail -n 1 "$GITHUB_PATH")/zig" ]] || return 1
  else
    [[ "$status" != 0 && "$(cat "$GITHUB_PATH")" == /existing/path ]] || {
      echo "Unexpected success or PATH mutation: $TEST_MODE (exit $status)" >&2
      cat "$fixture/output" "$GITHUB_PATH" >&2
      return 1
    }
  fi
  [[ "$(tr '\n' ',' < "$TEST_TRACE")" == "$3" ]] || {
    echo "Unexpected setup ordering: $TEST_MODE" >&2; cat "$TEST_TRACE" >&2; return 1;
  }
  count=$((count + 1))
}
run_case normal success 'index,download,checksum,extract,version,'
run_case index-failure failure 'index,'
run_case invalid-json failure 'index,'
run_case missing-master failure 'index,' 'del(.master)'
run_case missing-architecture failure 'index,' 'del(.master["x86_64-linux"])'
run_case invalid-version failure 'index,' '.master.version="../../untrusted"'
run_case invalid-checksum failure 'index,' '.master["x86_64-linux"].shasum="bad"'
run_case invalid-size failure 'index,' '.master["x86_64-linux"].size="-1"'
run_case excessive-size failure 'index,' '.master["x86_64-linux"].size="268435457"'
run_case off-host-url failure 'index,' '.master["x86_64-linux"].tarball="https://example.invalid/zig.tar.xz"'
run_case mismatched-url failure 'index,' '.master["x86_64-linux"].tarball="https://ziglang.org/builds/zig-aarch64-linux-0.18.0-dev.146+35accc06e.tar.xz"'
run_case download-failure failure 'index,download,'
run_case size-failure failure 'index,download,' '.master["x86_64-linux"].size="17"'
run_case checksum-failure failure 'index,download,checksum,'
run_case extraction-failure failure 'index,download,checksum,extract,'
run_case version-failure failure 'index,download,checksum,extract,version,'
run_case unsupported-os failure '' . Darwin
run_case unsupported-architecture failure '' . Linux aarch64
printf '%s nightly Zig bootstrap boundary checks passed\n' "$count"
