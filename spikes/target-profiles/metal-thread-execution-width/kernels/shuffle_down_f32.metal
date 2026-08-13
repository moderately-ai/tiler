#include <metal_stdlib>
using namespace metal;

// Negative control: the refused descending-stride / narrowing tree.
kernel void shuffle_down_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                             uint tid [[thread_position_in_grid]]) {
  float v = in[tid];
  v += simd_shuffle_down(v, ushort(16));
  v += simd_shuffle_down(v, ushort(8));
  v += simd_shuffle_down(v, ushort(4));
  v += simd_shuffle_down(v, ushort(2));
  v += simd_shuffle_down(v, ushort(1));
  out[tid] = v;
}
