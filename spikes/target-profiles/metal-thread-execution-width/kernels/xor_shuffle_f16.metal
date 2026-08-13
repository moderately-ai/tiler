#include <metal_stdlib>
using namespace metal;

kernel void xor_shuffle_f16(device half *out [[buffer(0)]], device const half *in [[buffer(1)]],
                            uint tid [[thread_position_in_grid]]) {
  half v = in[tid];
  v += simd_shuffle_xor(v, ushort(1));
  v += simd_shuffle_xor(v, ushort(2));
  v += simd_shuffle_xor(v, ushort(4));
  v += simd_shuffle_xor(v, ushort(8));
  v += simd_shuffle_xor(v, ushort(16));
  out[tid] = v;
}
