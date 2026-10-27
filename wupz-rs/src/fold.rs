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
    }
}
