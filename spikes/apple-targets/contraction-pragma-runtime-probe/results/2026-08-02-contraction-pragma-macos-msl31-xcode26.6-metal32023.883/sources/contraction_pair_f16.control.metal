#include <metal_stdlib>
using namespace metal;

kernel void tiler_probe(
        device const half *b0 [[buffer(0)]],
        device half *b1 [[buffer(1)]],
        uint tiler_global_invocation_index [[thread_position_in_grid]]) {
    ulong v0 = ulong(tiler_global_invocation_index);
    ulong v1 = 8ul;
    bool v2 = v0 < v1;
    if (v2) {
        half v3 = b0[v0];
        half v4 = as_type<half>(ushort(0x3e02u));
        half v5 = v3 * v4;
        half v6 = as_type<half>(ushort(0x3c00u));
        half v7 = v5 + v6;
        b1[v0] = v7;
    }
}
