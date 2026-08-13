#include <metal_stdlib>
using namespace metal;

kernel void add_f64(device double *out [[buffer(0)]], device const double *in [[buffer(1)]],
                    uint tid [[thread_position_in_grid]]) {
  out[tid] = in[tid] + 1.0;
}
