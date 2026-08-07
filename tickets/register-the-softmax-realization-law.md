---
id: register-the-softmax-realization-law
title: Register the softmax realization law
status: todo
priority: p1
dependencies: []
related: [admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, widen-the-staged-realization-law-to-the-registered-elementary-families, admit-the-softmax-family]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, numerics]
---
## User-visible outcome

`tiler::softmax-f32@1` carries a registered `IndexRealizationLaw`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for it and refinement can prove a provider's emitted region sequence realizes the occurrence. It is the third and last piece of the softmax vertical: the two walls are down.

## Why it has no dependencies

**Fact.** Both walls landed together on `tkt/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`. [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md) registered `tiler.scalar::maximum-f32@1`, and [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md) widened `VerifiedIndexRegionSequence` to a published value with several readers. Both are labelled drafts with acceptance nodes parked for Tom; neither node blocks *use* inside `tiler-ir`, exactly as the normalization law's did not.

## What the law must realize

**Fact.** `softmax_f32_reference_semantics` (`crates/tiler-ir/src/semantic/softmax.rs`) pins, over the single reduced axis and in this exact order: `m` = the strict left fold of the NaN-propagating `Maximum` family over the canonical contributor sequence seeded at the first contributor; `e_i = Exp(s_i - m)`; `d` = the strict left fold sum of `e` over the same sequence seeded at the first contributor; `c = 1.0 / d` as one division of one by the denominator; `r_i = e_i * c` as a multiplication by that reciprocal and deliberately not `e_i / d`.

**The staging, now expressible.** `the_softmax_staging_publishing_the_exponentials_chains` in `crates/tiler-ir/src/index/sequence.rs` checks the shape at the sequence layer: four stages, sources `[[Occurrence(0)], [Occurrence(0), Intermediate(0)], [Intermediate(1)], [Intermediate(1), Intermediate(2)]]`, with the exponentials published by stage one and `retained_through` stage three. That test proves the *chain* is well formed; it does not emit the softmax's scalar programs, which is this ticket's work.

## The three capabilities the existing emitters do not have

Stated so the elimination starts from what is missing rather than from a template guess. Compare `realize_root_mean_square_scale` in `crates/tiler-ir/src/index/law.rs`:

1. **A fold whose combiner is not `add-f32`.** `SumPlan::fold` hard-codes `add_reducer`, and the maximum fold has no identity, so it is seeded at the first contributor rather than at a constant — which `SumPlan`'s empty/tail split already contemplates for the sum but with a `0.0` seed. `SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS` says a zero-length axis yields a zero-length output with no scalar softmax evaluated, so the identity-less fold must never face an empty contributor domain, and the shape rule is what makes that unreachable rather than merely undefined.
2. **A middle stage that is neither a fold nor a two-operand pointwise pass.** Stage one reads the scores at their own coordinates and the row maximum at the kept coordinates, and writes `Exp(s_i - m)` — a subtraction and an elementary function between the read and the write. `emit_pointwise` applies exactly one scalar.
3. **A final stage reading two published values of different ranks.** `e` at the point coordinates and `d` at the kept coordinates, plus the reciprocal `c = 1.0 / d` computed once per row rather than once per point.

**Where generality should go, per the worked-examples discipline and the precedent this vertical already set.** The normalization's ticket closed three gaps as reusable *emitters* rather than as one family's inline code. The same rule applies here: a fold parameterized by its combiner and its seeding rule, and a stage that reads a reduced-rank published value at kept coordinates (which already exists), are the reusable pieces. The next staged family instantiates those.

## The scalar-program sibling need, named by the fold-with-epilogue acceptance

[`accept-the-root-mean-square-scale-realization-law`](accept-the-root-mean-square-scale-realization-law.md) records, as a choice worth objecting to and accepted with no exclusion: "**The chain is fixed; only the two attribute identifiers are law data.** … Carrying them would need a scalar-program language inside a law, which is the universal IR `law.rs`'s header refuses, or five independently settable keys whose combinations this module could no longer claim to interpret. The counter-argument, and the honest one: a second width's normalization would need a second variant rather than a second row."

**That consequence lands here, and it is why this ticket must not reach for a general template.** The softmax is the second family whose chain is fixed by the template rather than carried in it, so it takes its own variant with its own tag — tags `1..=10` are taken — and its own record-local attribute identifier. Two fixed-chain variants is where the trade the acceptance named starts costing: a reviewer should read this ticket's outcome asking whether a third one is still the right shape, or whether the accepted refusal of a scalar-program language inside a law needs reopening with new evidence. Reopening it is *not* in this ticket's scope; naming the evidence it produces is. Record, in the outcome, how much of the two variants' emission is shared machinery and how much is per-family chain, because that ratio is the measurement the reopening question would need.

## Public boundary

`IndexRealizationLaw` is `pub` and `#[non_exhaustive]`, so a new variant lands as a labelled draft with its own acceptance node parked for Tom, as `StagedRootMeanSquareScaleF32` did. The encoding tag must be appended with per-tag injectivity reasoning recorded at the encoding site, and `the_root_mean_square_law_tag_is_append_only_and_distinct` is the pattern.

## Non-goals

Making the compiler *recognize* the softmax as a program stage. Region formation's synthetic-intermediate record carries one consumer stage per handed value and needs widening first — [`carry-a-multi-reader-intermediate-through-region-formation`](carry-a-multi-reader-intermediate-through-region-formation.md). A registered law is useful without it: it lets refinement verify an emitted sequence.

## Closes when

`tiler::softmax-f32@1` resolves to a registered law that realizes a verified `VerifiedIndexRegionSequence` whose stages match the pinned reference step for step — the extrema family, the maximum subtraction, the exponential, the sum's seeding and order, the single reciprocal division, and the reciprocal multiplication — the new encoding tag is proved append-only and injective, every declared attribute is consumed or refused by name, every existing chain's identity is unchanged byte for byte, and the acceptance node is filed.
