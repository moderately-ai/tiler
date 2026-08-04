#include <metal_stdlib>

using namespace metal;

struct LayoutDims {
    uint heads;
    uint live_extent;
    uint width;
    uint head_stride;
};

kernel void copy_exact_head_major(device const float *input [[buffer(0)]],
                                  device float *output [[buffer(1)]],
                                  constant LayoutDims &dims [[buffer(2)]],
                                  uint gid [[thread_position_in_grid]]) {
    const uint count = dims.heads * dims.live_extent * dims.width;
    if (gid >= count) return;
    const uint component = gid % dims.width;
    const uint sequence = (gid / dims.width) % dims.live_extent;
    const uint head = gid / (dims.width * dims.live_extent);
    const ulong source = (ulong)head * dims.head_stride +
                         (ulong)sequence * dims.width + component;
    output[gid] = input[source];
}

kernel void copy_capacity_head_major(device const float *input [[buffer(0)]],
                                     device float *output [[buffer(1)]],
                                     constant LayoutDims &dims [[buffer(2)]],
                                     uint gid [[thread_position_in_grid]]) {
    const uint count = dims.heads * dims.live_extent * dims.width;
    if (gid >= count) return;
    const uint component = gid % dims.width;
    const uint sequence = (gid / dims.width) % dims.live_extent;
    const uint head = gid / (dims.width * dims.live_extent);
    const ulong source = (ulong)head * dims.head_stride +
                         (ulong)sequence * dims.width + component;
    output[gid] = input[source];
}

kernel void copy_sequence_major(device const float *input [[buffer(0)]],
                                device float *output [[buffer(1)]],
                                constant LayoutDims &dims [[buffer(2)]],
                                uint gid [[thread_position_in_grid]]) {
    const uint count = dims.heads * dims.live_extent * dims.width;
    if (gid >= count) return;
    const uint component = gid % dims.width;
    const uint sequence = (gid / dims.width) % dims.live_extent;
    const uint head = gid / (dims.width * dims.live_extent);
    const ulong source = (ulong)sequence * dims.heads * dims.width +
                         (ulong)head * dims.width + component;
    output[gid] = input[source];
}
