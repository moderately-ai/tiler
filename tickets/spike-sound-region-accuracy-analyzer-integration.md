---
id: spike-sound-region-accuracy-analyzer-integration
title: Spike sound region-accuracy analyzer integration
status: done
priority: p2
dependencies: [research-region-accuracy-contracts-and-analyzable-error-budgets]
related: []
scopes: [research/numerics]
shared_scopes: []
paths: []
tags: [tiler-research, numerics, accuracy, spike]
---
Integrate or invoke a sound roundoff analyzer such as Daisy or FPTaylor for a
bounded scalarization of fixed-shape, branch-free tensor regions. Compare
certified bounds against adversarial MPFR observations without confusing the
observations for proof.

Start with `+`, `-`, `*`, `/`, `sqrt`, casts, and explicit FMA. Add a small
fixed reduction only after scalar cases work; transcendental approximations
require a tool profile that actually proves their implementation error.

Measure supported/unsupported reason codes, bound tightness, proof and checking
time, sensitivity to interval subdivision and relational assumptions, and the
effect of casts, contraction, topology, overflow, and subnormal policies. Bind
every certificate to the canonical goal, candidate physical/numerical identity,
target profile, and assumptions. The outcome should decide whether a small
sound profile is practical; `Unknown` must remain a correct rejection.
