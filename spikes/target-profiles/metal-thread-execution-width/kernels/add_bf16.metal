#include <metal_stdlib>
using namespace metal;

kernel void add_bf16(device bfloat *out [[buffer(0)]], device const bfloat *in [[buffer(1)]],
                     uint tid [[thread_position_in_grid]]) {
  out[tid] = in[tid] + bfloat(1.0);
}
