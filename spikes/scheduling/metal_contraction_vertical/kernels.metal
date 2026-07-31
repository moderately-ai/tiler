// Realization candidates for the index structure `td,od->to`, which the L2
// derivation resolves 197 of the pinned workload's 253 contraction occurrences
// into. Operands are A[M, K] and B[N, K]; the contracted index is the LAST axis
// of both, because the checkpoint stores every projection weight
// [out_features, in_features]. No kernel here transposes an operand.
//
// Every kernel that claims the strict canonical fold seeds its accumulator from
// the FIRST product rather than from +0.0. That is not a stylistic choice:
// `fl(+0.0 + x)` equals `x` for every x except `x = -0.0`, where it is `+0.0`,
// so an accumulator seeded at +0.0 computes a reduction with an injected
// contributor. `docs/numerical-semantics.md` states the same rule for the
// registered strict sum ("Without an initial value, a nonempty sequence starts
// from x0; x0 is not combined with an implicit identity") and gives this exact
// signed-zero counterexample under reduction padding.
//
// `contract_direct_zero_seed` is the deliberately wrong twin, retained so the
// probe can demonstrate that its own classification distinguishes the two.

#include <metal_stdlib>
#include <metal_simdgroup_matrix>

using namespace metal;

struct ContractionDims {
    uint m_extent;   // M — free index `t`
    uint n_extent;   // N — free index `o`
    uint k_extent;   // K — contracted index `d`
    uint split;      // contracted-axis partition width, used by the split kernels only
};

// ---------------------------------------------------------------------------
// Candidate A — direct: one thread per output coordinate, strict left fold.
// ---------------------------------------------------------------------------

kernel void contract_direct(device const float *a [[buffer(0)]],
                            device const float *b [[buffer(1)]],
                            device float *c [[buffer(2)]],
                            constant ContractionDims &dims [[buffer(3)]],
                            uint2 gid [[thread_position_in_grid]]) {
    const uint m = gid.y;
    const uint n = gid.x;
    if (m >= dims.m_extent || n >= dims.n_extent) {
        return;
    }
    const uint k_extent = dims.k_extent;
    device const float *row = a + (ulong)m * k_extent;
    device const float *col = b + (ulong)n * k_extent;

    float accumulator = row[0] * col[0];
    for (uint k = 1; k < k_extent; ++k) {
        const float product = row[k] * col[k];
        accumulator = accumulator + product;
    }
    c[(ulong)m * dims.n_extent + n] = accumulator;
}

// ---------------------------------------------------------------------------
// Candidate A' — the same kernel with the idiomatic `+0.0` accumulator seed.
// Retained as a deliberate defect so the probe's signed-zero case has something
// to reject. It is never a delivery candidate.
// ---------------------------------------------------------------------------

kernel void contract_direct_zero_seed(device const float *a [[buffer(0)]],
                                      device const float *b [[buffer(1)]],
                                      device float *c [[buffer(2)]],
                                      constant ContractionDims &dims [[buffer(3)]],
                                      uint2 gid [[thread_position_in_grid]]) {
    const uint m = gid.y;
    const uint n = gid.x;
    if (m >= dims.m_extent || n >= dims.n_extent) {
        return;
    }
    const uint k_extent = dims.k_extent;
    device const float *row = a + (ulong)m * k_extent;
    device const float *col = b + (ulong)n * k_extent;

    float accumulator = 0.0f;
    for (uint k = 0; k < k_extent; ++k) {
        const float product = row[k] * col[k];
        accumulator = accumulator + product;
    }
    c[(ulong)m * dims.n_extent + n] = accumulator;
}

// ---------------------------------------------------------------------------
// Candidate B — tiled through threadgroup memory, contributor order preserved.
//
// The tiling is over the two FREE indices and over contiguous chunks of the
// contracted index, and each thread still folds its own output's contributors
// in ascending `d`. So this candidate changes the memory schedule and nothing
// about the reduction, which is the whole point of separating it from C.
//
// Structural precondition: K must be a positive multiple of TILE.
// ---------------------------------------------------------------------------

constant uint TILE = 16;

