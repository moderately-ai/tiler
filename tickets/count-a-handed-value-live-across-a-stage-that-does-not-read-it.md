---
id: count-a-handed-value-live-across-a-stage-that-does-not-read-it
title: Count a handed value live across a stage that does not read it
status: deferred
priority: p3
dependencies: []
related: [carry-a-multi-reader-intermediate-through-region-formation, admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, register-the-softmax-realization-law]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, indexing, deferred]
---
## User-visible outcome

A region covering a realization stage that a live handed value merely *spans* — published before it, read after it, and neither produced nor read by it — accounts for that value, or the record says why a region's live-value bound deliberately does not.

## Why this exists

**Fact.** `crates/tiler-compiler/src/region.rs`'s `region_shape` visits a member's handed values per covered stage and skips any value the stage neither publishes nor reads (`if !produced_here && !consumed_here { continue; }`). So a value published by stage `p` and read at stage `r` contributes nothing to a candidate covering a stage strictly between them, and `RegionShape::live_values` — the demand the `region-live-values` budget is compared against — does not count it.

**Fact.** The span is available and checked. `StagedIntermediate::retained_through` is derived by `VerifiedIndexRegionSequence::try_new` from the declared readers, and [`carry-a-multi-reader-intermediate-through-region-formation`](carry-a-multi-reader-intermediate-through-region-formation.md) carries it onto the compiler's per-value record, so `producer_stage < stage <= retained_through` is answerable at exactly the site that currently skips.

**Fact — nothing is wrong today, and the gap is expressible.** No law spells a chain with such a gap. `crates/tiler-ir/src/index/law.rs` carries exactly two staged forms — `StagedStrictSerialSumThenPointwiseF32` (sources `[[Occ(0)], [Occ(1), Int(0)]]`) and `StagedRootMeanSquareScaleF32` (`[[Occ(0)], [Occ(0), Occ(1), Int(0)]]`) — and both are two-stage chains whose one handed value is read by the stage immediately after its producer. The softmax's four-stage shape has no gap either: its reads are `(0,1), (1,2), (1,3), (2,3)`, so every stage between a producer and its last reader is itself a reader. The sequence vocabulary admits one, though: sources `[[Occ(0)], [Occ(0)], [Int(1)], [Int(0), Int(2)]]` publishes at stage zero for a stage-three reader with stages one and two spanning it, and `VerifiedIndexRegionSequence::try_new` accepts it.

## The decision this needs

Counting the spanning value is not obviously right, which is why this is parked rather than taken. A region that neither reads nor writes the value would then report a live value that appears in neither its boundary inputs nor its retained outputs, and the cover layer has no materialization edge for it — so the region's own boundary description and its live count would disagree. The alternative reading is that a spanning value must be threaded through the covering region as an explicit pass-through boundary, which changes boundary derivation and therefore region identity, and is a public-shape decision rather than a counter fix.

## Trigger

A registered realization law publishes a value whose last reader is later than a stage that does not read it, or a physical or cover authority asks region formation for a stage's live set rather than its boundary set.

## Closes when

Either the spanning value is accounted for at the named site with its consequence for boundaries and identity settled, or a recorded decision states that a region's live-value bound counts boundary and member-result values only, with the reason.

## Trigger check log

- 2026-08-06 — **not fired.** Both staged `IndexRealizationLaw` arms were read on `tkt/carry-a-multi-reader-intermediate-through-region-formation` at base `f0132c88`, and each declares a two-stage chain whose handed value is read at the stage after its producer. Reproducing check: `grep -n "Staged\|realizes_region_sequence" crates/tiler-ir/src/index/law.rs`, then read each staged arm's declared `StagedInputSource` lists for a producer whose readers skip a stage.
- 2026-08-09 — **not fired; the old two-arm population is stale.** The current production laws include the two-stage sum/pointwise and RMS-scale sequences plus the four-stage softmax sequence. Reading each `VerifiedIndexRegionSequence::try_new` source list shows every intermediate is read in the immediately following stage; softmax's second intermediate is also read again later, but the intervening denominator stage reads it too. No handed value crosses a stage that does not read it, so neither trigger disjunct is present. Recheck at the anchors `Builds the two-stage fold-then-pointwise realization`, `Builds the two-stage root-mean-square scale realization`, and `Builds the four-stage softmax realization` in `crates/tiler-ir/src/index/law.rs`.
