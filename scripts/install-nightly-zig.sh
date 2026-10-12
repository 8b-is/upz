#!/usr/bin/env bash
# Resolve one official master snapshot for the x86_64 Linux nightly lane.
set -euo pipefail

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
  echo 'Nightly Zig bootstrap requires x86_64 Linux' >&2
  exit 1
fi
: "${RUNNER_TEMP:?RUNNER_TEMP is required}"
: "${GITHUB_PATH:?GITHUB_PATH is required}"

install_dir=$(mktemp -d "$RUNNER_TEMP/wupz-zig-master.XXXXXX")
index="$install_dir/index.json"
download() {
  # No redirects: both the index and archive must be served by the named host.
  curl --fail --silent --show-error --proto '=https' --tlsv1.2 \
    --connect-timeout 15 --max-time 120 --retry 2 --retry-max-time 180 \
    --max-filesize "$3" "$1" --output "$2"
}
download 'https://ziglang.org/download/index.json' "$index" 1048576
version=$(jq -er '.master.version | strings' "$index")
sha256=$(jq -er '.master["x86_64-linux"].shasum | strings' "$index")
url=$(jq -er '.master["x86_64-linux"].tarball | strings' "$index")
size=$(jq -er '.master["x86_64-linux"].size | strings' "$index")
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]+\+[0-9a-f]+$ ]] ||
   ! [[ "$sha256" =~ ^[0-9a-f]{64}$ ]] ||
   ! [[ "$size" =~ ^[1-9][0-9]{0,8}$ && "$size" -le 268435456 ]]; then
  echo 'Invalid Zig master version, checksum or archive size' >&2
  exit 1
fi
name="zig-x86_64-linux-$version"
if [[ "$url" != "https://ziglang.org/builds/$name.tar.xz" ]]; then
  echo 'Unexpected Zig master archive URL' >&2
  exit 1
fi

# This receipt identifies the moving snapshot; it is not an independent pin.
printf 'Zig master snapshot: version=%s sha256=%s url=%s\n' "$version" "$sha256" "$url"
archive="$install_dir/$name.tar.xz"
download "$url" "$archive" "$size"
if [[ "$(wc -c < "$archive" | tr -d ' ')" != "$size" ]]; then
  echo 'Downloaded Zig archive has an unexpected size' >&2
  exit 1
fi
printf '%s  %s\n' "$sha256" "$archive" | sha256sum --check --status
tar -xJf "$archive" -C "$install_dir"
zig_dir="$install_dir/$name"
if [[ "$("$zig_dir/zig" version)" != "$version" ]]; then
  echo 'Downloaded Zig reports an unexpected version' >&2
  exit 1
fi
# Never expose a partially downloaded or unverified compiler to later steps.
printf '%s\n' "$zig_dir" >> "$GITHUB_PATH"
