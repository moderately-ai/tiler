#include <metal_stdlib>
using namespace metal;

kernel void tiler_probe(
        device const uchar *b0 [[buffer(0)]],
        device const uchar *b1 [[buffer(1)]],
        device const float *b2 [[buffer(2)]],
        device float *b3 [[buffer(3)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 65536ul;
    bool v2 = v0 < v1;
    if (v2) {
        uchar v3 = b0[v0];
        uchar v4 = b1[v0];
        int v5 = int(v3);
        int v6 = int(v4);
        int v7 = v5 - v6;
        float v8 = float(v7);
        float v9 = b2[0];
        float v10 = v8 * v9;
        b3[v0] = v10;
    }
}
