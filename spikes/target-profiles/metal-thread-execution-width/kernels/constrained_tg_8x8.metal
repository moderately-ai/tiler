#include <metal_stdlib>
using namespace metal;

kernel void constrained_tg_8x8(device uint *out [[buffer(0)]], uint tid [[thread_position_in_grid]])
    [[threads_per_threadgroup(8, 8, 1)]] {
  out[tid] = tid;
}
