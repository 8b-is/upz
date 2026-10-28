# upz (`wupz` × `up` Ultra-Evolved)

[![ci](https://github.com/8b-is/upz/actions/workflows/ci.yml/badge.svg)](https://github.com/8b-is/upz/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-AGPL--3.0--only-blue)
![rust](https://img.shields.io/badge/rust-stable-000?logo=rust)
![zig](https://img.shields.io/badge/zig-0.16-F7A41D?logo=zig)
![ABI](https://img.shields.io/badge/ABI-1-62e6c9)
![version](https://img.shields.io/badge/version-0.5.0-9dff5c)

**upz** = **up** (`pldev` sovereign Zig CLI) + **wupz** (**w**ild + **UPX** + **z**ig) — *kompress-super-ultra*.

An experimental **incremental ultra-fast linker, `kompress-super-ultra` packer, and Top-100 `8b-is` repository consolidator (`beehive`)**: the output is a packed, `0x62`-gate-sealed executable and layout image *by construction*, with zero-alloc hot paths, 64-byte cache-line-aligned `LitanyWireFrame` IPC, `BFP-64` Kleene bitplanes, `1.585 bits/trit` (`base-243`) `SPQT` ternary quantization, and deterministic zero-float (`p/q`) `RationalPQ` arithmetic across **Zig 0.16**, **Rust**, **C11**, **Perl 5**, and **Idris 2 QTT**.

> **Landing:** [wupz.vaked.dev](https://wupz.vaked.dev) — the spatial zen field.

---

## what is new in `upz` (ultra-evolved from `wupz`)

1. **64-Byte Cache-Line Aligned `LitanyWireFrame` (`align(64)` / `#[repr(C, align(64))]` / `wupz_litany_wire_frame_t`)**:
   - Fits in a single CPU cache line (`64B`, Apple Silicon M1–M4 NEON / x86_64 AVX-512):
     - `[0..32]`: 32B `0x62`-gate witness frame (`raw_fold`, `packed_fold`, `raw_len`, `packed_len`, `gate = 0x62`, `abi = 1`, 48-bit prefix XOR seal).
     - `[32..48]`: Disjoint `BFP-64` positive (`+1`) and negative (`-1`) bitplane words at `qt:55` (`(pos & neg) == 0`).
     - `[48..56]`: GCD-reduced zero-float `RationalPQ` (`ratio_p / ratio_q`, e.g. `3761/3314`, `39/40`, `31/32`, `6554/199`).
     - `[56..64]`: 3-byte `SPQT` (`1.585 bits/trit` `base-243`) 15-trit root signature, `qt_mask` (`0x55`), `lanes_active` (`0x0101` = `1/1`), and `wire_seal16` (16-bit XOR fold witness over `[0..62]`).
2. **QAT Sub-Window Incremental Reuse (`1/2 -> 15/16 -> 31/32`) & Zero-Float `RationalPQ`**:
   - `wupz-rs/src/incremental.rs` introduces `RationalPQ` (`gcd_u64`, `isqrt_u64`, `reuse_ratio_pq`, `reuse_ratio_qat_window_pq`), eliminating coarse `1/2` multi-section block invalidation leaks when only a single 4096B QAT window moves (`31/32` on 32-window layouts, `15/16` when 2 windows move).
3. **`.rodata.gaia.b158` `1.585 bits/trit` (`base-243`) Ternary Packer & In-Register Fold**:
   - `pack_gaia_b158_into` (zero-alloc into `Arena`) and `xorFoldGaiaB158` / `xor_fold_gaia_b158` pack balanced trits `{-1, 0, +1}` at `5 trits/byte` (`3^5 = 243 <= 256`, `6554 trits -> 1311B`).
4. **5-Stage `beehive` Top-100 `8b-is` Consolidation Pipeline (`ULTRABACKYARDLOOP`) & `DreamStateMachine`**:
   - `wupz-rs/src/beehive.rs`, `wupz-rs/src/bin/upz.rs`, and `pipeline/beehive-pipeline.mjs` consolidate the top 100 `8b-is` repositories (`upz`, `wupz`, `up`, `pldev`, `Apollo-11`, `spherepop-kernel`, `zeno`, `smart-tree`, `MLX-QUANT`, `wa-catalog`, `crabcc`, `music-quant-signs`, `8b-kit`, `gemini-diary`, `8B-IS-INFERENCE`, `zenUltraCooker`, etc.) into a single `6124B` zero-alloc `LayoutPlan` image sealed at `GATE_0X62 = 0x62` (`sweep_fold = 0x48031a0d4c054e69`, `target_fold = 0x206f642074737557`), plus `DreamStateMachine` for continuous inter-turn background wave consolidation (`432/528/864 Hz` integer carrier steps).
5. **Perl 5 & Idris 2 Dependent-Type Proofs (`verifiers/`) + Pure-Zig `up` CLI (`up/`)**:
   - `verifiers/perl/upz_verify.pl` (`1..7` TAP verification suite).
   - `verifiers/idris/UpzProofs.idr` (Idris 2 QTT `0, 1, ω` proofs for `LitanyWireWitness`, `RationalPQ`, Kleene `+AND=1` / `~NAND=-1`, and `AKPHSemantics` `K-PAX / Képes` `DoubleEntendre` + `30-semmi` ternary signal resolution).
   - `up/src/main.zig` (Pure-Zig `0.16.0` sovereign machine helper, `up fleet`, and `up verify`).

---

## quickstart

```bash
# 1. Full CI (Rust unit + integration + benchmark contract + Zig SIMD + C11 ABI smoke)
bash scripts/ci.sh

# 2. Run the `upz` Top-100 Beehive Consolidation + .rodata.gaia.b158 CLI
cargo run --release --bin upz

# 3. Run the Node.js BigInt Beehive 5-Stage Pipeline
node pipeline/beehive-pipeline.mjs

# 4. Run the Perl 5 64-bit Verifier & Pure-Zig `up` tests
perl verifiers/perl/upz_verify.pl
zig test up/src/main.zig
```

---

## benchmarks (Apple Silicon M-series, 64 MiB working set, median of 9)

| kernel | zig-simd | rust-scalar | verdict |
|---|---|---|---|
| `xor_fold` | **39.49 GiB/s** (`1.58 ms`) | 2.92 GiB/s (`21.39 ms`) | **13.5× — dual `@Vector(4, u64)` deferred `@reduce`** |
| `popcount` | **37.34 GiB/s** (`1.67 ms`) | 11.67 GiB/s (`5.35 ms`) | **3.2× — dual `@Vector(16, u8)` 992B deferred reduction** |
| `byte_diff` | **16.86 GiB/s** (`3.71 ms`) | 4.55 GiB/s (`13.75 ms`) | **3.7× — dual `@Vector(16, u8)` 4064B deferred reduction** |

## license

AGPL-3.0-only for `upz` / `wupz` code. Third-party terms in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

*8b-is · kompressed · 0 + 1 · fine touch from within · vaked.dev*
