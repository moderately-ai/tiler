#include <metal_stdlib>

using namespace metal;

kernel void tiler_aot_observer_probe(device uint *value [[buffer(0)]], uint index [[thread_position_in_grid]]) {
    value[index] += 1u;
}
