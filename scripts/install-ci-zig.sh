#!/usr/bin/env bash
# Official release bootstrap for the x86_64 Linux CI lane.
set -euo pipefail

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
  echo 'CI Zig bootstrap requires x86_64 Linux' >&2
  exit 1
fi
: "${RUNNER_TEMP:?RUNNER_TEMP is required}"
: "${GITHUB_PATH:?GITHUB_PATH is required}"

readonly version=0.16.0
# https://ziglang.org/download/index.json (0.16.0, x86_64-linux)
readonly sha256=70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
readonly name="zig-x86_64-linux-$version"
install_dir=$(mktemp -d "$RUNNER_TEMP/wupz-zig.XXXXXX")
archive="$install_dir/$name.tar.xz"

curl --fail --silent --show-error --location \
  --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --connect-timeout 15 --max-time 120 --retry 2 --retry-max-time 180 \
  "https://ziglang.org/download/$version/$name.tar.xz" --output "$archive"
printf '%s  %s\n' "$sha256" "$archive" | sha256sum --check --status
tar -xJf "$archive" -C "$install_dir"

zig_dir="$install_dir/$name"
if [[ "$("$zig_dir/zig" version)" != "$version" ]]; then
  echo 'Downloaded Zig reports an unexpected version' >&2
  exit 1
fi
# Publish only after download, checksum, extraction and version checks pass.
printf '%s\n' "$zig_dir" >> "$GITHUB_PATH"
