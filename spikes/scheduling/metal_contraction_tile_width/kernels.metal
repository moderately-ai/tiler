// Tile-width sweep kernels for the index structure `td,od->to`. Operands are
// A[M, K] and B[N, K]; the contracted index is the LAST axis of both. No kernel
// here transposes an operand. This mirrors the realization probe beside it
// (`../metal_contraction_vertical/kernels.metal`) so that the two measure the
// same subject.
//
// The governing protocol is `PROTOCOL-2026-08-22-contraction-tile-width.md`,
// committed before this file existed.
//
// WHAT IS PARAMETERIZED, AND WHY IT IS TWO NUMBERS RATHER THAN ONE
//
// The probe beside this one governs its tile with a single `constant uint TILE`
// that is simultaneously the M-block height, the N-block width, and the K-chunk
// depth. Those three are welded together by the load pattern rather than by
// intent: a TILE x TILE thread block loading one element each covers a
// TILE x TILE patch, which forces the staged k-extent to equal the block's
// other dimension. Sweeping that one constant therefore yields a compound and
// cannot say which of the three effects moved.
//
// So the body below takes TILE_M (M-block height) and TILE_W (N-block width AND
// K-chunk depth) separately. `TILE_M == TILE_W` is the square arm and
// reproduces the probe's kernel exactly; `TILE_M < TILE_W` is the rectangular
// arm, which decouples the M-block height from the staged k-extent and is the
// only thing in this sweep able to REFUTE the record's attribution of the M = 1
// loss to masked rows.
//
// BYTE-IDENTITY IS A REQUIRED PRE-RUN CHECK, NOT AN ASPIRATION
//
// `contract_tiled_reference` below is a verbatim copy of the probe's
// `contract_tiled`. The parameterized variant at (16, 16) must return bits
// identical to it. The generalization is written to make that true by
// construction rather than by luck: at TILE_M == TILE_W the B-load loop runs
// exactly one iteration and every index expression collapses to the reference's.
// The reference's B load and the general one write the same tile CONTENTS by
// different threads -- element (i, j) of the tile is written from
// b[(n0 + i) * K + k0 + j] in both -- and a barrier follows, so the contents a
// reader sees are identical, not merely equivalent.

#include <metal_stdlib>

using namespace metal;

struct ContractionDims {
    uint m_extent;   // M -- free index `t`
    uint n_extent;   // N -- free index `o`
    uint k_extent;   // K -- contracted index `d`
    uint split;      // unused here; kept so the manifest layout matches the probe's
};

// ---------------------------------------------------------------------------
// Reference A -- direct: one thread per output coordinate, strict left fold.
// Copied verbatim from `../metal_contraction_vertical/kernels.metal` so that
// the cross-variant oracle compares against the same reference the retained
// record used. The accumulator is seeded from the FIRST product rather than
// +0.0: `fl(+0.0 + x)` equals `x` for every x except `x = -0.0`, so a +0.0 seed
// computes a reduction with an injected contributor.
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
// Reference B -- the probe's `contract_tiled`, verbatim, at its compiled-in
// TILE of 16. Present only so the parameterized (16, 16) variant can be held
// against it bit for bit inside one process. It is not a sweep variant and the
// driver never times it as one.
// ---------------------------------------------------------------------------

constant uint TILE = 16;

