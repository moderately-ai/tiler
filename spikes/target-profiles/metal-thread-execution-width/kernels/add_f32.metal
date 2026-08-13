#include <metal_stdlib>
using namespace metal;

kernel void add_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                    uint tid [[thread_position_in_grid]]) {
  out[tid] = in[tid] + 1.0f;
}
