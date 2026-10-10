# wupz

[![ci](https://github.com/8b-is/wupz/actions/workflows/ci.yml/badge.svg)](https://github.com/8b-is/wupz/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-AGPL--3.0--only-blue)
![rust](https://img.shields.io/badge/rust-stable-000?logo=rust)
![zig](https://img.shields.io/badge/zig-0.16-F7A41D?logo=zig)
![ABI](https://img.shields.io/badge/ABI-1-62e6c9)
![version](https://img.shields.io/badge/version-0.2.0-9dff5c)

**wupz** = **w**(ild) + **UPX** + **z**(ig) — *kompressed*.

An experimental **incremental ultra-fast linker that also compresses**: the
output is a packed executable *by construction*, because the linker already
knows the layout it is about to write. Zero-alloc hot paths, SIMD kernels,
Zig ↔ Rust.

> **Landing:** [wupz.vaked.dev](https://wupz.vaked.dev) — the spatial zen field.
> **Status: experimental / research.** Nothing here is a production linker yet.
> This repo is the spec, the scaffold, and the receipts as they land.

---

## the thesis

Linking and packing are the same problem seen twice. Both transform an
executable image; both walk the same sections; both answer "where does this byte
go". Today they are two passes and a temporary file. wupz makes them **one pass**:

```
objects ─▶ [ wupz ] ─▶ packed, runnable image
             │  incremental  (only changed sections re-link, only changed blocks re-pack)
             │  zero-alloc   (arena + static buffers on the hot path)
             └─ SIMD         (hash · diff · bit-pack via Zig kernels)
```

- **incremental, ultra-fast** — content-addressed sections; a rebuild touches
  only what moved.
- **linker + compressor** — packing happens during layout, not after. No
  decompress-to-temp, no second walk.
- **zero-alloc SIMD** — the hot path allocates nothing; the kernels are vector
  code (AVX-512 / NEON) behind a C ABI.
- **Zig ↔ Rust** — Rust carries the linker model (wild's lineage); Zig carries
  the SIMD kernels and the build glue; they meet at a C ABI, like qwave's core.

## lineage — cited, not vendored

| upstream | what wupz takes | license |
|---|---|---|
| [**wild**](https://github.com/wild-linker/wild) | the incremental-linker speed model: "a very fast linker for Linux" | Apache-2.0 |
| [**UPX**](https://github.com/upx/upx) | the packer: the Ultimate Packer for eXecutables — packing as a first-class output stage | GPL-2.0-or-later (with the UPX exception) |

Neither is vendored. wupz re-derives the ideas at the layout level and cites the
sources. See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) — and note that
**linking UPX code pulls GPL terms into anything that includes it**, which is
why the UPX side stays an *external tool boundary* until proven otherwise.

## the crew

Admins: **poe** (Paul Showalter) · **standardgalactic** (Cogito Ergo Sum) ·
**8bit-wraith** (8bit-Wraith) — with peterlodri-sec.

## read next

- [`docs/DESIGN.md`](docs/DESIGN.md) — the technical plan and the non-goals.
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — phases, in order, each with a receipt.
- [`BENCHMARKS.md`](BENCHMARKS.md) — the measured top 5 (and the honest losses).

## what runs today

The scaffold is real code, not slides — and both toolchains are exercised:

```
include/wupz_abi.h     the frozen C ABI (one header, versioned)
wupz-rs/               Rust core: layout plan · incremental rebuild rule ·
                       pre-sized arena · deterministic plan image
wupz-zig/src/kernels.zig   Zig SIMD kernels: xorFold · byteDiff · popcount,
                       each with a scalar reference + seeded differential tests
tests/abi_smoke.c      links the Zig staticlib through the C header
scripts/ci.sh          runs the lot
```

```bash
scripts/ci.sh
#   rust core            23 tests ok (20 + the zig-parity seam)
#   zig kernels           5 differential tests ok
#   C ABI parity test     abi 1 ok (header == rust == zig)
```

**The seam is exercised, not assumed.** `wupz-rs/build.rs` compiles
`wupz-zig/src/kernels.zig` and links it, so `fold::xor_fold` runs the Zig SIMD
kernel; `tests/zig_parity.rs` asserts it agrees with the Rust scalar reference on
every length from 0 to 1088 and reports which path is live:

```
wupz fold backend = zig-simd      # WUPZ_NO_ZIG=1 → rust-scalar
```

Without zig the crate still builds and tests — the fallback keeps contributors
unblocked. (On macOS the archive is written with `libtool`; zig's own archive
members are not 8-byte aligned and Apple's `ld` rejects them.)

The plan-image writer rejects names or counts that exceed its wire-format fields
before consuming arena space; oversized names cannot silently wrap their length.

The zero-alloc claim is enforced, not asserted: `wupz-rs/tests/no_alloc.rs`
installs a counting global allocator and fails if `pack_into` allocates after the
arena is sized.

## benchmarks

Measured, not asserted — `scripts/bench.sh` runs both backends over a 64 MiB
working set (median of 9, black-boxed):

| kernel | zig-simd | rust-scalar | verdict |
|---|---|---|---|
| `byte_diff` | **18.44 GiB/s** | 5.70 GiB/s | **3.2× — the seam wins** |
| `xor_fold` | 45.65 GiB/s | **49.72 GiB/s** | scalar wins; honest loss |
| `popcount` | 14.24 GiB/s | 14.35 GiB/s | tie |

Both backends produce the same sink — the numbers are comparable. Full method,
findings and the P0 `upx` bar: [`BENCHMARKS.md`](BENCHMARKS.md).

The nightly lane ([`nightly-zig.yml`](.github/workflows/nightly-zig.yml)) runs the
kernels on **Zig master** — `0.18.0-dev.131` passes all five differential tests
with zero source changes.

## license

AGPL-3.0-only for wupz's own code. Third-party terms in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

*8b-is · kompressed · 0 + 1 · fine touch from within · vaked.dev*
