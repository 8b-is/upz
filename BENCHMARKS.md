> **Measured, not asserted.** Every number here comes from a run you can repeat:
> `scripts/bench.sh`. Hardware: Apple M1 · macOS 26 · rustc 1.99 · zig 0.16.0
> (and 0.18.0-dev.131 on the nightly lane). 64 MiB working set, 9 runs, median.

# Benchmarks

[**wupz-bench**](wupz-rs/src/bin/wupz-bench.rs) — *the kernel harness, dependency-free*

[**scripts/bench.sh**](scripts/bench.sh) — *runs both backends and prints the table*

[**The seam**](docs/DESIGN.md) — *why the kernels live in Zig at all*

[**The bar**](docs/ROADMAP.md) — *P0: what wupz must beat*

---

## Top 5 — Projects/tools

1. **[`byte_diff`](wupz-zig/src/kernels.zig)** · zig-simd — **18.44 GiB/s** — *3.2× the Rust scalar reference (5.70). The seam's clear win.*
2. **[`xor_fold`](wupz-zig/src/kernels.zig)** · rust-scalar — **49.72 GiB/s** — *the fallback beats the SIMD path. Honest loss, see Findings.*
3. **[`xor_fold`](wupz-zig/src/kernels.zig)** · zig-simd — **45.65 GiB/s** — *`@reduce(.Xor, …)` per 32 B loses to rustc's auto-vectorised loop.*
4. **[`popcount`](wupz-zig/src/kernels.zig)** · zig-simd — **14.24 GiB/s** — *a tie (scalar: 14.35). Not worth a separate kernel yet.*
5. **[`upx -9 --force-macos`](https://github.com/upx/upx)** · the P0 bar — **252,512 → 36,880 B (14.61%) in 0.01 s** — *unofficial on macOS; see The bar.*

| kernel | backend | GiB/s | median | vs scalar |
|---|---|---|---|---|
| `xor_fold` | zig-simd | 45.65 | 1.37 ms | **0.92×** |
| `xor_fold` | rust-scalar | 49.72 | 1.26 ms | 1.00× |
| `byte_diff` | zig-simd | 18.44 | 3.39 ms | **3.23×** |
| `byte_diff` | rust-scalar | 5.70 | 10.97 ms | 1.00× |
| `popcount` | zig-simd | 14.24 | 4.39 ms | 0.99× |
| `popcount` | rust-scalar | 14.35 | 4.36 ms | 1.00× |

Both backends produce the **same sink** (`fold=3a5e1d62251cee13`, `diff=66846195`,
`pop=268427185`) — the numbers are comparable because the answers are identical.

---

## Method

Three rules, and they are the whole method:

1. **One working set, both backends.** 64 MiB of deterministic LCG data; the
   *same* buffers, seeded, so a rerun is a rerun.
2. **Median of nine, after a warm-up.** First-run noise never enters the table.
3. **Black-boxed, never elided.** The result of every pass is folded into a sink
   and `black_box`'d, so the optimiser cannot delete the work it is being timed on.

```bash
scripts/bench.sh          # both backends, one table
```

The comparison script requires the first run to report `zig-simd` and the
second `rust-scalar`, then checks that their result sinks match before printing
the comparison. Missing Zig, a build fallback, an inherited `WUPZ_NO_ZIG`
opt-out, or unequal results makes the comparison fail. Individual scalar runs
remain available through `WUPZ_NO_ZIG=1 cargo run --release --bin wupz-bench`.
These controls check backend identity and output agreement; they do not prove
statistical significance, remove run-order effects, or validate the numbers
recorded above on a different machine.

## Findings

> *Is the Zig seam actually faster?*

**On `byte_diff`, emphatically yes** — 3.2×. The 16-byte compare-and-sum
vectorises cleanly, and the scalar loop cannot keep up.

**On `xor_fold`, no — the scalar Rust path is ~9% faster.** The Zig version folds
four `u64` lanes through `@reduce(.Xor, …)` every 32 bytes; rustc's
auto-vectorised scalar loop apparently schedules better. This is the interesting
result: *the seam is not a free win, it is a win per kernel.* The honest move is
to let `xor_fold` fall back to the scalar path (or re-write it without the
reduce) and keep `byte_diff` in Zig.

**On `popcount`, a tie.** Neither side has an edge; keep whichever is simpler.

> *Does the nightly Zig break the kernels?*

**No.** `0.18.0-dev.131+41f885830` runs all five differential tests green with
zero source changes — 0.16.0 → 0.18.0-dev compatibility, measured
([the nightly lane](.github/workflows/nightly-zig.yml)).

## The bar (P0)

> *What does wupz have to beat?*

`upx -9` on a real 252,512-byte binary: **→ 36,880 B, 14.61%, in 0.01 s.**

But the headless run is the finding:

```
upx: target: CantPackException: macOS is currently not supported (try --force-macos)
```

**UPX does not support macOS.** `--force-macos` packs a Mach-O (and does it well:
6.8×) but marks it unofficial, and a packed macOS binary must be unpacked before
it runs. So the fused linker-and-packer has **no incumbent to beat on macOS** —
which is exactly why [the design](docs/DESIGN.md) says *Linux/ELF first*. The P0
corpus must be ELF; the macOS number above is a curiosity, not a baseline.

`strip -x` on the same file: no change (already stripped), 1.56 s — the other
tool in the category, and not a size competitor.

## Reproduce

```bash
cargo run --release --bin wupz-bench               # zig-simd
WUPZ_NO_ZIG=1 cargo run --release --bin wupz-bench # rust-scalar
scripts/bench.sh                                   # both, formatted
scripts/ci.sh                                      # the correctness half
```

*prove it, don't assert it · 0 + 1 · fine touch from within · vaked.dev*
