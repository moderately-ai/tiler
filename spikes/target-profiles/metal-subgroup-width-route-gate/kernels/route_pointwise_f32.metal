#include <metal_stdlib>
using namespace metal;

// The materialized member's pointwise stage: scratch[i] = in[i] * 2.0 + 1.0.
// The packaged plan places the read binding (ABI slot 0) at argument-table
// index 1 and the write binding (ABI slot 1) at index 0 — the fixture's
// deliberately non-identity transport mapping — so the signature order here is
// the transport order, not the ABI order.
kernel void route_pointwise_f32(device float *scratch [[buffer(0)]],
                                device const float *in [[buffer(1)]],
                                uint gid [[thread_position_in_grid]]) {
  scratch[gid] = in[gid] * 2.0f + 1.0f;
}
