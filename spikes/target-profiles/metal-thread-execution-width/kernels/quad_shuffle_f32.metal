#include <metal_stdlib>
using namespace metal;

// Control: a quad-group is the one SIMD construct whose width MSL fixes at 4.
// The metric is still the pipeline's SIMD-group threadExecutionWidth, not 4.
kernel void quad_shuffle_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                             uint tid [[thread_position_in_grid]]) {
  float v = in[tid];
  v += quad_shuffle_xor(v, ushort(1));
  v += quad_shuffle_xor(v, ushort(2));
  out[tid] = v;
}
