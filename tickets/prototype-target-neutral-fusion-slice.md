---
id: prototype-target-neutral-fusion-slice
title: Fuse serial Sum into one verified target-neutral program
status: done
priority: p0
dependencies: [prototype-target-neutral-baseline-slice]
related: [prototype-shared-compiler-ir-ownership, harden-compiler-verifier-subject-binding-and-totality]
scopes: [implementation/compiler, implementation/artifact, implementation/ir, implementation/workspace]
shared_scopes: [project/tickets, contracts/optimizer, contracts/artifacts, contracts/foundation, contracts/numerics, contracts/navigation, implementation/cargo-lock]
paths: []
tags: [implementation, prototype, compiler, vertical-slice]
---
Starting from the proven materialized compiler path, add exactly the optimizer
behavior that the first value proof exists to test:

- retain complete singleton coverage and the verified two-stage materialized
  baseline as a complete equivalent program;
- enumerate the connected convex pointwise-plus-strict-`Sum` region and prove
  that removing its `f32` intermediate preserves operation order, canonical-NaN
  boundaries, dtype boundaries, shapes, accesses, and named results;
- schedule one thread per output so each thread computes the pointwise prologue
  for each contributor immediately before the strict left-fold combine;
- refine and verify the one-entry structured kernel, including exact semantic
  coverage, read bounds, unique writes, resource feasibility, and numerical
  realization; and
- deterministically select the fused complete program only after hard
  feasibility, using the bounded rule that it eliminates one dispatch and one
  global intermediate while preserving the materialized program as the
  explicit reference alternative.

The output contains both complete programs in a form the Metal proof can emit
and test, with the fused program selected by the fixed routing policy. Golden
tests must demonstrate one versus two kernel stages, absence versus presence of
the intermediate, deterministic identity, semantic equivalence evidence, and
stable explanations for both successful fusion and each rejected legality or
feasibility case. Do not add unrelated fusion patterns, a memo, a general
region partitioner, calibrated costing, Metal emission, serialization, or
device execution.

## Outcome

- Enumerated a deterministic seven-candidate occurrence projection with five
  singletons, the complete pointwise region, and the complete fused region;
  membership, boundaries, connectivity, convexity, and candidate budget fail
  closed.
- Proved the bounded fused plan preserves separate `f32` multiply/add
  operations, canonical arithmetic-NaN boundaries, original contributor order,
  positive-zero empty identity, dtype boundaries, and forbidden contraction,
  reassociation, and permutation.
- Added and verified a one-thread-per-output fused schedule and explicit
  structured kernel alongside the existing two-stage materialized program.
- Produced a deterministic two-alternative portfolio with provider revisions,
  exact dispatch/allocation/global-intermediate metrics, stable explanations,
  and selection only under strict structural Pareto dominance.
- Differentially checked the fused test interpreter against `tiler-reference`
  on finite, empty, singleton, signed-zero, subnormal, infinity, NaN, and
  contraction-sensitive vectors; malformed schedules, kernels, programs,
  providers, and budgets remain explicit failures or baseline retention.

This completes only the private target-neutral proof. It adds no public
compiler API, general fusion search, calibrated performance model, Metal
lowering, serialization, or runtime routing implementation.
