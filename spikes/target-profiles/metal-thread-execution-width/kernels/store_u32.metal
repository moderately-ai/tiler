#include <metal_stdlib>
using namespace metal;

kernel void store_u32(device uint *out [[buffer(0)]], uint tid [[thread_position_in_grid]]) {
  out[tid] = tid;
}
