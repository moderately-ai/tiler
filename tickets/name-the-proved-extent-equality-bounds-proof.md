---
id: name-the-proved-extent-equality-bounds-proof
title: Name the proved-extent-equality bounds proof
status: todo
priority: p1
dependencies: []
related: [bind-shapeenv-sources-into-tensor-boundaries-and-coefficients, name-the-unprovable-symbolic-extent-diagnostic, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir]
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
