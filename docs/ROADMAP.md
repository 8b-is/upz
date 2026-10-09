# wupz — roadmap

Each phase ends with a **receipt** (a measured number, a passing differential
test, or a reviewable artifact). No phase is "done" without one.

| Phase | What | Receipt | Status |
|---|---|---|---|
| **P0 · measure** | benchmark `wild` vs `ld` vs `mold` on a fixed corpus; benchmark `link → UPX` as the baseline wupz must beat | a table: link time, pack time, total, output size | **open** |
| **P1 · plan artifact** | emit the layout plan (section → addr → span → hash) from a minimal ELF linker; no packing yet | a plan for a hello-world that round-trips to an identical binary | open |
| **P2 · zig kernels** | `hash_blocks` + `diff_windows` with scalar refs + differential tests + C ABI header `wupz_abi.h` | SIMD == scalar on randomized inputs, pinned seeds | open |
| **P3 · pack-in-pass** | consume the plan and emit a packed image in the same pass; a working stub | hello-world wupz output runs; size ≤ UPX baseline | open |
| **P4 · incremental** | content-addressed sections + block map + the rebuild rule | edit one function → only N blocks re-packed; measured | open |
| **P5 · the fuse** | one command: objects → packed runnable; no temp file | end-to-end time < `link + upx` on the corpus | open |

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
