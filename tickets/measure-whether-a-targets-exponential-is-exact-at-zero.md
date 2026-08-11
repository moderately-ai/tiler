---
id: measure-whether-a-targets-exponential-is-exact-at-zero
title: Measure whether a target's exponential is exact at zero
status: in-progress
priority: p3
dependencies: []
related: [derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate]
scopes: [research/numerics, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, transcendentals, measurement, trigger-fired]
claimed_from: todo
assignee: sol-exp-zero
lease_expires_at: 1786423431
---
## User-visible outcome

Whether a target's `exp` returns exactly `1.0` at a zero argument is a measured target fact rather than an assumption, so the online-softmax rescaling bound can drop the elementary term it currently charges on the winning side of every merge.

## Why this exists

**Fact.** The merge operator `(m1,d1) + (m2,d2) = (max, d1*exp(m1-max) + d2*exp(m2-max))` evaluates `exp(0)` on whichever side holds the maximum. [The tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md) charges `eps_exp` and a rounding for it, because nothing in the target vocabulary says the result is exact.

**Fact.** Metal's Table 8.1 states `exp <= 4 ulp` under Apple's own definition, relayed in [the Metal elementary-function accuracy record](../docs/research/numerics/metal-elementary-function-accuracy.md). A 4-ulp bound at `1.0` does **not** imply exactness at zero, so the conservative reading is currently required rather than merely chosen.

**Inference.** If the result is exact, every path step whose rescale argument is zero (winning side of a merge, or a leaf already at the running max) drops its `eps_exp` charge and its rescale multiply is exact. That does not license halving `E`: the published price is path-based with `E = 1 + D`, and under a strict max jump at every merge the deepest always-losing path may still pay a non-zero rescale at every level, so exactness of `exp(0)` can drop far less than half of that path's charges. The conservative unsharpened first-order price remains `D * (u + eps_exp)`; any sharpened form must be re-derived from the path counting rather than obtained by slogan. The saving is therefore a bounded constant improvement on an already-small quantity — which is why this is a bounded measurement rather than a research wave.

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
- 2026-08-09 — **fired.** The related ticket `expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate` is `done`. `elementary_relative_accuracy` now obtains an exact rational from the governed target/requirement authority, including `24u` for the registered softmax and SiLU requirements and a typed magnitude domain. A real target row can therefore instantiate the parametric quantity this zero-point measurement sharpens. The measurement is no longer speculative and moves to `todo`; `research/apple-targets` is added because the required device result and environment row belong in the Apple target record, not only in the generic numerical derivation.
