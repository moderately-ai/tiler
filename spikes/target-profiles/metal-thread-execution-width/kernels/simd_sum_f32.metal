#include <metal_stdlib>
using namespace metal;

// Negative control: the refused subgroup collective, not an authorized family.
kernel void simd_sum_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                         uint tid [[thread_position_in_grid]]) {
  out[tid] = simd_sum(in[tid]);
}
