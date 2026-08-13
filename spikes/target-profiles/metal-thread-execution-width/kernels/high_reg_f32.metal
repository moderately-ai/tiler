#include <metal_stdlib>
using namespace metal;

kernel void high_reg_f32(device float *out [[buffer(0)]], device const float *in [[buffer(1)]],
                         uint tid [[thread_position_in_grid]]) {
  float x = in[tid];
  float a0 = x * 1.01f;
  float a1 = x * 1.02f;
  float a2 = x * 1.03f;
  float a3 = x * 1.04f;
  float a4 = x * 1.05f;
  float a5 = x * 1.06f;
  float a6 = x * 1.07f;
  float a7 = x * 1.08f;
  float a8 = x * 1.09f;
  float a9 = x * 1.10f;
  float a10 = x * 1.11f;
  float a11 = x * 1.12f;
  float a12 = x * 1.13f;
  float a13 = x * 1.14f;
  float a14 = x * 1.15f;
  float a15 = x * 1.16f;
  float a16 = x * 1.17f;
  float a17 = x * 1.18f;
  float a18 = x * 1.19f;
  float a19 = x * 1.20f;
  float a20 = x * 1.21f;
  float a21 = x * 1.22f;
  float a22 = x * 1.23f;
  float a23 = x * 1.24f;
  float a24 = x * 1.25f;
  float a25 = x * 1.26f;
  float a26 = x * 1.27f;
  float a27 = x * 1.28f;
  float a28 = x * 1.29f;
  float a29 = x * 1.30f;
  float a30 = x * 1.31f;
  float a31 = x * 1.32f;
  out[tid] = a0 + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9 + a10 + a11 + a12 + a13 + a14 + a15 +
             a16 + a17 + a18 + a19 + a20 + a21 + a22 + a23 + a24 + a25 + a26 + a27 + a28 + a29 +
             a30 + a31;
}
