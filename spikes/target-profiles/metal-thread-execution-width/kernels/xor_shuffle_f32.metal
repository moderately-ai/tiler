#include <metal_stdlib>
using namespace metal;

// Ascending-mask XOR butterfly at the masks a width-32 InRangeXorShuffle subject would emit.
// The masks are the authorized family's transfer, not a width guess used as the metric.
kernel void xor_shuffle_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                            uint tid [[thread_position_in_grid]]) {
  float v = in[tid];
  v += simd_shuffle_xor(v, ushort(1));
  v += simd_shuffle_xor(v, ushort(2));
  v += simd_shuffle_xor(v, ushort(4));
  v += simd_shuffle_xor(v, ushort(8));
  v += simd_shuffle_xor(v, ushort(16));
  out[tid] = v;
}
