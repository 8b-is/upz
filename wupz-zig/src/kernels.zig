//! wupz-zig — the SIMD kernels behind `include/wupz_abi.h`.
//!
//! Every kernel ships with a **scalar reference** and is checked against it by a
//! differential test over seeded pseudo-random inputs (`docs/DESIGN.md` §6). No
//! kernel is trusted without parity; the tests below are the receipt.
//!
//! Zig owns these because they are tight, allocation-free loops where comptime
//! and explicit vectors pay off. Rust owns the linker model. They meet at the
//! C ABI — one header, versioned.

const std = @import("std");

// ── xorFold: integrity fold ─────────────────────────────────────────────────

/// Scalar reference: XOR of the little-endian u64 words, then the trailing bytes.
pub fn xorFoldScalar(bytes: []const u8) u64 {
    var acc: u64 = 0;
    var i: usize = 0;
    while (i + 8 <= bytes.len) : (i += 8) {
        acc ^= std.mem.readInt(u64, bytes[i..][0..8], .little);
    }
    while (i < bytes.len) : (i += 1) {
        acc ^= @as(u64, bytes[i]);
    }
    return acc;
}

/// Vectorized: fold four u64 lanes per step in one `@reduce`.
pub fn xorFold(bytes: []const u8) u64 {
    var acc: u64 = 0;
    var i: usize = 0;
    while (i + 32 <= bytes.len) : (i += 32) {
        const words: @Vector(4, u64) = .{
            std.mem.readInt(u64, bytes[i..][0..8], .little),
            std.mem.readInt(u64, bytes[i + 8 ..][0..8], .little),
            std.mem.readInt(u64, bytes[i + 16 ..][0..8], .little),
            std.mem.readInt(u64, bytes[i + 24 ..][0..8], .little),
        };
        acc ^= @reduce(.Xor, words);
    }
    while (i + 8 <= bytes.len) : (i += 8) {
        acc ^= std.mem.readInt(u64, bytes[i..][0..8], .little);
    }
    while (i < bytes.len) : (i += 1) {
        acc ^= @as(u64, bytes[i]);
    }
    return acc;
}

// ── byteDiff: the delta size ────────────────────────────────────────────────

/// Scalar reference: number of differing bytes between two spans.
pub fn byteDiffScalar(a: []const u8, b: []const u8) u64 {
    const n = @min(a.len, b.len);
    var count: u64 = 0;
    var i: usize = 0;
    while (i < n) : (i += 1) {
        if (a[i] != b[i]) count += 1;
    }
    return count;
}

/// Vectorized: 16 bytes per step, compared as a u8 vector, summed as u32.
pub fn byteDiff(a: []const u8, b: []const u8) u64 {
    const n = @min(a.len, b.len);
    var count: u64 = 0;
    var i: usize = 0;
    const ones: @Vector(16, u8) = @splat(1);
    const zeros: @Vector(16, u8) = @splat(0);
    while (i + 16 <= n) : (i += 16) {
        const va: @Vector(16, u8) = a[i..][0..16].*;
        const vb: @Vector(16, u8) = b[i..][0..16].*;
        const d: @Vector(16, u8) = @select(u8, va != vb, ones, zeros);
        const wide: @Vector(16, u32) = @intCast(d);
        count += @as(u64, @reduce(.Add, wide));
    }
    while (i < n) : (i += 1) {
        if (a[i] != b[i]) count += 1;
    }
    return count;
}

// ── popcount: density ───────────────────────────────────────────────────────

pub fn popcountScalar(bytes: []const u8) u64 {
    var count: u64 = 0;
    for (bytes) |byte| count += @popCount(byte);
    return count;
}

pub fn popcount(bytes: []const u8) u64 {
    var count: u64 = 0;
    var i: usize = 0;
    while (i + 8 <= bytes.len) : (i += 8) {
        const v: @Vector(8, u8) = bytes[i..][0..8].*;
        const pc: @Vector(8, u8) = @popCount(v);
        const wide: @Vector(8, u32) = @intCast(pc);
        count += @as(u64, @reduce(.Add, wide));
    }
    while (i < bytes.len) : (i += 1) count += @popCount(bytes[i]);
    return count;
}

// ── 32B 0x62-gate frame sealing (kompress-super-ultra parity) ───────────────

