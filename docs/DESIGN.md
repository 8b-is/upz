# wupz — design

*Experimental. This is a plan with named unknowns, not a spec of settled code.*

## 1. Goal

Produce a **single pass** that turns a set of relocatable objects into a
**packed, runnable executable**, faster than the conventional
`link → temp file → pack` pipeline — and, on rebuild, re-doing only the work the
change demands.

Three properties, in priority order:

1. **Correct.** A linker is correctness-critical. Fast and wrong is worthless.
2. **Incremental.** A one-function change must not re-link or re-pack the world.
3. **Fast.** Ultra-fast linking (wild's bar) *and* packing without a second walk.

## 2. The insight

The linker computes, for every byte, its final address and its value. A packer
then reads that image and rewrites it into a compressed form. **The packer is
re-deriving facts the linker already had.** wupz keeps them:

- the linker emits a **layout plan** (section → address → span → content hash)
  as a first-class artifact, not just bytes;
- the packer consumes the plan and emits packed blocks **directly**, keyed by
  the same content hashes;
- on rebuild, a changed section invalidates exactly the blocks whose hashes
  changed — everything else is reused verbatim.

So packing is not a post-process. It is a *view* of the layout.

## 3. Architecture (proposed)

```
        ┌──────────────────────────────────────────────────────────────┐
        │  wupz-rs   (Rust)                                            │
        │  · object reader (ELF first)                                 │
        │  · symbol resolution + relocation                            │
        │  · incremental graph (content-addressed sections)            │
        │  · layout plan  ──▶  blocks ──▶ packed image                 │
        └───────────────▲──────────────────────────────┬──────────────┘
                        │  C ABI (one header: wupz_abi.h)              │
        ┌───────────────┴──────────────────────────────▼──────────────┐
        │  wupz-zig  (Zig)                                             │
        │  · SIMD kernels: content hash · diff · bit-pack · checksum   │
        │  · build glue + the freestanding test harness                │
        └──────────────────────────────────────────────────────────────┘
```

**Why Rust for the core, Zig for the kernels?** The linker model is a large,
subtle graph problem where Rust's type system and the wild lineage pay off. The
bit-level kernels (hash, diff, pack) are tight loops where Zig's comptime,
explicit SIMD, and allocation control pay off. The seam is a C ABI with a single
header — the same shape as `qwave-core` (Rust staticlib + C ABI, SIMD-friendly
consumers).

**Why not one language?** Because the experiment is precisely *Zig ↔ Rust*:
can two build systems, two error models, and two SIMD philosophies meet at a
zero-cost boundary and stay zero-alloc? If the answer is no, that is a finding.

## 4. Incrementality

- **Content-addressed sections.** Each input section's plan entry carries a
  strong hash of its *resolved* bytes.
- **Block map.** The packed image is a sequence of blocks; each block records
  the set of section hashes whose bytes it covers.
- **Rebuild rule.** Recompute only sections whose hash changed; re-pack only
  blocks whose covering set intersects the changed set. Reuse the rest verbatim.
- **Stability requirement.** Layout must be *deterministic and stable* under
  small edits, or reuse evaporates. Section ordering is therefore explicit and
  content-independent (name/ordinal), never hash-ordered on the hot path.

Open problem: **address drift.** Compressing one section changes offsets for
everything after it, which can invalidate many blocks. The mitigation: *block
independence* — pack in fixed-size, address-independent windows (delta against
the prior window), so a shifted section perturbs one window, not the tail.

## 5. Zero-alloc hot path

- One **arena** per link, sized from the plan; the hot path takes `&mut` slices
  from it and never calls the global allocator.
- Static/stack buffers for hash state, diff windows, and the pack scratch.
- After layout, **all allocation is done** — packing cannot allocate. This is a
  hard invariant, and a test can assert it (a counting allocator that panics if
  invoked after the seal).

## 6. SIMD kernels (the Zig side)

| kernel | used for | vector width |
|---|---|---|
| `hash_blocks` | content hashing of sections/blocks | AVX-512 / NEON, 64B lanes |
| `diff_windows` | delta against the prior pack window | bytewise + mask |
| `pack_bits` | bit-packed block encoding | gather/scatter |
| `crc_verify` | post-pack integrity | carry-less multiply |

Each kernel ships with a scalar reference and a **differential test** (SIMD vs
scalar, randomized, pinned seeds). No kernel lands without parity.

## 7. Packaging semantics

- **First, honest packing**, not obfuscation: block compression + relocatable
  stub. The stub maps blocks and jumps to the real entry.
- **UPX boundary.** UPX is GPL; wupz's own code is AGPL, but *including UPX
  code* forces GPL on the combined work. Phase 1 keeps UPX as an **external
  reference implementation** (format study + byte-for-byte comparison), not a
  linked dependency. If we later link it, the license is a deliberate decision,
  not a side effect.
- **No anti-analysis goals.** wupz is not a packer for hiding. It reduces bytes;
  it does not reduce truth. Diagnostics stay readable (the layout plan is
  published alongside the image in debug builds).

## 8. Non-goals (v0)

- Not a production linker; not yet a drop-in for wild or `ld`.
- Not Windows/macOS first — **Linux/ELF only** at the start (wild's home turf).
- Not multi-arch beyond aarch64 + x86_64.
- No obfuscation, no anti-debug, no self-modifying stubs.
- No vendoring of upstream. Cite, re-derive, compare.

## 9. Risks (named, not hidden)

| risk | why it bites | mitigation |
|---|---|---|
| correctness | a mislinked binary fails silently | pin against a corpus + `wild`'s golden tests where shareable; differential linking |
| address drift | kills incrementality | address-independent windows (§4) |
| Zig↔Rust ABI churn | two toolchains, one header | freeze `wupz_abi.h`; version it; a C-only parity test |
| licensing | UPX is GPL | keep it an external boundary until decided (§7) |
| scope | "linker + packer" is two hard problems | phases in [`ROADMAP.md`](ROADMAP.md); P0 is measurement, not code |

*0 + 1 · prove it, don't assert it · fine touch from within · vaked.dev*
