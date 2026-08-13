#include <metal_stdlib>
using namespace metal;

// Source-side product 8*8*1. `threads_per_threadgroup(N)` is a parameter
// builtin, not a function attribute; the function constraint is this one.
kernel void constrained_tg_8x8 [[max_total_threads_per_threadgroup(64)]] (
    device uint *out [[buffer(0)]], uint tid [[thread_position_in_grid]]) {
  out[tid] = tid;
}
