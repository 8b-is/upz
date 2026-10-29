# Zero-copy snapshots and `SIGBUS`

Status: recorded, no code change — `upz` maps no files.

## Context

`mmap` reserves address space eagerly and loads blocks on first touch. If the
backing inode shrinks (`ftruncate`, `O_TRUNC`) while a mapping is live, or a
page past EOF is touched, the kernel cannot back the virtual page and delivers
`SIGBUS`, which kills the process. Catching it in user space (`sigaction` +
`sigsetjmp`) needs a jump buffer around every dereference, inhibits dead-store
and aliasing optimisation, and is unsound once tasks migrate across threads.

## Finding in this repository

- `wupz-rs/src/arena.rs` is a bump arena over an owned `Vec<u8>`, and `Seal`
  only freezes allocation. Nothing is file-backed, so no truncation can
  invalidate it and no fault can raise `SIGBUS`.
- `plan.rs` resolves sections and blocks with an `O(1)` dense index and a
  linear fallback; there is no sorted key table to binary-search.
- The packer writes images; it does not map them back for reads.

No file-backed mapping exists, so no code change is made. This mirrors the
aurora finding: the memory is owned, not a projection of a mutable inode.

## Invariant if the packer ever maps an image for zero-copy reads

Four layers, in this order:

1. **Immutable inode replacement** — write a new temporary inode, `fsync`,
   `0400`, then `rename`. Never truncate a live file.
2. **Unlinked-inode lifetime** — POSIX keeps replaced blocks until the last
   `munmap`; readers survive rotation.
3. **Pre-map `fstat` envelope** — physical `st_size` must equal the header's
   declared length before mapping.
4. **Prefault and verify** — `MADV_WILLNEED`, then footer digest verification
   at open.

Canonical, tested implementation: `8b-is/aurora` PR #5 —
`tools/zero_copy_snapshot.py` plus `crates/aurora-storage`, a safe zero-dependency
Rust index over the same buffer. Its `get_by_id` compares the full id per probe;
`get_by_id_prefixed` is the opt-in 8-byte-prefix fast path.
