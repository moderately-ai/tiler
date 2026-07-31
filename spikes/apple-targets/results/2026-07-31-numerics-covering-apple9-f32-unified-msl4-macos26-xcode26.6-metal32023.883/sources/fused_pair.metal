#include <metal_stdlib>
using namespace metal;

kernel void tiler_probe(
        device const float *b0 [[buffer(0)]],
        device float *b1 [[buffer(1)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 8ul;
    bool v2 = v0 < v1;
    if (v2) {
        float v3 = b0[v0];
        float v4 = as_type<float>(0x3fc00000u);
        float v5 = as_type<float>(0x3f800000u);
        float v6 = fma(v3, v4, v5);
        b1[v0] = v6;
    }
}
