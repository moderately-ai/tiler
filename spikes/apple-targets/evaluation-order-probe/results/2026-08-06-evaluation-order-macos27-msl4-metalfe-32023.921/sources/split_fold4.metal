#include <metal_stdlib>
using namespace metal;

kernel void tiler_probe(
        device const float *b0 [[buffer(0)]],
        device float *b1 [[buffer(1)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 12ul;
    bool v2 = v0 < v1;
    if (v2) {
        ulong v3 = (v0 / 4ul) * 4ul;
        float v4 = b0[v3 + 0ul];
        float v5 = b0[v3 + 1ul];
        float v6 = b0[v3 + 2ul];
        float v7 = b0[v3 + 3ul];
        float v8 = v4 + v5;
        float v9 = v6 + v7;
        float v10 = v8 + v9;
        b1[v0] = v10;
    }
}
