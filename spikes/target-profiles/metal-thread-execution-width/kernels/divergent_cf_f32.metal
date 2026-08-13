#include <metal_stdlib>
using namespace metal;

kernel void divergent_cf_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                             uint tid [[thread_position_in_grid]]) {
  float v = in[tid];
  if ((tid & 1u) == 0u) {
    v = v * 1.5f;
  } else {
    v = v + 1.0f;
  }
  out[tid] = v;
}
