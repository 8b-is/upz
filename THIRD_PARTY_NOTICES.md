# Third-party notices

wupz cites and re-derives; it does not vendor its upstreams. This file records
the terms of the projects wupz studies, compares against, or may link.

## wild — Apache-2.0

<https://github.com/wild-linker/wild> · "A very fast linker for Linux" · Rust ·
**Apache License 2.0**.

wupz borrows the *incremental-linker speed model* as a design reference and as a
benchmark baseline. No wild source is copied unless explicitly noted in a file
header; any such file keeps Apache-2.0 and its `NOTICE` obligations.

## UPX — GPL-2.0-or-later (with the UPX special exception)

<https://github.com/upx/upx> · "the Ultimate Packer for eXecutables" · C++ ·
**GNU GPL v2 or later**, plus the UPX exception that permits packing non-GPL
programs.

**License boundary.** Linking or including UPX source in a wupz binary makes the
combined work subject to the GPL. Until that is a deliberate decision, wupz
treats UPX as an **external reference** — a format study and a byte-for-byte
comparison on the command line, never an embedded dependency. See
[`docs/DESIGN.md`](docs/DESIGN.md) §7.

## wupz's own code — AGPL-3.0-only

See [`LICENSE`](LICENSE).

*prove it, don't assert it · fine touch from within · vaked.dev*
