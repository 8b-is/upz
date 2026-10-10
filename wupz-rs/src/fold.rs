//! The integrity fold — the Zig SIMD kernel when it is linked, the scalar
//! reference otherwise.
//!
//! Both paths are XOR of the little-endian `u64` words plus the trailing bytes,
//! so they must agree bit for bit. `tests/zig_parity.rs` asserts exactly that
//! across every length from 0 to 1088 — the seam is exercised, not assumed.

/// The scalar reference. Also the fallback when zig is absent.
pub fn xor_fold_scalar(bytes: &[u8]) -> u64 {
    let mut acc: u64 = 0;
    let mut i = 0usize;
    while i + 8 <= bytes.len() {
        let mut word = [0u8; 8];
        word.copy_from_slice(&bytes[i..i + 8]);
        acc ^= u64::from_le_bytes(word);
        i += 8;
    }
    while i < bytes.len() {
        acc ^= bytes[i] as u64;
        i += 1;
    }
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
