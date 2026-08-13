#include <metal_stdlib>
using namespace metal;

kernel void loop_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                     uint tid [[thread_position_in_grid]]) {
  float v = in[tid];
  for (uint i = 0; i < 8; i++) {
    v = v * 1.000001f + float(i);
  }
  out[tid] = v;
}
