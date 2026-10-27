//! The integrity fold — the Zig SIMD kernel when it is linked, the scalar
//! reference otherwise.
//!
//! Both paths are XOR of the little-endian `u64` words plus the trailing bytes,
//! so they must agree bit for bit. `tests/zig_parity.rs` asserts exactly that
//! across every length from 0 to 1088 — the seam is exercised, not assumed.

/// The scalar reference. Also the fallback when zig is absent.
/// Folds each byte directly into the accumulator without staging 8-byte slices
/// onto the stack (`word: [u8; 8]`), preventing stack retention across all
/// optimization levels (`-O0`, `-O1`, `-O2`) (`ZA-0002`, `ZA-0003`).
pub fn xor_fold_scalar(bytes: &[u8]) -> u64 {
    let full_words_len = bytes.len() & !7usize;
    let mut acc: u64 = 0;
    let mut i = 0usize;
    while i < full_words_len {
        acc ^= (bytes[i] as u64) << ((i & 7) * 8);
        i += 1;
    }
    while i < bytes.len() {
        acc ^= bytes[i] as u64;
        i += 1;
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    acc
}

#[cfg(have_zig)]
mod zig {
    unsafe extern "C" {
        pub fn wupz_xor_fold(ptr: *const u8, len: usize) -> u64;
        pub fn wupz_byte_diff(a: *const u8, b: *const u8, len: usize) -> u64;
        pub fn wupz_popcount(ptr: *const u8, len: usize) -> u64;
    }
}

/// Fold a span. Dispatches to the Zig kernel when `cfg(have_zig)` is set.
#[cfg(have_zig)]
pub fn xor_fold(bytes: &[u8]) -> u64 {
    // SAFETY: pointer and length come from the same slice, and the kernel reads
    // exactly `len` bytes.
    unsafe { zig::wupz_xor_fold(bytes.as_ptr(), bytes.len()) }
}

#[cfg(not(have_zig))]
pub fn xor_fold(bytes: &[u8]) -> u64 {
    xor_fold_scalar(bytes)
}

/// Which fold path is compiled in: `"zig-simd"` or `"rust-scalar"`.
#[cfg(have_zig)]
pub const fn backend() -> &'static str {
    "zig-simd"
}

/// Which fold path is compiled in: `"zig-simd"` or `"rust-scalar"`.
#[cfg(not(have_zig))]
pub const fn backend() -> &'static str {
    "rust-scalar"
}

// ── byteDiff / popcount: the same seam, the other two kernels ───────────────

/// Scalar reference: number of differing bytes between two spans.
pub fn byte_diff_scalar(a: &[u8], b: &[u8]) -> u64 {
    let n = a.len().min(b.len());
    let mut count = 0u64;
    for i in 0..n {
        if a[i] != b[i] {
            count += 1;
        }
    }
    count
}

/// Population count over a span — the scalar reference.
pub fn popcount_scalar(bytes: &[u8]) -> u64 {
    bytes.iter().map(|b| b.count_ones() as u64).sum()
}

/// Differing bytes between two spans (the delta size a repack would touch).
#[cfg(have_zig)]
pub fn byte_diff(a: &[u8], b: &[u8]) -> u64 {
    // SAFETY: both pointers are valid for `n` bytes, and `n` is the min length.
    unsafe { zig::wupz_byte_diff(a.as_ptr(), b.as_ptr(), a.len().min(b.len())) }
}

#[cfg(not(have_zig))]
pub fn byte_diff(a: &[u8], b: &[u8]) -> u64 {
    byte_diff_scalar(a, b)
}

/// Population count over a span (density).
#[cfg(have_zig)]
pub fn popcount(bytes: &[u8]) -> u64 {
    // SAFETY: pointer and length come from the same slice.
    unsafe { zig::wupz_popcount(bytes.as_ptr(), bytes.len()) }
}

#[cfg(not(have_zig))]
pub fn popcount(bytes: &[u8]) -> u64 {
    popcount_scalar(bytes)
}

/// Default `kompress-super-ultra` gate byte (`0x62`).
pub const GATE_0X62: u8 = 0x62;
/// 32B frame ABI version (`1`).
pub const FRAME_ABI_V1: u8 = 0x01;
/// Alternating 0x55 quantization bitplane mask byte (`qt:55`).
pub const QUANT_QT55: u8 = 0x55;
/// Canonical 64-bit `kompress-super-ultra` fold signature at `qt:55` and gate `0x62`.
pub const FOLD_0XE1034E575C371AB0: u64 = 0xe103_4e57_5c37_1ab0;

/// Broadcast an 8-bit quantization mask across all 8 bytes of a 64-bit word.
#[inline]
pub const fn qt_broadcast_mask(qt: u8) -> u64 {
    (qt as u64).wrapping_mul(0x0101_0101_0101_0101)
}

/// Pack a 64-bit fold signature into disjoint `BFP-64` positive and negative bitplanes
/// partitioned by the quantization byte `qt`.
#[inline]
pub const fn pack_fold_bfp64(fold: u64, qt: u8) -> (u64, u64) {
    let mask = qt_broadcast_mask(qt);
    (fold & mask, fold & !mask)
}

