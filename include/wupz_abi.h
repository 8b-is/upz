/*
 * wupz_abi.h — the one boundary between wupz-rs (Rust) and wupz-zig (Zig).
 *
 * FROZEN. Any change bumps WUPZ_ABI_VERSION and both sides in lockstep.
 * This header is a contract, not a convenience: if it churns, the seam is
 * broken and the "two toolchains, zero-cost" thesis is falsified.
 */
#ifndef WUPZ_ABI_H
#define WUPZ_ABI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define WUPZ_ABI_VERSION 1u

/* wupz-rs: the ABI version the Rust core speaks. */
uint32_t wupz_abi_version(void);

/* wupz-zig: the ABI version the Zig kernels speak. Both must equal
 * WUPZ_ABI_VERSION — tests/abi_smoke.c asserts it at link time. */
uint32_t wupz_zig_abi_version(void);

/* wupz-zig: integrity fold. XOR of the little-endian u64 words, then the
 * trailing bytes. Allocation-free, order-stable, and its own witness: fold a
 * valid frame and you get zero if it was built as an XOR seal. */
uint64_t wupz_xor_fold(const uint8_t *ptr, size_t len);

/* wupz-zig: the number of differing bytes between two equal-length spans
 * (the delta size an incremental repack would touch). */
uint64_t wupz_byte_diff(const uint8_t *a, const uint8_t *b, size_t len);

/* wupz-zig: population count over a span (density hint). */
uint64_t wupz_popcount(const uint8_t *ptr, size_t len);

/* DSE-resistant memory scrub helper for C/FFI callers (ZA-0003).
 * Uses volatile byte stores + compiler memory barrier so -O2/-O3 Dead Store
 * Elimination cannot strip stack/heap wipes prior to scope exit. */
static inline void wupz_zeroize(void *ptr, size_t len) {
    if (ptr == NULL || len == 0) {
        return;
    }
    volatile uint8_t *p = (volatile uint8_t *)ptr;
    for (size_t i = 0; i < len; ++i) {
        p[i] = 0u;
    }
#if defined(__GNUC__) || defined(__clang__)
    __asm__ __volatile__("" : : "r"(ptr) : "memory");
#endif
}

#ifdef __cplusplus
}
#endif

#endif /* WUPZ_ABI_H */
