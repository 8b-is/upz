/* abi_smoke.c — prove the Zig side honors include/wupz_abi.h.
 *
 * Built and run by scripts/ci.sh, against build/libwupz_zig.a. If this links and
 * passes, the Zig ↔ Rust seam (one header, versioned) is real. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "wupz_abi.h"

int main(void) {
    const char *s = "wupz — kompressed";
    const size_t n = strlen(s);
    const uint8_t *p = (const uint8_t *)s;

    const uint64_t fold = wupz_xor_fold(p, n);
    const uint64_t pop = wupz_popcount(p, n);
    const uint64_t diff = wupz_byte_diff(p, p, n);
    const uint64_t diff2 = wupz_byte_diff(p, (const uint8_t *)"wupz — KOMPRESSED", n);

    /* the frozen ABI: header, Rust core, and Zig kernels must agree. */
    const uint32_t hdr = WUPZ_ABI_VERSION;
    const uint32_t rust = wupz_abi_version();
    const uint32_t zig = wupz_zig_abi_version();
    if (hdr != rust || hdr != zig) {
        fprintf(stderr, "abi_smoke: ABI mismatch — header=%u rust=%u zig=%u\n", hdr, rust, zig);
        return 1;
    }

    if (diff != 0) {
        fprintf(stderr, "abi_smoke: identical spans must differ by 0\n");
        return 1;
    }
    if (diff2 == 0) {
        fprintf(stderr, "abi_smoke: differing spans must differ by > 0\n");
        return 1;
    }

    /* SECURITY.md fail-closed null-pointer invariant + wupz_zeroize check */
    if (wupz_xor_fold(NULL, 32) != 0 || wupz_popcount(NULL, 32) != 0 || wupz_byte_diff(NULL, p, 16) != 0) {
        fprintf(stderr, "abi_smoke: null pointers must fail closed with 0\n");
        return 1;
    }
    uint8_t secret_buf[16];
    memcpy(secret_buf, "SECRET_KEY_2026!", 16);
    wupz_zeroize(secret_buf, sizeof(secret_buf));
    for (size_t i = 0; i < sizeof(secret_buf); ++i) {
        if (secret_buf[i] != 0) {
            fprintf(stderr, "abi_smoke: wupz_zeroize failed to scrub byte %zu\n", i);
            return 1;
        }
    }

    printf("abi %u ok (header=rust=zig=%u) · fold=0x%016llx · popcount=%llu · diff=%llu\n",
           WUPZ_ABI_VERSION, hdr,
           (unsigned long long)fold,
           (unsigned long long)pop,
           (unsigned long long)diff2);
    return 0;
}