kernel void contract_tiled(device const float *a [[buffer(0)]],
                           device const float *b [[buffer(1)]],
                           device float *c [[buffer(2)]],
                           constant ContractionDims &dims [[buffer(3)]],
                           uint2 tgid [[threadgroup_position_in_grid]],
                           uint2 tid [[thread_position_in_threadgroup]]) {
    threadgroup float a_tile[TILE * TILE];
    threadgroup float b_tile[TILE * TILE];

    const uint local_m = tid.y;
    const uint local_n = tid.x;
    const uint m = tgid.y * TILE + local_m;
    const uint n = tgid.x * TILE + local_n;
    const uint k_extent = dims.k_extent;
    const bool writes = (m < dims.m_extent && n < dims.n_extent);

    // Tile k = [0, TILE). Loaded by every thread of the group, so the barrier is
    // uniform across the threadgroup even for threads whose output is masked.
    a_tile[local_m * TILE + local_n] =
        (m < dims.m_extent) ? a[(ulong)m * k_extent + local_n] : 0.0f;
    b_tile[local_n * TILE + local_m] =
        (n < dims.n_extent) ? b[(ulong)n * k_extent + local_m] : 0.0f;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    float accumulator = a_tile[local_m * TILE + 0] * b_tile[local_n * TILE + 0];
    for (uint kk = 1; kk < TILE; ++kk) {
        const float product = a_tile[local_m * TILE + kk] * b_tile[local_n * TILE + kk];
        accumulator = accumulator + product;
    }

    for (uint k0 = TILE; k0 < k_extent; k0 += TILE) {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        a_tile[local_m * TILE + local_n] =
            (m < dims.m_extent) ? a[(ulong)m * k_extent + k0 + local_n] : 0.0f;
        b_tile[local_n * TILE + local_m] =
            (n < dims.n_extent) ? b[(ulong)n * k_extent + k0 + local_m] : 0.0f;
        threadgroup_barrier(mem_flags::mem_threadgroup);

        for (uint kk = 0; kk < TILE; ++kk) {
            const float product = a_tile[local_m * TILE + kk] * b_tile[local_n * TILE + kk];
            accumulator = accumulator + product;
        }
    }

    if (writes) {
        c[(ulong)m * dims.n_extent + n] = accumulator;
    }
}

// ---------------------------------------------------------------------------
// Candidate C — contracted-axis split into CONTIGUOUS intervals, merged in
// ascending interval order.
//
// Each of `split` lanes folds one contiguous interval of the canonical
// contributor sequence, and lane 0 then folds the lane partials in ascending
// lane order. Under `docs/research/numerics/reduction-semantics-and-legality.md`
// that composed tree has the canonical leaves in canonical order and differs
// from the left fold only in grouping, so it consumes reassociation and does
// NOT consume permutation.
//
// Structural precondition: K must be a positive multiple of `split`.
// ---------------------------------------------------------------------------

constant uint MAX_SPLIT = 32;
constant uint SPLIT_ROWS = 8;

