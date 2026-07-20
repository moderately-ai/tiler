---
id: research-region-accuracy-contracts-and-analyzable-error-budgets
title: Research region accuracy contracts and analyzable error budgets
status: done
priority: p2
dependencies: [numerical-policy-contract, reference-evaluator-slice]
related: []
scopes: [research/numerics]
shared_scopes: []
paths: []
tags: [tiler-research, research, numerics, accuracy, spike]
---
Define the optional accuracy layer above mandatory per-operation numerical
contracts. A region/output goal is a hard feasibility constraint over an
observable result; it does not replace operation semantics or become another
optimizer cost.

The contract model must name:

- observable output or region;
- explicit reference semantics or oracle;
- input value/range assumptions and shape bounds;
- metric such as absolute, relative, ULP, or a deliberately supported mixed
  metric;
- tolerance and exceptional-value behavior; and
- evidence status distinguishing sound proof, empirical validation with a
  named test definition, and unknown.

Do not assume local error bounds compose by addition. Account for sensitivity,
cancellation, correlation, casts/materialization, overflow/subnormals,
transcendentals, reductions, and branch/path instability. Preserve local
semantic legality independently: a graph budget cannot authorize contraction,
reassociation, or rounding-boundary removal that the operation contracts
forbid.

## Initial research synthesis

LLVM, XLA, StableHLO, and PyTorch primarily expose local operation permissions,
algorithms, or accuracy controls. Whole-expression tools form a distinct
layer: FPTuner assigns operator precision against a worst-case output error over
bounded inputs; Daisy soundly over-approximates error from a real-valued
reference and preconditions; Precimonious and Herbie use empirical/sample-based
objectives that do not establish the same guarantee.

Local-only semantics remain extensible if casts, materialization boundaries,
reduction topology, input/shape assumptions, and rewrite provenance survive.
An opaque `fast` boolean or erased reference graph would block later sound
analysis.

Primary starting points:

- https://llvm.org/docs/LangRef.html#fast-math-flags
- https://openxla.org/xla/operation_semantics
- https://openxla.org/stablehlo/compatibility
- https://soarlab.org/papers/2017_popl_cbbsgr.pdf
- https://link.springer.com/chapter/10.1007/978-3-319-89960-2_15
- https://herbie.uwplse.org/doc/latest/error.html

## Bounded spike

Restrict the first experiment to fixed-shape, branch-free, acyclic regions with
`+`, `-`, `*`, `/`, `sqrt`, `exp`, `expm1`, casts, FMA, and reductions at a
small fixed set of extents. Compare strict reference, locally relaxed plans,
and globally tuned precision/order choices against MPFR. Where scalarization is
manageable, compare a sound analyzer such as Daisy or FPTaylor with observed
absolute, relative, and ULP errors.

Record proof-bound tightness, analysis time, chosen precision/casts/topology,
runtime, and unsupported cases. Random/adversarial measurements remain
empirical evidence and cannot satisfy a sound contract. The gate is whether a
sound analyzer can certify modest fused tensor regions quickly enough and with
non-vacuous bounds.

## Completed outcome

The typed goal/evidence model and compiler boundary are recorded in
`docs/research/numerics/region-accuracy-contract.md`. A dependency-light
`mpmath` adversarial probe exercises materialization removal, cancellation,
reference choice, relative error at zero, and reduction topology. It is
explicitly empirical and therefore does not claim a worst-case certificate.

Actual sound-analyzer integration, certificate tightness, and proof-time
measurement are separated into
`spike-sound-region-accuracy-analyzer-integration`; delegated region budgets
remain disabled until that feasibility gate passes.
