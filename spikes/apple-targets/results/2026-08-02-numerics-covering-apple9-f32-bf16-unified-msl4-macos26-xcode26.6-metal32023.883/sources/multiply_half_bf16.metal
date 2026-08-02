#include <metal_stdlib>
using namespace metal;

// Replaces an arithmetic NaN with the canonical pattern 0x7fc0, spelled as
// an integer test exactly as the Metal emitter spells it.
static inline bfloat tiler_canonicalize_nan_bf16_7fc0(bfloat value) {
    ushort pattern = as_type<ushort>(value);
    bool nan = (pattern & 0x7f80u) == 0x7f80u
        && (pattern & 0x007fu) != 0x0000u;
    return nan ? as_type<bfloat>(ushort(0x7fc0u)) : value;
}

kernel void tiler_probe(
        device const bfloat *b0 [[buffer(0)]],
        device bfloat *b1 [[buffer(1)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 8ul;
    bool v2 = v0 < v1;
    if (v2) {
        bfloat v3 = b0[v0];
        bfloat v4 = as_type<bfloat>(ushort(0x3f00u));
        bfloat v5 = v3 * v4;
        bfloat v6 = tiler_canonicalize_nan_bf16_7fc0(v5);
        b1[v0] = v6;
    }
}
