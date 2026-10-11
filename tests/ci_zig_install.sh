#!/usr/bin/env bash
# Exercise setup failure boundaries without downloading or executing a compiler.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/runner temp"
export RUNNER_TEMP="$fixture/runner temp"
export GITHUB_PATH="$fixture/github-path"
export TEST_TRACE="$fixture/trace"
export PATH="$fixture/bin:$PATH"

cat > "$fixture/bin/uname" <<'MOCK'
#!/usr/bin/env bash
case "$1" in
  -s) printf '%s\n' "${TEST_OS:-Linux}" ;;
  -m) printf '%s\n' x86_64 ;;
  *) exit 1 ;;
esac
MOCK
cat > "$fixture/bin/curl" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo download >> "$TEST_TRACE"
[[ "$TEST_MODE" != download-failure ]] || exit 22
while (($#)); do
  if [[ "$1" == --output ]]; then
    printf '%s\n' 'fixture archive' > "$2"
    exit 0
  fi
  shift
done
exit 1
MOCK
cat > "$fixture/bin/sha256sum" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo checksum >> "$TEST_TRACE"
[[ "$*" == '--check --status' ]]
read -r digest archive
[[ "$digest" == 70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00 ]]
[[ -f "$archive" ]]
[[ "$TEST_MODE" != checksum-failure ]]
MOCK
cat > "$fixture/bin/tar" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo extract >> "$TEST_TRACE"
[[ "$1" == -xJf && -f "$2" && "$3" == -C ]]
[[ "$TEST_MODE" != extraction-failure ]] || exit 2
mkdir -p "$4/zig-x86_64-linux-0.16.0"
cat > "$4/zig-x86_64-linux-0.16.0/zig" <<'ZIG'
#!/usr/bin/env bash
set -euo pipefail
echo version >> "$TEST_TRACE"
[[ "$1" == version ]]
if [[ "$TEST_MODE" == version-failure ]]; then
  echo unexpected-version
else
  echo 0.16.0
fi
ZIG
chmod +x "$4/zig-x86_64-linux-0.16.0/zig"
MOCK
chmod +x "$fixture/bin/"*

run_case() {
  export TEST_MODE="$1" TEST_OS="${4:-Linux}"
  : > "$TEST_TRACE"
  : > "$GITHUB_PATH"
  local status=0
  bash "$repo/scripts/install-ci-zig.sh" > "$fixture/output" 2>&1 || status=$?
  if [[ "$2" == success ]]; then
    [[ "$status" == 0 ]] || { cat "$fixture/output" >&2; return 1; }
    [[ "$(wc -l < "$GITHUB_PATH" | tr -d ' ')" == 1 ]]
    [[ -x "$(cat "$GITHUB_PATH")/zig" ]]
  else
    [[ "$status" != 0 && ! -s "$GITHUB_PATH" ]]
  fi
  [[ "$(tr '\n' ',' < "$TEST_TRACE")" == "$3" ]]
}

run_case normal success 'download,checksum,extract,version,'
run_case download-failure failure 'download,'
run_case checksum-failure failure 'download,checksum,'
run_case extraction-failure failure 'download,checksum,extract,'
run_case version-failure failure 'download,checksum,extract,version,'
run_case unsupported-host failure '' Darwin
echo '6 CI Zig bootstrap boundary checks passed'
