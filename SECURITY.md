# Security policy

## reporting

Report a vulnerability privately — **do not open a public issue**. Use GitHub's
[private vulnerability reporting](https://github.com/8b-is/wupz/security/advisories/new)
on this repo.

## scope

wupz is an experimental linker and packer. The threat surface that matters:

- **memory safety in the kernels** — the Zig kernels take a pointer and a
  length. A caller that passes a wrong length is a bug wupz must not amplify.
  Report any path where length is not derived from the same buffer.
- **the C ABI** — `include/wupz_abi.h` is the trust boundary. A null or
  mis-sized pointer must fail closed, not crash the caller.
- **the packer** — packing must never change program *semantics*. A packed
  binary that behaves differently from the unpacked one is a correctness bug,
  and we treat it as security-relevant.
- **the landing page** (`wupz.vaked.dev`) — a static page with no server. It
  fetches nothing; if it ever does, that is a finding.

## out of scope

- Upstream `wild` and `UPX` bugs — report those upstream.
- The `docs/` plans: this is a research repo, and an unshipped design is not a
  vulnerability.

## posture

Zero dependencies in the Rust core; no network on any hot path; the frozen ABI
is versioned. *No state change without a witness.*
