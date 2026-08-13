#include <metal_stdlib>
using namespace metal;

kernel void add_i32(device int *out [[buffer(0)]], device const int *in [[buffer(1)]],
                    uint tid [[thread_position_in_grid]]) {
  out[tid] = in[tid] + 1;
}
