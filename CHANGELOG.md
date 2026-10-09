# Changelog

All notable changes to wupz are documented here.
The format is [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org).

## [0.2.0] — 2026-10-10

### Added
- **The Zig path, linked into the Rust core.** `wupz-rs/build.rs` builds
  `wupz-zig/src/kernels.zig` when `zig` is on `PATH` and links the static
  library, so `wupz_core::fold::xor_fold` runs the SIMD kernel. Without zig the
  crate still builds and tests: it falls back to the scalar reference and reports
  `backend() == "rust-scalar"`. Disable explicitly with `WUPZ_NO_ZIG=1`.
- **Backend parity test** (`wupz-rs/tests/zig_parity.rs`) — the linked Zig kernel
  and the Rust scalar reference must agree on every length from 0 to 1088.
- `fold::backend()` — which path is compiled in (`zig-simd` or `rust-scalar`).
- Repo furniture: `CONTRIBUTING.md`, `SECURITY.md`, `CODEOWNERS`,
  `.editorconfig`, `.gitattributes`, `rust-toolchain.toml`.
- README badges (CI, license, ABI, toolchains).

### Changed
- Crate version `0.1.0` → `0.2.0`.
- `wupz-rs` gains a build script; the ABI version it reports is unchanged (`1`).

## [0.1.0] — 2026-10-10

### Added
- The layout plan, the incremental rebuild rule, the pre-sized arena and the
  deterministic plan image (`wupz-rs/`).
- The first Zig SIMD kernels — `xorFold`, `byteDiff`, `popcount` — each with a
  scalar reference and seeded differential tests.
- The frozen C ABI (`include/wupz_abi.h`) and the parity smoke test that links
  both static libraries.
- `scripts/ci.sh` and the CI workflow.
- The docs: `docs/DESIGN.md`, `docs/ROADMAP.md`.
