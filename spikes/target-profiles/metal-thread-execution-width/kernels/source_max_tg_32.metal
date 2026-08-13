#include <metal_stdlib>
using namespace metal;

kernel void source_max_tg_32 [[max_total_threads_per_threadgroup(32)]] (
    device uint *out [[buffer(0)]], uint tid [[thread_position_in_grid]]) {
  out[tid] = tid;
}
