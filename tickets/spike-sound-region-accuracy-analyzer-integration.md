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

## Outcome

The [research report](../docs/research/numerics/sound-region-analyzer-spike.md)
and [preserved Daisy harness](../spikes/numerics/sound_accuracy/README.md)
demonstrate a bounded trusted-analyzer profile for fixed branch-free scalarized
regions. The measured profile is practical with caching, but produces no
independently checkable certificate; FPTaylor/HOL Light remains a
[deferred follow-up](spike-hermetic-fptaylor-certificate-checking.md).

## Evidence correction (2026-07-21)

The [numerical witness repair](repair-numerical-witness-integrity.md) and
[current report](../docs/research/numerics/sound-region-analyzer-spike.md)
show that the historical Daisy bounds lack retained raw analyzer streams and
cannot be freshly reproduced by the repaired adapter, which returns `Unknown`.
They are historical analyzer-reported values, not current `SoundProof`
evidence; current empirical maxima come from the retained observation record.
