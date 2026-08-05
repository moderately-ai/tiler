#include <metal_stdlib>
using namespace metal;

// One invocation per grid point, writing a value only that invocation can produce.
//
// `uint tid [[thread_position_in_grid]]` is not an arbitrary choice: it is the
// launch-index realization the authoritative macOS profile selects
// (`LaunchIndexRealization::ThreadPositionInGridUInt`), so the measurement is
// about the grid this profile's own emission can address.
//
// The written value is `tid ^ salt` rather than `tid` for two reasons. The salt
// arrives in a buffer at dispatch time, so no zero fill, no stale mapping, and
// no host-side memset can reproduce the expected pattern; and because XOR with a
// fixed salt is injective, two invocations that collided on one slot would have
// to have carried the same position, which is what a truncated grid index would
// look like.
kernel void grid_extent_probe(device uint *out [[buffer(0)]],
                              constant uint &salt [[buffer(1)]],
                              uint tid [[thread_position_in_grid]]) {
  out[tid] = tid ^ salt;
}
