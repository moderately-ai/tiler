---
id: spike-first-metal-contraction-vertical
title: Spike the first workload-derived Metal contraction vertical
status: done
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, prototype-metal-runtime-proof]
related: [scope-einsum-contraction-support, implement-opaque-physical-call-providers, implement-parallel-reduction-strategies, implement-analytical-component-cost-model]
scopes: [research/scheduling, research/apple-targets, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [spike, research, contraction, matmul, metal, language-model]
---
## User-visible outcome

The first contraction profile is *bounded and measured* — which matmul/batched-matmul shapes and dtypes, under which realization (direct, tiled, simdgroup, or library call), at what measured cost on the bench host — instead of an attempt at general einsum. The measurements are the evidence `scope-first-quantized-lm-profile` and the cost calibration consume.

Use the selected workload to bound and measure the first tensor-contraction
profile rather than attempting general einsum support.

## Questions the spike must answer

- Which fixed matmul or batched-matmul shapes and dtypes constitute the first
  useful profile?
- Which semantic identity, structural validation, access relation, reduction
  order, accumulation dtype, and exceptional-value rules does it require?
- Which direct, tiled, simdgroup, or opaque/library realization candidates
  survive correctness and Metal feasibility checks?
- What padding, layout, synchronization, resource, and numerical obligations
  eliminate a candidate?
- What can be measured on the selected Apple target, and what remains unknown?

Preserve a reproducible harness, exact environment, raw or summarized results,
unsupported cases, and stop conditions under `spikes/`. A candidate with
unknown numerical behavior is not a viable implementation merely because it is
fast.

## Ticket-producing outcome

File separate dependency-ordered delivery tickets for the surviving semantic
profile, normative reference, direct Metal realization, optimized schedule
portfolio, qualified opaque alternative if one survives, runtime integration,
and conformance evidence. Do not file work for eliminated candidates.

## Closes when

At least one bounded contraction path is shown feasible or every tested path is
rejected with reproducible reasons; the architecture and measurement boundary
are recorded; and the surviving work is represented by scoped vertical
tickets with explicit user-visible outcomes.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L3** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L2 lists the contraction shapes **and** milestone 6 settles whether a contraction is one keyed family or a set of per-shape keys.

**Rests on:** L2, plus the milestone 6 open question.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Outcome — 2026-07-31

**Delivered:** the [first Metal contraction realizations](../docs/research/scheduling/first-metal-contraction-realizations.md) record and the [realization probe](../spikes/scheduling/metal_contraction_vertical/README.md), with correctness measured on an Apple M4 Max and timing on the M3 Pro bench host. The status is not "L3 runs end to end": what this rung produced is the profile, the elimination, and two measured host rows, and the end-to-end remainder is [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md).

**The profile.** Index structure 1, `td,od->to`, which L2 resolved 197 of the workload's 253 contraction occurrences into, at the workload's own extents in F32: six correctness cells and eight timing cells covering all six of L1's weight shape classes. Structures 2 and 3 are deferred with a derivation, not omitted — both take a cached `K`/`V` operand whose production is the KV-state model L5 owns and L2 recorded as absent in both candidate mechanisms.

**The elimination, with the ground per candidate.** Six realizations measured; each attributed against twenty-two named reduction topologies computed in exact rational arithmetic.

| Candidate | Survives | Ground |
| --- | --- | --- |
| direct | yes, no permission | uniquely `strict_fold+ftz`; bit-identical to the host oracle at every cell |
| tiled | **yes, no permission — the surviving realization** | same attribution and byte-identical results, 2.6x–4.3x faster at prefill |
| contiguous K-split | only under reassociation | uniquely `contiguous_split+ftz`; fastest candidate at the full decode vocabulary projection |
| strided K-split | only under reassociation *and* permutation | uniquely `strided_split+ftz`; the measured demonstration that the two permissions are different plans |
| `simdgroup_float8x8` | no, under the governed contract | delivers a fused multiply-add under `-ffp-contract=off`, and seeds its accumulator at `+0.0`; also refuses `M=1` and `M=10` |
| `MPSMatrixMultiplication` | no, and for a different reason | refuted against all twenty-two topologies, and shape-dependent on one device — inadmissible before it is costed |

**Three measurements worth carrying forward.** `-ffp-contract=off` does not reach a matrix-multiply-accumulate instruction, which is finding 16 of the Apple numerical record at a new construct. The accumulator seed is observable, so the idiomatic `acc = 0` matmul loop computes a *seeded* reduction rather than the unseeded fold. And the price of the strict contract is shape-dependent rather than uniform: 12% at the complete vocabulary projection, which is bandwidth-bound at about 146 GB/s and where the reassociation-only split is the *fastest* candidate; 3.9x at the small per-layer decode projection, a cell whose 4.19 MB weight is cache-resident and therefore does not extrapolate to a real decode step's 1.76 GiB; and 7.6x at prefill.

**Seven dependency-ordered delivery tickets filed**, for surviving work only: `admit-the-contraction-semantic-profile`, `admit-the-contraction-normative-reference`, `realize-the-strict-contraction-on-metal`, `admit-reassociated-contraction-schedule-alternatives`, `qualify-the-simdgroup-matrix-contraction-realization`, `integrate-the-contraction-vertical-into-the-runtime`, `retain-contraction-conformance-evidence`. Nothing is filed for the MPS route as a realization; its consequences are recorded on [`exercise-opaque-admissions-downstream-of-the-frontier`](exercise-opaque-admissions-downstream-of-the-frontier.md) per the graph maintenance below.

**Measurement boundary.** Correctness on one Apple M4 Max under macOS build `26A5388g`; timing on one Apple M3 Pro under build `26A5378n`; both Apple9, one offline toolchain `metalfe-32023.883`, offline compilation only. No iOS family, no other Apple generation, no F16/BF16/quantized dtype, and no runtime-compilation path. The `simdgroup` and MPS topology results are empirical eliminations over a finite named set, not guarantees.

**Nothing under `crates/` was touched and nothing moved the support matrix.** The contraction row stays at R1.

## Graph maintenance

- **This is a spike**: it lives under `spikes/`, runs from its own directory with the invocation its README records, and no `make` target reaches it. Keep the harness, inputs, and result fixtures checked in; `.gitignore` only regenerable outputs.
- **An opaque/library realization candidate is an opaque physical call** — if the spike shows the library route wins, the admission machinery already exists (declaration, registration, frontier admission) and the gap is caller-supplied providers plus lowering; record that on `exercise-opaque-admissions-downstream-of-the-frontier` and the enforcers ticket rather than inventing a separate integration path.
- **Measurements happen on the M3 bench host, serially** — never in parallel agents; interleave A/B; record exact environment per row.
- **On close, update the roadmap ladder rung** and hand the shape/dtype profile to `derive-transformer-operation-and-shape-surface` if it is still open.

## Closed (2026-07-31)

Integrated at `28d7c9d`. The rung's closing condition is met on its own terms: one bounded contraction path (the tiled strict realization) is shown feasible with reproducible attribution, the eliminations carry their grounds, the measurement boundary is recorded, and the surviving work is seven dependency-ordered vertical tickets. The roadmap L3 row and the two catalog lines are coordinator-owed and land when `contracts/navigation` frees from the live p0 worker.