/// Reconstruct the 64-bit fold signature from `BFP-64` bitplanes and verify zero bit leakage.
#[inline]
pub const fn unpack_fold_bfp64(pos_plane: u64, neg_plane: u64, qt: u8) -> Option<u64> {
    if (pos_plane & neg_plane) != 0 {
        return None;
    }
    let mask = qt_broadcast_mask(qt);
    if (pos_plane & !mask) != 0 || (neg_plane & mask) != 0 {
        return None;
    }
    Some(pos_plane | neg_plane)
}

/// 64-lane Kleene three-valued AND (`+AND=1`) over `BFP-64` bitplane words.
#[inline]
pub const fn bfp64_and_kleene(pos_a: u64, neg_a: u64, pos_b: u64, neg_b: u64) -> (u64, u64) {
    let neg_out = neg_a | neg_b;
    let pos_out = (pos_a & pos_b) & !neg_out;
    (pos_out, neg_out)
}

/// 64-lane Kleene three-valued NAND (`~NAND=-1`) over `BFP-64` bitplane words.
#[inline]
pub const fn bfp64_nand_kleene(pos_a: u64, neg_a: u64, pos_b: u64, neg_b: u64) -> (u64, u64) {
    let (p, n) = bfp64_and_kleene(pos_a, neg_a, pos_b, neg_b);
    (n, p)
}

/// Seal `(raw, packed)` into a 32-byte `0x62`-gated `wupz` frame.
pub fn seal_frame_32b(raw: &[u8], packed: &[u8], gate: u8) -> [u8; 32] {
    let mut frame = [0u8; 32];
    frame[0..8].copy_from_slice(&xor_fold(raw).to_le_bytes());
    frame[8..16].copy_from_slice(&xor_fold(packed).to_le_bytes());
    frame[16..20].copy_from_slice(&(raw.len() as u32).to_le_bytes());
    frame[20..24].copy_from_slice(&(packed.len() as u32).to_le_bytes());
    frame[24] = gate;
    frame[25] = FRAME_ABI_V1;
    let prefix = xor_fold(&frame[0..26]).to_le_bytes();
    frame[26..32].copy_from_slice(&prefix[0..6]);
    frame
}

/// Verify a 32-byte `wupz` frame against `(raw, packed, gate)` in constant shape.
pub fn verify_frame_32b(frame: &[u8; 32], raw: &[u8], packed: &[u8], gate: u8) -> bool {
    if frame[24] != gate || frame[25] != FRAME_ABI_V1 {
        return false;
    }
    let expected = seal_frame_32b(raw, packed, gate);
    byte_diff(frame, &expected) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_frame_32b_fold_and_incremental_diff() {
        // 32-byte 8b-is Dynamic Environment Equalizer frame (ABI v1)
        let mut frame_a = [0u8; 32];
        frame_a[0..2].copy_from_slice(&(-251i16).to_le_bytes()); // -25.1 dBFS
        frame_a[8..10].copy_from_slice(&(213i16).to_le_bytes()); // +2.13 dB 60Hz
        frame_a[18..20].copy_from_slice(&(1000i16).to_le_bytes()); // 1.000 Left Jack Vol
        frame_a[20..22].copy_from_slice(&(1000i16).to_le_bytes()); // 1.000 Right AirPods Vol
        frame_a[24] = 0x59; // qt:59
        frame_a[25] = 1; // ABI v1
        let prefix = xor_fold(&frame_a[0..26]).to_le_bytes();
        frame_a[26..32].copy_from_slice(&prefix[0..6]);

        assert_eq!(xor_fold(&frame_a), xor_fold_scalar(&frame_a));
        assert_eq!(byte_diff(&frame_a, &frame_a), 0);
        assert!(popcount(&frame_a) > 0);

        let mut frame_b = frame_a;
        frame_b[12..14].copy_from_slice(&(280i16).to_le_bytes()); // +2.80 dB 1kHz
        assert_eq!(
            byte_diff(&frame_a, &frame_b),
            byte_diff_scalar(&frame_a, &frame_b)
        );
        assert!(byte_diff(&frame_a, &frame_b) > 0);
    }

    #[test]
    fn kompress_super_ultra_32b_seal_at_0x62_gate() {
        let raw = [0x62u8; 3761];
        let packed = [0x33u8; 3314];
        let mut frame = seal_frame_32b(&raw, &packed, GATE_0X62);
        assert_eq!(frame[24], GATE_0X62);
        assert_eq!(frame[25], FRAME_ABI_V1);
        assert!(verify_frame_32b(&frame, &raw, &packed, GATE_0X62));

        // Any amplitude spike or bit flip trips the 0x62 gate seal
        frame[31] ^= 0x01;
        assert!(!verify_frame_32b(&frame, &raw, &packed, GATE_0X62));

        let (pos, neg) = pack_fold_bfp64(FOLD_0XE1034E575C371AB0, QUANT_QT55);
        assert_eq!(pos & neg, 0);
        let (and_p, and_n) = bfp64_and_kleene(pos, 0, pos, 0);
        let (nand_p, nand_n) = bfp64_nand_kleene(neg, 0, neg, 0);
        assert_eq!((and_p, and_n), (pos, 0));
        assert_eq!((nand_p, nand_n), (0, neg));
        assert_eq!(
            unpack_fold_bfp64(and_p, nand_n, QUANT_QT55),
            Some(FOLD_0XE1034E575C371AB0)
        );
        assert_eq!(unpack_fold_bfp64(pos | 0x02, neg, QUANT_QT55), None);
    }
}