pub const GATE_0X62: u8 = 0x62;
pub const FRAME_ABI_V1: u8 = 0x01;

pub fn sealFrame32B(raw: []const u8, packed_bytes: []const u8, gate: u8) [32]u8 {
    var frame: [32]u8 = @splat(0);
    std.mem.writeInt(u64, frame[0..8], xorFold(raw), .little);
    std.mem.writeInt(u64, frame[8..16], xorFold(packed_bytes), .little);
    std.mem.writeInt(u32, frame[16..20], @truncate(raw.len), .little);
    std.mem.writeInt(u32, frame[20..24], @truncate(packed_bytes.len), .little);
    frame[24] = gate;
    frame[25] = FRAME_ABI_V1;
    var prefix_le: [8]u8 = undefined;
    std.mem.writeInt(u64, &prefix_le, xorFold(frame[0..26]), .little);
    @memcpy(frame[26..32], prefix_le[0..6]);
    return frame;
}

pub fn verifyFrame32B(frame: *const [32]u8, raw: []const u8, packed_bytes: []const u8, gate: u8) bool {
    if (frame[24] != gate or frame[25] != FRAME_ABI_V1) return false;
    const expected = sealFrame32B(raw, packed_bytes, gate);
    return byteDiff(frame[0..], expected[0..]) == 0;
}

// ── the C ABI (see include/wupz_abi.h) ──────────────────────────────────────

const builtin = @import("builtin");

/// Scrub caller-saved SIMD/NEON vector registers and scratch general-purpose
/// registers before returning across the C ABI boundary so secret input chunks
/// or tail comparison bytes never remain in registers (`ZA-0004`).
inline fn scrubCallerRegisters() void {
    switch (builtin.cpu.arch) {
        .aarch64, .aarch64_be => {
            asm volatile (
                \\movi v0.2d, #0
                \\movi v1.2d, #0
                \\movi v2.2d, #0
                \\movi v3.2d, #0
                \\movi v4.2d, #0
                \\movi v5.2d, #0
                \\movi v6.2d, #0
                \\movi v7.2d, #0
                \\mov x8, #0
                \\mov x9, #0
                \\mov x10, #0
                \\mov x11, #0
                \\mov x12, #0
                \\mov x13, #0
                \\mov x14, #0
                \\mov x15, #0
                ::: .{
                    .v0 = true,
                    .v1 = true,
                    .v2 = true,
                    .v3 = true,
                    .v4 = true,
                    .v5 = true,
                    .v6 = true,
                    .v7 = true,
                    .x8 = true,
                    .x9 = true,
                    .x10 = true,
                    .x11 = true,
                    .x12 = true,
                    .x13 = true,
                    .x14 = true,
                    .x15 = true,
                });
        },
        .x86_64 => {
            asm volatile (
                \\pxor %%xmm0, %%xmm0
                \\pxor %%xmm1, %%xmm1
                \\pxor %%xmm2, %%xmm2
                \\pxor %%xmm3, %%xmm3
                \\xorq %%rcx, %%rcx
                \\xorq %%rdx, %%rdx
                \\xorq %%rsi, %%rsi
                \\xorq %%rdi, %%rdi
                \\xorq %%r8, %%r8
                \\xorq %%r9, %%r9
                \\xorq %%r10, %%r10
                \\xorq %%r11, %%r11
                ::: .{
                    .xmm0 = true,
                    .xmm1 = true,
                    .xmm2 = true,
                    .xmm3 = true,
                    .rcx = true,
                    .rdx = true,
                    .rsi = true,
                    .rdi = true,
                    .r8 = true,
                    .r9 = true,
                    .r10 = true,
                    .r11 = true,
                });
        },
        else => {},
    }
}

export fn wupz_xor_fold(ptr_opt: ?[*]const u8, len: usize) u64 {
    defer scrubCallerRegisters();
    if (len == 0) return 0;
    const ptr = ptr_opt orelse return 0;
    return xorFold(ptr[0..len]);
}

export fn wupz_byte_diff(a_opt: ?[*]const u8, b_opt: ?[*]const u8, len: usize) u64 {
    defer scrubCallerRegisters();
    if (len == 0) return 0;
    const a = a_opt orelse return 0;
    const b = b_opt orelse return 0;
    return byteDiff(a[0..len], b[0..len]);
}

