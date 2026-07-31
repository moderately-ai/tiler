---
id: admit-the-contraction-normative-reference
title: Admit the contraction normative reference and its exceptional-value corpus
status: todo
priority: p1
dependencies: [admit-the-contraction-semantic-profile]
related: [implement-parallel-reduction-strategies, reduction-semantics-contract]
scopes: [implementation/reference, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, numerics, contraction, reductions]
---
## User-visible outcome

The reference evaluator computes the admitted contraction, so every later realization has a target-independent answer to be bit-compared against instead of being compared to itself.

## The formula, and the part of it that is easy to get wrong

**Proposal — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md).** For each output coordinate `(t, o)`, over the canonical ascending contributor sequence `d = 0 .. K-1`:

```text
p_d = fl(A[t, d] * B[o, d])      # one rounding each, round-to-nearest ties-to-even
acc = p_0                        # the FIRST product, not +0.0
for d in 1 .. K-1: acc = fl(acc + p_d)
```

**Measurement — the seed is observable and the idiomatic loop gets it wrong.** `fl(+0.0 + x)` equals `x` for every binary32 `x` except `x = -0.0`, where it is `+0.0`. On the spike's `negative_zero_seed` case, where every product is `-0.0`, a first-product-seeded fold returns `0x80000000` and a `+0.0`-seeded one returns `0x00000000`. [Numerical semantics](../docs/numerical-semantics.md) states the same rule for the registered strict sum and gives the same counterexample under reduction padding. **This must be a regression test that fails before the evaluator is written correctly**, not a comment.

**Fact — a `+0.0` seed is a different operation, not a defect.** It is a reduction carrying an explicit `initial`, which the reduction contract admits as one logical contributor. The evaluator must be able to express both and must not silently supply one.

## Required delivery

- A `tiler-reference` evaluator for the admitted key, with the accumulator dtype, the contributor order, the empty-domain declaration, and the seed all read from the operation's own signature rather than defaulted.
- Per-combine and result-boundary canonicalization to `tiler::canonical-arithmetic-nan-f32@1`, on the same rule the registered strict serial sum already carries. **Open decision D-8** of the L3 record asks whether per-combine canonicalization is required of a contraction or only at its boundary; this ticket is where it gets answered, because the answer changes what a matrix instruction could ever satisfy.
- The exceptional-value corpus, at minimum the spike's eight cases: an execution witness, order absorption, a separately-rounded-against-fused discriminator, the signed-zero seed, a non-canonical NaN payload, `inf * 0` formed inside the reduction, a subnormal product, and a vector separating the contiguous from the strided split. Their exact operand bit patterns are retained in `spikes/scheduling/metal_contraction_vertical/results/.../semantics-candidates.tsv`.
- A statement of which conformance level the evaluator's own results claim.

## Non-goals

Any schedule, any backend, any tolerance for a model-level comparison.

## Closes when

The evaluator reproduces every retained candidate value for `strict_fold` in the spike's semantics record, the signed-zero seed test was watched failing against a `+0.0`-seeded implementation, and D-8 is answered in the operation's declared signature rather than left to the implementation.
