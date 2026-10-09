# wupz — roadmap

Each phase ends with a **receipt** (a measured number, a passing differential
test, or a reviewable artifact). No phase is "done" without one.

| Phase | What | Receipt | Status |
|---|---|---|---|
| **P0 · measure** | benchmark `wild` vs `ld` vs `mold` on a fixed corpus; benchmark `link → UPX` as the baseline wupz must beat | a table: link time, pack time, total, output size | **open** |
| **P1 · plan artifact** | emit the layout plan (section → addr → span → hash) from a minimal ELF linker; no packing yet | a plan for a hello-world that round-trips to an identical binary | **partial** — plan + deterministic plan-image + digest landed; the ELF reader is not written |
| **P2 · zig kernels** | `hash_blocks` + `diff_windows` with scalar refs + differential tests + C ABI header `wupz_abi.h` | SIMD == scalar on randomized inputs, pinned seeds | **partial** — `xorFold` · `byteDiff` · `popcount` landed with scalar refs, seeded differential tests, and the linked C ABI; the hash/bit-pack/crc kernels are open |
| **P3 · pack-in-pass** | consume the plan and emit a packed image in the same pass; a working stub | hello-world wupz output runs; size ≤ UPX baseline | open |
| **P4 · incremental** | content-addressed sections + block map + the rebuild rule | edit one function → only N blocks re-packed; measured | **partial** — the rebuild rule (`changed_sections` · `blocks_to_repack` · `windows_touched` · `reuse_ratio`) landed and is tested; not yet wired to a linker |
| **P5 · the fuse** | one command: objects → packed runnable; no temp file | end-to-end time < `link + upx` on the corpus | open |

## receipts so far (`scripts/ci.sh`)

```
rust core (plan · incremental · arena · pack-image)  23 tests ok
zig kernels (differential: vectorized vs scalar)      5 tests ok
static libs + C ABI parity test                       abi 1 ok (header == rust == zig)
zig↔rust fold seam (0..=1088)                         backend = zig-simd · fallback = rust-scalar
```

The zero-alloc invariant is enforced, not asserted: `wupz-rs/tests/no_alloc.rs`
installs a counting global allocator and fails if `pack_into` touches it after
the arena is sized.


## the bar

- **Correctness first:** never trade a failing differential test for speed.
- **Zero-alloc invariant:** a counting allocator that panics if the hot path
  (post-seal) allocates — asserted in CI.
- **Cite, don't vendor:** wild and UPX are references; comparisons are
  byte-for-byte, not binary-embedded.

## open questions (for the crew)

1. Rust-core/Zig-kernel — or Zig-core/Rust-tooling? (P1 decides with a spike.)
2. Is "pack in pass" viable for *all* sections, or only leaf sections?
3. Do we ever link UPX, or always shell out? (license → §7 of DESIGN.)
4. What is the honest hand-off to `wild` for the non-packed path?

*0 + 1 · the loop has an exit · fine touch from within · vaked.dev*
