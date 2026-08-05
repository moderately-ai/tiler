// Folding probe for elementary-function identities. Each kernel isolates
// exactly one MSL spelling, so the emitted opcode signature attributes to that
// spelling alone — kernels are separate rather than statements in one body
// because common-subexpression elimination across statements would make a
// spelling's count depend on which other spellings sat beside it.
//
// This file is compiled, never executed. A signature says which operations the
// offline front end emitted for a spelling; it says nothing about what any of
// them returns.
//
// The identity kernels come in pairs where one exists: the spelling that would
// be rewritten, and the spelling it would be rewritten into. A fold shows up as
// the first kernel's signature becoming the second's.

#include <metal_stdlib>
using namespace metal;

// --- exp(a) * exp(b) = exp(a + b) ------------------------------------------
// The functional equation the online-softmax rescaling fold telescopes through.
kernel void exp_product(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                        device const float *b [[buffer(2)]],
                        uint t [[thread_position_in_grid]]) { o[t] = exp(a[t]) * exp(b[t]); }
kernel void exp_of_sum(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                       device const float *b [[buffer(2)]],
                       uint t [[thread_position_in_grid]]) { o[t] = exp(a[t] + b[t]); }

// The same identity in the exact shape the rescaling fold uses: a contributor
// exponentiated against a running maximum, then rescaled onto a later one.
kernel void exp_telescope(device float *o [[buffer(0)]], device const float *x [[buffer(1)]],
                          device const float *m [[buffer(2)]],
                          uint t [[thread_position_in_grid]]) {
  float xi = x[t], m1 = m[t], m2 = m[t + 1];
  o[t] = exp(xi - m1) * exp(m1 - m2);
}
kernel void exp_telescoped(device float *o [[buffer(0)]], device const float *x [[buffer(1)]],
                           device const float *m [[buffer(2)]],
                           uint t [[thread_position_in_grid]]) {
  float xi = x[t], m2 = m[t + 1];
  o[t] = exp(xi - m2);
}

// --- exp(a) / exp(b) = exp(a - b), and 1 / exp(a) = exp(-a) ----------------
kernel void exp_quotient(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                         device const float *b [[buffer(2)]],
                         uint t [[thread_position_in_grid]]) { o[t] = exp(a[t]) / exp(b[t]); }
kernel void exp_reciprocal(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                           uint t [[thread_position_in_grid]]) { o[t] = 1.0f / exp(a[t]); }

// --- log(a) + log(b) = log(a * b), and log(a) - log(b) = log(a / b) --------
// Present because the exponential's identity and the logarithm's differ in
// error behaviour, and a compiler that folded one need not fold the other.
kernel void log_sum(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                    device const float *b [[buffer(2)]],
                    uint t [[thread_position_in_grid]]) { o[t] = log(a[t]) + log(b[t]); }
kernel void log_of_product(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                           device const float *b [[buffer(2)]],
                           uint t [[thread_position_in_grid]]) { o[t] = log(a[t] * b[t]); }
kernel void log_difference(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                           device const float *b [[buffer(2)]],
                           uint t [[thread_position_in_grid]]) { o[t] = log(a[t]) - log(b[t]); }

// --- sqrt(a) * sqrt(b) = sqrt(a * b) ---------------------------------------
// An algebraic rather than transcendental elementary function, included so the
// question is about elementary identities rather than about transcendentals.
kernel void sqrt_product(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                         device const float *b [[buffer(2)]],
                         uint t [[thread_position_in_grid]]) { o[t] = sqrt(a[t]) * sqrt(b[t]); }
kernel void sqrt_of_product(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                            device const float *b [[buffer(2)]],
                            uint t [[thread_position_in_grid]]) { o[t] = sqrt(a[t] * b[t]); }

// --- pow(x, k) = x * x, and pow(x, 1/2) = sqrt(x) --------------------------
// The exponent is a literal, so these are the cheapest identity folds a
// compiler could perform on an elementary call and the likeliest to be present.
kernel void pow_square(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                       uint t [[thread_position_in_grid]]) { o[t] = pow(a[t], 2.0f); }
kernel void pow_half(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                     uint t [[thread_position_in_grid]]) { o[t] = pow(a[t], 0.5f); }

// --- positive controls -----------------------------------------------------
// Each is a rewrite whose presence or absence the same counting method has to
// register, so that a row of "no fold" is evidence the probe looked rather than
// evidence it ran. ctl_double is rewritten under a relaxing mode and not under
// the governed one; ctl_muladd is contracted under the governed one; and
// ctl_exp_of_zero is the constant-folding case whose answer explains the rest.
kernel void ctl_double(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                       uint t [[thread_position_in_grid]]) { float x = a[t]; o[t] = x + x; }
kernel void ctl_muladd(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                       device const float *b [[buffer(2)]],
                       uint t [[thread_position_in_grid]]) {
  float x = a[t];
  o[t] = x * b[t] + x;
}
kernel void ctl_exp_of_zero(device float *o [[buffer(0)]], device const float *a [[buffer(1)]],
                            uint t [[thread_position_in_grid]]) { o[t] = exp(0.0f) + a[t]; }
