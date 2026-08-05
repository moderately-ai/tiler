---
id: search-the-p-elem-sign-assignment-for-the-model-level-band
title: Search the P-elem sign assignment for the model-level band
status: deferred
priority: p2
dependencies: [measure-the-model-level-comparison-envelope-under-the-target-realization]
related: [design-model-level-qualification-and-optimization, define-the-model-level-conformance-corpus, define-first-metal-lm-workload, prove-the-c1-complete-model-execution]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, conformance, measurement, deferred]
---
## The gap this exists to close, and why closing it now would be waste

**Fact.** [`measure-the-model-level-comparison-envelope-under-the-target-realization`](measure-the-model-level-comparison-envelope-under-the-target-realization.md) measured the joint band at **2.2101e-4** by moving every subordinate elementary-function result to the edge of its registered contract's admitted band, under two sign policies — `outward`, which is worst where a perturbation propagates through a sum and cancels exactly inside a softmax normalization, and `alternating`, which is taken from the result's own low mantissa bit and so does not cancel there. The retained record states the limitation in three places: each policy is a full-magnitude *sample* of the admitted band, not a search over the 2^N per-element sign assignments, so the true worst case within `Ulp(tiler::ulp-reference-gap@1, 12)` and `Faithful` is at least the measured band and is **not bounded above by it**.

**Inference — the asymmetry decides whether that matters.** A Tiler result *outside* the band is read as a defect. If the searched worst case is materially wider than the sampled band, that reading produces a false defect on a legal realization. On C1 the question is currently academic: the band sits about **1,204×** below this row's smallest runner-up gap, and the exact-greedy gate would survive a band three orders of magnitude wider. Searching now would spend a per-output adversarial search on a margin nothing is near.

## Activation triggers

Any one of these fires it; none has:

1. A Tiler result lands within **10×** of the retained band at any C1 position, in whole-vocabulary or top-32 terms.
2. A Tiler result lands *outside* the band and the greedy token nonetheless agrees, which is the shape a too-narrow band produces and a genuine defect usually does not.
3. A corpus row is admitted whose smallest runner-up gap is within **100×** of the band, so the exact-greedy gate stops having room to spare.
4. A registered accuracy contract subordinate to `tiler::softmax-f32@1`, `tiler::silu-f32@1`, or `tiler::rms-norm-f32@1` widens, or a fourth subordinate elementary function joins them.

## What the work would be

Bound the worst case rather than sample it. The tractable direction is per-output rather than global: for one retained observable at a time — a top-32 logit, or an attention output lane — the locally worst sign assignment is derivable from the sensitivity of that observable to each perturbed result, which is a directional derivative the reference can supply. That yields a per-observable worst case, and the honest deliverable is the maximum over the retained observables together with an explicit statement of what a whole-vocabulary claim would still need. A brute-force search over sign assignments is not the work; it is exponential and would produce a number with no argument attached.

## Explicit non-goals

No threshold: this widens or confirms a measured band and decides nothing about what is gated on it. No new prompt, checkpoint, or target row — a broader corpus is [`define-the-model-level-conformance-corpus`](define-the-model-level-conformance-corpus.md)'s. No re-derivation of the contracts themselves.

## Trigger check log

- 2026-08-04 — **not fired.** Triggers 1, 2, and 3 all require a *Tiler* result at a C1 position, and no Tiler execution of the workload exists ([`prove-the-c1-complete-model-execution`](prove-the-c1-complete-model-execution.md) is `todo`). Trigger 4 is unmet: no accuracy contract subordinate to `tiler::softmax-f32@1`, `tiler::silu-f32@1`, or `tiler::rms-norm-f32@1` has widened and no fourth subordinate has joined them.
