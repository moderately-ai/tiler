---
id: name-the-proved-extent-equality-bounds-proof
title: Name the proved-extent-equality bounds proof
status: done
priority: p1
dependencies: []
related: [bind-shapeenv-sources-into-tensor-boundaries-and-coefficients, name-the-unprovable-symbolic-extent-diagnostic, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, shapes, correctness]
---
**Fact — what landed.** `bind-shapeenv-sources-into-tensor-boundaries-and-coefficients` made a tensor boundary's extents `SourcedExtent`s and taught `write_is_permutation` to compare them through `ExtentSources::proves_equal`, which decides equality from the `ShapeEnv` equality classes. A dynamically shaped write's *ownership* argument therefore now succeeds with no value known for either extent.

**Fact — bounds do not, and the gap is structural rather than a missing environment fact.** For the ordinary caller-sized copy — input `[m]`, output `[m]`, domain sized `n`, `m == n`, nothing determined — the region is refused. `crates/tiler-ir/src/index/sourced.rs::a_wholly_undetermined_dynamic_copy_is_refused_rather_than_approximated` measures exactly this: `BoundsNotProven` on the read, `WriteOwnershipNotProven` on the write, and deliberately **not** `ProofResourceLimit`.

The reason is that interval propagation cannot express the argument. `n`'s interval is the whole extent domain, so `max(i)` is nowhere below `m`'s floor. The sound argument is a different one and it is cheap: a coordinate that *is* `IndexNode::Dimension(d)` ranges over `[0, extent(d))` by construction, so when the environment proves `extent(d)` equal to the axis extent, the coordinate is in bounds in every model — with no bound on either needed. That is the same per-axis obligation `write_is_permutation` already discharges, used for a read and one axis at a time.

**Why it was not done there.** The retained evidence has to say *how* an access was proved. `BoundsProofView` offers `VacuousEmptyDomain`, `Interval`, and `Exhaustive`, and each would misdescribe this argument — `Interval` most dangerously, since nothing about either interval closed the question. An access whose recorded proof kind is wrong is worse than one that is refused, so the rule was left out rather than landed under a borrowed name. Adding the variant is a public API addition on `tiler_ir::index::BoundsProofView`, which is owner-reserved.

**The shape of the public change.** `BoundsProofView` is already `#[non_exhaustive]`, so a variant is additive for out-of-crate matchers. The internal `BoundsProof` is `pub(super)`, and `TensorAccessRef::bounds_proof` is the only mapping that must grow.

## Scope

Add the proof kind, admit the per-axis structural bounds rule wherever `interval_verdict`'s answer is consumed, and record it in `remap_access` so the retained evidence names the argument actually used. Preserve the existing precedence — interval first, this second, enumeration last, refusal otherwise — because an access interval propagation already proved must keep recording `Interval`, or existing regions' retained evidence silently changes meaning.

One decision this ticket owns: whether the same rule also relaxes the `!interval_proved` conjunct that a proven `CoordinatePermutation` write is currently *also* required to satisfy. For a static boundary the permutation implies it, so the conjunct is redundant there; for a symbolic one it is exactly what blocks the case above.

## Closes when

The wholly undetermined `[n] -> [n]` copy verifies, its retained bounds evidence names the equality argument rather than an interval or an enumeration, the neighbour whose environment does not prove the extents equal is still refused, `a_wholly_undetermined_dynamic_copy_is_refused_rather_than_approximated` is replaced by its accepting successor rather than deleted, and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

`BoundsProofView::ProvedExtentEquality` landed, additively on the already-`#[non_exhaustive]` enum, with the private `BoundsProof::ProvedExtentEquality` beside it and `TensorAccessRef::bounds_proof` growing the one arm the ticket predicted.

### The rule

`IndexRegionBuilder::coordinates_are_bounded_dimensions` holds when every coordinate of an access *is* an `IndexNode::Dimension(d)`, `d` is iterated by that access, and `extents_proved_equal` proves `extent(d)` and the axis it indexes are one extent. That is per-axis and needs no bound on either side, which is exactly why it decides the case interval propagation cannot: a wholly undetermined symbol's interval is the whole extent domain, so no comparison against it closes.

It is deliberately **not** a permutation check. Two axes may name the same dimension and each still be in bounds; covering a boundary exactly once stays `write_is_permutation`'s obligation. Conflating them would either refuse a legal read or let a write claim ownership it has not shown.

### The decision this ticket owned — the `!interval_proved` conjunct is subsumed, not removed

The ticket asked whether the same rule should also relax the interval conjunct a proven `CoordinatePermutation` write is additionally required to satisfy. It needed no separate answer: `write_is_permutation` requires *exactly* the per-axis condition above plus distinct dimensions and equal ranks, so it implies `coordinates_are_bounded_dimensions`. Once the bounds disjunction admits the structural argument, a write that owns its boundary satisfies the bounds obligation through the argument that actually holds. The conjunct was therefore not deleted; it stopped being reachable for a proven permutation, and the verifier still discharges bounds explicitly rather than inheriting them from ownership.

### Precedence, and why it is the evidence that constrains it

`verify_accesses`, `access_needs_exhaustive_proof`, and `remap_access` all read one predicate, `bounds_proved_without_enumeration` — interval first, structural equality second — so the pass that decides an enumeration is needed and the pass that records what proved it cannot drift. The order is load-bearing for the retained evidence rather than for soundness: an access interval propagation already proved keeps recording `Interval`, so no existing region's evidence changes meaning. `VacuousEmptyDomain` still precedes both.

### Identity is untouched, deliberately

`tiler.index-region.v6` encodes an access as mode, tensor, domain, and coordinates, and folds neither `bounds_proof` nor `ownership_proof`. The proof kind is retained evidence beside the identity rather than part of it, so a new variant needs no domain bump and no existing identity byte moves. The exact check is `grep -n "bounds_proof" crates/tiler-ir/src/index/builder.rs`, whose hits are confined to `remap_access` and the encoder is not among them.

### Tests

`a_wholly_undetermined_dynamic_copy_is_refused_rather_than_approximated` is replaced by `a_wholly_undetermined_dynamic_copy_verifies_by_proved_extent_equality`, which asserts the region verifies, that **both** accesses record `ProvedExtentEquality` rather than an interval or an enumeration, and that the write's ownership is still the permutation argument. Its neighbour `an_undetermined_copy_whose_extents_are_never_proved_equal_is_still_refused` runs the same fixture over an environment that declares `m` and `n` and relates them not at all: both accesses are refused, and neither refusal is `ProofResourceLimit`, because nothing was enumerated. The shared `undetermined_dynamic_copy` fixture is what makes the pair differ in the environment and in nothing else.

`cargo nextest run --workspace`: 790 tests, all passing. `uv run --locked python scripts/check_repository.py` passes.

### Contract updated

`docs/ir.md`'s index-layer capability list named "interval bounds proofs, resource-bounded finite fallback when a conservative interval overlaps a boundary"; it now names the structural proved-extent-equality proof between them, because the finite fallback is no longer what a symbolic equality falls through to.
