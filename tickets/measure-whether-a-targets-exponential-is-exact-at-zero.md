---
id: measure-whether-a-targets-exponential-is-exact-at-zero
title: Measure whether a target's exponential is exact at zero
status: deferred
priority: p3
dependencies: []
related: [derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, transcendentals]
---
## User-visible outcome

Whether a target's `exp` returns exactly `1.0` at a zero argument is a measured target fact rather than an assumption, so the online-softmax rescaling bound can drop the elementary term it currently charges on the winning side of every merge.

## Why this exists

**Fact.** The merge operator `(m1,d1) + (m2,d2) = (max, d1*exp(m1-max) + d2*exp(m2-max))` evaluates `exp(0)` on whichever side holds the maximum. [The tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md) charges `eps_exp` and a rounding for it, because nothing in the target vocabulary says the result is exact.

**Fact.** Metal's Table 8.1 states `exp <= 4 ulp` under Apple's own definition, relayed in [the Metal elementary-function accuracy record](../docs/research/numerics/metal-elementary-function-accuracy.md). A 4-ulp bound at `1.0` does **not** imply exactness at zero, so the conservative reading is currently required rather than merely chosen.

**Inference.** If the result is exact, the winning side's rescale factor is exactly `1`, its multiply is exact, and roughly half of the elementary evaluations and half the rescale roundings leave the bound. The derived price is `D * (u + eps_exp)`, so the saving is a constant factor on an already-small quantity — which is why this is a bounded measurement rather than a research wave.

## What this ticket must produce

- One measurement per honoured target row: the exact bits `exp(+0.0)` and `exp(-0.0)` return, on the device rather than in an offline compiler, since a runtime-compiled kernel is what would execute the fold.
- The result recorded against the target row that claims it, per the discipline that a device measurement belongs to the host whose hardware row it claims.
- The sharpened bound stated in the tree-fold record if and only if a target answers exactly, with the unmeasured case staying conservative rather than assumed.

## Non-goals

Widening the transcendental accuracy contract; measuring `exp` anywhere but at zero; changing the bound's form.

## Trigger

The first target profile that declares a numeric elementary accuracy a parametric bound can instantiate from — because until a bound is instantiated on a real target, sharpening its constant changes no answer.

## Trigger check log

- 2026-08-06 — not fired. No target authority yields a numeric `eps_exp`; [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md) is the ticket that would change that and is open. Reproduce with `tkt show expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`.
