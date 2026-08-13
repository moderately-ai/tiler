#include <metal_stdlib>
using namespace metal;

kernel void add_f16(device half *out [[buffer(0)]], device const half *in [[buffer(1)]],
                    uint tid [[thread_position_in_grid]]) {
  out[tid] = in[tid] + half(1.0);
}
