#include <metal_stdlib>
using namespace metal;

kernel void exp_at_zero(device const float *input [[buffer(0)]],
                        device float *output [[buffer(1)]],
                        uint tid [[thread_position_in_grid]]) {
  output[tid] = precise::exp(input[tid]);
}
