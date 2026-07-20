---
id: prototype-target-neutral-fusion-slice
title: Fuse serial Sum into one verified target-neutral program
status: todo
priority: p0
dependencies: [prototype-target-neutral-baseline-slice]
related: []
scopes: [implementation/compiler, implementation/artifact, implementation/ir]
shared_scopes: [project/tickets, contracts/optimizer, contracts/artifacts, contracts/foundation, contracts/numerics]
paths: [Cargo.lock]
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
