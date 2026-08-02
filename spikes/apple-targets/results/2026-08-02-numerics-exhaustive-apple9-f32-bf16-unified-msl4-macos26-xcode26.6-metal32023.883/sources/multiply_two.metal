#include <metal_stdlib>
using namespace metal;

// Replaces an arithmetic NaN with the canonical pattern 0x7fc00000, spelled as
// an integer test exactly as the Metal emitter spells it.
static inline float tiler_canonicalize_nan_f32_7fc00000(float value) {
    uint pattern = as_type<uint>(value);
    bool nan = (pattern & 0x7f800000u) == 0x7f800000u
        && (pattern & 0x007fffffu) != 0x00000000u;
    return nan ? as_type<float>(0x7fc00000u) : value;
}

kernel void tiler_probe(
        device const float *b0 [[buffer(0)]],
        device float *b1 [[buffer(1)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 8ul;
    bool v2 = v0 < v1;
    if (v2) {
        float v3 = b0[v0];
        float v4 = as_type<float>(0x40000000u);
        float v5 = v3 * v4;
        float v6 = tiler_canonicalize_nan_f32_7fc00000(v5);
        b1[v0] = v6;
    }
}