kernel void contract_ksplit_contiguous(device const float *a [[buffer(0)]],
                                       device const float *b [[buffer(1)]],
                                       device float *c [[buffer(2)]],
                                       constant ContractionDims &dims [[buffer(3)]],
                                       uint2 tgid [[threadgroup_position_in_grid]],
                                       uint2 tid [[thread_position_in_threadgroup]]) {
    threadgroup float partials[SPLIT_ROWS * MAX_SPLIT];

    const uint lane = tid.x;
    const uint row = tid.y;
    const uint split = dims.split;
    const uint m = tgid.y;
    const uint n = tgid.x * SPLIT_ROWS + row;
    const uint k_extent = dims.k_extent;
    const uint span = k_extent / split;

    if (lane < split && n < dims.n_extent && m < dims.m_extent) {
        device const float *a_row = a + (ulong)m * k_extent;
        device const float *b_row = b + (ulong)n * k_extent;
        const uint start = lane * span;
        float accumulator = a_row[start] * b_row[start];
        for (uint offset = 1; offset < span; ++offset) {
            const uint k = start + offset;
            accumulator = accumulator + a_row[k] * b_row[k];
        }
        partials[row * MAX_SPLIT + lane] = accumulator;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (lane == 0 && n < dims.n_extent && m < dims.m_extent) {
        float total = partials[row * MAX_SPLIT + 0];
        for (uint j = 1; j < split; ++j) {
            total = total + partials[row * MAX_SPLIT + j];
        }
        c[(ulong)m * dims.n_extent + n] = total;
    }
}

// ---------------------------------------------------------------------------
// Candidate F — contracted-axis split into STRIDED subsets, merged in ascending
// lane order. Identical to C except that lane `j` consumes `j, j+split, ...`,
// which is a noncontiguous subset of the canonical sequence. The composed tree
// therefore reorders leaves and consumes permutation in addition to
// reassociation. It exists to make that distinction a measured difference
// rather than a definitional one.
// ---------------------------------------------------------------------------

kernel void contract_ksplit_strided(device const float *a [[buffer(0)]],
                                    device const float *b [[buffer(1)]],
                                    device float *c [[buffer(2)]],
                                    constant ContractionDims &dims [[buffer(3)]],
                                    uint2 tgid [[threadgroup_position_in_grid]],
                                    uint2 tid [[thread_position_in_threadgroup]]) {
    threadgroup float partials[SPLIT_ROWS * MAX_SPLIT];

    const uint lane = tid.x;
    const uint row = tid.y;
    const uint split = dims.split;
    const uint m = tgid.y;
    const uint n = tgid.x * SPLIT_ROWS + row;
    const uint k_extent = dims.k_extent;

    if (lane < split && n < dims.n_extent && m < dims.m_extent) {
        device const float *a_row = a + (ulong)m * k_extent;
        device const float *b_row = b + (ulong)n * k_extent;
        float accumulator = a_row[lane] * b_row[lane];
        for (uint k = lane + split; k < k_extent; k += split) {
            accumulator = accumulator + a_row[k] * b_row[k];
        }
        partials[row * MAX_SPLIT + lane] = accumulator;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    if (lane == 0 && n < dims.n_extent && m < dims.m_extent) {
        float total = partials[row * MAX_SPLIT + 0];
        for (uint j = 1; j < split; ++j) {
            total = total + partials[row * MAX_SPLIT + j];
        }
        c[(ulong)m * dims.n_extent + n] = total;
    }
}

// ---------------------------------------------------------------------------
// Candidate D — `simdgroup_float8x8` matrix-multiply-accumulate.
//
// The B operand is loaded with `transpose_matrix = true`, so the [N, K] weight
// layout is consumed in place. Nothing here states, and nothing in the vendor
// documentation this spike could find states, in what order
// `simdgroup_multiply_accumulate` combines the eight products of a row-column
// pair or at what precision it holds the accumulator. The probe classifies the
// returned bits and reports what it can distinguish.
//
// Structural precondition: M, N, and K must all be positive multiples of 8.
// ---------------------------------------------------------------------------

kernel void contract_simdgroup(device const float *a [[buffer(0)]],
                               device const float *b [[buffer(1)]],
                               device float *c [[buffer(2)]],
                               constant ContractionDims &dims [[buffer(3)]],
                               uint2 tgid [[threadgroup_position_in_grid]]) {
    const uint m0 = tgid.y * 8;
    const uint n0 = tgid.x * 8;
    if (m0 >= dims.m_extent || n0 >= dims.n_extent) {
        return;
    }
    const uint k_extent = dims.k_extent;

    simdgroup_float8x8 accumulator = make_filled_simdgroup_matrix<float, 8, 8>(0.0f);
    for (uint k0 = 0; k0 < k_extent; k0 += 8) {
        simdgroup_float8x8 left;
        simdgroup_float8x8 right;
        simdgroup_load(left, a + (ulong)m0 * k_extent + k0, k_extent);
        simdgroup_load(right, b + (ulong)n0 * k_extent + k0, k_extent, 0, true);
        simdgroup_multiply_accumulate(accumulator, left, right, accumulator);
    }
    simdgroup_store(accumulator, c + (ulong)m0 * dims.n_extent + n0, dims.n_extent);
}