kernel void contract_tiled_reference(device const float *a [[buffer(0)]],
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
// The parameterized tiled body.
//
// Thread block is (x = TILE_W, y = TILE_M), so `local_n` is the fastest-varying
// index, exactly as in the reference.
//
// A tile is TILE_M rows by TILE_W staged k-values: TILE_M * TILE_W elements for
// TILE_M * TILE_W threads, so one load each, unchanged from the reference.
//
// B tile is TILE_W rows by TILE_W staged k-values: TILE_W * TILE_W elements for
// TILE_M * TILE_W threads, so TILE_W / TILE_M loads each. The loop generalizes
// the reference's single B load along the k axis -- thread (local_m, local_n)
// stages k offsets `local_m + r * TILE_M` of row `n0 + local_n` -- which is why
// it collapses to exactly the reference's statement when TILE_M == TILE_W.
//
// Structural preconditions, checked by the host rather than assumed here:
// K must be a positive multiple of TILE_W, and TILE_M must divide TILE_W.
//
// Every load is guarded by a ternary rather than an early return, so all
// barriers stay threadgroup-uniform even for threads whose output is masked.
// The B-load trip count is a compile-time constant for the same reason.
// ---------------------------------------------------------------------------

template <uint TILE_M, uint TILE_W>
static void tiled_body(device const float *a,
                       device const float *b,
                       device float *c,
                       constant ContractionDims &dims,
                       uint2 tgid,
                       uint2 tid,
                       threadgroup float *a_tile,
                       threadgroup float *b_tile) {
    static_assert(TILE_W % TILE_M == 0, "TILE_M must divide TILE_W");

    constexpr uint b_loads = TILE_W / TILE_M;

    const uint local_m = tid.y;
    const uint local_n = tid.x;
    const uint n0 = tgid.x * TILE_W;
    const uint m = tgid.y * TILE_M + local_m;
    const uint n = n0 + local_n;
    const uint k_extent = dims.k_extent;
    const bool writes = (m < dims.m_extent && n < dims.n_extent);

    a_tile[local_m * TILE_W + local_n] =
        (m < dims.m_extent) ? a[(ulong)m * k_extent + local_n] : 0.0f;
    for (uint r = 0; r < b_loads; ++r) {
        const uint kk = local_m + r * TILE_M;
        b_tile[local_n * TILE_W + kk] =
            (n < dims.n_extent) ? b[(ulong)n * k_extent + kk] : 0.0f;
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);

    float accumulator = a_tile[local_m * TILE_W + 0] * b_tile[local_n * TILE_W + 0];
    for (uint kk = 1; kk < TILE_W; ++kk) {
        const float product = a_tile[local_m * TILE_W + kk] * b_tile[local_n * TILE_W + kk];
        accumulator = accumulator + product;
    }

    for (uint k0 = TILE_W; k0 < k_extent; k0 += TILE_W) {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        a_tile[local_m * TILE_W + local_n] =
            (m < dims.m_extent) ? a[(ulong)m * k_extent + k0 + local_n] : 0.0f;
        for (uint r = 0; r < b_loads; ++r) {
            const uint kk = local_m + r * TILE_M;
            b_tile[local_n * TILE_W + kk] =
                (n < dims.n_extent) ? b[(ulong)n * k_extent + k0 + kk] : 0.0f;
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);

        for (uint kk = 0; kk < TILE_W; ++kk) {
            const float product = a_tile[local_m * TILE_W + kk] * b_tile[local_n * TILE_W + kk];
            accumulator = accumulator + product;
        }
    }

    if (writes) {
        c[(ulong)m * dims.n_extent + n] = accumulator;
    }
}

// ---------------------------------------------------------------------------
// Explicit instantiations. Threadgroup memory must be allocated in the kernel
// function itself, so each variant declares its own arrays and hands the body
// two pointers.
//
// The population is frozen by the protocol: six square widths and twelve
// rectangular pairs, eighteen in total. The driver asserts that count against
// the pipelines the device actually prepared, so a variant that silently fails
// to compile fails the run instead of shrinking the sweep.
// ---------------------------------------------------------------------------

#define TILED_VARIANT(MM, WW)                                                        \
    kernel void contract_tiled_m##MM##_w##WW(                                        \
        device const float *a [[buffer(0)]],                                         \
        device const float *b [[buffer(1)]],                                         \
        device float *c [[buffer(2)]],                                               \
        constant ContractionDims &dims [[buffer(3)]],                                \
        uint2 tgid [[threadgroup_position_in_grid]],                                 \
        uint2 tid [[thread_position_in_threadgroup]]) {                              \
        threadgroup float a_tile[(MM) * (WW)];                                       \
        threadgroup float b_tile[(WW) * (WW)];                                       \
        tiled_body<MM, WW>(a, b, c, dims, tgid, tid, a_tile, b_tile);                \
    }

// Square arm: TILE_M == TILE_W.
TILED_VARIANT(1, 1)
TILED_VARIANT(2, 2)
TILED_VARIANT(4, 4)
TILED_VARIANT(8, 8)
TILED_VARIANT(16, 16)
TILED_VARIANT(32, 32)

// Rectangular arm: TILE_M < TILE_W, TILE_M divides TILE_W.
TILED_VARIANT(1, 8)
TILED_VARIANT(2, 8)
TILED_VARIANT(4, 8)
TILED_VARIANT(1, 16)
TILED_VARIANT(2, 16)
TILED_VARIANT(4, 16)
TILED_VARIANT(8, 16)
TILED_VARIANT(1, 32)
TILED_VARIANT(2, 32)
TILED_VARIANT(4, 32)
TILED_VARIANT(8, 32)
TILED_VARIANT(16, 32)

// ---------------------------------------------------------------------------
// A deliberately wrong twin of the (16, 16) variant, differing ONLY in seeding
// its accumulator at +0.0 instead of from the first product. It exists so the
// cross-variant oracle has something it must reject: a classification that
// could not separate these two would separate nothing. The driver never treats
// it as a sweep variant and the protocol's perturbation check names it.
// ---------------------------------------------------------------------------

kernel void contract_tiled_m16_w16_zero_seed(device const float *a [[buffer(0)]],
                                             device const float *b [[buffer(1)]],
                                             device float *c [[buffer(2)]],
                                             constant ContractionDims &dims [[buffer(3)]],
                                             uint2 tgid [[threadgroup_position_in_grid]],
                                             uint2 tid [[thread_position_in_threadgroup]]) {
    threadgroup float a_tile[16 * 16];
    threadgroup float b_tile[16 * 16];

    const uint local_m = tid.y;
    const uint local_n = tid.x;
    const uint n0 = tgid.x * 16u;
    const uint m = tgid.y * 16u + local_m;
    const uint n = n0 + local_n;
    const uint k_extent = dims.k_extent;
    const bool writes = (m < dims.m_extent && n < dims.n_extent);

    float accumulator = 0.0f;
    for (uint k0 = 0; k0 < k_extent; k0 += 16u) {
        threadgroup_barrier(mem_flags::mem_threadgroup);
        a_tile[local_m * 16u + local_n] =
            (m < dims.m_extent) ? a[(ulong)m * k_extent + k0 + local_n] : 0.0f;
        b_tile[local_n * 16u + local_m] =
            (n < dims.n_extent) ? b[(ulong)n * k_extent + k0 + local_m] : 0.0f;
        threadgroup_barrier(mem_flags::mem_threadgroup);

        for (uint kk = 0; kk < 16u; ++kk) {
            const float product = a_tile[local_m * 16u + kk] * b_tile[local_n * 16u + kk];
            accumulator = accumulator + product;
        }
    }

    if (writes) {
        c[(ulong)m * dims.n_extent + n] = accumulator;
    }
}
