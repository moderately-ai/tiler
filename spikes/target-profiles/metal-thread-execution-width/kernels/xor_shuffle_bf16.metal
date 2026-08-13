#include <metal_stdlib>
using namespace metal;

// Authorized-family candidate. MSL Table 6.14 excludes bfloat from simd_shuffle.
// A compile failure is a retained row, not a reason to drop this identity.
kernel void xor_shuffle_bf16(device bfloat *out [[buffer(0)]], device const bfloat *in [[buffer(1)]],
                             uint tid [[thread_position_in_grid]]) {
  bfloat v = in[tid];
  v += simd_shuffle_xor(v, ushort(1));
  v += simd_shuffle_xor(v, ushort(2));
  v += simd_shuffle_xor(v, ushort(4));
  v += simd_shuffle_xor(v, ushort(8));
  v += simd_shuffle_xor(v, ushort(16));
  out[tid] = v;
}
