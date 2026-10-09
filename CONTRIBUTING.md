# Contributing to wupz

Experimental, but not casual: a linker is correctness-critical, so the bar is
receipts, not vibes.

## the loop

`reflect → improve → wire → polish → push → readme`. A change that stops before
readme is a change that didn't happen.

## the rules

1. **Prove it, don't assert it.** Every performance claim starts as a baseline.
   Every invariant gets a test, not a comment.
2. **Cite, don't vendor.** `wild` and `UPX` are references. Compare against them
   byte for byte; do not embed them. See `THIRD_PARTY_NOTICES.md` — *linking UPX
   source makes the combined work GPL.*
3. **The ABI is frozen.** Any change to `include/wupz_abi.h` bumps
   `WUPZ_ABI_VERSION` and lands on both sides at once. `tests/abi_smoke.c` will
   fail if they drift — that is the point.
4. **Kernels ship with a scalar reference.** A new SIMD kernel is not done
   without a seeded differential test proving `vectorized == scalar`.
5. **No stub code.** `docs/ROADMAP.md` marks what is open; do not fake it.

## the check

```bash
scripts/ci.sh
```

Runs: Rust tests · Zig differential tests · both static libs linked into the C
ABI parity test. Green before you push.

## toolchains

- Rust — stable (`rust-toolchain.toml`); edition 2024.
- Zig — 0.16.x. Without zig the Rust crate still builds and tests; the fold
  falls back to the scalar reference (`WUPZ_NO_ZIG=1` forces that path).

## commits

Conventional commits. Small, honest subject lines; a body only when the *why*
isn't obvious.
