#include <metal_stdlib>
using namespace metal;

kernel void tiler_probe(
        device const bfloat *b0 [[buffer(0)]],
        device bfloat *b1 [[buffer(1)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 8ul;
    bool v2 = v0 < v1;
    if (v2) {
        bfloat v3 = b0[v0];
        bfloat v4 = as_type<bfloat>(ushort(0x3b80u));
        bfloat v5 = v3 + v4;
        bfloat v6 = as_type<bfloat>(ushort(0x3b80u));
        bfloat v7 = v5 + v6;
        b1[v0] = v7;
    }
}
