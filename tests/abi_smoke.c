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

    if (diff != 0) {
        fprintf(stderr, "abi_smoke: identical spans must differ by 0\n");
        return 1;
    }
    if (diff2 == 0) {
        fprintf(stderr, "abi_smoke: differing spans must differ by > 0\n");
        return 1;
    }

    printf("abi %u ok · fold=0x%016llx · popcount=%llu · diff=%llu\n",
           WUPZ_ABI_VERSION,
           (unsigned long long)fold,
           (unsigned long long)pop,
           (unsigned long long)diff2);
    return 0;
}