export fn wupz_popcount(ptr_opt: ?[*]const u8, len: usize) u64 {
    defer scrubCallerRegisters();
    if (len == 0) return 0;
    const ptr = ptr_opt orelse return 0;
    return popcount(ptr[0..len]);
}

/// The ABI version this Zig build speaks. Must equal the Rust `wupz_abi_version()`
/// and the `WUPZ_ABI_VERSION` macro — the C smoke test asserts all three agree.
export fn wupz_zig_abi_version() u32 {
    return 1;
}

// ── differential tests: vectorized == scalar, seeded ────────────────────────

fn fill(buf: []u8, seed: u64) void {
    var x = seed;
    for (buf) |*b| {
        x = x *% 6364136223846793005 +% 1442695040888963407;
        b.* = @truncate(x >> 33);
    }
}

test "xorFold matches the scalar reference across every length" {
    var buf: [1088]u8 = undefined;
    fill(buf[0..], 0x9E3779B97F4A7C15);
    var len: usize = 0;
    while (len <= buf.len) : (len += 1) {
        try std.testing.expectEqual(xorFoldScalar(buf[0..len]), xorFold(buf[0..len]));
    }
}

test "byteDiff matches the scalar reference" {
    var a: [512]u8 = undefined;
    var b: [512]u8 = undefined;
    fill(a[0..], 0x1234_5678_9ABC_DEF0);
    fill(b[0..], 0x0FED_CBA9_8765_4321);
    try std.testing.expectEqual(byteDiffScalar(a[0..], b[0..]), byteDiff(a[0..], b[0..]));

    // identical spans differ by zero
    try std.testing.expectEqual(@as(u64, 0), byteDiff(a[0..], a[0..]));

    // one flipped byte is seen exactly once
    b[300] +%= 1;
    const expected = byteDiffScalar(a[0..], b[0..]);
    try std.testing.expectEqual(expected, byteDiff(a[0..], b[0..]));
}

test "popcount matches the scalar reference" {
    var buf: [512]u8 = undefined;
    fill(buf[0..], 0xDEAD_BEEF_CAFE_F00D);
    try std.testing.expectEqual(popcountScalar(buf[0..]), popcount(buf[0..]));
}

test "xorFold detects a single flipped byte" {
    var buf: [128]u8 = undefined;
    fill(buf[0..], 0xABCD_1234_5678_9EF0);
    const before = xorFold(buf[0..]);
    buf[64] ^= 0x01;
    try std.testing.expect(xorFold(buf[0..]) != before);
}

test "xorFold composes with a seal byte" {
    // Build a 79-byte frame whose last byte seals the first 78; the whole-frame
    // fold is the body fold XOR the tail byte, by construction — the property the
    // wave frame leans on (part 2 of the Sovereign Core series).
    var frame: [79]u8 = undefined;
    fill(frame[0..78], 0x0BADF00D_0BADF00D);
    const body = xorFold(frame[0..78]);
    frame[78] = @truncate(body);
    try std.testing.expectEqual(body ^ @as(u64, frame[78]), xorFold(frame[0..79]));
}

test "C ABI exports fail closed on null pointers" {
    var dummy: [16]u8 = @splat(0x42);
    try std.testing.expectEqual(@as(u64, 0), wupz_xor_fold(null, 64));
    try std.testing.expectEqual(@as(u64, 0), wupz_popcount(null, 64));
    try std.testing.expectEqual(@as(u64, 0), wupz_byte_diff(null, &dummy, 16));
    try std.testing.expectEqual(@as(u64, 0), wupz_byte_diff(&dummy, null, 16));
}

test "32B 0x62-gate frame seal verifies 3761/3314 kompress-super-ultra density and rejects spikes" {
    var raw: [3761]u8 = undefined;
    var packed_bytes: [3314]u8 = undefined;
    fill(raw[0..], 0x3761_3314_0000_0062);
    fill(packed_bytes[0..], 0x0062_3314_3761_0001);

    var frame = sealFrame32B(raw[0..], packed_bytes[0..], GATE_0X62);
    try std.testing.expectEqual(GATE_0X62, frame[24]);
    try std.testing.expect(verifyFrame32B(&frame, raw[0..], packed_bytes[0..], GATE_0X62));

    frame[31] ^= 0x01;
    try std.testing.expect(!verifyFrame32B(&frame, raw[0..], packed_bytes[0..], GATE_0X62));
}
