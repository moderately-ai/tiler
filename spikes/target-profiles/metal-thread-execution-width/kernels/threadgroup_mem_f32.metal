#include <metal_stdlib>
using namespace metal;

kernel void threadgroup_mem_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                                uint tid [[thread_position_in_grid]],
                                uint lid [[thread_index_in_threadgroup]]) {
  threadgroup float tile[4096];
  tile[lid] = in[tid];
  threadgroup_barrier(mem_flags::mem_threadgroup);
  out[tid] = tile[lid];
}
