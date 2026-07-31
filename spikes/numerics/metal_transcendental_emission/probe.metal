// Emission probe for the transcendental and reduction primitives the pinned
// Qwen3-0.6B-Base F32 workload needs. Each kernel isolates exactly one MSL
// spelling so the emitted AIR call target attributes to that spelling alone.
//
// This file is compiled, never executed. It establishes which AIR intrinsic a
// spelling selects under a given flag set; it establishes nothing about the
// values any of those intrinsics returns.

#include <metal_stdlib>
using namespace metal;

// --- softmax numerator: exponential, three spellings -----------------------
kernel void exp_default(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                        uint t [[thread_position_in_grid]]) { o[t] = exp(i[t]); }
kernel void exp_precise(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                        uint t [[thread_position_in_grid]]) { o[t] = precise::exp(i[t]); }
kernel void exp_fast(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                     uint t [[thread_position_in_grid]]) { o[t] = fast::exp(i[t]); }
kernel void exp2_default(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                         uint t [[thread_position_in_grid]]) { o[t] = exp2(i[t]); }

// --- RMS normalization: reciprocal square root, three spellings ------------
kernel void rsqrt_default(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                          uint t [[thread_position_in_grid]]) { o[t] = rsqrt(i[t]); }
kernel void rsqrt_precise(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                          uint t [[thread_position_in_grid]]) { o[t] = precise::rsqrt(i[t]); }
kernel void rsqrt_fast(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                       uint t [[thread_position_in_grid]]) { o[t] = fast::rsqrt(i[t]); }
kernel void sqrt_precise(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                         uint t [[thread_position_in_grid]]) { o[t] = precise::sqrt(i[t]); }

// --- SiLU: there is no sigmoid intrinsic; both spellings are compositions ---
kernel void silu_div(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                     uint t [[thread_position_in_grid]]) {
  float x = i[t];
  o[t] = x / (1.0f + precise::exp(-x));
}
kernel void silu_mul_recip(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                           uint t [[thread_position_in_grid]]) {
  float x = i[t];
  o[t] = x * (1.0f / (1.0f + precise::exp(-x)));
}

// --- softmax denominator: division, two spellings --------------------------
kernel void divide_operator(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                            device const float *b [[buffer(2)]],
                            uint t [[thread_position_in_grid]]) { o[t] = a[t] / b[t]; }
kernel void divide_precise(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                           device const float *b [[buffer(2)]],
                           uint t [[thread_position_in_grid]]) { o[t] = precise::divide(a[t], b[t]); }

// --- softmax row maximum: the number-preferring scalar extremum ------------
kernel void fmax_default(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                         device const float *b [[buffer(2)]],
                         uint t [[thread_position_in_grid]]) { o[t] = fmax(a[t], b[t]); }

// --- the reduction primitives a fused softmax or RMSNorm would use ---------
kernel void simd_sum_f32(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                         uint t [[thread_position_in_grid]]) { o[t] = simd_sum(i[t]); }
kernel void simd_max_f32(device float *o [[buffer(0)]], device const float *i [[buffer(1)]],
                         uint t [[thread_position_in_grid]]) { o[t] = simd_max(i[t]); }
