---
id: widen-shapeenv-factorization-fragment
title: Decide support for factorizations with multiple runtime-unknown terms
status: deferred
priority: p3
dependencies: []
related: [implement-shapeenv-constraints, implement-shapeenv-index-bindings]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [shapes, indexing, mature-product, deferred]
---
The current ShapeEnv deliberately accepts a decidable fragment. It rejects a split relation such as `parent == outer * tile` when more than one factor remains unknown at runtime.

## Deferred product boundary

No admitted product requires one artifact to support a split whose factors all remain unknown until launch. The current fail-closed behavior is correct and tested, so there is no live choice to present.

- **Refuse it.** Every built ShapeEnv remains decided; a frontend specializes on a static factor and that value participates in artifact identity. This is the current fail-closed behavior.
- **Support it.** First introduce either an explicit specialization input that supplies and identities a compile-time value before ShapeEnv construction, or an `Undecided` constraint state propagated through every consumer. An availability phase alone is not a third option.

## Recommendation

Keep fully launch-dynamic factorizations unsupported until a frontend or product requirement needs one artifact to serve arbitrary runtime factors. Explicit refusal preserves the accepted meaning that a built environment is decided and does not preclude a later specialization-input design.

**The refusal is already a checked behaviour, not merely a stated one.** `crates/tiler-ir/src/shape/env.rs:1447`, `a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored`, builds `n == a * b` over three unknowns and asserts `build()` returns `ShapeEnvError::UnsupportedRelation { violation: FragmentViolation::UnderdeterminedFactorization { undetermined: 3 } }` — the exact case this ticket is about. The same test separately asserts that the same relation supplied as a `VariantGuard` is refused too, on the stated ground that an undecidable guard leaves the variant's selectability unknown rather than merely unselected, and it pins the in-fragment contrast alongside: `128 == 8 * outer` is *solved* to `outer == 16` rather than stored. So "refuse it" is the tested status quo, and a change to "support it" has a named test that must move.

## Counterpoint

If one artifact must serve arbitrary runtime tile sizes without recompilation, specialization is insufficient; Tiler must accept the broader undecided-state cost and define how every optimizer, verifier, artifact, and runtime consumer handles it.

## The derivation

- Non-static `BindingSource` variants name where a future value comes from; they do not carry that value.
- `AvailabilityPhase` says when a binding may become available. It cannot make an unknown value determined.
- Only `BindingSource::Static(Extent)` currently contributes a constant to fragment checking.
- General nonlinear integer constraints over multiple 64-bit unknowns do not have a bounded complete procedure suitable for this compile path without a budget and an explicit undecided result.

The former proposed resolution—treat a compile-phase caller parameter as known from its phase—cannot be implemented in the current model because no supplied value exists to substitute.

`BindingSource` has exactly four variants at `crates/tiler-ir/src/shape/env.rs:259-287` — `Static(Extent)`, `InputDimension { input, axis }`, `InterfaceParameter { key }`, and `TargetProperty { key }` — and only the first contributes a constant, which is what the third fact above states and what makes the retired resolution unimplementable rather than merely unimplemented.

**The reopening trigger, supplied 2026-07-28.** Reopen on the first frontend or product requirement for a single artifact to serve arbitrary runtime tile sizes without recompilation — the exact case the counterpoint above names. Until one exists, specialization on a static factor satisfies every requirement this repository has, and the undecided-state cost buys nothing. A demand that can be met by recompiling per tile size is not the trigger; the trigger is a caller for whom recompilation is not available.

## Closes when

When activated, the shape contract records either the durable explicit refusal or the admitted specialization-input/undecided model, and no implementation infers a known value from availability alone. If the durable refusal and reopening trigger land elsewhere first, close this ticket as superseded by that contract.

## Trigger check log

- 2026-08-04 — **not fired.** No frontend or product requires one artifact to serve arbitrary runtime tile sizes without recompilation; every admitted route specializes on a static factor, and the refusal remains the tested behaviour at `crates/tiler-ir/src/shape/env.rs`'s `a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored`. A demand satisfiable by recompiling per tile size is explicitly not the trigger. Recheck: `grep -n 'a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored' crates/tiler-ir/src/shape/env.rs`.
- 2026-08-09 — **not fired.** No admitted frontend or product requires one artifact to serve arbitrary launch-time factor pairs without recompilation. `a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored` still pins `UnderdeterminedFactorization { undetermined: 3 }`, so the tested fail-closed boundary and the specialization alternative remain current.
