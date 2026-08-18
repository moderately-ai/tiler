#include <metal_stdlib>
using namespace metal;

// The materialized member's reduction stage: out[row] = the strict ascending
// serial sum of that row's three mapped contributors. This entry's transports
// are the identity ([0, 1]), the opposite of the pointwise stage's, which is
// what makes a per-entry transport resolution observable on the route.
kernel void route_reduce_f32(device const float *scratch [[buffer(0)]],
                             device float *out [[buffer(1)]],
                             uint gid [[thread_position_in_grid]]) {
  float acc = scratch[gid * 3u + 0u];
  acc += scratch[gid * 3u + 1u];
  acc += scratch[gid * 3u + 2u];
  out[gid] = acc;
}
