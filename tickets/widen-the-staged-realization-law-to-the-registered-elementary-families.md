---
id: widen-the-staged-realization-law-to-the-registered-elementary-families
title: Widen the staged realization law to the registered elementary families
status: in-progress
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-law-widening
lease_expires_at: 1786036406
---
## User-visible outcome

`tiler::rms-norm-f32@1` -- and, once its scalar lands, `tiler::softmax-f32@1` -- carries a registered `IndexRealizationLaw`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for them and refinement can prove a provider's emitted region sequence realizes the occurrence.

## Why this exists: the accepted staged template expresses neither family

**Fact, and it corrects a premise.** [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) states that "the law registrations then use the accepted staged template (or a single-region law where the family is one region)". Read against `crates/tiler-ir/src/index/law.rs`, that is false for both registered elementary families. `IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32` (`law.rs:106-111`) is realized by `realize_staged_sum_then_pointwise` (`law.rs:953-1017`), and its exact shape is:

- **stage zero** is `SumPlan::for_boundaries` over operand zero with no prologue -- a plain strict left fold of the operand's own elements (`law.rs:967-982`);
- **stage one** is `emit_pointwise` applying one *binary* scalar to operand one and the published fold, with the fold legible only at the result shape or at rank zero (`law.rs:984-1004`).

The law's own doc-comment (`law.rs:98-105`) already says it "is deliberately *not* the normalization's own law". What the ticket above assumed is that the remaining gap was scalar keys; it is the template.

### The normalization

**Fact.** `rms_norm_f32_reference_semantics` (`crates/tiler-ir/src/semantic/rms_norm.rs:228-238`) pins `q_i = x_i * x_i`, `a = fold(q)`, `u = a / N`, `t = u + eps`, `r = Rsqrt(t)`, `y_i = w_i * (x_i * r)`. Three of those are outside the template:

1. the fold is over `x_i * x_i`, and the template's stage zero folds the operand's elements directly -- `SumPlan` has no prologue;
2. the published intermediate is transformed before the pointwise pass consumes it (`/ N`, `+ eps`, `Rsqrt`), and the template's stage one applies exactly one scalar;
3. stage one reads *three* values -- the weight, the normalized value, and the intermediate -- where `emit_pointwise` refuses any operand count other than two (`law.rs:707-709`, rule `pointwise-operand-arity`).

**And a silent-wrongness hazard if the template were registered anyway.** `reduction_axes` reads its attribute by field ID and tolerates extra fields (`law.rs:1396-1402`), while `realize_constant` refuses a record whose field set it does not expect. The normalization declares two attributes -- the axis and the exact `eps` bits -- so registering the staged template for it would drop `eps` with no refusal, and `eps` is part of the operation's identity (`rms_norm.rs:76-94`). Any widening must consume every declared attribute or refuse by name.

### The softmax

**Fact.** `softmax_f32_reference_semantics` (`crates/tiler-ir/src/semantic/softmax.rs:394-408`) pins a *maximum* fold, then `e_i = Exp(s_i - m)`, then a *second* fold summing `e`, then `c = 1.0 / d`, then `r_i = e_i * c`. That is at least three regions with two distinct folds, the first of which has no registered scalar combiner at all -- [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md) owns that. The template has one fold and one pointwise pass.

## Scope, and what is reachable now

The normalization half is reachable today: `rsqrt_f32_scalar_op` landed as a draft and every other scalar the reference names is registered. The softmax half is not, and waits on the maximum key.

The widening is a public surface: `IndexRealizationLaw` is `pub` and `#[non_exhaustive]`, so a new variant lands as a labelled draft with its own acceptance node. Its encoding tag must be appended -- tags `1..=9` are taken, and `the_staged_law_tag_is_append_only_and_distinct` (`law.rs`) is the pattern -- with per-tag injectivity reasoning recorded at the encoding site.

## Non-goals

Making the compiler *recognize* the family as a program stage. That is blocked on a separate fork -- see [`resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`](resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md) -- and a registered law is useful without it: it flips `the_normalization_still_refuses_for_an_absent_law_and_not_for_the_vocabulary` (`crates/tiler-compiler/tests/two_region_occurrence_lowering.rs:1005-1049`) and lets refinement verify an emitted sequence.

## Closes when

At least the normalization's law is registered and realizes a verified `VerifiedIndexRegionSequence` whose stages match the pinned reference step for step, every declared attribute is consumed or refused by name, the new encoding tag is proved append-only and injective, and the wall test above is flipped rather than deleted.
